<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import SkillRow from "./SkillRow.vue";
import SkillTree from "./SkillTree.vue";
import AgentMatrix from "./AgentMatrix.vue";
import BatchSyncPanel from "./BatchSyncPanel.vue";
import IssueRepairPanel from "./IssueRepairPanel.vue";
import InstallDialog from "../skills/InstallDialog.vue";
import EmptyState from "../common/EmptyState.vue";
import SkeletonCard from "../common/SkeletonCard.vue";
import type { Agent } from "../../types";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const isRefreshing = ref(false);

// ── 初始化 ──────────────────────────────────────
onMounted(async () => {
  if (skillsStore.skills.length === 0) await skillsStore.fetchSkills();
  if (agentsStore.agents.length === 0) await agentsStore.fetchAgents();
  if (skillsStore.issues.length === 0) await skillsStore.fetchIssues();
});

// ── 视图模式 ──────────────────────────────────────
type ViewMode = "list" | "tree";
const viewMode = ref<ViewMode>(
  (localStorage.getItem("vibe-manage-view") as ViewMode) || "list"
);
function setViewMode(mode: ViewMode) {
  viewMode.value = mode;
  localStorage.setItem("vibe-manage-view", mode);
}

// ── 筛选状态 ──────────────────────────────────────
type StatusFilter = "conflict" | "dangling" | "independent" | "unlinked" | "linked" | "missing_lib" | "only_lib" | "duplicate";
const activeStatusFilters = ref<Set<StatusFilter>>(new Set());
const selectedAgentFilter = ref<Set<string>>(new Set());
const agentFilterMode = ref<"include" | "exclude">("include");
const sortBy = ref<"status" | "name" | "sources">("status");
const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

// ── Skill 多选 ──────────────────────────────────────
const selectedSkills = ref<Set<string>>(new Set());

function toggleSkillSelect(skillId: string) {
  const newSet = new Set(selectedSkills.value);
  if (newSet.has(skillId)) newSet.delete(skillId);
  else newSet.add(skillId);
  selectedSkills.value = newSet;
}

function selectAllSkills() {
  selectedSkills.value = new Set(displaySkills.value.map((s) => s.id));
}

function deselectAllSkills() {
  selectedSkills.value = new Set();
}

// ── 折叠状态 ──────────────────────────────────────
const agentOverviewExpanded = ref(true);
const matrixExpanded = ref(false);
const expandedSkillId = ref<string | null>(null);

// ── 安装弹窗 ──────────────────────────────────────
const showInstall = ref(false);

// ── 快捷键 ──────────────────────────────────────
function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "f")) {
    e.preventDefault();
    searchInput.value?.focus();
  }
}
onMounted(() => document.addEventListener("keydown", handleKeydown));
onUnmounted(() => document.removeEventListener("keydown", handleKeydown));

// ── Agent 概览（= Agent 筛选） ──────────────────────
const detectedAgents = computed(() => agentsStore.agents.filter((a) => a.detected));

interface AgentOverview {
  agent: Agent;
  skillCount: number;
  linkedCount: number;
  conflictCount: number;
}

const agentOverviews = computed<AgentOverview[]>(() =>
  detectedAgents.value.map((agent) => ({
    agent,
    skillCount: skillsStore.skills.filter((s) =>
      s.sources.some((src) => src.from === agent.id)
    ).length,
    linkedCount: skillsStore.skills.filter((s) =>
      s.sources.some((src) => src.from === agent.id && src.is_symlink)
    ).length,
    conflictCount: skillsStore.skills.filter(
      (s) => s.has_conflict && s.sources.some((src) => src.from === agent.id)
    ).length,
  }))
);

function selectAgentFromOverview(agentId: string) {
  const set = new Set(selectedAgentFilter.value);
  if (set.has(agentId)) set.delete(agentId);
  else set.add(agentId);
  selectedAgentFilter.value = set;
}

// ── 状态 chips 定义 ──────────────────────────────
const hasAnyDuplicate = computed(() => skillsStore.skills.some((s) => s.is_duplicate));

interface StatusChipDef {
  id: StatusFilter;
  labelKey: string;
  color: string;
  group: "issue" | "status" | "other";
  icon: string;
}

const statusChipDefs: StatusChipDef[] = [
  { id: "conflict", labelKey: "manage.status_conflict", color: "var(--c-warning)", group: "issue", icon: "TriangleAlert" },
  { id: "dangling", labelKey: "manage.status_dangling", color: "var(--c-danger)", group: "issue", icon: "XCircle" },
  { id: "independent", labelKey: "manage.status_independent", color: "var(--c-primary)", group: "status", icon: "Circle" },
  { id: "unlinked", labelKey: "manage.status_unlinked", color: "var(--c-text-secondary)", group: "status", icon: "CircleDashed" },
  { id: "linked", labelKey: "manage.status_linked", color: "var(--c-primary)", group: "status", icon: "Circle" },
  { id: "missing_lib", labelKey: "manage.quick_filter_missing_lib", color: "var(--c-text-secondary)", group: "other", icon: "CircleDashed" },
  { id: "only_lib", labelKey: "manage.quick_filter_only_lib", color: "var(--c-text-secondary)", group: "other", icon: "CircleDashed" },
  { id: "duplicate", labelKey: "manage.status_duplicate", color: "var(--c-info)", group: "other", icon: "Copy" },
];

function toggleStatusFilter(id: StatusFilter) {
  const set = new Set(activeStatusFilters.value);
  if (set.has(id)) set.delete(id);
  else set.add(id);
  activeStatusFilters.value = set;
}

// ── Chip 计数（基于 displaySkills 的筛选子集） ──────
const chipCounts = computed(() => {
  // 先应用除当前 chip 类型以外的所有筛选，得到"基础列表"
  let base = searchQuery.value.trim() ? skillsStore.searchResults : skillsStore.skills;

  // Agent 筛选
  if (selectedAgentFilter.value.size > 0) {
    base = base.filter((s) => {
      const hasAny = [...selectedAgentFilter.value].some((agentId) =>
        s.sources.some((src) => src.from === agentId)
      );
      return agentFilterMode.value === "include" ? hasAny : !hasAny;
    });
  }

  // 其他 status 筛选（排除当前 chip 自身）
  const otherFilters = new Set(activeStatusFilters.value);
  if (otherFilters.size > 0) {
    base = base.filter((s) => {
      for (const filter of otherFilters) {
        if (!matchesFilter(s, filter)) return false;
      }
      return true;
    });
  }

  return {
    conflict: base.filter((s) => s.has_conflict).length,
    dangling: base.filter((s) => s.has_dangling).length,
    independent: base.filter((s) => s.sources.some((src) => !src.is_symlink && src.from !== "vibe-lib")).length,
    unlinked: base.filter((s) => !s.sources.some((src) => src.from !== "vibe-lib" && src.is_symlink)).length,
    linked: base.filter((s) => s.sources.some((src) => src.from !== "vibe-lib" && src.is_symlink)).length,
    missing_lib: base.filter((s) => !s.sources.some((src) => src.from === "vibe-lib")).length,
    only_lib: base.filter((s) => s.sources.filter((src) => src.from !== "vibe-lib").length === 0).length,
    duplicate: base.filter((s) => s.is_duplicate).length,
  };
});

function matchesFilter(s: { has_conflict: boolean; has_dangling: boolean; is_duplicate: boolean; sources: { from: string; is_symlink: boolean }[] }, filter: StatusFilter): boolean {
  switch (filter) {
    case "conflict": return s.has_conflict;
    case "dangling": return s.has_dangling;
    case "independent": return s.sources.some((src) => !src.is_symlink && src.from !== "vibe-lib");
    case "unlinked": return !s.sources.some((src) => src.from !== "vibe-lib" && src.is_symlink);
    case "linked": return s.sources.some((src) => src.from !== "vibe-lib" && src.is_symlink);
    case "missing_lib": return !s.sources.some((src) => src.from === "vibe-lib");
    case "only_lib": return s.sources.filter((src) => src.from !== "vibe-lib").length === 0;
    case "duplicate": return s.is_duplicate;
  }
}

// ── 排序选项 ──────────────────────────────────────
const sortOptions = computed(() => [
  { value: "status", label: t("manage.sort_by_status_priority") || "需操作优先" },
  { value: "name", label: t("manage.sort_by_name") || "名称" },
  { value: "sources", label: t("manage.sort_by_sources") || "来源数" },
]);

// ── 清除筛选 ──────────────────────────────────────
function clearAllFilters() {
  activeStatusFilters.value = new Set();
  selectedAgentFilter.value = new Set();
  agentFilterMode.value = "include";
  searchQuery.value = "";
}

async function refreshManageData() {
  if (isRefreshing.value) return;
  isRefreshing.value = true;
  try {
    await Promise.all([
      skillsStore.fetchSkills(),
      agentsStore.fetchAgents(),
      skillsStore.fetchIssues(),
    ]);
    if (
      expandedSkillId.value &&
      !skillsStore.skills.some((skill) => skill.id === expandedSkillId.value)
    ) {
      expandedSkillId.value = null;
    }
    const currentIds = new Set(skillsStore.skills.map((skill) => skill.id));
    selectedSkills.value = new Set(
      [...selectedSkills.value].filter((skillId) => currentIds.has(skillId))
    );
  } finally {
    isRefreshing.value = false;
  }
}

const hasActiveFilters = computed(
  () =>
    activeStatusFilters.value.size > 0 ||
    selectedAgentFilter.value.size > 0 ||
    searchQuery.value.trim() !== ""
);

// ── Stats bar ──────────────────────────────────────
const totalSkills = computed(() => skillsStore.skills.length);
const sharedSkills = computed(() =>
  skillsStore.skills.filter((s) => s.sources.filter((src) => src.from !== "vibe-lib").length > 1)
);
const uniqueSkills = computed(() =>
  skillsStore.skills.filter((s) => s.sources.filter((src) => src.from !== "vibe-lib").length === 1)
);
const issueSkills = computed(() =>
  skillsStore.skills.filter((s) => s.has_conflict || s.has_dangling)
);

// ── 核心筛选逻辑 ──────────────────────────────────
const displaySkills = computed(() => {
  let list = searchQuery.value.trim() ? skillsStore.searchResults : skillsStore.skills;

  // 状态筛选
  if (activeStatusFilters.value.size > 0) {
    list = list.filter((s) => {
      for (const filter of activeStatusFilters.value) {
        if (!matchesFilter(s, filter)) return false;
      }
      return true;
    });
  }

  // Agent 筛选
  if (selectedAgentFilter.value.size > 0) {
    list = list.filter((s) => {
      const hasAny = [...selectedAgentFilter.value].some((agentId) =>
        s.sources.some((src) => src.from === agentId)
      );
      return agentFilterMode.value === "include" ? hasAny : !hasAny;
    });
  }

  // 排序
  const sorted = [...list];
  sorted.sort((a, b) => {
    if (sortBy.value === "status") {
      const priority = (s: typeof a): number => {
        if (s.has_conflict) return 0;
        if (s.has_dangling) return 1;
        if (s.sources.some((src) => !src.is_symlink && src.from !== "vibe-lib")) return 2;
        if (!s.sources.some((src) => src.from !== "vibe-lib" && src.is_symlink)) return 3;
        return 4;
      };
      return priority(a) - priority(b);
    }
    if (sortBy.value === "name") return (a.name || a.id).localeCompare(b.name || b.id);
    if (sortBy.value === "sources") return b.sources.length - a.sources.length;
    return 0;
  });

  return sorted;
});

watch(searchQuery, (val) => {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(() => skillsStore.searchSkills(val), 300);
});

// ── 批量操作面板（多 skill 同步交互，详见 docs/multi-skill-sync-interaction.v3.md） ──
const showBatch = ref(false);
const selectedSkillIds = computed(() => [...selectedSkills.value]);
const batchRepairContext = ref<string | null>(null);

function openBatchPanel() {
  if (selectedSkills.value.size === 0) return;
  batchRepairContext.value = null;
  showBatch.value = true;
}

function removeSkillFromSelection(skillId: string) {
  const s = new Set(selectedSkills.value);
  s.delete(skillId);
  selectedSkills.value = s;
}

function closeBatchPanel() {
  showBatch.value = false;
  batchRepairContext.value = null;
}

function resolveConflictFromBatch(skillId: string) {
  showBatch.value = false;
  batchRepairContext.value = "conflict";
  viewMode.value = "list";
  expandedSkillId.value = skillId;
  setTimeout(() => {
    document.getElementById(`skill-${skillId}`)?.scrollIntoView({ behavior: "smooth", block: "center" });
  }, 100);
}

function onBatchApplied() {
  // 面板内部已统一 refreshSkills + fetchAgents；此处保留选择，便于继续操作
}

function selectIssueGroup(skillIds: string[], openBatch: boolean, repairContext: string) {
  if (skillIds.length === 0) return;
  viewMode.value = "list";
  if (openBatch) {
    selectedSkills.value = new Set(skillIds);
    batchRepairContext.value = repairContext;
    showBatch.value = true;
    return;
  }

  selectedSkills.value = new Set();
  batchRepairContext.value = repairContext;
  showBatch.value = false;
  expandedSkillId.value = skillIds[0];
  setTimeout(() => {
    document.getElementById(`skill-${skillIds[0]}`)?.scrollIntoView({ behavior: "smooth", block: "center" });
  }, 100);
}

// ── 矩阵操作 ──────────────────────────────────────
function handleMatrixExpand(skillId: string) {
  expandedSkillId.value = skillId;
  setTimeout(() => {
    document.getElementById(`skill-${skillId}`)?.scrollIntoView({ behavior: "smooth", block: "center" });
  }, 100);
}

// ── 状态 chips 分组 ──────────────────────────────
const chipGroups = computed(() => {
  const groups: { label: string; chips: StatusChipDef[] }[] = [];
  const issue = statusChipDefs.filter((c) => c.group === "issue");
  const status = statusChipDefs.filter((c) => c.group === "status");
  const other = statusChipDefs.filter((c) => c.group === "other");
  if (issue.length) groups.push({ label: t("manage.filter_group_issue") || "异常", chips: issue });
  if (status.length) groups.push({ label: t("manage.filter_group_status") || "状态", chips: status });
  if (other.length) groups.push({ label: t("manage.filter_group_other") || "其他", chips: other });
  return groups;
});
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
            v-if="hasActiveFilters"
            class="action-toolbar-text"
            @click="clearAllFilters"
          >
            {{ t("manage.clear_filters") || "清除筛选" }}
          </button>
          <button
            class="action-toolbar-icon disabled:opacity-50 disabled:cursor-not-allowed"
            :title="t('manage.refresh')"
            :disabled="isRefreshing"
            @click="refreshManageData"
          >
            <RefreshCw :size="15" :class="{ 'animate-spin': isRefreshing }" />
          </button>
          <button
            class="action-toolbar-primary"
            @click="showInstall = true"
          >
            <Plus :size="15" />
            {{ t("skills.install") }}
          </button>
          <div class="action-toolbar-segment">
            <button
              class="action-toolbar-segment-button"
              :style="{ background: viewMode === 'list' ? 'var(--c-primary)' : 'transparent', color: viewMode === 'list' ? 'white' : 'var(--c-text-secondary)' }"
              :title="t('manage.view_list') || '列表视图'"
              @click="setViewMode('list')"
            >
              <List :size="15" />
            </button>
            <button
              class="action-toolbar-segment-button"
              :style="{ background: viewMode === 'tree' ? 'var(--c-primary)' : 'transparent', color: viewMode === 'tree' ? 'white' : 'var(--c-text-secondary)' }"
              :title="t('manage.view_tree') || '树视图'"
              @click="setViewMode('tree')"
            >
              <ListTree :size="15" />
            </button>
          </div>
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

    <!-- Agent 概览（= Agent 筛选入口） -->
    <section class="workspace-panel" v-if="detectedAgents.length > 0">
      <div
        class="flex items-center gap-2 cursor-pointer select-none"
        @click="agentOverviewExpanded = !agentOverviewExpanded"
      >
        <ChevronRight class="text-xs transition-transform" :size="14" :style="{ transform: agentOverviewExpanded ? 'rotate(90deg)' : 'rotate(0deg)', color: 'var(--c-text-secondary)' }" />
        <span class="text-xs font-semibold" style="color: var(--c-text-strong);">
          {{ t("manage.agent_overview") || "Agent 概览" }}
        </span>
        <span v-if="selectedAgentFilter.size > 0" class="text-[10px] ml-1" style="color: var(--c-primary);">
          {{ selectedAgentFilter.size }} {{ t("manage.agent_selected") || "个已选" }}
        </span>
      </div>
      <div v-if="agentOverviewExpanded" class="grid gap-2 mt-3" style="grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));">
        <div
          v-for="overview in agentOverviews"
          :key="overview.agent.id"
          class="agent-overview-card p-3 cursor-pointer transition-all hover:shadow-sm"
          :style="{
            background: selectedAgentFilter.has(overview.agent.id)
              ? (agentFilterMode === 'include' ? 'var(--c-primary-light)' : 'var(--c-danger-light)')
              : 'var(--c-surface)',
            borderColor: selectedAgentFilter.has(overview.agent.id)
              ? (agentFilterMode === 'include' ? 'var(--c-primary)' : 'var(--c-danger)')
              : 'var(--c-border)',
          }"
          @click="selectAgentFromOverview(overview.agent.id)"
        >
          <div class="flex items-center gap-1.5 mb-1">
            <span class="w-1.5 h-1.5 rounded-full shrink-0" :style="{ background: overview.agent.detected ? 'var(--c-success)' : '#94a3b8' }" />
            <span class="text-xs font-medium truncate" style="color: var(--c-text);">{{ overview.agent.name }}</span>
          </div>
          <div class="flex items-center gap-2 text-[10px]">
            <span style="color: var(--c-text-secondary);">{{ overview.skillCount }}</span>
            <span class="inline-flex items-center gap-0.5" style="color: var(--c-primary);"><Check :size="12" /> {{ overview.linkedCount }}</span>
            <span v-if="overview.conflictCount > 0" class="inline-flex items-center gap-0.5" style="color: var(--c-warning);"><TriangleAlert :size="12" /> {{ overview.conflictCount }}</span>
          </div>
        </div>
      </div>
      <!-- include/exclude 切换 -->
      <div v-if="selectedAgentFilter.size > 0" class="flex items-center gap-2 mt-1.5">
        <span class="text-[10px]" style="color: var(--c-text-secondary);">
          {{ t("manage.agent_selected") || "已选" }} {{ selectedAgentFilter.size }}
        </span>
        <button
          class="text-[10px] px-1.5 py-0.5 rounded cursor-pointer"
          style="border: 1px solid var(--c-border); background: var(--c-bg);"
          :style="{ color: agentFilterMode === 'include' ? 'var(--c-primary)' : 'var(--c-danger)' }"
          @click="agentFilterMode = agentFilterMode === 'include' ? 'exclude' : 'include'"
        >
          {{ agentFilterMode === "include" ? t("manage.filter_include") : t("manage.filter_exclude") }}
        </button>
      </div>
    </section>

    <!-- 状态筛选 chips（分组 + 计数） -->
    <section class="workspace-panel">
      <div v-for="(group, gIdx) in chipGroups" :key="group.label" class="flex gap-1.5 mb-1.5 flex-wrap items-center">
        <span v-if="gIdx > 0" class="w-px h-3 mx-1" style="background: var(--c-border);" />
        <span class="text-[9px] mr-0.5 shrink-0 uppercase tracking-wider" style="color: var(--c-text-secondary);">
          {{ group.label }}
        </span>
        <button
          v-for="chip in group.chips"
          :key="chip.id"
          class="filter-chip inline-flex items-center gap-1 text-[10px] px-2.5 py-1 rounded-full cursor-pointer transition-colors"
          :style="
            (chip.id === 'duplicate' && !hasAnyDuplicate) || chipCounts[chip.id] === 0
              ? 'opacity: 0.35; pointer-events: none;'
              : activeStatusFilters.has(chip.id)
                ? `background: ${chip.color}; color: white;`
                : ''
          "
          @click="toggleStatusFilter(chip.id)"
        >
          <component :is="chip.icon" :size="12" />
          {{ t(chip.labelKey) }}
          <span class="text-[9px] opacity-70">({{ chipCounts[chip.id] }})</span>
        </button>
      </div>

    <!-- 工具行：排序 + 搜索 -->
    <div class="flex gap-2 mt-3 items-center">
      <select
        v-model="sortBy"
        class="toolbar-control appearance-none px-3 py-2 pr-6 text-[11px] rounded-md outline-none cursor-pointer"
        style="min-width: 116px;"
      >
        <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
      </select>
      <input
        ref="searchInput"
        v-model="searchQuery"
        :placeholder="t('skills.search') + ' (Ctrl+K)'"
        class="toolbar-control flex-1 px-3 py-2 text-xs rounded-md outline-none transition-colors min-w-[140px]"
      />
    </div>
    </section>

    <IssueRepairPanel
      :skills="skillsStore.skills"
      :agents="agentsStore.agents"
      @select-group="selectIssueGroup"
    />

    <!-- Loading -->
    <div v-if="skillsStore.loading" class="space-y-3">
      <SkeletonCard v-for="i in 4" :key="i" />
    </div>

    <!-- Error -->
    <div v-else-if="skillsStore.error" class="text-sm" style="color: var(--c-danger);">{{ skillsStore.error }}</div>

    <!-- Empty -->
    <EmptyState
      v-else-if="displaySkills.length === 0"
      icon="Package"
      :title="t('skills.no_skills')"
      :description="t('skills.no_skills_hint')"
      :action-label="t('skills.install')"
      @action="showInstall = true"
    />

    <!-- Skill list (list mode) -->
    <div
      v-else-if="viewMode === 'list'"
      class="space-y-2"
      :style="{ paddingBottom: selectedSkills.size > 0 ? '56px' : '0' }"
    >
      <SkillRow
        v-for="skill in displaySkills"
        :key="skill.id"
        :id="`skill-${skill.id}`"
        :skill="skill"
        :agents="agentsStore.agents"
        :expanded="expandedSkillId === skill.id"
        :selected="selectedSkills.has(skill.id)"
        @update:expanded="(v) => expandedSkillId = v ? skill.id : null"
        @toggle:select="toggleSkillSelect"
      />
    </div>

    <!-- Skill grid (card mode) -->
    <!-- Skill tree (tree mode) -->
    <div
      v-else
      :style="{ paddingBottom: selectedSkills.size > 0 ? '56px' : '0' }"
    >
      <SkillTree
        :skills="displaySkills"
        :agents="agentsStore.agents"
        :selected-ids="selectedSkills"
        :expanded-skill-id="expandedSkillId"
        @toggle:select="toggleSkillSelect"
        @open:detail="(id) => expandedSkillId = id"
      />
    </div>

    <!-- 浮动批量操作栏 -->
    <Transition
      enter-active-class="transition duration-200 ease-out"
      leave-active-class="transition duration-200 ease-in"
      enter-from-class="translate-y-full opacity-0"
      enter-to-class="translate-y-0 opacity-100"
      leave-from-class="translate-y-0 opacity-100"
      leave-to-class="translate-y-full opacity-0"
    >
      <div
        v-if="selectedSkills.size > 0"
        class="fixed bottom-4 left-1/2 -translate-x-1/2 z-40 flex items-center gap-3 px-4 py-2.5 rounded-lg shadow-lg"
        style="background: var(--c-surface); border: 1px solid var(--c-border);"
      >
        <input
          type="checkbox"
          :checked="selectedSkills.size === displaySkills.length && displaySkills.length > 0"
          class="w-3.5 h-3.5 rounded cursor-pointer"
          style="accent-color: var(--c-primary);"
          @change="selectedSkills.size === displaySkills.length ? deselectAllSkills() : selectAllSkills()"
        />
        <span class="text-xs" style="color: var(--c-text);">
          {{ t("manage.selected_count", { count: selectedSkills.size }) }}
        </span>
        <button
          class="text-[11px] px-3 py-1.5 rounded-md cursor-pointer transition-colors"
          style="background: var(--c-primary); color: white;"
          @click="openBatchPanel"
        >
          {{ t("manage.batch_panel_open") || "批量操作" }}
        </button>
        <button
          class="text-[11px] px-2 py-1 rounded cursor-pointer"
          style="color: var(--c-text-secondary);"
          @click="deselectAllSkills"
        >
          {{ t("manage.deselect_all") || "取消" }}
        </button>
      </div>
    </Transition>

    <!-- 关系矩阵 -->
    <div class="mt-4" v-if="detectedAgents.length > 0">
      <div class="flex items-center gap-2 cursor-pointer mb-2 select-none" @click="matrixExpanded = !matrixExpanded">
        <ChevronRight class="text-xs transition-transform" :size="14" :style="{ transform: matrixExpanded ? 'rotate(90deg)' : 'rotate(0deg)', color: 'var(--c-text-secondary)' }" />
        <span class="text-xs font-semibold" style="color: var(--c-text);">{{ t("manage.agent_matrix") }}</span>
      </div>
      <AgentMatrix
        v-if="matrixExpanded"
        :skills="skillsStore.skills"
        :agents="agentsStore.agents"
        :expanded-skill-id="expandedSkillId"
        @expand-skill="handleMatrixExpand"
      />
    </div>

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
