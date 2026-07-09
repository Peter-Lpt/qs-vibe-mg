<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { useToast } from "../../composables/useToast";
import {
  useSkillAgentStatus,
  actionLabel,
  type AgentStatus,
  type AgentAction,
} from "../../composables/useSkillAgentStatus";
import type { Skill, Agent } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  selectedSkillIds: string[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "remove-skill", skillId: string): void;
  (e: "applied"): void;
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const toast = useToast();

type Mode = "sync" | "link_only" | "unlink_only";
const mode = ref<Mode>("sync");
const operating = ref(false);
const showConflictConfirm = ref(false);
const confirmAck = ref(false);
const hasConflictSelected = ref(false);

const selectedCells = ref<Set<string>>(new Set());

const result = ref<{ synced: number; errors: { skillId: string; agentId: string; message: string }[] } | null>(null);

const panelSkills = computed(() =>
  skillsStore.skills.filter((s) => props.selectedSkillIds.includes(s.id))
);
const detectedAgents = computed(() => agentsStore.agents.filter((a) => a.detected));

interface Row {
  skill: Skill;
  statuses: AgentStatus[];
}

// 复用 useSkillAgentStatus 计算每格状态（判定同源），每次数据刷新重建。
const rows = ref<Row[]>([]);
watch(
  () => [panelSkills.value, detectedAgents.value] as const,
  ([skills, agents]) => {
    rows.value = skills.map((skill) => {
      const { allAgentStatuses } = useSkillAgentStatus(
        computed(() => skill),
        computed(() => agents),
        (k, p) => t(k, p as Record<string, unknown>)
      );
      return { skill, statuses: allAgentStatuses.value };
    });
    selectedCells.value = defaultSelection();
    result.value = null;
  },
  { immediate: true }
);

// 切换总开关也重置为默认勾选，避免遗留不可操作的勾选项。
watch(mode, () => {
  selectedCells.value = defaultSelection();
});

const ACTION_COLOR: Record<AgentAction, string> = {
  none: "var(--c-text-secondary)",
  link: "var(--c-primary)",
  unlink: "var(--c-text-secondary)",
  sync_to_vibe: "var(--c-primary)",
  replace_with_link: "var(--c-text)",
  relink: "var(--c-warning)",
  remove_dangling: "var(--c-danger)",
};

function hasVibe(skill: Skill): boolean {
  return skill.sources.some((s) => s.from === "vibe-lib");
}

// §4.4 总开关覆盖规则
function applySwitch(
  m: Mode,
  status: string,
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

function isConflictCell(status: string, action: AgentAction, vibe: boolean): boolean {
  return status === "independent" && action === "sync_to_vibe" && vibe;
}

function defaultSelection(): Set<string> {
  const sel = new Set<string>();
  for (const row of rows.value) {
    const vibe = hasVibe(row.skill);
    for (const st of row.statuses) {
      const sw = applySwitch(mode.value, st.status, st.action, vibe);
      if (sw.selectable && !isConflictCell(st.status, st.action, vibe)) {
        sel.add(`${row.skill.id}::${st.agent.id}`);
      }
    }
  }
  return sel;
}

function statusOf(row: Row, agentId: string): AgentStatus | undefined {
  return row.statuses.find((s) => s.agent.id === agentId);
}

interface CellView {
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

function cellOf(row: Row, agent: Agent): CellView {
  const key = `${row.skill.id}::${agent.id}`;
  const st = statusOf(row, agent.id);
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
  const sw = applySwitch(mode.value, st.status, st.action, vibe);
  const isConflict = isConflictCell(st.status, st.action, vibe);
  const needsImport = st.status === "unlinked" && !vibe;
  const checked = selectedCells.value.has(key);

  let label = "";
  let color = "";
  let muted = false;
  if (!sw.selectable) {
    muted = true;
    if (st.status === "origin") label = t("manage.status_origin");
    else if (st.status === "synced") label = t("manage.status_synced");
    else if (needsImport) label = t("manage.batch_panel_needs_import");
    else label = t("manage.batch_panel_none");
  } else {
    label = actionLabel(t, sw.effectiveAction) || st.statusLabel;
    color = ACTION_COLOR[sw.effectiveAction];
  }
  return {
    skillId: row.skill.id,
    agentId: agent.id,
    selectable: sw.selectable,
    effectiveAction: sw.effectiveAction,
    isConflict,
    needsImport,
    checked,
    label,
    color,
    muted,
  };
}

// 预览摘要：基于当前勾选
const selectedSummary = computed(() => {
  let exec = 0;
  let conflict = 0;
  for (const key of selectedCells.value) {
    const idx = key.indexOf("::");
    const skillId = key.slice(0, idx);
    const agentId = key.slice(idx + 2);
    const row = rows.value.find((r) => r.skill.id === skillId);
    if (!row) continue;
    const st = row.statuses.find((s) => s.agent.id === agentId);
    if (!st) continue;
    const sw = applySwitch(mode.value, st.status, st.action, hasVibe(row.skill));
    if (!sw.selectable) continue;
    if (isConflictCell(st.status, st.action, hasVibe(row.skill))) conflict++;
    else exec++;
  }
  return { exec, conflict };
});

const importNeededCount = computed(() => {
  let k = 0;
  for (const row of rows.value) {
    const vibe = hasVibe(row.skill);
    for (const st of row.statuses) {
      if (st.status === "unlinked" && !vibe) k++;
    }
  }
  return k;
});

// ── 选择操作 ──────────────────────────────────
function toggleCell(skillId: string, agentId: string) {
  const key = `${skillId}::${agentId}`;
  const s = new Set(selectedCells.value);
  if (s.has(key)) s.delete(key);
  else s.add(key);
  selectedCells.value = s;
}

function selectableKeysForRow(row: Row): string[] {
  const vibe = hasVibe(row.skill);
  return row.statuses
    .filter((st) => applySwitch(mode.value, st.status, st.action, vibe).selectable)
    .map((st) => `${row.skill.id}::${st.agent.id}`);
}

function toggleRow(row: Row) {
  const keys = selectableKeysForRow(row);
  const s = new Set(selectedCells.value);
  const allSel = keys.length > 0 && keys.every((k) => s.has(k));
  keys.forEach((k) => (allSel ? s.delete(k) : s.add(k)));
  selectedCells.value = s;
}

function selectableKeysForCol(agentId: string): string[] {
  const keys: string[] = [];
  for (const row of rows.value) {
    const st = row.statuses.find((x) => x.agent.id === agentId);
    if (st && applySwitch(mode.value, st.status, st.action, hasVibe(row.skill)).selectable) {
      keys.push(`${row.skill.id}::${agentId}`);
    }
  }
  return keys;
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

// ── 执行 ──────────────────────────────────────
async function execute() {
  result.value = null;
  hasConflictSelected.value = false;
  const cells: { skillId: string; agentId: string; action: AgentAction }[] = [];
  for (const key of selectedCells.value) {
    const idx = key.indexOf("::");
    const skillId = key.slice(0, idx);
    const agentId = key.slice(idx + 2);
    const row = rows.value.find((r) => r.skill.id === skillId);
    if (!row) continue;
    const st = row.statuses.find((s) => s.agent.id === agentId);
    if (!st) continue;
    const sw = applySwitch(mode.value, st.status, st.action, hasVibe(row.skill));
    if (!sw.selectable || sw.effectiveAction === "none") continue;
    if (isConflictCell(st.status, st.action, hasVibe(row.skill))) {
      hasConflictSelected.value = true;
    }
    cells.push({ skillId, agentId, action: sw.effectiveAction });
  }

  if (cells.length === 0) {
    toast.show(t("manage.batch_panel_no_selection"), "warning");
    return;
  }
  if (hasConflictSelected.value && !confirmAck.value) {
    showConflictConfirm.value = true;
    return;
  }
  await runExecute(cells);
}

async function runExecute(cells: { skillId: string; agentId: string; action: AgentAction }[]) {
  operating.value = true;
  // 按 (skillId, 有效动作) 分组 → 同一 skill 行内混合动作自动拆成独立调用，不串味
  const groups = new Map<string, { skillId: string; action: AgentAction; agentIds: string[] }>();
  for (const c of cells) {
    const k = `${c.skillId}::${c.action}`;
    if (!groups.has(k)) groups.set(k, { skillId: c.skillId, action: c.action, agentIds: [] });
    groups.get(k)!.agentIds.push(c.agentId);
  }

  let totalSynced = 0;
  const errors: { skillId: string; agentId: string; message: string }[] = [];
  for (const g of groups.values()) {
    try {
      const res = await skillsStore.batchSkillAction(g.skillId, g.agentIds, g.action, true);
      totalSynced += res.synced_count;
      for (const e of res.errors) {
        const ci = e.indexOf(": ");
        const agentId = ci >= 0 ? e.slice(0, ci) : "";
        const message = ci >= 0 ? e.slice(ci + 2) : e;
        errors.push({ skillId: g.skillId, agentId, message });
      }
    } catch (e: unknown) {
      errors.push({ skillId: g.skillId, agentId: "", message: String(e) });
    }
  }

  // 全部完成后统一刷新，使"先入库再链接"工作流能刷新出新状态
  await skillsStore.refreshSkills();
  await agentsStore.fetchAgents();
  operating.value = false;
  confirmAck.value = false;
  hasConflictSelected.value = false;
  result.value = { synced: totalSynced, errors };

  if (errors.length === 0) {
    toast.show(t("manage.batch_panel_result_success", { count: totalSynced }), "success");
  } else {
    toast.show(
      t("manage.batch_panel_result_error", { success: totalSynced, error: errors.length }),
      "warning"
    );
  }
  emit("applied");
}

function onConfirmConflict() {
  showConflictConfirm.value = false;
  confirmAck.value = true;
  // 冲突项保留在勾选中，执行后会如实进入失败明细（诚实失败）
  execute();
}

function closePanel() {
  emit("close");
}
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center"
    style="background: rgba(0, 0, 0, 0.45);"
    @click.self="closePanel"
  >
    <div
      class="flex flex-col w-[min(92vw,920px)] h-[min(88vh,720px)] rounded-xl overflow-hidden"
      style="background: var(--c-surface); border: 1px solid var(--c-border);"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b" style="border-color: var(--c-border);">
        <div>
          <h3 class="text-sm font-semibold" style="color: var(--c-text);">
            {{ t("manage.batch_panel_title", { count: panelSkills.length }) }}
          </h3>
          <p class="text-[10px] mt-0.5" style="color: var(--c-text-secondary);">
            {{ t("manage.batch_panel_steps") }}
          </p>
        </div>
        <button
          class="w-7 h-7 flex items-center justify-center rounded cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
          style="color: var(--c-text-secondary);"
          @click="closePanel"
        >
          ✕
        </button>
      </div>

      <!-- Mode switch -->
      <div class="flex items-center gap-2 px-4 py-2 border-b" style="border-color: var(--c-border);">
        <button
          v-for="m in (['sync', 'link_only', 'unlink_only'] as Mode[])"
          :key="m"
          class="text-[11px] px-3 py-1.5 rounded-md cursor-pointer transition-colors"
          :style="
            mode === m
              ? 'background: var(--c-primary); color: white;'
              : 'background: var(--c-bg); color: var(--c-text-secondary); border: 1px solid var(--c-border);'
          "
          @click="mode = m"
        >
          {{ t(m === 'sync' ? 'manage.batch_panel_mode_sync' : m === 'link_only' ? 'manage.batch_panel_mode_link' : 'manage.batch_panel_mode_unlink') }}
        </button>
        <div class="ml-auto flex items-center gap-2">
          <button class="text-[10px] px-2 py-1 rounded cursor-pointer" style="color: var(--c-text-secondary); border: 1px solid var(--c-border);" @click="selectAll">
            {{ t("manage.batch_panel_select_all") }}
          </button>
          <button class="text-[10px] px-2 py-1 rounded cursor-pointer" style="color: var(--c-text-secondary); border: 1px solid var(--c-border);" @click="clearSelection">
            {{ t("manage.batch_panel_clear") }}
          </button>
        </div>
      </div>

      <!-- Matrix -->
      <div class="flex-1 overflow-auto px-4 py-3">
        <div
          v-if="panelSkills.length === 0"
          class="text-xs py-8 text-center"
          style="color: var(--c-text-secondary);"
        >
          {{ t("manage.batch_panel_no_skills") }}
        </div>
        <div
          v-else-if="detectedAgents.length === 0"
          class="text-xs py-8 text-center"
          style="color: var(--c-text-secondary);"
        >
          {{ t("manage.batch_panel_no_agents") }}
        </div>
        <table v-else class="w-full text-xs border-collapse">
          <thead>
            <tr>
              <th
                class="sticky left-0 z-10 px-2 py-2 text-left font-medium whitespace-nowrap"
                style="background: var(--c-surface); color: var(--c-text-secondary); min-width: 150px; border-bottom: 1px solid var(--c-border);"
              >
                {{ t("manage.title") }}
              </th>
              <th
                v-for="agent in detectedAgents"
                :key="agent.id"
                class="px-2 py-2 text-center font-medium cursor-pointer select-none"
                style="color: var(--c-text-secondary); border-bottom: 1px solid var(--c-border);"
                :title="t('manage.batch_panel_col_tip')"
                @click="toggleCol(agent.id)"
              >
                <div class="flex flex-col items-center gap-1">
                  <span class="truncate max-w-[80px] block">{{ agent.name }}</span>
                  <input
                    type="checkbox"
                    :checked="selectableKeysForCol(agent.id).length > 0 && selectableKeysForCol(agent.id).every((k) => selectedCells.has(k))"
                    class="w-3 h-3 rounded cursor-pointer"
                    style="accent-color: var(--c-primary);"
                    @click.stop.prevent="toggleCol(agent.id)"
                  />
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="row.skill.id" style="border-bottom: 1px solid var(--c-border);">
              <td
                class="sticky left-0 z-10 px-2 py-1.5 cursor-pointer select-none"
                style="background: var(--c-surface);"
                :title="t('manage.batch_panel_row_tip')"
                @click="toggleRow(row)"
              >
                <div class="flex items-center gap-2">
                  <input
                    type="checkbox"
                    :checked="selectableKeysForRow(row).length > 0 && selectableKeysForRow(row).every((k) => selectedCells.has(k))"
                    class="w-3.5 h-3.5 rounded cursor-pointer shrink-0"
                    style="accent-color: var(--c-primary);"
                    @click.stop.prevent="toggleRow(row)"
                  />
                  <span class="text-xs font-medium truncate max-w-[110px]" style="color: var(--c-text);">
                    {{ row.skill.name || row.skill.id }}
                  </span>
                  <button
                    class="shrink-0 text-[10px] px-1 rounded cursor-pointer"
                    style="color: var(--c-text-secondary);"
                    :title="t('manage.batch_panel_remove')"
                    @click.stop="emit('remove-skill', row.skill.id)"
                  >
                    ✕
                  </button>
                </div>
              </td>
              <td
                v-for="agent in detectedAgents"
                :key="agent.id"
                class="px-1 py-1.5 text-center"
              >
                <button
                  v-if="cellOf(row, agent).selectable"
                  class="w-full min-w-[64px] px-1.5 py-1 rounded border text-[10px] cursor-pointer transition-colors"
                  :style="{
                    borderColor: cellOf(row, agent).isConflict
                      ? 'var(--c-danger)'
                      : cellOf(row, agent).checked
                        ? (cellOf(row, agent).color || 'var(--c-primary)')
                        : 'var(--c-border)',
                    background: cellOf(row, agent).checked
                      ? (cellOf(row, agent).isConflict ? 'var(--c-danger-light)' : 'var(--c-primary-light)')
                      : 'transparent',
                    color: cellOf(row, agent).checked
                      ? (cellOf(row, agent).isConflict ? 'var(--c-danger)' : 'var(--c-primary)')
                      : (cellOf(row, agent).isConflict ? 'var(--c-danger)' : (cellOf(row, agent).color || 'var(--c-text)')),
                  }"
                  @click="toggleCell(row.skill.id, agent.id)"
                >
                  {{ cellOf(row, agent).label }}
                </button>
                <span
                  v-else
                  class="inline-block px-1.5 py-1 text-[10px] rounded"
                  :style="{
                    color: cellOf(row, agent).needsImport ? 'var(--c-text-secondary)' : 'var(--c-text-secondary)',
                    background: cellOf(row, agent).isConflict ? 'var(--c-danger-light)' : 'transparent',
                    border: cellOf(row, agent).isConflict ? '1px solid var(--c-danger)' : '1px dashed var(--c-border)',
                  }"
                  :title="cellOf(row, agent).needsImport ? t('manage.batch_panel_needs_import_tip') : ''"
                >
                  {{ cellOf(row, agent).label }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Result detail -->
      <div v-if="result && result.errors.length > 0" class="px-4 pb-2 max-h-[120px] overflow-auto">
        <div class="text-[11px] font-medium mb-1" style="color: var(--c-danger);">
          {{ t("manage.batch_panel_error_detail") }}
        </div>
        <div
          v-for="(err, i) in result.errors"
          :key="i"
          class="text-[10px] py-0.5"
          style="color: var(--c-text-secondary);"
        >
          {{ skillsStore.skills.find((s) => s.id === err.skillId)?.name || err.skillId }}
          @ {{ agentsStore.agents.find((a) => a.id === err.agentId)?.name || err.agentId || "?" }}
          : {{ err.message }}
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center gap-3 px-4 py-3 border-t" style="border-color: var(--c-border);">
        <span class="text-[11px]" style="color: var(--c-text-secondary);">
          {{ t("manage.batch_panel_summary", {
            exec: selectedSummary.exec,
            conflict: selectedSummary.conflict,
            import: importNeededCount,
          }) }}
        </span>
        <div class="ml-auto flex items-center gap-2">
          <button
            class="text-[11px] px-3 py-1.5 rounded-md cursor-pointer"
            style="color: var(--c-text-secondary);"
            @click="closePanel"
          >
            {{ t("manage.batch_panel_close") }}
          </button>
          <button
            class="text-[11px] px-4 py-1.5 rounded-md cursor-pointer"
            :style="{ background: 'var(--c-primary)', color: 'white' }"
            :disabled="operating"
            @click="execute"
          >
            {{ operating ? "..." : t("manage.batch_panel_execute") }}
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-if="showConflictConfirm"
      :title="t('manage.batch_panel_conflict_confirm_title')"
      :message="t('manage.batch_panel_conflict_confirm_msg', { count: selectedSummary.conflict })"
      :confirm-text="t('manage.batch_panel_execute')"
      :danger="true"
      @confirm="onConfirmConflict"
      @cancel="showConflictConfirm = false"
    />
  </div>
</template>
