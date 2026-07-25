<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { useAppStore } from "../../stores/app";
import { useToast } from "../../composables/useToast";
import { SMART_VIEWS, viewToFilterPreset } from "../../composables/useSmartViews";
import { useBatchActions, type BatchIntent } from "../../composables/useBatchActions";
import SkillRow from "./SkillRow.vue";
import IssueRepairPanel, { type RepairGroupId } from "./IssueRepairPanel.vue";
import InstallDialog from "../skills/InstallDialog.vue";
import SkeletonCard from "../common/SkeletonCard.vue";

import {
  useManageFilters,
  useManageSelection,
  type LibraryScope,
} from "./manageFilters";
import type { SmartViewId } from "../../types";

type ManageFilterModel = ReturnType<typeof useManageFilters>;

const props = defineProps<{
  view: SmartViewId;
  filterModel: ManageFilterModel;
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const appStore = useAppStore();
const toast = useToast();

const filterModel = props.filterModel;
const selectionModel = useManageSelection(filterModel.filteredSkills);

// ── 视图与预设 ──────────────────────────────────────
const viewDef = computed(() => SMART_VIEWS.find((v) => v.id === props.view));
const viewTitle = computed(() => (viewDef.value ? t(viewDef.value.labelKey) : props.view));
const presetScopes = computed<Set<LibraryScope>>(
  () => viewToFilterPreset(props.view).libraryScope ?? new Set()
);

function applyViewPreset(view: SmartViewId) {
  const preset = viewToFilterPreset(view);
  filterModel.statusPreset.value = preset.statusPreset ?? "all";
  filterModel.libraryScope.value = preset.libraryScope ? new Set(preset.libraryScope) : new Set();
  filterModel.domain.value = preset.domain;
}

// ── 列表数据 ──────────────────────────────────────
const isRefreshing = ref(false);
const checkingUpdates = ref(false);
const availableUpdateCount = computed(
  () => Object.values(skillsStore.updateChecks).filter((r) => r.available).length
);
const detectedAgents = computed(() => agentsStore.agents.filter((a) => a.detected && a.enabled));
const displaySkills = filterModel.filteredSkills;
const totalSkills = computed(() => skillsStore.skills.length);

// ── 统计概览 ──────────────────────────────────
const stats = computed(() => {
  const all = skillsStore.skills;
  const linked = all.filter((s) => s.linked_agents.length > 0);
  const unlinked = all.filter((s) => s.linked_agents.length === 0);
  const issues = all.filter((s) => s.has_conflict || s.has_dangling || s.is_duplicate);
  return {
    total: all.length,
    linked: linked.length,
    unlinked: unlinked.length,
    issues: issues.length,
  };
});

const selectedSkills = selectionModel.selectedIds;
const allDisplayedSelected = selectionModel.allVisibleSelected;
const someDisplayedSelected = selectionModel.partiallyVisibleSelected;

// ── 批量操作（意图驱动） ──────────────────────────────────────
const selectedSkillsList = computed(() =>
  skillsStore.skills.filter((s) => selectedSkills.value.has(s.id))
);
const { allPairs, intents, operating: batchOperating, result: batchResult, execute: executeBatchIntent, clearResult } =
  useBatchActions(selectedSkillsList, detectedAgents, t);

const batchShowConfirm = ref(false);
const batchConfirmIntent = ref<BatchIntent | null>(null);
const batchShowResult = ref(false);
const batchExpandDetails = ref(false);

function onIntentClick(intent: BatchIntent) {
  if (intent.needsConfirm) {
    batchConfirmIntent.value = intent;
    batchShowConfirm.value = true;
  } else {
    // 安全操作直接执行
    runIntent(intent);
  }
}

async function runIntent(intent: BatchIntent) {
  await executeBatchIntent(intent);
  batchShowResult.value = true;
  batchExpandDetails.value = false;
  selectionModel.pruneMissing(skillsStore.skills);
}

function onBatchConfirm() {
  const intent = batchConfirmIntent.value;
  if (!intent) return;
  batchShowConfirm.value = false;
  batchConfirmIntent.value = null;
  runIntent(intent);
}

function closeBatchResult() {
  batchShowResult.value = false;
  clearResult();
}

// ── 工具栏：排序 / 搜索 ──────────────────────
const sortMenuOpen = ref(false);
const sortMenuRef = ref<HTMLElement | null>(null);
const searchInput = ref<HTMLInputElement | null>(null);

const sortOptions = computed(() =>
  [
    { value: "status", label: t("manage.sort_by_status_priority") },
    { value: "updated", label: t("manage.sort_by_updated") },
    { value: "name", label: t("manage.sort_by_name") },
    { value: "linked_agents", label: t("manage.sort_by_linked_agents") },
  ].filter((option) => option.value !== "linked_agents" || detectedAgents.value.length > 1)
);
const currentSortLabel = computed(
  () => sortOptions.value.find((o) => o.value === filterModel.sort.value)?.label || t("manage.sort_label")
);
function chooseSort(value: "status" | "updated" | "name" | "linked_agents") {
  filterModel.sort.value = value;
  sortMenuOpen.value = false;
}

// ── 用户条件与 Token ──────────────────────────────────
const userScopes = computed(() =>
  [...filterModel.libraryScope.value].filter((s) => !presetScopes.value.has(s))
);
const userFilterCount = computed(
  () =>
    Number(Boolean(filterModel.query.value.trim())) +
    filterModel.issues.value.size +
    userScopes.value.length +
    Number(filterModel.agentIds.value.size > 0)
);
const hasUserFilters = computed(() => userFilterCount.value > 0);

interface Token {
  key: string;
  label: string;
  onRemove: () => void;
}
const tokens = computed<Token[]>(() => {
  const list: Token[] = [];
  if (props.view !== "all" && props.view !== "plugins") {
    list.push({
      key: "view",
      label: viewTitle.value,
      onRemove: () => appStore.setActiveView("all"),
    });
  }
  if (filterModel.query.value.trim()) {
    list.push({ key: "query", label: filterModel.query.value, onRemove: () => filterModel.clearQuery() });
  }
  for (const issue of filterModel.issues.value) {
    list.push({
      key: `issue-${issue}`,
      label: t(`manage.status_${issue}`),
      onRemove: () => filterModel.removeIssue(issue),
    });
  }
  for (const scope of userScopes.value) {
    list.push({
      key: `scope-${scope}`,
      label: t(scope === "missing_library" ? "manage.quick_filter_missing_lib" : "manage.quick_filter_only_lib"),
      onRemove: () => filterModel.removeLibraryScope(scope),
    });
  }
  for (const agentId of filterModel.agentIds.value) {
    list.push({
      key: `agent-${agentId}`,
      label: agentsStore.agents.find((a) => a.id === agentId)?.name || agentId,
      onRemove: () => filterModel.removeAgent(agentId),
    });
  }
  return list;
});

function clearUserFilters() {
  filterModel.clearFilters();
  applyViewPreset(props.view);
  selectionModel.clearSelection();
  expandedSkillId.value = null;
}

// ── 修复中心 ──────────────────────────────────
function handleFilterGroup(group: RepairGroupId) {
  if (group === "conflict") {
    filterModel.issues.value = new Set(["conflict"]);
  } else if (group === "dangling") {
    filterModel.issues.value = new Set(["dangling"]);
  } else {
    filterModel.libraryScope.value = new Set([...filterModel.libraryScope.value, "missing_library"]);
  }
  // 自动全选（浮动条会自动出现）
  selectionModel.toggleAllVisible();
}

// ── 详情展开 / 选择 ──────────────────────────────────
const expandedSkillId = ref<string | null>(null);
function toggleSkillSelect(skillId: string) {
  selectionModel.toggleOne(skillId);
}
function toggleAllDisplayedSkills() {
  selectionModel.toggleAllVisible();
}
function deselectAllSkills() {
  selectionModel.clearSelection();
}

// ── 安装 ──────────────────────────────────
const showInstall = ref(false);

// ── 刷新 / 更新检查 ──────────────────────────────────
async function refreshManageData() {
  if (isRefreshing.value) return;
  isRefreshing.value = true;
  try {
    await Promise.all([
      skillsStore.refreshSkills(),
      agentsStore.fetchAgents(),
      skillsStore.fetchIssues(),
    ]);
    filterModel.normalizeAgents(detectedAgents.value);
    selectionModel.pruneMissing(skillsStore.skills);
    if (expandedSkillId.value && !skillsStore.skills.some((s) => s.id === expandedSkillId.value)) {
      expandedSkillId.value = null;
    }
  } finally {
    isRefreshing.value = false;
  }
}

async function checkAllUpdates() {
  if (checkingUpdates.value) return;
  checkingUpdates.value = true;
  try {
    const results = await skillsStore.checkAllSkillUpdates();
    const available = results.filter((r) => r.available).length;
    toast.show(
      available > 0 ? t("manage.updates_found", { count: available }) : t("manage.no_updates_found"),
      available > 0 ? "info" : "success"
    );
  } catch (error: unknown) {
    toast.show(String(error), "error");
  } finally {
    checkingUpdates.value = false;
  }
}

// ── 回到顶部 ──────────────────────────────────
const showScrollToTop = ref(false);
let scrollContainer: HTMLElement | null = null;
function getScrollContainer(): HTMLElement | null {
  if (!scrollContainer) scrollContainer = document.querySelector(".app-shell-content");
  return scrollContainer;
}
function scrollToTop() {
  getScrollContainer()?.scrollTo({ top: 0, behavior: "smooth" });
}
function handleScroll() {
  const container = getScrollContainer();
  if (container) showScrollToTop.value = container.scrollTop > 300;
}

// ── 快捷键与全局事件 ──────────────────────────────────
function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && (event.key === "k" || event.key === "f")) {
    const target = event.target as HTMLElement | null;
    if (target?.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target?.tagName || "")) return;
    event.preventDefault();
    searchInput.value?.focus();
  }
  if (event.key === "Escape") {
    if (sortMenuOpen.value) { sortMenuOpen.value = false; return; }
    if (batchShowConfirm.value) { batchShowConfirm.value = false; return; }
  }
}
function handlePointerDown(event: PointerEvent) {
  const target = event.target as Node;
  if (sortMenuRef.value && !sortMenuRef.value.contains(target)) sortMenuOpen.value = false;
}

// ── 视图切换 ──────────────────────────────────
watch(
  () => props.view,
  (view) => {
    applyViewPreset(view);
    expandedSkillId.value = null;
    batchShowConfirm.value = false;
    batchConfirmIntent.value = null;
  },
  { immediate: true }
);

onMounted(async () => {
  document.addEventListener("keydown", handleKeydown);
  document.addEventListener("pointerdown", handlePointerDown);
  setTimeout(() => {
    getScrollContainer()?.addEventListener("scroll", handleScroll);
  }, 100);
  if (skillsStore.skills.length === 0) await skillsStore.fetchSkills();
  if (agentsStore.agents.length === 0) await agentsStore.fetchAgents();
  if (skillsStore.issues.length === 0) await skillsStore.fetchIssues();
  filterModel.normalizeAgents(detectedAgents.value);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
  document.removeEventListener("pointerdown", handlePointerDown);
  getScrollContainer()?.removeEventListener("scroll", handleScroll);
});

defineExpose({
  clearSelection: selectionModel.clearSelection,
});
</script>

<template>
  <div class="space-y-4">
    <!-- 顶部统计概览 -->
    <section class="workspace-panel !p-3">
      <div class="stats-overview">
        <div class="stats-overview__item">
          <span class="stats-overview__value">{{ stats.total }}</span>
          <span class="stats-overview__label">{{ t("sidebar.view_all") }}</span>
        </div>
        <div class="stats-overview__divider" />
        <div class="stats-overview__item">
          <span class="stats-overview__value stats-overview__value--linked">{{ stats.linked }}</span>
          <span class="stats-overview__label">{{ t("sidebar.view_linked") }}</span>
        </div>
        <div class="stats-overview__divider" />
        <div class="stats-overview__item">
          <span class="stats-overview__value stats-overview__value--unlinked">{{ stats.unlinked }}</span>
          <span class="stats-overview__label">{{ t("sidebar.view_unlinked") }}</span>
        </div>
        <div class="stats-overview__divider" />
        <div class="stats-overview__item">
          <span class="stats-overview__value stats-overview__value--issues">{{ stats.issues }}</span>
          <span class="stats-overview__label">{{ t("sidebar.view_attention") }}</span>
        </div>
      </div>
    </section>

    <!-- 工具栏 -->
    <section class="workspace-panel !p-3">
      <div class="flex flex-wrap items-center gap-2">
        <h2 class="text-sm font-semibold" style="color: var(--c-text-strong);">
          {{ viewTitle }}
        </h2>
        <span
          class="rounded-full px-2 py-0.5 text-xs"
          style="background: var(--c-primary-light); color: var(--c-primary);"
        >
          {{ displaySkills.length }}/{{ totalSkills }}
        </span>
        <span class="hidden text-[11px] lg:inline" style="color: var(--c-text-secondary);">
          {{ t("manage.workspace_hint") }}
        </span>
        <div class="action-toolbar ml-auto">
          <button
            class="action-toolbar-icon disabled:opacity-50 disabled:cursor-not-allowed"
            :title="t('manage.refresh')"
            :disabled="isRefreshing"
            @click="refreshManageData"
          >
            <RefreshCw :size="15" :class="{ 'animate-spin': isRefreshing }" />
          </button>
          <button
            class="action-toolbar-icon relative disabled:opacity-50 disabled:cursor-not-allowed"
            :title="t('manage.check_all_updates')"
            :disabled="checkingUpdates || isRefreshing || totalSkills === 0"
            @click="checkAllUpdates"
          >
            <CloudDownload :size="15" :class="{ 'animate-spin': checkingUpdates }" />
            <span
              v-if="availableUpdateCount > 0"
              class="absolute -right-1 -top-1 min-w-3.5 rounded-full px-1 text-[9px] leading-3.5"
              style="background: var(--c-warning); color: white;"
            >{{ availableUpdateCount }}</span>
          </button>
          <button class="action-toolbar-primary" @click="showInstall = true">
            <Plus :size="15" />
            {{ t("skills.install") }}
          </button>
        </div>
      </div>

      <div class="manage-filter-toolbar mt-2.5">
        <div class="relative min-w-0 flex-1">
          <Search
            :size="14"
            class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2"
            style="color: var(--c-text-secondary);"
          />
          <input
            ref="searchInput"
            v-model="filterModel.query.value"
            :placeholder="t('skills.search') + ' (Ctrl+K)'"
            class="toolbar-control w-full rounded-md py-2 pl-9 pr-9 text-xs outline-none transition-colors"
          />
          <button
            v-if="filterModel.query.value"
            type="button"
            class="absolute right-2 top-1/2 inline-flex -translate-y-1/2 items-center justify-center rounded-full p-1"
            style="color: var(--c-text-secondary);"
            :aria-label="t('manage.clear_query')"
            @click="filterModel.clearQuery"
          >
            <X :size="13" />
          </button>
        </div>

        <div ref="sortMenuRef" class="relative shrink-0" style="min-width: 118px;">
          <button
            class="toolbar-control inline-flex w-full items-center justify-between gap-2 rounded-md px-3 py-2 text-[11px] outline-none transition-colors"
            :class="{ 'border-[var(--c-primary)]': sortMenuOpen }"
            type="button"
            aria-haspopup="listbox"
            :aria-expanded="sortMenuOpen"
            :aria-label="t('manage.sort_label')"
            @click.stop="sortMenuOpen = !sortMenuOpen"
          >
            <span class="truncate">{{ currentSortLabel }}</span>
            <ChevronDown
              :size="13"
              class="shrink-0 transition-transform"
              :style="{ transform: sortMenuOpen ? 'rotate(180deg)' : 'rotate(0deg)' }"
            />
          </button>
          <div
            v-if="sortMenuOpen"
            class="absolute left-0 top-full z-20 mt-1 w-full min-w-[138px] rounded-lg border p-1 shadow-lg"
            style="background: var(--c-surface-raised); border-color: var(--c-border);"
            role="listbox"
          >
            <button
              v-for="option in sortOptions"
              :key="option.value"
              class="flex w-full items-center justify-between rounded-md px-2.5 py-2 text-left text-[11px] transition-colors hover:bg-[var(--c-surface-hover)]"
              :style="{ color: option.value === filterModel.sort.value ? 'var(--c-primary)' : 'var(--c-text)' }"
              type="button"
              role="option"
              :aria-selected="option.value === filterModel.sort.value"
              @click="chooseSort(option.value as 'status' | 'updated' | 'name' | 'linked_agents')"
            >
              <span>{{ option.label }}</span>
              <Check v-if="option.value === filterModel.sort.value" :size="13" />
            </button>
          </div>
        </div>

        <button
          class="manage-filter-clear"
          :class="{ 'manage-filter-clear-active': hasUserFilters }"
          :disabled="!hasUserFilters"
          type="button"
          @click="clearUserFilters"
        >
          <X :size="13" />
          {{ t("manage.clear_filters") }}
        </button>
      </div>

      <!-- 已启用条件 Token -->
      <div v-if="tokens.length > 0" class="manage-active-filter-row mt-2">
        <span class="manage-active-filter-label">{{ t("manage.filter_active_summary") }}</span>
        <button
          v-for="token in tokens"
          :key="token.key"
          class="manage-filter-token"
          type="button"
          :title="token.key === 'view' ? t('sidebar.view_token_tip') : undefined"
          @click="token.onRemove"
        >
          {{ token.label }} <X :size="11" />
        </button>
        <span class="manage-filter-result-count">{{ displaySkills.length }} / {{ totalSkills }}</span>
      </div>
    </section>

    <!-- 待处理视图：修复分组卡 -->
    <section v-if="view === 'attention'">
      <IssueRepairPanel
        :skills="skillsStore.skills"
        :agents="detectedAgents"
        compact
        @filter-group="handleFilterGroup"
      />
    </section>

    <!-- Loading -->
    <div v-if="skillsStore.loading" class="space-y-3">
      <SkeletonCard v-for="i in 4" :key="i" />
    </div>

    <!-- Error -->
    <div v-else-if="skillsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ skillsStore.error }}
    </div>

    <!-- Empty -->
    <section v-else-if="displaySkills.length === 0" class="manage-empty-state">
      <div class="manage-empty-icon">
        <SearchX :size="28" />
      </div>
      <template v-if="totalSkills === 0">
        <h3>{{ t("skills.no_skills") }}</h3>
        <p>{{ t("skills.no_skills_hint") }}</p>
        <div class="manage-empty-actions">
          <button class="manage-empty-primary" type="button" @click="showInstall = true">
            {{ t("skills.install") }}
          </button>
        </div>
      </template>
      <template v-else>
        <h3>{{ t("manage.no_filter_results") }}</h3>
        <p>{{ t("manage.no_filter_results_hint") }}</p>
        <div class="manage-empty-actions">
          <button
            v-if="hasUserFilters"
            class="manage-empty-secondary"
            type="button"
            @click="clearUserFilters"
          >
            <X :size="14" />
            {{ t("manage.clear_filters") }}
          </button>
          <button class="manage-empty-primary" type="button" @click="showInstall = true">
            {{ t("skills.install") }}
          </button>
        </div>
      </template>
    </section>

    <!-- 列表 -->
    <template v-else>
      <div
        class="flex items-center gap-2 rounded-md border px-3 py-2"
        style="background: var(--c-surface); border-color: var(--c-border);"
      >
        <input
          type="checkbox"
          :checked="allDisplayedSelected"
          :indeterminate="someDisplayedSelected"
          class="h-3.5 w-3.5 cursor-pointer rounded"
          style="accent-color: var(--c-primary);"
          :title="t('manage.workbench_select_filtered')"
          :aria-label="t('manage.workbench_select_filtered')"
          @change="toggleAllDisplayedSkills"
        />
        <span class="text-xs" style="color: var(--c-text-secondary);">
          {{ t("manage.workbench_select_filtered") }}
        </span>
        <span class="ml-auto text-[11px]" style="color: var(--c-text-tertiary);">
          {{ selectedSkills.size }}/{{ displaySkills.length }}
        </span>
      </div>

      <!-- 技能列表 -->
      <div class="space-y-2">
        <SkillRow
          v-for="skill in displaySkills"
          :key="skill.id"
          :id="`skill-${skill.id}`"
          :skill="skill"
          :agents="agentsStore.agents"
          :selected="selectedSkills.has(skill.id)"
          :expanded="expandedSkillId === skill.id"
          @toggle:select="toggleSkillSelect"
          @update:expanded="expandedSkillId = $event ? skill.id : null"
        />
      </div>

      <!-- 浮动批量操作条（sticky 底部） -->
      <div
        v-if="selectedSkills.size > 0 && intents.length > 0"
        class="sticky bottom-0 z-10 rounded-lg border px-4 py-3"
        style="background: var(--c-surface); border-color: var(--c-border); box-shadow: 0 -4px 12px rgba(0,0,0,0.08);"
      >
        <!-- 摘要 -->
        <div class="flex items-center gap-2 text-[11px] mb-2" style="color: var(--c-text-secondary);">
          <span>{{ t("batch.selected_summary", { skills: selectedSkills.size, pairs: intents.reduce((s, i) => s + i.count, 0) }) }}</span>
          <button
            class="ml-auto text-[10px] cursor-pointer hover:underline"
            style="color: var(--c-text-tertiary);"
            type="button"
            @click="batchExpandDetails = !batchExpandDetails"
          >
            {{ batchExpandDetails ? t("batch.hide_details") : t("batch.show_details") }}
          </button>
        </div>

        <!-- 意图按钮 -->
        <div class="flex flex-wrap gap-2">
          <button
            v-for="intent in intents"
            :key="intent.id"
            type="button"
            class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-[11px] font-medium cursor-pointer transition-colors border disabled:opacity-50"
            :style="{
              background: 'var(--c-primary-light)',
              borderColor: 'var(--c-primary)',
              color: 'var(--c-primary)',
            }"
            :disabled="batchOperating"
            @click="onIntentClick(intent)"
          >
            <component :is="intent.icon" :size="13" />
            {{ t(intent.labelKey, { count: intent.count }) }}
          </button>

          <button
            type="button"
            class="inline-flex items-center gap-1 rounded-md px-2.5 py-1.5 text-[11px] cursor-pointer"
            style="color: var(--c-text-secondary);"
            @click="deselectAllSkills"
          >
            {{ t("manage.deselect_all") }}
          </button>
        </div>

        <!-- 展开详情 -->
        <div v-if="batchExpandDetails" class="mt-2 rounded border text-[10px] overflow-hidden" style="border-color: var(--c-border);">
          <div class="max-h-32 overflow-y-auto">
            <div
              v-for="(pair, i) in allPairs.slice(0, 100)"
              :key="i"
              class="flex items-center gap-2 px-2.5 py-1"
              :style="{ background: i % 2 ? 'var(--c-surface)' : 'transparent' }"
            >
              <span class="truncate flex-1" style="color: var(--c-text);">{{ pair.skillName }}</span>
              <span style="color: var(--c-text-tertiary);">→</span>
              <span class="truncate" style="color: var(--c-text-secondary);">{{ pair.agentName }}</span>
              <span class="shrink-0 rounded px-1 text-[9px]" style="background: var(--c-surface-hover); color: var(--c-text-tertiary);">
                {{ t(`manage.btn_${pair.action}`) }}
              </span>
            </div>
            <div v-if="allPairs.length > 100" class="px-2.5 py-1 text-[10px]" style="color: var(--c-text-tertiary);">
              ... {{ allPairs.length - 100 }} {{ t("manage.batch_detail_more") }}
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>

  <!-- 批量确认弹窗 -->
  <Teleport to="body">
    <div
      v-if="batchShowConfirm && batchConfirmIntent"
      class="modal-backdrop fixed inset-0 z-50 flex items-center justify-center p-4"
      @click.self="batchShowConfirm = false"
    >
      <div class="modal-shell flex w-full max-w-lg flex-col" style="max-height: 80vh;">
        <div class="modal-header shrink-0">
          <h3 class="text-sm font-semibold" style="color: var(--c-text);">
            {{ t(`batch.confirm_${batchConfirmIntent.id}_title`) }}
          </h3>
          <button
            class="inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md"
            style="color: var(--c-text-secondary);"
            type="button"
            @click="batchShowConfirm = false"
          >
            &times;
          </button>
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto p-4 space-y-3">
          <p class="text-xs" style="color: var(--c-text-secondary);">
            {{ t(`batch.confirm_${batchConfirmIntent.id}_msg`, { count: batchConfirmIntent.count }) }}
          </p>
          <div class="rounded border text-[11px] overflow-hidden" style="border-color: var(--c-border);">
            <div class="max-h-48 overflow-y-auto">
              <div
                v-for="(pair, i) in batchConfirmIntent.pairs.slice(0, 50)"
                :key="i"
                class="flex items-center gap-2 px-3 py-1.5"
                :style="{ background: i % 2 ? 'var(--c-surface)' : 'transparent' }"
              >
                <span class="truncate flex-1" style="color: var(--c-text);">{{ pair.skillName }}</span>
                <span style="color: var(--c-text-tertiary);">→</span>
                <span class="truncate" style="color: var(--c-text-secondary);">{{ pair.agentName }}</span>
                <span class="shrink-0 rounded px-1 text-[9px]" style="background: var(--c-surface-hover); color: var(--c-text-tertiary);">
                  {{ t(`manage.btn_${pair.action}`) }}
                </span>
              </div>
              <div v-if="batchConfirmIntent.pairs.length > 50" class="px-3 py-1.5 text-[10px]" style="color: var(--c-text-tertiary);">
                ... {{ batchConfirmIntent.pairs.length - 50 }} {{ t("manage.batch_detail_more") }}
              </div>
            </div>
          </div>
        </div>
        <div class="flex shrink-0 items-center justify-end gap-2 border-t px-4 py-2.5" style="border-color: var(--c-border);">
          <button
            class="rounded-md px-3 py-1.5 text-[11px] cursor-pointer"
            style="color: var(--c-text-secondary); border: 1px solid var(--c-border);"
            @click="batchShowConfirm = false"
          >
            {{ t("common.cancel") }}
          </button>
          <button
            class="rounded-md px-4 py-1.5 text-[11px] font-medium cursor-pointer disabled:opacity-50"
            style="background: var(--c-primary); color: white;"
            :disabled="batchOperating"
            @click="onBatchConfirm"
          >
            {{ batchOperating ? "..." : t("manage.batch_panel_execute") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- 批量结果弹窗 -->
  <Teleport to="body">
    <div
      v-if="batchShowResult && batchResult"
      class="modal-backdrop fixed inset-0 z-50 flex items-center justify-center p-4"
      @click.self="closeBatchResult"
    >
      <div class="modal-shell flex w-full max-w-lg flex-col" style="max-height: 80vh;">
        <div class="modal-header shrink-0">
          <h3 class="text-sm font-semibold" style="color: var(--c-text);">
            {{ t("batch.result_title") }}
          </h3>
          <button
            class="inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md"
            style="color: var(--c-text-secondary);"
            type="button"
            @click="closeBatchResult"
          >
            &times;
          </button>
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto p-4 space-y-3">
          <div class="flex items-center gap-3 text-xs">
            <span class="rounded-full px-1.5 py-0.5 text-[10px]" style="background: var(--c-success-light, rgba(34,197,94,0.15)); color: var(--c-success);">
              {{ batchResult.success }} {{ t("batch.result_success") }}
            </span>
            <span v-if="batchResult.errors > 0" class="rounded-full px-1.5 py-0.5 text-[10px]" style="background: var(--c-danger-light, rgba(239,68,68,0.15)); color: var(--c-danger);">
              {{ batchResult.errors }} {{ t("batch.result_error") }}
            </span>
          </div>
          <div class="rounded border text-[11px] overflow-hidden" style="border-color: var(--c-border);">
            <div class="max-h-64 overflow-y-auto">
              <div
                v-for="(detail, i) in batchResult.details.slice(0, 100)"
                :key="i"
                class="flex items-center gap-2 px-3 py-1.5"
                :style="{
                  background: i % 2 ? 'var(--c-surface)' : 'transparent',
                  color: detail.status === 'success' ? 'var(--c-text)' : 'var(--c-danger)',
                }"
              >
                <span class="shrink-0 w-3">{{ detail.status === "success" ? "✓" : "✗" }}</span>
                <span class="truncate flex-1">{{ detail.skillName }}</span>
                <span style="color: var(--c-text-tertiary);">→</span>
                <span class="truncate" style="color: var(--c-text-secondary);">{{ detail.agentName }}</span>
                <span v-if="detail.message" class="truncate text-[10px] max-w-[120px]" :title="detail.message">({{ detail.message }})</span>
              </div>
              <div v-if="batchResult.details.length > 100" class="px-3 py-1.5 text-[10px]" style="color: var(--c-text-tertiary);">
                ... {{ batchResult.details.length - 100 }} {{ t("manage.batch_detail_more") }}
              </div>
            </div>
          </div>
        </div>
        <div class="flex shrink-0 items-center justify-end border-t px-4 py-2.5" style="border-color: var(--c-border);">
          <button
            class="rounded-md px-4 py-1.5 text-[11px] font-medium cursor-pointer"
            style="background: var(--c-primary); color: white;"
            @click="closeBatchResult"
          >
            {{ t("common.confirm") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <InstallDialog v-if="showInstall" @close="showInstall = false" />

  <!-- 回到顶部 -->
  <Transition
    enter-active-class="transition duration-200 ease-out"
    leave-active-class="transition duration-200 ease-in"
    enter-from-class="opacity-0 scale-90"
    enter-to-class="opacity-100 scale-100"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-90"
  >
    <button
      v-if="showScrollToTop"
      class="scroll-to-top-btn"
      type="button"
      :title="t('manage.scroll_to_top')"
      @click="scrollToTop"
    >
      <ArrowUp :size="18" />
    </button>
  </Transition>
</template>

<style scoped>
.stats-overview {
  display: flex;
  align-items: center;
  gap: 0;
}

.stats-overview__item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.stats-overview__value {
  font-size: 18px;
  font-weight: 600;
  line-height: 1.2;
  color: var(--c-text-strong);
}

.stats-overview__value--linked {
  color: var(--c-primary);
}

.stats-overview__value--unlinked {
  color: var(--c-text-tertiary);
}

.stats-overview__value--issues {
  color: var(--c-warning, #f59e0b);
}

.stats-overview__label {
  font-size: 10px;
  color: var(--c-text-tertiary);
  white-space: nowrap;
}

.stats-overview__divider {
  width: 1px;
  height: 28px;
  background: var(--c-border);
  flex-shrink: 0;
}
</style>
