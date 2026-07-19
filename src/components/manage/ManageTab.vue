<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import SkillWorkbench from "./SkillWorkbench.vue";
import BatchSyncPanel from "./BatchSyncPanel.vue";
import IssueRepairPanel from "./IssueRepairPanel.vue";
import InstallDialog from "../skills/InstallDialog.vue";
import AddAgentDialog from "../agents/AddAgentDialog.vue";
import AgentCard from "../agents/AgentCard.vue";
import SkeletonCard from "../common/SkeletonCard.vue";
import {
  useManageFilters,
  useManageSelection,
  type IssueFilter,
  type LibraryScope,
  type StatusPreset,
} from "./manageFilters";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const isRefreshing = ref(false);
const filtersExpanded = ref(false);
const sortMenuOpen = ref(false);
const sortMenuRef = ref<HTMLElement | null>(null);
const searchInput = ref<HTMLInputElement | null>(null);
const workbenchViewport = ref<HTMLElement | null>(null);
const expandedSkillId = ref<string | null>(null);
const showAgentManager = ref(false);
const showAddAgent = ref(false);
const showInstall = ref(false);
const showBatch = ref(false);
const batchRepairContext = ref<string | null>(null);

const detectedAgents = computed(() => agentsStore.agents.filter((agent) => agent.detected && agent.enabled));
const skillList = computed(() => skillsStore.skills);
const filterModel = useManageFilters(skillList, detectedAgents);
const selectionModel = useManageSelection(filterModel.filteredSkills);

const displaySkills = filterModel.filteredSkills;
const selectedSkills = selectionModel.selectedIds;
const selectedSkillIds = computed(() => [...selectedSkills.value]);
const allDisplayedSelected = selectionModel.allVisibleSelected;
const someDisplayedSelected = selectionModel.partiallyVisibleSelected;
const hasActiveFilters = filterModel.hasActiveFilters;
const selectedVisibleCount = selectionModel.selectedVisibleCount;
const facetCounts = filterModel.facetCounts;
const activeFilterCount = filterModel.activeFilterCount;
const activeIssueTokens = computed(() => [...filterModel.issues.value]);
const activeLibraryTokens = computed(() => [...filterModel.libraryScope.value]);
const activeAgentTokens = computed(() => [...filterModel.agentIds.value]);

const agentOverviews = computed(() => detectedAgents.value.map((agent) => ({
  agent,
  skillCount: skillsStore.skills.filter((skill) => skill.sources.some((source) => source.from === agent.id)).length,
  linkedCount: skillsStore.skills.filter((skill) => skill.sources.some((source) => source.from === agent.id && source.is_symlink)).length,
  conflictCount: skillsStore.skills.filter((skill) => skill.has_conflict && skill.sources.some((source) => source.from === agent.id)).length,
})))

function agentSkillCount(agentId: string): number {
  return skillsStore.skills.filter((skill) => skill.sources.some((source) => source.from === agentId)).length;
}
;

const totalSkills = computed(() => skillsStore.skills.length);
const sharedSkills = computed(() => skillsStore.skills.filter((skill) => skill.sources.filter((source) => source.from !== "vibe-lib").length > 1));
const uniqueSkills = computed(() => skillsStore.skills.filter((skill) => skill.sources.filter((source) => source.from !== "vibe-lib").length === 1));
const issueSkills = computed(() => skillsStore.skills.filter((skill) => skill.has_conflict || skill.has_dangling));
const sortOptions = computed(() => [
  { value: "status", label: t("manage.sort_by_status_priority") || "需处理优先" },
  { value: "updated", label: t("manage.sort_by_updated") || "最近更新" },
  { value: "name", label: t("manage.sort_by_name") || "名称 A-Z" },
  { value: "linked_agents", label: t("manage.sort_by_linked_agents") || "关联 Agent 数" },
].filter((option) => option.value !== "linked_agents" || detectedAgents.value.length > 1));
const currentSortLabel = computed(() => sortOptions.value.find((option) => option.value === filterModel.sort.value)?.label || t("manage.sort_label") || "显示顺序");

function clearAllFilters() {
  filterModel.clearFilters();
  selectionModel.clearSelection();
  expandedSkillId.value = null;
}

function toggleSkillSelect(skillId: string) {
  selectionModel.toggleOne(skillId);
}

function toggleAllDisplayedSkills() {
  selectionModel.toggleAllVisible();
}

function deselectAllSkills() {
  selectionModel.clearSelection();
}

function chooseSort(value: "status" | "updated" | "name" | "linked_agents") {
  filterModel.sort.value = value;
  sortMenuOpen.value = false;
}

function toggleIssue(issue: IssueFilter) {
  filterModel.toggleIssue(issue);
}

function toggleLibraryScope(scope: LibraryScope) {
  filterModel.toggleLibraryScope(scope);
}

function setStatusPreset(preset: StatusPreset) {
  filterModel.statusPreset.value = preset;
}

function selectAgentFromOverview(agentId: string) {
  filterModel.toggleAgent(agentId);
}

function removeIssue(issue: IssueFilter) {
  filterModel.removeIssue(issue);
}

function removeLibraryScope(scope: LibraryScope) {
  filterModel.removeLibraryScope(scope);
}

function removeAgent(agentId: string) {
  filterModel.removeAgent(agentId);
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && (event.key === "k" || event.key === "f")) {
    const target = event.target as HTMLElement | null;
    if (target?.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target?.tagName || "")) return;
    event.preventDefault();
    searchInput.value?.focus();
  }
  if (event.key === "Escape") {
    sortMenuOpen.value = false;
    if (filtersExpanded.value) filtersExpanded.value = false;
  }
}

function handleSortOutside(event: PointerEvent) {
  if (!sortMenuRef.value?.contains(event.target as Node)) sortMenuOpen.value = false;
}

onMounted(async () => {
  document.addEventListener("keydown", handleKeydown);
  document.addEventListener("pointerdown", handleSortOutside);
  if (skillsStore.skills.length === 0) await skillsStore.fetchSkills();
  if (agentsStore.agents.length === 0) await agentsStore.fetchAgents();
  if (skillsStore.issues.length === 0) await skillsStore.fetchIssues();
  filterModel.normalizeAgents(detectedAgents.value);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
  document.removeEventListener("pointerdown", handleSortOutside);
});

async function refreshManageData() {
  if (isRefreshing.value) return;
  isRefreshing.value = true;
  try {
    await Promise.all([skillsStore.refreshSkills(), agentsStore.fetchAgents(), skillsStore.fetchIssues()]);
    filterModel.normalizeAgents(detectedAgents.value);
    selectionModel.pruneMissing(skillsStore.skills);
    if (expandedSkillId.value && !skillsStore.skills.some((skill) => skill.id === expandedSkillId.value)) {
      expandedSkillId.value = null;
    }
  } finally {
    isRefreshing.value = false;
  }
}

function openBatchPanel() {
  if (selectedSkillIds.value.length === 0) return;
  batchRepairContext.value = null;
  showBatch.value = true;
}

function removeSkillFromSelection(skillId: string) {
  const next = new Set(selectedSkills.value);
  next.delete(skillId);
  selectedSkills.value = next;
}

function closeBatchPanel() {
  showBatch.value = false;
  batchRepairContext.value = null;
}

function resolveConflictFromBatch(skillId: string) {
  showBatch.value = false;
  batchRepairContext.value = "conflict";
  expandedSkillId.value = skillId;
    setTimeout(() => {
    const row = document.getElementById(`skill-${skillId}`);
    const viewport = workbenchViewport.value;
    if (!row || !viewport) return;
    const header = viewport.querySelector<HTMLElement>(".workbench-header-row");
    const headerHeight = header?.offsetHeight ?? 0;
    const viewportRect = viewport.getBoundingClientRect();
    const rowRect = row.getBoundingClientRect();
    const topLimit = viewportRect.top + headerHeight + 8;
    const bottomLimit = viewportRect.bottom - 8;
    if (rowRect.top < topLimit) {
      viewport.scrollBy({ top: rowRect.top - topLimit, behavior: "smooth" });
    } else if (rowRect.bottom > bottomLimit) {
      viewport.scrollBy({ top: rowRect.bottom - bottomLimit, behavior: "smooth" });
    }
  }, 100);
}

function onBatchApplied() {
  selectionModel.pruneMissing(skillsStore.skills);
}

function selectIssueGroup(skillIds: string[], openBatch: boolean, repairContext: string) {
  if (repairContext === "conflict" || repairContext === "dangling") {
    filterModel.toggleIssue(repairContext);
    filtersExpanded.value = true;
    selectionModel.clearSelection();
    expandedSkillId.value = null;
    return;
  }
  if (repairContext === "missing_lib") {
    filterModel.toggleLibraryScope("missing_library");
    filtersExpanded.value = true;
    selectionModel.clearSelection();
    expandedSkillId.value = null;
    return;
  }

  const visibleIds = new Set(displaySkills.value.map((skill) => skill.id));
  const selectableIds = hasActiveFilters.value ? skillIds.filter((skillId) => visibleIds.has(skillId)) : skillIds;
  if (selectableIds.length === 0) return;
  selectedSkills.value = new Set(selectableIds);
  batchRepairContext.value = repairContext;
  if (openBatch) showBatch.value = true;
  else expandedSkillId.value = selectableIds[0];
}
</script>

<template>
  <div class="space-y-4">
    <!-- Header -->
    <section class="workspace-hero">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-semibold" style="color: var(--c-text-strong);">
              {{ t("manage.title") || "软连接管理" }}
            </h2>
            <span class="text-xs px-2 py-0.5 rounded-full" style="background: var(--c-primary-light); color: var(--c-primary);">
              {{ displaySkills.length }}/{{ totalSkills }}
            </span>
          </div>
          <p class="text-xs mt-1" style="color: var(--c-text-secondary);">
            {{ t("manage.workspace_hint") }}
          </p>
        </div>
        <div class="action-toolbar">
          <button
            class="action-toolbar-icon disabled:opacity-50 disabled:cursor-not-allowed"
            :title="t('manage.refresh')"
            :disabled="isRefreshing"
            @click="refreshManageData"
          >
            <RefreshCw :size="15" :class="{ 'animate-spin': isRefreshing }" />
          </button>
          <button
            class="action-toolbar-icon"
            :title="t('manage.agent_management')"
            @click="showAgentManager = true"
          >
            <Settings :size="15" />
          </button>
          <button
            class="action-toolbar-primary"
            @click="showInstall = true"
          >
            <Plus :size="15" />
            {{ t("skills.install") }}
          </button>
        </div>
      </div>

      <div class="grid gap-3 mt-4" style="grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));">
        <div class="rounded-lg px-3 py-2" style="background: var(--c-bg); border: 1px solid var(--c-border-subtle);">
          <div class="text-[10px] uppercase tracking-wide" style="color: var(--c-text-secondary);">{{ t("manage.total_skills") || "共" }}</div>
          <div class="text-lg font-semibold" style="color: var(--c-text-strong);">{{ totalSkills }}</div>
        </div>
        <div class="rounded-lg px-3 py-2" style="background: var(--c-primary-light); border: 1px solid color-mix(in srgb, var(--c-primary) 22%, transparent);">
          <div class="text-[10px] uppercase tracking-wide" style="color: var(--c-primary);">{{ t("manage.linked_count") || "共享" }}</div>
          <div class="text-lg font-semibold" style="color: var(--c-primary);">{{ sharedSkills.length }}</div>
        </div>
        <div class="rounded-lg px-3 py-2" style="background: var(--c-bg); border: 1px solid var(--c-border-subtle);">
          <div class="text-[10px] uppercase tracking-wide" style="color: var(--c-text-secondary);">{{ t("manage.status_unlinked") || "独立" }}</div>
          <div class="text-lg font-semibold" style="color: var(--c-text);">{{ uniqueSkills.length }}</div>
        </div>
        <div class="rounded-lg px-3 py-2" style="background: var(--c-warning-light); border: 1px solid color-mix(in srgb, var(--c-warning) 22%, transparent);">
          <div class="text-[10px] uppercase tracking-wide" style="color: var(--c-warning);">{{ t("manage.conflict_count") || "异常" }}</div>
          <div class="text-lg font-semibold" style="color: var(--c-warning);">{{ issueSkills.length }}</div>
        </div>
      </div>
    </section>

    <!-- Filter control center -->
    <section class="workspace-panel !p-3">
      <div class="manage-filter-panel">
        <div class="manage-filter-toolbar">
          <div class="relative min-w-0">
            <Search :size="14" class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2" style="color: var(--c-text-secondary);" />
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
              :aria-label="t('manage.clear_query') || '清除关键词'"
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
              :aria-label="t('manage.sort_label') || '显示顺序'"
              @click.stop="sortMenuOpen = !sortMenuOpen"
            >
              <span class="truncate">{{ currentSortLabel }}</span>
              <ChevronDown :size="13" class="shrink-0 transition-transform" :style="{ transform: sortMenuOpen ? 'rotate(180deg)' : 'rotate(0deg)' }" />
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
            class="inline-flex items-center justify-center gap-1 rounded-md border px-2.5 py-2 text-[11px] cursor-pointer transition-colors"
            :style="{ borderColor: filtersExpanded || activeFilterCount > 0 ? 'var(--c-primary)' : 'var(--c-border)', color: filtersExpanded || activeFilterCount > 0 ? 'var(--c-primary)' : 'var(--c-text-secondary)' }"
            type="button"
            :aria-expanded="filtersExpanded"
            @click="filtersExpanded = !filtersExpanded"
          >
            <SlidersHorizontal :size="13" />
            {{ t('manage.more_filters') || '更多筛选' }}
            <span v-if="activeFilterCount > 0" class="rounded-full px-1.5 text-[9px]" style="background: var(--c-primary-light);">{{ activeFilterCount }}</span>
          </button>

          <button
            class="manage-filter-clear"
            :class="{ 'manage-filter-clear-active': hasActiveFilters }"
            :disabled="!hasActiveFilters"
            type="button"
            @click="clearAllFilters"
          >
            <X :size="13" />
            {{ t('manage.clear_filters') || '清除筛选' }}
          </button>
        </div>

        <div v-if="hasActiveFilters" class="manage-active-filter-row">
          <span class="manage-active-filter-label">{{ t('manage.filter_active_summary') || '当前筛选' }}</span>
          <button v-if="filterModel.query.value" class="manage-filter-token" type="button" @click="filterModel.clearQuery">
            {{ filterModel.query.value }} <X :size="11" />
          </button>
          <button v-if="filterModel.statusPreset.value !== 'all'" class="manage-filter-token" type="button" @click="setStatusPreset('all')">
            {{ t(`manage.status_preset_${filterModel.statusPreset.value}`) }} <X :size="11" />
          </button>
          <button v-for="issue in activeIssueTokens" :key="issue" class="manage-filter-token" type="button" @click="removeIssue(issue)">
            {{ t(`manage.status_${issue}`) }} <X :size="11" />
          </button>
          <button v-for="scope in activeLibraryTokens" :key="scope" class="manage-filter-token" type="button" @click="removeLibraryScope(scope)">
            {{ t(`manage.${scope === 'missing_library' ? 'quick_filter_missing_lib' : 'quick_filter_only_lib'}`) }} <X :size="11" />
          </button>
          <button v-for="agentId in activeAgentTokens" :key="agentId" class="manage-filter-token" type="button" @click="removeAgent(agentId)">
            {{ agentsStore.agents.find((agent) => agent.id === agentId)?.name || agentId }} <X :size="11" />
          </button>
          <span class="manage-filter-result-count">{{ displaySkills.length }} / {{ totalSkills }}</span>
        </div>

        <div v-if="filtersExpanded" class="manage-filter-expanded">
          <div class="manage-filter-group">
            <span class="manage-filter-group-label">{{ t('manage.filter_group_status') || '状态' }}</span>
            <button v-for="preset in (['all', 'needs_attention', 'linked_any', 'unlinked_all'] as const)" :key="preset" class="filter-chip" :class="{ 'filter-chip-active': filterModel.statusPreset.value === preset }" :disabled="facetCounts.status[preset] === 0 && filterModel.statusPreset.value !== preset" type="button" @click="setStatusPreset(preset)">
              {{ t(`manage.status_preset_${preset}`) }}
              <span class="filter-chip-count">{{ facetCounts.status[preset] }}</span>
            </button>
          </div>
          <div class="manage-filter-group">
            <span class="manage-filter-group-label">{{ t('manage.filter_group_issue') || '问题' }}</span>
            <button v-for="issue in (['conflict', 'dangling', 'duplicate'] as const)" :key="issue" class="filter-chip" :class="{ 'filter-chip-active': filterModel.issues.value.has(issue) }" :disabled="facetCounts.issues[issue] === 0 && !filterModel.issues.value.has(issue)" type="button" @click="toggleIssue(issue)">
              {{ t(`manage.status_${issue}`) }}
              <span class="filter-chip-count">{{ facetCounts.issues[issue] }}</span>
            </button>
          </div>
          <div class="manage-filter-group">
            <span class="manage-filter-group-label">{{ t('manage.filter_group_other') || '来源' }}</span>
            <button v-for="scope in (['missing_library', 'library_only'] as const)" :key="scope" class="filter-chip" :class="{ 'filter-chip-active': filterModel.libraryScope.value.has(scope) }" :disabled="facetCounts.library[scope] === 0 && !filterModel.libraryScope.value.has(scope)" type="button" @click="toggleLibraryScope(scope)">
              {{ t(`manage.${scope === 'missing_library' ? 'quick_filter_missing_lib' : 'quick_filter_only_lib'}`) }}
              <span class="filter-chip-count">{{ facetCounts.library[scope] }}</span>
            </button>
          </div>
          <div class="manage-filter-group" :class="{ 'manage-filter-group-disabled': detectedAgents.length === 0 }">
            <span class="manage-filter-group-label">{{ t('manage.agent_filter_label') || 'Agent 范围' }}</span>
            <span v-if="detectedAgents.length === 0" class="text-[11px]" style="color: var(--c-text-tertiary);">{{ t('manage.no_agent_filter') || '暂无可筛选 Agent' }}</span>
            <template v-else>
              <button v-for="overview in agentOverviews" :key="overview.agent.id" class="filter-chip" :class="{ 'filter-chip-active': filterModel.agentIds.value.has(overview.agent.id) }" type="button" @click="selectAgentFromOverview(overview.agent.id)">
                <span class="h-1.5 w-1.5 rounded-full" style="background: var(--c-success);" />
                {{ overview.agent.name }} <span class="filter-chip-count">{{ overview.skillCount }}</span>
              </button>
              <button v-if="filterModel.agentIds.value.size > 0" class="manage-agent-match-toggle" type="button" @click="filterModel.agentMatch.value = filterModel.agentMatch.value === 'any' ? 'exclude' : 'any'">
                {{ filterModel.agentMatch.value === 'any' ? (t('manage.filter_include') || '任一匹配') : (t('manage.filter_exclude') || '排除') }}
              </button>
            </template>
          </div>
        </div>
      </div>

    </section>

    <section class="workspace-panel !p-3 manage-issue-summary">
      <IssueRepairPanel :skills="skillsStore.skills" :agents="agentsStore.agents" compact @select-group="selectIssueGroup" />
    </section>

    <!-- Loading -->
    <div v-if="skillsStore.loading" class="space-y-3">
      <SkeletonCard v-for="i in 4" :key="i" />
    </div>

    <!-- Error -->
    <div v-else-if="skillsStore.error" class="text-sm" style="color: var(--c-danger);">{{ skillsStore.error }}</div>

    <!-- Empty -->
    <section v-else-if="displaySkills.length === 0" class="manage-empty-state">
      <div class="manage-empty-icon">
        <SearchX :size="28" />
      </div>
      <h3>{{ hasActiveFilters ? (t("manage.no_filter_results") || "未找到匹配的 Skill") : t("skills.no_skills") }}</h3>
      <p>{{ hasActiveFilters ? (t("manage.no_filter_results_hint") || "尝试调整关键词或清除筛选条件") : t("skills.no_skills_hint") }}</p>
      <div class="manage-empty-actions">
        <button v-if="hasActiveFilters" class="manage-empty-secondary" type="button" @click="clearAllFilters">
          <X :size="14" />
          {{ t("manage.clear_filters") || "清除筛选" }}
        </button>
        <button class="manage-empty-primary" type="button" @click="showInstall = true">
          {{ t("skills.install") }}
        </button>
      </div>
    </section>

    <div v-else class="manage-workbench-layout">
      <div ref="workbenchViewport" class="manage-workbench-viewport">
        <SkillWorkbench
          :skills="displaySkills"
          :agents="detectedAgents"
          :selected-ids="selectedSkills"
          :expanded-skill-id="expandedSkillId"
          :all-visible-selected="allDisplayedSelected"
          :partially-visible-selected="someDisplayedSelected"
          @toggle:select="toggleSkillSelect"
          @toggle:all="toggleAllDisplayedSkills"
          @open:detail="(id) => expandedSkillId = id || null"
          @request:add-agent="showAgentManager = true"
        />
      </div>

      <div v-if="selectedSkills.size > 0" class="manage-selection-bar">
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
          {{ t("manage.selected_scope_count", { selected: selectedVisibleCount, total: displaySkills.length }) || `已选 ${selectedVisibleCount} / 当前结果 ${displaySkills.length}` }}
        </span>
        <button class="manage-selection-primary" type="button" @click="openBatchPanel">
          {{ t("manage.batch_panel_open") || "批量操作" }}
        </button>
        <button class="manage-selection-clear" type="button" @click="deselectAllSkills">
          {{ t("manage.deselect_all") || "取消选择" }}
        </button>
      </div>
    </div>
    <Teleport to="body">
      <div
        v-if="showAgentManager"
         class="modal-backdrop fixed inset-0 z-50 flex items-center justify-center p-4"
        @click.self="showAgentManager = false"
      >
        <div
          class="modal-shell flex w-full max-w-3xl flex-col"
        >
          <div class="modal-header shrink-0">
            <div>
              <h3 class="text-sm font-semibold" style="color: var(--c-text);">
                {{ t("manage.agent_management") }}
              </h3>
              <p class="text-[11px] mt-0.5" style="color: var(--c-text-secondary);">
                {{ t("manage.agent_management_hint") }}
              </p>
            </div>
            <div class="flex items-center gap-2">
              <button
                class="inline-flex items-center gap-1 rounded-md px-3 py-1.5 text-xs cursor-pointer"
                style="background: var(--c-primary); color: white;"
                type="button"
                @click="showAddAgent = true"
              >
                <Plus :size="14" />
                {{ t("agents.add") }}
              </button>
              <button
                class="w-7 h-7 inline-flex items-center justify-center rounded-md cursor-pointer"
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

    <AddAgentDialog
      v-if="showAddAgent"
      @close="showAddAgent = false"
      @added="showAddAgent = false"
    />

    <!-- Install dialog -->
    <InstallDialog v-if="showInstall" @close="showInstall = false" />

    <!-- 批量同步矩阵面板 -->
    <BatchSyncPanel
      v-if="showBatch"
      :selected-skill-ids="selectedSkillIds"
      :repair-context="batchRepairContext"
      @close="closeBatchPanel"
      @remove-skill="removeSkillFromSelection"
      @resolve-conflict="resolveConflictFromBatch"
      @applied="onBatchApplied"
    />
  </div>
</template>
