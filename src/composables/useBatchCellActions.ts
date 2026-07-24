import { computed, type Ref } from "vue";
import type { Agent, Skill } from "../types";
import { useSkillAgentStatus, type AgentStatus, type AgentStatusType } from "./useSkillAgentStatus";
import { actionColor, actionLabel, type AgentAction, type TFunc } from "./skillActionRegistry";

// ── 类型 ─────────────────────────────────────────────
export type Mode = "sync" | "link_only" | "unlink_only";
export type CellScope = "all" | "dangling";

export type BatchActionCardId = "sync" | "link" | "unlink" | "clean_dangling";

export interface BatchActionCard {
  id: BatchActionCardId;
  mode: Mode;
  cellScope: CellScope;
  labelKey: string;
  descKey: string;
}

export const BATCH_ACTION_CARDS: readonly BatchActionCard[] = [
  { id: "sync", mode: "sync", cellScope: "all", labelKey: "batch.action_sync", descKey: "batch.action_sync_desc" },
  { id: "link", mode: "link_only", cellScope: "all", labelKey: "batch.action_link", descKey: "batch.action_link_desc" },
  { id: "unlink", mode: "unlink_only", cellScope: "all", labelKey: "batch.action_unlink", descKey: "batch.action_unlink_desc" },
  { id: "clean_dangling", mode: "sync", cellScope: "dangling", labelKey: "batch.action_clean_dangling", descKey: "batch.action_clean_dangling_desc" },
];

export type RepairContext = "conflict" | "dangling" | "missing_lib";

// 修复上下文 → 预设动作卡（首次接通 repairContext 通路）
export const REPAIR_PRESETS: Record<RepairContext, { cardId: BatchActionCardId; labelKey: string; hintKey: string }> = {
  conflict: { cardId: "sync", labelKey: "batch.repair_conflict", hintKey: "batch.repair_conflict_hint" },
  dangling: { cardId: "clean_dangling", labelKey: "batch.repair_dangling", hintKey: "batch.repair_dangling_hint" },
  missing_lib: { cardId: "sync", labelKey: "batch.repair_missing_lib", hintKey: "batch.repair_missing_lib_hint" },
};

export interface BatchRow {
  skill: Skill;
  statuses: AgentStatus[];
}

export interface CellView {
  skillId: string;
  agentId: string;
  selectable: boolean;
  effectiveAction: AgentAction;
  isConflict: boolean;
  needsImport: boolean;
  checked: boolean;
  label: string;
  color: string;
  muted: boolean;
}

export type DryRunCategory = "execute" | "skip" | "conflict" | "blocked";

export interface DryRunItem {
  key: string;
  skillId: string;
  skillName: string;
  agentId: string;
  agentName: string;
  action: AgentAction | "conflict" | "needs_import" | "skipped";
  category: DryRunCategory;
  reason: string;
}

export interface DryRunCounts {
  execute: number;
  link: number;
  relink: number;
  clean: number;
  sync: number;
  skipped: number;
  conflict: number;
  blocked: number;
}

export interface BatchResult {
  synced: number;
  success: DryRunItem[];
  failed: { item: DryRunItem | null; skillId: string; agentId: string; message: string }[];
  warnings: { skillId: string; message: string }[];
  skipped: DryRunItem[];
  conflicts: DryRunItem[];
  blocked: DryRunItem[];
}

// ── 纯函数（自 BatchSyncPanel.vue 搬迁，逻辑一字不改） ──
export function hasVibe(skill: Skill): boolean {
  return skill.sources.some((s) => s.from === "vibe-lib");
}

// §4.4 总开关覆盖规则
export function applySwitch(
  m: Mode,
  status: AgentStatusType | string,
  action: AgentAction,
  vibe: boolean
): { effectiveAction: AgentAction; selectable: boolean } {
  const needsImport = status === "unlinked" && !vibe;
  if (m === "sync") {
    if (status === "origin" || status === "synced" || needsImport)
      return { effectiveAction: "none", selectable: false };
    return { effectiveAction: action, selectable: true };
  }
  if (m === "link_only") {
    if (status === "unlinked" && !needsImport)
      return { effectiveAction: "link", selectable: true };
    return { effectiveAction: "none", selectable: false };
  }
  // unlink_only：唯一有意覆盖基础动作的地方（relink → unlink）
  if (status === "synced" || status === "linked_elsewhere")
    return { effectiveAction: "unlink", selectable: true };
  return { effectiveAction: "none", selectable: false };
}

export function isConflictCell(status: string, action: AgentAction, vibe: boolean): boolean {
  return status === "independent" && action === "sync_to_vibe" && vibe;
}

// 逐 skill 调 useSkillAgentStatus 取 allAgentStatuses（判定同源）
export function buildBatchRows(
  skills: readonly Skill[],
  agents: Agent[],
  t: TFunc
): BatchRow[] {
  return skills.map((skill) => {
    const { allAgentStatuses } = useSkillAgentStatus(
      computed(() => skill),
      computed(() => agents),
      t
    );
    return { skill, statuses: allAgentStatuses.value };
  });
}

// ── 纯计算（矩阵组件与 composable 共用，判定同源） ──
export function computeCellView(
  row: BatchRow,
  agent: Agent,
  mode: Mode,
  selectedCells: ReadonlySet<string>,
  t: TFunc
): CellView {
  const key = `${row.skill.id}::${agent.id}`;
  const st = row.statuses.find((s) => s.agent.id === agent.id);
  const vibe = hasVibe(row.skill);
  if (!st) {
    return {
      skillId: row.skill.id,
      agentId: agent.id,
      selectable: false,
      effectiveAction: "none",
      isConflict: false,
      needsImport: false,
      checked: false,
      label: "",
      color: "",
      muted: true,
    };
  }
  const sw = applySwitch(mode, st.status, st.action, vibe);
  const isConflict = isConflictCell(st.status, st.action, vibe);
  const needsImport = st.status === "unlinked" && !vibe;
  const checked = selectedCells.has(key);

  let label = "";
  let color = "";
  let muted = false;
  if (isConflict) {
    muted = true;
    label = t("manage.batch_panel_conflict");
  } else if (!sw.selectable) {
    muted = true;
    if (st.status === "origin") label = t("manage.status_origin");
    else if (st.status === "synced") label = t("manage.status_synced");
    else if (needsImport) label = t("manage.batch_panel_needs_import");
    else label = t("manage.batch_panel_none");
  } else {
    label = actionLabel(t, sw.effectiveAction) || st.statusLabel;
    color = actionColor(sw.effectiveAction);
  }
  return {
    skillId: row.skill.id,
    agentId: agent.id,
    selectable: isConflict ? false : sw.selectable,
    effectiveAction: sw.effectiveAction,
    isConflict,
    needsImport,
    checked,
    label,
    color,
    muted,
  };
}

export function computeRowSelectableKeys(row: BatchRow, mode: Mode, cellScope: CellScope): string[] {
  const vibe = hasVibe(row.skill);
  return row.statuses
    .filter((st) => {
      if (cellScope === "dangling" && st.status !== "dangling") return false;
      return (
        applySwitch(mode, st.status, st.action, vibe).selectable &&
        !isConflictCell(st.status, st.action, vibe)
      );
    })
    .map((st) => `${row.skill.id}::${st.agent.id}`);
}

export function computeColSelectableKeys(
  rows: readonly BatchRow[],
  agentId: string,
  mode: Mode,
  cellScope: CellScope
): string[] {
  const keys: string[] = [];
  for (const row of rows) {
    const st = row.statuses.find((x) => x.agent.id === agentId);
    const vibe = hasVibe(row.skill);
    if (!st) continue;
    if (cellScope === "dangling" && st.status !== "dangling") continue;
    if (
      applySwitch(mode, st.status, st.action, vibe).selectable &&
      !isConflictCell(st.status, st.action, vibe)
    ) {
      keys.push(`${row.skill.id}::${agentId}`);
    }
  }
  return keys;
}

// ── 响应式主体 ────────────────────────────────────────
export function useBatchCellActions(
  rows: Ref<BatchRow[]>,
  actionCard: Ref<BatchActionCard>,
  selectedCells: Ref<Set<string>>,
  t: TFunc
) {
  function cellOf(row: BatchRow, agent: Agent): CellView {
    return computeCellView(row, agent, actionCard.value.mode, selectedCells.value, t);
  }

  // 默认勾选：cellScope 收窄（"clean_dangling" 仅勾 dangling 格 —— v0.3 新行为）
  function defaultSelection(): Set<string> {
    const sel = new Set<string>();
    for (const row of rows.value) {
      const vibe = hasVibe(row.skill);
      for (const st of row.statuses) {
        if (actionCard.value.cellScope === "dangling" && st.status !== "dangling") continue;
        const sw = applySwitch(actionCard.value.mode, st.status, st.action, vibe);
        if (sw.selectable && !isConflictCell(st.status, st.action, vibe)) {
          sel.add(`${row.skill.id}::${st.agent.id}`);
        }
      }
    }
    return sel;
  }

  const selectedTargetAgentIds = computed(() => {
    const ids = new Set<string>();
    for (const key of selectedCells.value) {
      const idx = key.indexOf("::");
      if (idx >= 0) ids.add(key.slice(idx + 2));
    }
    return ids;
  });

  const dryRunItems = computed<DryRunItem[]>(() => {
    const items: DryRunItem[] = [];
    const targetAgentIds = selectedTargetAgentIds.value;
    if (targetAgentIds.size === 0) return items;
    for (const row of rows.value) {
      const vibe = hasVibe(row.skill);
      for (const st of row.statuses) {
        if (!targetAgentIds.has(st.agent.id)) continue;
        const key = `${row.skill.id}::${st.agent.id}`;
        const sw = applySwitch(actionCard.value.mode, st.status, st.action, vibe);
        const selected = selectedCells.value.has(key);
        const base = {
          key,
          skillId: row.skill.id,
          skillName: row.skill.name || row.skill.id,
          agentId: st.agent.id,
          agentName: st.agent.name,
        };

        if (isConflictCell(st.status, st.action, vibe)) {
          items.push({
            ...base,
            action: "conflict",
            category: "conflict",
            reason: t("manage.batch_panel_reason_conflict"),
          });
        } else if (st.status === "unlinked" && !vibe) {
          items.push({
            ...base,
            action: "needs_import",
            category: "blocked",
            reason: t("manage.batch_panel_reason_needs_import"),
          });
        } else if (selected && sw.selectable && sw.effectiveAction !== "none") {
          items.push({
            ...base,
            action: sw.effectiveAction,
            category: "execute",
            reason: actionLabel(t, sw.effectiveAction) || st.statusLabel,
          });
        } else if (!selected && sw.selectable && sw.effectiveAction !== "none") {
          items.push({
            ...base,
            action: "skipped",
            category: "skip",
            reason: t("manage.batch_panel_reason_not_selected"),
          });
        } else if (!sw.selectable) {
          items.push({
            ...base,
            action: "skipped",
            category: "skip",
            reason: st.statusLabel,
          });
        }
      }
    }
    return items;
  });

  const dryRunCounts = computed<DryRunCounts>(() => ({
    execute: dryRunItems.value.filter((i) => i.category === "execute").length,
    link: dryRunItems.value.filter((i) => i.action === "link").length,
    relink: dryRunItems.value.filter((i) => i.action === "relink").length,
    clean: dryRunItems.value.filter((i) => i.action === "remove_dangling").length,
    sync: dryRunItems.value.filter((i) => i.action === "sync_to_vibe" || i.action === "replace_with_link").length,
    skipped: dryRunItems.value.filter((i) => i.category === "skip").length,
    conflict: dryRunItems.value.filter((i) => i.category === "conflict").length,
    blocked: dryRunItems.value.filter((i) => i.category === "blocked").length,
  }));

  // ── 选择操作 ──────────────────────────────────
  function toggleCell(skillId: string, agentId: string) {
    const key = `${skillId}::${agentId}`;
    const s = new Set(selectedCells.value);
    if (s.has(key)) s.delete(key);
    else s.add(key);
    selectedCells.value = s;
  }

  function selectableKeysForRow(row: BatchRow): string[] {
    return computeRowSelectableKeys(row, actionCard.value.mode, actionCard.value.cellScope);
  }

  function toggleRow(row: BatchRow) {
    const keys = selectableKeysForRow(row);
    const s = new Set(selectedCells.value);
    const allSel = keys.length > 0 && keys.every((k) => s.has(k));
    keys.forEach((k) => (allSel ? s.delete(k) : s.add(k)));
    selectedCells.value = s;
  }

  function selectableKeysForCol(agentId: string): string[] {
    return computeColSelectableKeys(rows.value, agentId, actionCard.value.mode, actionCard.value.cellScope);
  }

  function toggleCol(agentId: string) {
    const keys = selectableKeysForCol(agentId);
    const s = new Set(selectedCells.value);
    const allSel = keys.length > 0 && keys.every((k) => s.has(k));
    keys.forEach((k) => (allSel ? s.delete(k) : s.add(k)));
    selectedCells.value = s;
  }

  function selectAll() {
    selectedCells.value = defaultSelection();
  }

  function clearSelection() {
    selectedCells.value = new Set();
  }

  return {
    cellOf,
    defaultSelection,
    dryRunItems,
    dryRunCounts,
    selectedTargetAgentIds,
    selectableKeysForRow,
    selectableKeysForCol,
    toggleCell,
    toggleRow,
    toggleCol,
    selectAll,
    clearSelection,
  };
}
