import { computed, ref, type ComputedRef, type Ref, watch } from "vue";
import type { Agent, Skill, SkillSource } from "../../types";

export type StatusPreset = "all" | "needs_attention" | "linked_any" | "unlinked_all";
export type IssueFilter = "conflict" | "dangling" | "duplicate";
export type LibraryScope = "missing_library" | "library_only";
export type AgentMatch = "any" | "exclude";
export type SortMode = "status" | "updated" | "name" | "linked_agents";

export interface ManageFilterState {
  query: string;
  statusPreset: StatusPreset;
  issues: Set<IssueFilter>;
  libraryScope: Set<LibraryScope>;
  agentIds: Set<string>;
  agentMatch: AgentMatch;
  sort: SortMode;
}

export interface SourceClassification {
  hasLibrary: boolean;
  hasAgent: boolean;
  hasProject: boolean;
  hasExternal: boolean;
  hasMarketplace: boolean;
  hasAgentSymlink: boolean;
  hasIndependentAgentCopy: boolean;
  hasLinkedElsewhere: boolean;
}

// Plugin 类型到 Agent ID 的映射
export const PLUGIN_AGENT_MAP: Record<string, string> = {
  "claude-plugin": "claude-code",
  "codex-plugin": "codex",
};

function sourceKind(source: SkillSource, agentIds: ReadonlySet<string>): "library" | "agent" | "project" | "external" | "marketplace" {
  if (source.source_kind === "marketplace") return "marketplace";
  if (source.source_kind) return source.source_kind;
  if (source.from === "vibe-lib") return "library";
  if (source.from.startsWith("project:")) return "project";
  if (source.from.startsWith("claude-plugin:") || source.from.startsWith("codex-plugin:")) return "marketplace";
  if (agentIds.has(source.from)) return "agent";
  return "external";
}

// 获取 plugin 来源对应的 agent ID
export function getPluginAgentId(source: SkillSource): string | null {
  for (const [prefix, agentId] of Object.entries(PLUGIN_AGENT_MAP)) {
    if (source.from.startsWith(prefix + ":")) {
      return agentId;
    }
  }
  return null;
}

// 检查 skill 是否属于指定的 agent（包括 plugin 来源）
export function skillBelongsToAgent(skill: Skill, agentId: string): boolean {
  return skill.sources.some((source) => {
    if (source.from === agentId) return true;
    const pluginAgentId = getPluginAgentId(source);
    return pluginAgentId === agentId;
  });
}

function normalizePath(path: string): string {
  return path.replace(/\\/g, "/").replace(/\/+$/, "").toLocaleLowerCase();
}

export function classifySkillSources(skill: Skill, agents: readonly Agent[] = []): SourceClassification {
  const agentIds = new Set(agents.map((agent) => agent.id));
  const classification = {
    hasLibrary: false,
    hasAgent: false,
    hasProject: false,
    hasExternal: false,
    hasMarketplace: false,
    hasAgentSymlink: false,
    hasIndependentAgentCopy: false,
    hasLinkedElsewhere: false,
  };

  for (const source of skill.sources) {
    const kind = sourceKind(source, agentIds);
    if (kind === "library") classification.hasLibrary = true;
    if (kind === "agent") {
      classification.hasAgent = true;
      if (source.is_symlink) classification.hasAgentSymlink = true;
      else classification.hasIndependentAgentCopy = true;
    }
    if (kind === "project") classification.hasProject = true;
    if (kind === "external") classification.hasExternal = true;
    if (kind === "marketplace") classification.hasMarketplace = true;
  }

  const librarySource = skill.sources.find((source) => sourceKind(source, agentIds) === "library");
  classification.hasLinkedElsewhere = !!librarySource?.path && skill.sources.some((source) => {
    if (sourceKind(source, agentIds) !== "agent" || !source.is_symlink || !source.symlink_target) return false;
    return normalizePath(source.symlink_target) !== normalizePath(librarySource.path);
  });

  return classification;
}

export function matchesStatusPreset(
  skill: Skill,
  preset: StatusPreset,
  agents: readonly Agent[] = []
): boolean {
  if (preset === "all") return true;
  // Plugin 来源的 skill 不参与状态筛选（只在 "all" 时显示）
  if (skill.from_plugin) return false;
  const sources = classifySkillSources(skill, agents);
  if (preset === "needs_attention") {
    return skill.has_conflict || skill.has_dangling || sources.hasLinkedElsewhere;
  }
  if (preset === "linked_any") return sources.hasAgentSymlink || sources.hasMarketplace;
  return !sources.hasAgentSymlink && !sources.hasMarketplace;
}

export function matchesIssues(skill: Skill, issues: ReadonlySet<IssueFilter>): boolean {
  if (issues.size === 0) return true;
  return (
    (issues.has("conflict") && skill.has_conflict) ||
    (issues.has("dangling") && skill.has_dangling) ||
    (issues.has("duplicate") && skill.is_duplicate)
  );
}

export function matchesLibraryScope(
  skill: Skill,
  scopes: ReadonlySet<LibraryScope>,
  agents: readonly Agent[] = []
): boolean {
  if (scopes.size === 0) return true;
  const sourceInfo = classifySkillSources(skill, agents);
  return (
    (scopes.has("missing_library") && !sourceInfo.hasLibrary) ||
    (scopes.has("library_only") && sourceInfo.hasLibrary && !sourceInfo.hasAgent && !sourceInfo.hasProject && !sourceInfo.hasExternal && !sourceInfo.hasMarketplace)
  );
}

export function matchesAgentScope(
  skill: Skill,
  agentIds: ReadonlySet<string>,
  mode: AgentMatch
): boolean {
  if (agentIds.size === 0) return true;
  const matched = [...agentIds].some((agentId) =>
    skill.sources.some((source) => {
      // 直接匹配 agent ID
      if (source.from === agentId) return true;
      // 匹配 agent 类型的 source
      if (source.source_kind === "agent" && source.from === agentId) return true;
      // 匹配 plugin 来源的 skill（映射到对应的 agent）
      const pluginAgentId = getPluginAgentId(source);
      if (pluginAgentId && pluginAgentId === agentId) return true;
      return false;
    })
  );
  return mode === "exclude" ? !matched : matched;
}

function normalizedText(value: string | undefined): string {
  return (value || "").toLocaleLowerCase();
}

export function matchesQuery(skill: Skill, query: string): boolean {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  if (!normalizedQuery) return true;
  return [skill.id, skill.name, skill.description].some((value) => normalizedText(value).includes(normalizedQuery));
}

function statusPriority(skill: Skill, agents: readonly Agent[]): number {
  if (skill.has_conflict) return 0;
  if (skill.has_dangling) return 1;
  if (classifySkillSources(skill, agents).hasLinkedElsewhere) return 2;
  if (!matchesStatusPreset(skill, "linked_any", agents)) return 3;
  return 4;
}

function skillName(skill: Skill): string {
  return skill.name || skill.id;
}

function modifiedTimestamp(skill: Skill): number {
  const timestamp = Date.parse(skill.modified_at || "");
  return Number.isNaN(timestamp) ? 0 : timestamp;
}

function compareName(left: Skill, right: Skill): number {
  return skillName(left).localeCompare(skillName(right), undefined, { sensitivity: "base" }) || left.id.localeCompare(right.id);
}

export function sortSkills(skills: readonly Skill[], sort: SortMode, agents: readonly Agent[]): Skill[] {
  return [...skills].sort((left, right) => {
    if (sort === "name") return compareName(left, right);
    if (sort === "updated") {
      return modifiedTimestamp(right) - modifiedTimestamp(left) || compareName(left, right);
    }
    if (sort === "linked_agents") {
      return right.linked_agents.length - left.linked_agents.length || compareName(left, right);
    }
    return statusPriority(left, agents) - statusPriority(right, agents) || modifiedTimestamp(right) - modifiedTimestamp(left) || compareName(left, right);
  });
}

export function filterSkills(skills: readonly Skill[], state: ManageFilterState, agents: readonly Agent[]): Skill[] {
  const filtered = skills.filter((skill) =>
    matchesQuery(skill, state.query) &&
    matchesStatusPreset(skill, state.statusPreset, agents) &&
    matchesIssues(skill, state.issues) &&
    matchesLibraryScope(skill, state.libraryScope, agents) &&
    matchesAgentScope(skill, state.agentIds, state.agentMatch)
  );
  return sortSkills(filtered, state.sort, agents);
}

export interface FacetCounts {
  status: Record<StatusPreset, number>;
  issues: Record<IssueFilter, number>;
  library: Record<LibraryScope, number>;
}

export function computeFacetCounts(
  skills: readonly Skill[],
  state: ManageFilterState,
  agents: readonly Agent[]
): FacetCounts {
  const withoutStatus = { ...state, statusPreset: "all" as const };
  const withoutIssues = { ...state, issues: new Set<IssueFilter>() };
  const withoutLibrary = { ...state, libraryScope: new Set<LibraryScope>() };
  const count = (candidate: ManageFilterState): number => filterSkills(skills, candidate, agents).length;
  return {
    status: {
      all: count(withoutStatus),
      needs_attention: count({ ...withoutStatus, statusPreset: "needs_attention" }),
      linked_any: count({ ...withoutStatus, statusPreset: "linked_any" }),
      unlinked_all: count({ ...withoutStatus, statusPreset: "unlinked_all" }),
    },
    issues: {
      conflict: count({ ...withoutIssues, issues: new Set(["conflict"]) }),
      dangling: count({ ...withoutIssues, issues: new Set(["dangling"]) }),
      duplicate: count({ ...withoutIssues, issues: new Set(["duplicate"]) }),
    },
    library: {
      missing_library: count({ ...withoutLibrary, libraryScope: new Set(["missing_library"]) }),
      library_only: count({ ...withoutLibrary, libraryScope: new Set(["library_only"]) }),
    },
  };
}

export function useManageFilters(
  skills: ComputedRef<Skill[]> | Ref<Skill[]>,
  agents: ComputedRef<Agent[]> | Ref<Agent[]>
) {
  const query = ref("");
  const statusPreset = ref<StatusPreset>("all");
  const issues = ref<Set<IssueFilter>>(new Set());
  const libraryScope = ref<Set<LibraryScope>>(new Set());
  const agentIds = ref<Set<string>>(new Set());
  const agentMatch = ref<AgentMatch>("any");
  const sort = ref<SortMode>("status");

  const state = computed<ManageFilterState>(() => ({
    query: query.value,
    statusPreset: statusPreset.value,
    issues: issues.value,
    libraryScope: libraryScope.value,
    agentIds: agentIds.value,
    agentMatch: agentMatch.value,
    sort: sort.value,
  }));
  const filteredSkills = computed(() => filterSkills(skills.value, state.value, agents.value));
  const facetCounts = computed(() => computeFacetCounts(skills.value, state.value, agents.value));
  const activeFilterCount = computed(() =>
    Number(Boolean(query.value.trim())) +
    Number(statusPreset.value !== "all") +
    issues.value.size +
    libraryScope.value.size +
    Number(agentIds.value.size > 0)
  );
  const hasActiveFilters = computed(() => activeFilterCount.value > 0);

  function clearQuery() {
    query.value = "";
  }

  function clearFilters() {
    clearQuery();
    statusPreset.value = "all";
    issues.value = new Set();
    libraryScope.value = new Set();
    agentIds.value = new Set();
    agentMatch.value = "any";
  }

  function toggleIssue(issue: IssueFilter) {
    const next = new Set(issues.value);
    if (next.has(issue)) next.delete(issue);
    else next.add(issue);
    issues.value = next;
  }

  function toggleLibraryScope(scope: LibraryScope) {
    const next = new Set(libraryScope.value);
    if (next.has(scope)) next.delete(scope);
    else next.add(scope);
    libraryScope.value = next;
  }

  function toggleAgent(agentId: string) {
    const next = new Set(agentIds.value);
    if (next.has(agentId)) next.delete(agentId);
    else next.add(agentId);
    agentIds.value = next;
  }

  function normalizeAgents(validAgents: readonly Agent[]) {
    const validIds = new Set(validAgents.map((agent) => agent.id));
    const next = new Set([...agentIds.value].filter((agentId) => validIds.has(agentId)));
    if (next.size !== agentIds.value.size) agentIds.value = next;
    if (next.size === 0) agentMatch.value = "any";
  }

  function removeIssue(issue: IssueFilter) {
    const next = new Set(issues.value);
    next.delete(issue);
    issues.value = next;
  }

  function removeLibraryScope(scope: LibraryScope) {
    const next = new Set(libraryScope.value);
    next.delete(scope);
    libraryScope.value = next;
  }

  function removeAgent(agentId: string) {
    const next = new Set(agentIds.value);
    next.delete(agentId);
    agentIds.value = next;
    if (next.size === 0) agentMatch.value = "any";
  }

  watch(agents, (value) => normalizeAgents(value), { deep: true });

  return {
    query,
    statusPreset,
    issues,
    libraryScope,
    agentIds,
    agentMatch,
    sort,
    state,
    filteredSkills,
    facetCounts,
    activeFilterCount,
    hasActiveFilters,
    clearQuery,
    clearFilters,
    toggleIssue,
    toggleLibraryScope,
    toggleAgent,
    normalizeAgents,
    removeIssue,
    removeLibraryScope,
    removeAgent,
  };
}

export function useManageSelection(visibleSkills: ComputedRef<Skill[]> | Ref<Skill[]>) {
  const selectedIds = ref<Set<string>>(new Set());
  const visibleIds = computed(() => new Set(visibleSkills.value.map((skill) => skill.id)));
  const selectedVisibleCount = computed(() => [...selectedIds.value].filter((id) => visibleIds.value.has(id)).length);
  const allVisibleSelected = computed(() => visibleSkills.value.length > 0 && selectedVisibleCount.value === visibleSkills.value.length);
  const partiallyVisibleSelected = computed(() => selectedVisibleCount.value > 0 && !allVisibleSelected.value);

  function toggleOne(skillId: string) {
    const next = new Set(selectedIds.value);
    if (next.has(skillId)) next.delete(skillId);
    else if (visibleIds.value.has(skillId)) next.add(skillId);
    selectedIds.value = next;
  }

  function toggleAllVisible() {
    const next = new Set(selectedIds.value);
    if (allVisibleSelected.value) {
      visibleIds.value.forEach((id) => next.delete(id));
    } else {
      visibleIds.value.forEach((id) => next.add(id));
    }
    selectedIds.value = next;
  }

  function clearSelection() {
    selectedIds.value = new Set();
  }

  function pruneInvisible() {
    const next = new Set([...selectedIds.value].filter((id) => visibleIds.value.has(id)));
    if (next.size !== selectedIds.value.size) selectedIds.value = next;
  }

  function pruneMissing(allSkills: readonly Skill[]) {
    const validIds = new Set(allSkills.map((skill) => skill.id));
    const next = new Set([...selectedIds.value].filter((id) => validIds.has(id)));
    if (next.size !== selectedIds.value.size) selectedIds.value = next;
  }

  watch(visibleSkills, pruneInvisible, { deep: true });

  return {
    selectedIds,
    selectedVisibleCount,
    allVisibleSelected,
    partiallyVisibleSelected,
    toggleOne,
    toggleAllVisible,
    clearSelection,
    pruneInvisible,
    pruneMissing,
  };
}
