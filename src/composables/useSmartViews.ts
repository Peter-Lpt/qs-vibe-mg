import { computed, type ComputedRef, type Ref } from "vue";
import type { Agent, Skill, SmartViewId, ViewId } from "../types";
import type { LibraryScope, ManageFilterState } from "../components/manage/manageFilters";
import { computeFacetCounts } from "../components/manage/manageFilters";

export type { SmartViewId, ViewId };

export interface SmartViewDef {
  id: SmartViewId;
  labelKey: string;
  icon: string;
  showBadge?: boolean;
}

// ── 视图注册表（侧边栏与快捷键的单一真源，替代 App.vue 硬编码 tabs） ──
// Plugin skills 现在由独立的 PluginSkillsView 处理
export const SMART_VIEWS: readonly SmartViewDef[] = [
  { id: "all", labelKey: "sidebar.view_all", icon: "List" },
  { id: "attention", labelKey: "sidebar.view_attention", icon: "TriangleAlert", showBadge: true },
  { id: "linked", labelKey: "sidebar.view_linked", icon: "Link2" },
  { id: "unlinked", labelKey: "sidebar.view_unlinked", icon: "CircleDashed" },
  { id: "plugins", labelKey: "sidebar.plugins", icon: "Puzzle" },
];

export interface ViewFilterPreset {
  statusPreset?: ManageFilterState["statusPreset"];
  libraryScope?: Set<LibraryScope>;
}

// ── 视图 → 筛选预设（不写状态，纯映射） ──
// query/issues/agentIds/agentMatch/sort 属用户高级条件，跨视图保留，不出现在预设里。
export function viewToFilterPreset(view: SmartViewId): ViewFilterPreset {
  switch (view) {
    case "attention":
      return { statusPreset: "needs_attention" };
    case "linked":
      return { statusPreset: "linked_any" };
    case "unlinked":
      return { statusPreset: "unlinked_all" };
    case "plugins":
      // Plugin view 由独立组件处理，这里返回默认预设
      return { statusPreset: "all" };
    case "all":
    default:
      return { statusPreset: "all" };
  }
}

// ── 视图计数（facet 合同） ──
export function useSmartViews(
  skills: Ref<Skill[]> | ComputedRef<Skill[]>,
  agents: Ref<Agent[]> | ComputedRef<Agent[]>,
  state: ComputedRef<ManageFilterState>
) {
  const viewCounts = computed<Record<SmartViewId, number>>(() => {
    // 计数语义 = "用户若切到该视图会看到什么"：
    // 视图持有字段（statusPreset/libraryScope）中和到基线，
    // 用户高级条件（query/issues/agentIds/agentMatch）保留参与计数。
    const baseState: ManageFilterState = {
      ...state.value,
      statusPreset: "all",
      libraryScope: new Set(),
    };

    const facets = computeFacetCounts(skills.value, baseState, agents.value);

    return {
      all: facets.status.all,
      attention: facets.status.needs_attention,
      linked: facets.status.linked_any,
      unlinked: facets.status.unlinked_all,
      plugins: 0, // Plugin skills 由独立 API 处理，这里返回 0
    };
  });

  const attentionCount = computed(() => viewCounts.value.attention);

  return {
    views: SMART_VIEWS,
    viewCounts,
    attentionCount,
  };
}
