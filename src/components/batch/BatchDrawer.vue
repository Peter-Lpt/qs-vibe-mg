<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { useToast } from "../../composables/useToast";
import {
  BATCH_ACTION_CARDS,
  REPAIR_PRESETS,
  buildBatchRows,
  useBatchCellActions,
  type BatchActionCard,
  type BatchResult,
  type BatchRow,
  type DryRunItem,
  type RepairContext,
} from "../../composables/useBatchCellActions";
import type { AgentAction } from "../../composables/skillActionRegistry";
import BatchActionStep from "./BatchActionStep.vue";
import BatchTargetMatrix from "./BatchTargetMatrix.vue";
import BatchPreviewStep from "./BatchPreviewStep.vue";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = withDefaults(
  defineProps<{
    open: boolean;
    selectedSkillIds: string[];
    repairContext?: RepairContext | null;
    overlay?: boolean;
  }>(),
  { repairContext: null, overlay: false }
);

const emit = defineEmits<{
  (e: "close"): void;
  (e: "remove-skill", skillId: string): void;
  (e: "resolve-conflict", skillId: string): void;
  (e: "applied"): void;
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const toast = useToast();

type DrawerStep = 1 | 2 | 3;

const step = ref<DrawerStep>(1);
const actionCard = ref<BatchActionCard>(BATCH_ACTION_CARDS[0]);
const selectedCells = ref<Set<string>>(new Set());
const result = ref<BatchResult | null>(null);
const contextStale = ref(false);
const operating = ref(false);
const confirmAck = ref(false);
const showConflictConfirm = ref(false);

const panelSkills = computed(() =>
  skillsStore.skills.filter((s) => props.selectedSkillIds.includes(s.id))
);
const detectedAgents = computed(() => agentsStore.agents.filter((a) => a.detected));

const rows = ref<BatchRow[]>([]);
const cellActions = useBatchCellActions(rows, actionCard, selectedCells, (k, p) =>
  t(k, p as Record<string, unknown>)
);

// rows 重建（后台刷新 / 选择变化 / 打开抽屉）：重置勾选 + "上下文已变化"提示（§10.4）
let firstBuild = true;
watch(
  () => [panelSkills.value, detectedAgents.value] as const,
  ([skills, agents]) => {
    rows.value = buildBatchRows(skills, agents, (k, p) => t(k, p as Record<string, unknown>));
    selectedCells.value = cellActions.defaultSelection();
    if (firstBuild) {
      firstBuild = false;
      return;
    }
    if (!operating.value) {
      result.value = null;
      contextStale.value = true;
    }
  },
  { immediate: true }
);

// repairContext 首次接线：预设动作卡（§9.2）
watch(
  () => props.repairContext,
  (ctx) => {
    actionCard.value = ctx
      ? BATCH_ACTION_CARDS.find((c) => c.id === REPAIR_PRESETS[ctx].cardId) ?? BATCH_ACTION_CARDS[0]
      : BATCH_ACTION_CARDS[0];
    selectedCells.value = cellActions.defaultSelection();
    result.value = null;
  },
  { immediate: true }
);

// 切换动作卡：重置为默认勾选，避免遗留不可操作的勾选项
watch(actionCard, () => {
  selectedCells.value = cellActions.defaultSelection();
  result.value = null;
});

function goNext() {
  if (step.value < 3) step.value = (step.value + 1) as DrawerStep;
}

function goPrev() {
  if (step.value > 1) step.value = (step.value - 1) as DrawerStep;
}

function backToMatrix() {
  result.value = null;
  step.value = 2;
}

function toggleRowById(skillId: string) {
  const row = rows.value.find((r) => r.skill.id === skillId);
  if (row) cellActions.toggleRow(row);
}

// ── 执行管线（§10.3） ──────────────────────────────────
async function execute() {
  result.value = null;
  const plan = cellActions.dryRunItems.value;
  const hasConflict = plan.some((item) => item.category === "conflict");
  const cells = plan
    .filter((item): item is DryRunItem & { action: AgentAction } => item.category === "execute")
    .map((item) => ({ skillId: item.skillId, agentId: item.agentId, action: item.action }));

  if (cells.length === 0) {
    toast.show(t("batch.no_selection"), "warning");
    return;
  }
  if (hasConflict && !confirmAck.value) {
    showConflictConfirm.value = true;
    return;
  }
  await runExecute(cells, plan);
}

async function runExecute(
  cells: { skillId: string; agentId: string; action: AgentAction }[],
  plan: DryRunItem[]
) {
  operating.value = true;
  // 按 (skillId, 有效动作) 分组 —— 同一 skill 行内混合动作拆成独立调用，不串味
  const groups = new Map<string, { skillId: string; action: AgentAction; agentIds: string[] }>();
  for (const c of cells) {
    const k = `${c.skillId}::${c.action}`;
    if (!groups.has(k)) groups.set(k, { skillId: c.skillId, action: c.action, agentIds: [] });
    groups.get(k)!.agentIds.push(c.agentId);
  }

  let totalSynced = 0;
  const errors: { skillId: string; agentId: string; message: string }[] = [];
  const warnings: { skillId: string; message: string }[] = [];
  for (const g of groups.values()) {
    try {
      const res = await skillsStore.batchSkillAction(g.skillId, g.agentIds, g.action, true);
      totalSynced += res.synced_count;
      for (const warning of res.warnings) {
        warnings.push({ skillId: g.skillId, message: warning });
      }
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

  // 全部完成后统一刷新（合同：不得逐组刷新）
  await skillsStore.refreshSkills();
  await agentsStore.fetchAgents();
  operating.value = false;
  confirmAck.value = false;

  const failedKeys = new Set(errors.map((e) => `${e.skillId}::${e.agentId}`));
  const success = plan.filter((item) => item.category === "execute" && !failedKeys.has(item.key));
  result.value = {
    synced: totalSynced,
    success,
    failed: errors.map((e) => ({
      item: plan.find((item) => item.key === `${e.skillId}::${e.agentId}`) ?? null,
      ...e,
    })),
    warnings,
    skipped: plan.filter((item) => item.category === "skip"),
    conflicts: plan.filter((item) => item.category === "conflict"),
    blocked: plan.filter((item) => item.category === "blocked"),
  };

  if (errors.length === 0 && warnings.length === 0) {
    toast.show(t("manage.batch_panel_result_success", { count: totalSynced }), "success");
  } else if (errors.length === 0) {
    toast.show(t("manage.batch_panel_result_warning", { success: totalSynced, warning: warnings.length }), "warning");
  } else {
    toast.show(t("manage.batch_panel_result_error", { success: totalSynced, error: errors.length }), "warning");
  }
  emit("applied");
}

function onConfirmConflict() {
  showConflictConfirm.value = false;
  confirmAck.value = true;
  execute();
}

function close() {
  if (operating.value) return;
  emit("close");
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && props.open && !operating.value && !showConflictConfirm.value) {
    event.stopPropagation();
    close();
  }
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      document.addEventListener("keydown", handleKeydown, true);
    } else {
      document.removeEventListener("keydown", handleKeydown, true);
    }
  },
  { immediate: true }
);

const wrapperClass = computed(() =>
  props.overlay ? "fixed inset-0 z-50" : "sticky top-4 w-[min(520px,42vw)] shrink-0 self-start"
);
const panelClass = computed(() =>
  props.overlay
    ? "absolute right-0 top-0 flex h-full w-[min(520px,92vw)] flex-col border-l shadow-xl"
    : "flex max-h-[calc(100vh-110px)] flex-col rounded-lg border shadow-lg"
);

defineExpose({
  resetToStep(target: DrawerStep) {
    step.value = target;
  },
  markContextStale() {
    contextStale.value = true;
  },
});
</script>

<template>
  <Teleport to="body" :disabled="!overlay">
    <div v-if="open" :class="wrapperClass">
      <div
        v-if="overlay"
        class="absolute inset-0"
        style="background: rgba(0, 0, 0, 0.4);"
        @click="close"
      />
      <div :class="panelClass" style="background: var(--c-surface); border-color: var(--c-border);">
        <!-- Header -->
        <div class="flex shrink-0 items-center gap-2 border-b px-4 py-3" style="border-color: var(--c-border);">
          <div class="min-w-0">
            <h3 class="text-sm font-semibold" style="color: var(--c-text);">
              {{ t("batch.drawer_title", { count: rows.length }) }}
            </h3>
            <div class="mt-1 flex items-center gap-1.5 text-[9px]" style="color: var(--c-text-tertiary);">
              <span :style="step === 1 ? 'color: var(--c-primary); font-weight: 600;' : ''">{{ t("batch.step_action") }}</span>
              <span>→</span>
              <span :style="step === 2 ? 'color: var(--c-primary); font-weight: 600;' : ''">{{ t("batch.step_target") }}</span>
              <span>→</span>
              <span :style="step === 3 ? 'color: var(--c-primary); font-weight: 600;' : ''">{{ t("batch.step_preview") }}</span>
            </div>
          </div>
          <button
            type="button"
            class="ml-auto flex h-7 w-7 items-center justify-center rounded cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)] disabled:opacity-40"
            style="color: var(--c-text-secondary);"
            :disabled="operating"
            @click="close"
          >
            <X :size="14" />
          </button>
        </div>

        <!-- 上下文已变化提示 -->
        <div
          v-if="contextStale"
          class="flex shrink-0 items-center gap-2 border-b px-4 py-2 text-[10px]"
          style="background: var(--c-warning-light); color: var(--c-warning); border-color: var(--c-border);"
        >
          <CircleAlert :size="12" class="shrink-0" />
          <span class="flex-1">{{ t("batch.context_stale") }}</span>
          <button type="button" class="cursor-pointer" @click="contextStale = false">
            <X :size="12" />
          </button>
        </div>

        <!-- Step body -->
        <div class="min-h-0 flex-1 overflow-y-auto px-4 py-3">
          <BatchActionStep
            v-if="step === 1"
            :model-value="actionCard"
            :repair-context="repairContext"
            @update:model-value="actionCard = $event"
          />
          <BatchTargetMatrix
            v-else-if="step === 2"
            :rows="rows"
            :agents="detectedAgents"
            :action-card="actionCard"
            :selected-cells="selectedCells"
            @toggle-cell="cellActions.toggleCell"
            @toggle-row="toggleRowById"
            @toggle-col="cellActions.toggleCol"
            @select-all="cellActions.selectAll"
            @clear="cellActions.clearSelection"
            @remove-skill="emit('remove-skill', $event)"
            @resolve-conflict="emit('resolve-conflict', $event)"
          />
          <BatchPreviewStep
            v-else
            :items="cellActions.dryRunItems.value"
            :counts="cellActions.dryRunCounts.value"
            :result="result"
            :operating="operating"
            @execute="execute"
            @back-to-matrix="backToMatrix"
          />
        </div>

        <!-- Footer nav -->
        <div
          v-if="step !== 3 || result"
          class="flex shrink-0 items-center gap-2 border-t px-4 py-2.5"
          style="border-color: var(--c-border);"
        >
          <button
            v-if="step > 1"
            type="button"
            class="rounded-md px-3 py-1.5 text-[11px] cursor-pointer"
            style="color: var(--c-text-secondary); border: 1px solid var(--c-border);"
            :disabled="operating"
            @click="goPrev"
          >
            {{ t("batch.prev_step") }}
          </button>
          <div class="flex-1" />
          <button
            v-if="step < 3"
            type="button"
            class="rounded-md px-4 py-1.5 text-[11px] font-medium cursor-pointer"
            style="background: var(--c-primary); color: white;"
            @click="goNext"
          >
            {{ t("batch.next_step") }}
          </button>
        </div>
      </div>

      <ConfirmDialog
        v-if="showConflictConfirm"
        :title="t('manage.batch_panel_conflict_confirm_title')"
        :message="t('manage.batch_panel_conflict_confirm_msg', { count: cellActions.dryRunCounts.value.conflict })"
        :confirm-text="t('manage.batch_panel_execute')"
        :danger="true"
        @confirm="onConfirmConflict"
        @cancel="showConflictConfirm = false"
      />
    </div>
  </Teleport>
</template>
