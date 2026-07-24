<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { useAppStore } from "../../stores/app";
import { useToast } from "../../composables/useToast";
import { SMART_VIEWS, viewToFilterPreset } from "../../composables/useSmartViews";
import type { RepairContext } from "../../composables/useBatchCellActions";
import SkillRow from "./SkillRow.vue";
import IssueRepairPanel, { type RepairGroupId } from "./IssueRepairPanel.vue";
import FilterPopover from "./FilterPopover.vue";
import BatchDrawer from "../batch/BatchDrawer.vue";
import InstallDialog from "../skills/InstallDialog.vue";
import AddAgentDialog from "../agents/AddAgentDialog.vue";
import AgentCard from "../agents/AgentCard.vue";
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
const isPluginsView = computed(() => props.view === "plugins");
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

const selectedSkills = selectionModel.selectedIds;
const selectedSkillIds = computed(() => [...selectedSkills.value]);
const allDisplayedSelected = selectionModel.allVisibleSelected;
const someDisplayedSelected = selectionModel.partiallyVisibleSelected;

// ── 插件视图分组 ──────────────────────────────────
const pluginGroups = computed(() => {
  if (!isPluginsView.value) return [];
  const groups = new Map<string, typeof displaySkills.value>();
  for (const skill of displaySkills.value) {
    const source = skill.plugin_source || "unknown";
    const group = groups.get(source) || [];
    group.push(skill);
    groups.set(source, group);
  }
  return Array.from(groups.entries()).map(([source, skills]) => ({ source, skills }));
});
const expandedPluginGroups = ref<Set<string>>(new Set());
function togglePluginGroup(source: string) {
  const next = new Set(expandedPluginGroups.value);
  if (next.has(source)) next.delete(source);
  else next.add(source);
  expandedPluginGroups.value = next;
}
function isPluginGroupExpanded(source: string): boolean {
  return expandedPluginGroups.value.has(source);
}

// ── 工具栏：排序 / 筛选弹层 / 搜索 ──────────────────────
const sortMenuOpen = ref(false);
const sortMenuRef = ref<HTMLElement | null>(null);
const filterOpen = ref(false);
const filterWrapRef = ref<HTMLElement | null>(null);
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
const domainOverridden = computed(
  () => filterModel.domain.value !== viewToFilterPreset(props.view).domain
);
const userFilterCount = computed(
  () =>
    Number(Boolean(filterModel.query.value.trim())) +
    filterModel.issues.value.size +
    userScopes.value.length +
    Number(filterModel.agentIds.value.size > 0) +
    Number(domainOverridden.value)
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
  if (domainOverridden.value) {
    list.push({
      key: "domain",
      label: t(filterModel.domain.value === "local" ? "filter.domain_local" : "filter.domain_plugin"),
      onRemove: () => {
        filterModel.domain.value = viewToFilterPreset(props.view).domain;
      },
    });
  }
  return list;
});

function clearUserFilters() {
  filterModel.clearFilters();
  // 视图持有字段按当前视图预设回填（清除用户条件，不动视图本身）
  applyViewPreset(props.view);
  selectionModel.clearSelection();
  expandedSkillId.value = null;
}

// ── 修复上下文（首次接线 repairContext） ──────────────────────
const repairContext = ref<RepairContext | null>(null);

function handleFilterGroup(group: RepairGroupId) {
  if (group === "conflict") {
    filterModel.issues.value = new Set(["conflict"]);
    repairContext.value = "conflict";
  } else if (group === "dangling") {
    filterModel.issues.value = new Set(["dangling"]);
    repairContext.value = "dangling";
  } else {
    filterModel.libraryScope.value = new Set([...filterModel.libraryScope.value, "missing_library"]);
    repairContext.value = "missing_lib";
  }
}

// 修复上下文与其筛选条件同生共死：条件被移除/清除时上下文一并失效
watch([filterModel.issues, filterModel.libraryScope], () => {
  const ctx = repairContext.value;
  if (!ctx) return;
  const active =
    (ctx === "conflict" && filterModel.issues.value.has("conflict")) ||
    (ctx === "dangling" && filterModel.issues.value.has("dangling")) ||
    (ctx === "missing_lib" && filterModel.libraryScope.value.has("missing_library"));
  if (!active) repairContext.value = null;
});

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

// ── 批量抽屉 ──────────────────────────────────
const drawerOpen = ref(false);
const drawerRef = ref<{ markContextStale: () => void } | null>(null);
const isNarrow = ref(window.innerWidth < 960);

function openBatchDrawer() {
  if (selectedSkillIds.value.length === 0) return;
  if (isNarrow.value) expandedSkillId.value = null; // 覆盖模式下收起行内详情（§15 Q5）
  drawerOpen.value = true;
}
function closeBatchDrawer() {
  drawerOpen.value = false;
}
function removeSkillFromSelection(skillId: string) {
  const next = new Set(selectedSkills.value);
  next.delete(skillId);
  selectedSkills.value = next;
}
function resolveConflictFromDrawer(skillId: string) {
  drawerOpen.value = false;
  repairContext.value = "conflict";
  expandedSkillId.value = skillId;
}
function onBatchApplied() {
  selectionModel.pruneMissing(skillsStore.skills);
}

// ── Agent 管理 / 安装 ──────────────────────────────────
const showAgentManager = ref(false);
const showAddAgent = ref(false);
const showInstall = ref(false);
function agentSkillCount(agentId: string): number {
  return skillsStore.skills.filter((skill) =>
    skill.sources.some((s) => s.from === agentId)
  ).length;
}

// ── 插件同步 ──────────────────────────────────
async function handleSyncPlugin(skillId: string) {
  try {
    const skill = skillsStore.skills.find((s) => s.id === skillId);
    if (!skill) return;
    const pluginSource = skill.sources.find((s) => s.source_kind === "marketplace");
    if (!pluginSource) return;
    await skillsStore.installSkill(pluginSource.path, false);
    toast.show(t("manage.sync_to_library_success"), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

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
    if (sortMenuOpen.value) {
      sortMenuOpen.value = false;
      return;
    }
    if (filterOpen.value) {
      filterOpen.value = false;
      return;
    }
  }
}
function handlePointerDown(event: PointerEvent) {
  const target = event.target as Node;
  if (sortMenuRef.value && !sortMenuRef.value.contains(target)) sortMenuOpen.value = false;
  if (filterWrapRef.value && !filterWrapRef.value.contains(target)) filterOpen.value = false;
}
function handleWindowResize() {
  isNarrow.value = window.innerWidth < 960;
}

// ── 视图切换：应用预设 + 重置局部状态 ──────────────────────
watch(
  () => props.view,
  (view) => {
    applyViewPreset(view);
    expandedSkillId.value = null;
    repairContext.value = null;
    if (drawerOpen.value) drawerRef.value?.markContextStale();
  },
  { immediate: true }
);

onMounted(async () => {
  document.addEventListener("keydown", handleKeydown);
  document.addEventListener("pointerdown", handlePointerDown);
  window.addEventListener("resize", handleWindowResize);
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
  window.removeEventListener("resize", handleWindowResize);
  getScrollContainer()?.removeEventListener("scroll", handleScroll);
});

defineExpose({
  clearSelection: selectionModel.clearSelection,
});
</script>

<template>
  <div class="flex items-start gap-4">
    <div class="min-w-0 flex-1 space-y-4">
      <!-- 工具栏 -->
      <section class="workspace-panel !p-3">
        <div class="flex flex-wrap items-center gap-2">
          <h2 class="text-sm font-semibold" style="color: var(--c-text-strong);">
            {{ isPluginsView ? t("plugins.view_title") : viewTitle }}
          </h2>
          <span
            class="rounded-full px-2 py-0.5 text-xs"
            style="background: var(--c-primary-light); color: var(--c-primary);"
          >
            {{ displaySkills.length }}/{{ totalSkills }}
          </span>
          <span class="hidden text-[11px] lg:inline" style="color: var(--c-text-secondary);">
            {{ isPluginsView ? t("plugins.view_hint") : t("manage.workspace_hint") }}
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
            <button
              class="action-toolbar-icon"
              :title="t('manage.agent_management')"
              @click="showAgentManager = true"
            >
              <Settings :size="15" />
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

          <div ref="filterWrapRef" class="relative shrink-0">
            <button
              class="inline-flex items-center justify-center gap-1 rounded-md border px-2.5 py-2 text-[11px] cursor-pointer transition-colors"
              :style="{
                borderColor: filterOpen || userFilterCount > 0 ? 'var(--c-primary)' : 'var(--c-border)',
                color: filterOpen || userFilterCount > 0 ? 'var(--c-primary)' : 'var(--c-text-secondary)',
              }"
              type="button"
              :aria-expanded="filterOpen"
              @click.stop="filterOpen = !filterOpen"
            >
              <SlidersHorizontal :size="13" />
              {{ t("filter.trigger") }}
              <span
                v-if="userFilterCount > 0"
                class="rounded-full px-1.5 text-[9px]"
                style="background: var(--c-primary-light);"
              >{{ userFilterCount }}</span>
            </button>
            <FilterPopover
              :open="filterOpen"
              :filter-model="filterModel"
              :facet-counts="filterModel.facetCounts.value"
              :agents="detectedAgents"
              :default-domain="viewDef?.domain ?? 'local'"
              @close="filterOpen = false"
            />
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
        <template v-if="isPluginsView">
          <h3>{{ t("plugins.empty") }}</h3>
          <p>{{ t("plugins.view_hint") }}</p>
        </template>
        <template v-else-if="totalSkills === 0">
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

        <!-- 本地视图：技能列表 -->
        <div v-if="!isPluginsView" class="space-y-2">
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
            @sync-plugin="handleSyncPlugin"
          />
        </div>

        <!-- 插件视图：按市场分组 -->
        <div v-else class="space-y-3">
          <div v-for="group in pluginGroups" :key="group.source" class="plugin-group">
            <div
              class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 select-none hover:bg-[var(--c-surface-hover)]"
              @click="togglePluginGroup(group.source)"
            >
              <ChevronRight
                :size="13"
                class="transition-transform"
                :style="{
                  color: 'var(--c-plugin, #8b5cf6)',
                  transform: isPluginGroupExpanded(group.source) ? 'rotate(90deg)' : 'rotate(0deg)',
                }"
              />
              <Puzzle :size="13" style="color: var(--c-plugin, #8b5cf6);" />
              <span class="text-[11px] font-semibold uppercase tracking-wide" style="color: var(--c-plugin, #8b5cf6);">
                {{ group.source }}
              </span>
              <span
                class="rounded-full px-1.5 text-[9px]"
                style="background: var(--c-plugin-light, rgba(139, 92, 246, 0.15)); color: var(--c-plugin, #8b5cf6);"
              >{{ group.skills.length }}</span>
            </div>
            <div v-if="isPluginGroupExpanded(group.source)" class="mt-1.5 space-y-2">
              <SkillRow
                v-for="skill in group.skills"
                :key="`plugin-${group.source}-${skill.id}`"
                :id="`skill-${skill.id}`"
                :skill="skill"
                :agents="agentsStore.agents"
                :selected="selectedSkills.has(skill.id)"
                :expanded="expandedSkillId === skill.id"
                @toggle:select="toggleSkillSelect"
                @update:expanded="expandedSkillId = $event ? skill.id : null"
                @sync-plugin="handleSyncPlugin"
              />
            </div>
          </div>
        </div>

        <!-- 内联批量操作条（文档流，非 fixed 覆盖） -->
        <div
          v-if="selectedSkills.size > 0"
          class="flex items-center gap-3 rounded-lg border px-4 py-2.5"
          style="background: var(--c-primary-light); border-color: color-mix(in srgb, var(--c-primary) 30%, transparent);"
        >
          <input
            type="checkbox"
            :checked="allDisplayedSelected"
            :indeterminate="someDisplayedSelected"
            class="h-3.5 w-3.5 cursor-pointer rounded"
            style="accent-color: var(--c-primary);"
            :aria-label="t('manage.workbench_select_filtered')"
            @change="toggleAllDisplayedSkills"
          />
          <span class="text-xs" style="color: var(--c-text);">
            {{ t("manage.selected_scope_count", { selected: selectedSkills.size, total: displaySkills.length }) }}
          </span>
          <div class="ml-auto flex items-center gap-2">
            <button
              class="cursor-pointer rounded-md px-3 py-1.5 text-[11px] font-medium transition-colors"
              style="background: var(--c-primary); color: white;"
              type="button"
              @click="openBatchDrawer"
            >
              {{ t("manage.batch_panel_open") }}
            </button>
            <button
              class="cursor-pointer rounded px-2 py-1 text-[11px]"
              style="color: var(--c-text-secondary);"
              type="button"
              @click="deselectAllSkills"
            >
              {{ t("manage.deselect_all") }}
            </button>
          </div>
        </div>
      </template>
    </div>

    <!-- 批量抽屉（docked：flex 兄弟；overlay：组件内 Teleport 覆盖） -->
    <BatchDrawer
      v-if="drawerOpen"
      ref="drawerRef"
      :open="drawerOpen"
      :selected-skill-ids="selectedSkillIds"
      :repair-context="repairContext"
      :overlay="isNarrow"
      @close="closeBatchDrawer"
      @remove-skill="removeSkillFromSelection"
      @resolve-conflict="resolveConflictFromDrawer"
      @applied="onBatchApplied"
    />
  </div>

  <!-- Agent 管理浮层 -->
  <Teleport to="body">
    <div
      v-if="showAgentManager"
      class="modal-backdrop fixed inset-0 z-50 flex items-center justify-center p-4"
      @click.self="showAgentManager = false"
    >
      <div class="modal-shell flex w-full max-w-3xl flex-col">
        <div class="modal-header shrink-0">
          <div>
            <h3 class="text-sm font-semibold" style="color: var(--c-text);">
              {{ t("manage.agent_management") }}
            </h3>
            <p class="mt-0.5 text-[11px]" style="color: var(--c-text-secondary);">
              {{ t("manage.agent_management_hint") }}
            </p>
          </div>
          <div class="flex items-center gap-2">
            <button
              class="inline-flex cursor-pointer items-center gap-1 rounded-md px-3 py-1.5 text-xs"
              style="background: var(--c-primary); color: white;"
              type="button"
              @click="showAddAgent = true"
            >
              <Plus :size="14" />
              {{ t("agents.add") }}
            </button>
            <button
              class="inline-flex h-7 w-7 cursor-pointer items-center justify-center rounded-md"
              style="color: var(--c-text-secondary);"
              type="button"
              @click="showAgentManager = false"
            >
              &times;
            </button>
          </div>
        </div>
        <div class="overflow-y-auto p-4">
          <div class="grid gap-3" style="grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));">
            <AgentCard
              v-for="agent in agentsStore.agents"
              :key="agent.id"
              :agent="agent"
              :skill-count="agentSkillCount(agent.id)"
            />
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <AddAgentDialog v-if="showAddAgent" @close="showAddAgent = false" @added="showAddAgent = false" />
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
