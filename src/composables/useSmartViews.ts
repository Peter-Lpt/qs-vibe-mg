import { computed, type ComputedRef, type Ref } from "vue";
import type { Agent, Skill, SmartViewId, ViewId } from "../types";
import type { DomainScope, LibraryScope, ManageFilterState } from "../components/manage/manageFilters";
import {
  computeFacetCounts,
  filterSkills,
  matchesDomain,
} from "../components/manage/manageFilters";

export type { SmartViewId, ViewId, DomainScope };

export interface SmartViewDef {
  id: SmartViewId;
  domain: DomainScope;
  labelKey: string;
  icon: string;
  showBadge?: boolean;
}

// ── 视图注册表（侧边栏与快捷键的单一真源，替代 App.vue 硬编码 tabs） ──
export const SMART_VIEWS: readonly SmartViewDef[] = [
  { id: "all", domain: "local", labelKey: "sidebar.view_all", icon: "List" },
  { id: "attention", domain: "local", labelKey: "sidebar.view_attention", icon: "TriangleAlert", showBadge: true },
  { id: "linked", domain: "local", labelKey: "sidebar.view_linked", icon: "Link2" },
  { id: "unlinked", domain: "local", labelKey: "sidebar.view_unlinked", icon: "CircleDashed" },
  { id: "plugins", domain: "plugin", labelKey: "sidebar.plugins", icon: "Puzzle" },
];

export interface ViewFilterPreset {
  statusPreset?: ManageFilterState["statusPreset"];
  libraryScope?: Set<LibraryScope>;
  domain: DomainScope;
}

// ── 视图 → 筛选预设（不写状态，纯映射） ──
// query/issues/agentIds/agentMatch/sort 属用户高级条件，跨视图保留，不出现在预设里。
export function viewToFilterPreset(view: SmartViewId): ViewFilterPreset {
  switch (view) {
    case "attention":
      return { domain: "local", statusPreset: "needs_attention" };
    case "linked":
      return { domain: "local", statusPreset: "linked_any" };
    case "unlinked":
      return { domain: "local", statusPreset: "unlinked_all" };
    case "plugins":
      return { domain: "plugin", statusPreset: "all" };
    case "all":
    default:
      return { domain: "local", statusPreset: "all" };
  }
}

// ── 视图计数（domain 前置 + facet 合同） ──
export function useSmartViews(
  skills: Ref<Skill[]> | ComputedRef<Skill[]>,
  agents: Ref<Agent[]> | ComputedRef<Agent[]>,
  state: ComputedRef<ManageFilterState>
) {
  const viewCounts = computed<Record<SmartViewId, number>>(() => {
    // 计数语义 = "用户若切到该视图会看到什么"：
    // 视图持有字段（statusPreset/libraryScope/domain）中和到基线，
    // 用户高级条件（query/issues/agentIds/agentMatch）保留参与计数。
    const baseState: ManageFilterState = {
      ...state.value,
      statusPreset: "all",
      libraryScope: new Set(),
    };

    const localScoped = skills.value.filter((s) => matchesDomain(s, "local"));
    const pluginScoped = skills.value.filter((s) => matchesDomain(s, "plugin"));

    const facets = computeFacetCounts(localScoped, { ...baseState, domain: "local" }, agents.value);
    const pluginCount = filterSkills(pluginScoped, { ...baseState, domain: "plugin" }, agents.value).length;

    return {
      all: facets.status.all,
      attention: facets.status.needs_attention,
      linked: facets.status.linked_any,
      unlinked: facets.status.unlinked_all,
      plugins: pluginCount,
    };
  });

  const attentionCount = computed(() => viewCounts.value.attention);

  return {
    views: SMART_VIEWS,
    viewCounts,
    attentionCount,
  };
}
