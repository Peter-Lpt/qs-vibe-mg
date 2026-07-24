<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { actionLabel } from "../../composables/skillActionRegistry";
import type {
  BatchResult,
  DryRunCounts,
  DryRunItem,
} from "../../composables/useBatchCellActions";

const props = defineProps<{
  items: DryRunItem[];
  counts: DryRunCounts;
  result: BatchResult | null;
  operating: boolean;
}>();

const emit = defineEmits<{
  (e: "execute"): void;
  (e: "back-to-matrix", key?: string): void;
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();

const importNeededCount = computed(
  () => props.items.filter((item) => item.action === "needs_import").length
);

const previewGroups = computed(() => [
  { key: "execute", label: t("manage.batch_result_success_group"), color: "var(--c-primary)", items: props.items.filter((i) => i.category === "execute") },
  { key: "skip", label: t("manage.batch_result_skipped_group"), color: "var(--c-text-secondary)", items: props.items.filter((i) => i.category === "skip") },
  { key: "conflict", label: t("manage.batch_result_conflict_group"), color: "var(--c-warning)", items: props.items.filter((i) => i.category === "conflict") },
  { key: "blocked", label: t("manage.batch_result_blocked_group"), color: "var(--c-danger)", items: props.items.filter((i) => i.category === "blocked") },
]);

const resultGroups = computed(() => {
  if (!props.result) return [];
  return [
    { key: "success", label: t("manage.batch_result_success_group"), color: "var(--c-success)", items: props.result.success },
    { key: "failed", label: t("manage.batch_result_failed_group"), color: "var(--c-danger)", items: props.result.failed },
    { key: "warnings", label: t("manage.batch_result_warning_group"), color: "var(--c-warning)", items: props.result.warnings },
    { key: "skipped", label: t("manage.batch_result_skipped_group"), color: "var(--c-text-secondary)", items: props.result.skipped },
    { key: "conflicts", label: t("manage.batch_result_conflict_group"), color: "var(--c-warning)", items: props.result.conflicts },
    { key: "blocked", label: t("manage.batch_result_blocked_group"), color: "var(--c-danger)", items: props.result.blocked },
  ].filter((group) => group.items.length > 0);
});

function actionName(action: DryRunItem["action"]): string {
  if (action === "conflict") return t("manage.batch_panel_conflict");
  if (action === "needs_import") return t("manage.batch_panel_needs_import");
  if (action === "skipped") return t("manage.batch_result_skipped");
  return actionLabel(t, action) || action;
}

function resultEntryText(
  entry: DryRunItem | BatchResult["failed"][number] | BatchResult["warnings"][number]
): string {
  if ("item" in entry) {
    if (entry.item) return `${entry.item.skillName} @ ${entry.item.agentName}: ${entry.message}`;
    const skillName = skillsStore.skills.find((s) => s.id === entry.skillId)?.name || entry.skillId;
    const agentName = agentsStore.agents.find((a) => a.id === entry.agentId)?.name || entry.agentId || "?";
    return `${skillName} @ ${agentName}: ${entry.message}`;
  }
  if ("message" in entry) {
    const skillName = skillsStore.skills.find((s) => s.id === entry.skillId)?.name || entry.skillId;
    return `${skillName}: ${entry.message}`;
  }
  return `${entry.skillName} @ ${entry.agentName} · ${entry.reason}`;
}

function failedEntryKey(entry: BatchResult["failed"][number]): string | undefined {
  return entry.item?.key;
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-3">
    <!-- Dry-run 预览 -->
    <template v-if="!result">
      <div class="text-[10px]" style="color: var(--c-text-secondary);">
        {{ t("manage.batch_panel_summary", {
          exec: counts.execute,
          conflict: counts.conflict,
          import: importNeededCount,
          skipped: counts.skipped,
          blocked: counts.blocked,
        }) }}
      </div>

      <div class="min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
        <div
          v-for="group in previewGroups"
          v-show="group.items.length > 0"
          :key="group.key"
          class="rounded border p-2"
          style="border-color: var(--c-border);"
        >
          <div class="mb-1 flex items-center gap-2 text-[10px] font-medium" :style="{ color: group.color }">
            {{ group.label }} ({{ group.items.length }})
            <span v-if="group.key === 'conflict'" class="font-normal" style="color: var(--c-text-secondary);">
              · {{ t("batch.conflict_group_note") }}
            </span>
          </div>
          <div
            v-for="item in group.items.slice(0, 50)"
            :key="item.key + item.category"
            class="grid gap-2 py-0.5 text-[10px]"
            style="grid-template-columns: minmax(110px,1fr) minmax(80px,.7fr) minmax(80px,.6fr) minmax(120px,1fr); color: var(--c-text-secondary);"
          >
            <span class="truncate" :title="item.skillName">{{ item.skillName }}</span>
            <span class="truncate" :title="item.agentName">{{ item.agentName }}</span>
            <span :style="{ color: group.color }">{{ actionName(item.action) }}</span>
            <span class="truncate" :title="item.reason">{{ item.reason }}</span>
          </div>
        </div>
      </div>

      <button
        type="button"
        class="w-full rounded-md px-4 py-2 text-xs font-medium cursor-pointer disabled:opacity-50"
        style="background: var(--c-primary); color: white;"
        :disabled="operating"
        @click="emit('execute')"
      >
        {{ operating ? "..." : t("manage.batch_panel_execute") }}
      </button>
    </template>

    <!-- 执行结果 -->
    <template v-else>
      <div class="min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
        <div
          v-for="group in resultGroups"
          :key="group.key"
          class="rounded border p-2"
          style="border-color: var(--c-border);"
        >
          <div class="mb-1 text-[10px] font-medium" :style="{ color: group.color }">
            {{ group.label }} ({{ group.items.length }})
          </div>
          <div
            v-for="(entry, i) in group.items.slice(0, 50)"
            :key="i"
            class="flex items-center gap-2 py-0.5 text-[10px]"
            style="color: var(--c-text-secondary);"
          >
            <span class="min-w-0 flex-1 truncate">{{ resultEntryText(entry) }}</span>
            <button
              v-if="group.key === 'failed'"
              type="button"
              class="shrink-0 cursor-pointer"
              style="color: var(--c-primary);"
              @click="emit('back-to-matrix', failedEntryKey(entry as BatchResult['failed'][number]))"
            >
              {{ t("batch.back_to_matrix") }}
            </button>
          </div>
        </div>
      </div>

      <button
        type="button"
        class="w-full rounded-md border px-4 py-2 text-xs cursor-pointer"
        style="border-color: var(--c-border); color: var(--c-text);"
        @click="emit('back-to-matrix')"
      >
        {{ t("batch.back_to_matrix") }}
      </button>
    </template>
  </div>
</template>
