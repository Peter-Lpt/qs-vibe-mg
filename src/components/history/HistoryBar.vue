<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useHistoryStore } from "../../stores/history";
import { useSkillsStore } from "../../stores/skills";

const { t } = useI18n();
const historyStore = useHistoryStore();
const skillsStore = useSkillsStore();

onMounted(async () => {
  await historyStore.fetchHistory();
  historyStore.updateUndoRedoState();
});

async function handleUndo() {
  try {
    await historyStore.undo();
    await skillsStore.fetchSkills();
    historyStore.updateUndoRedoState();
  } catch (e: unknown) {
    alert(String(e));
  }
}

async function handleRedo() {
  try {
    await historyStore.redo();
    await skillsStore.fetchSkills();
    historyStore.updateUndoRedoState();
  } catch (e: unknown) {
    alert(String(e));
  }
}

function formatTime(ts: string): string {
  try {
    const date = new Date(ts);
    return date.toLocaleTimeString();
  } catch {
    return ts;
  }
}

function getActionLabel(entry: { action: string; skill_id: string; agent_id?: string; mode?: string; undone: boolean }): string {
  const skill = entry.skill_id;
  const agent = entry.agent_id || "";
  const mode = entry.mode || "symlink";

  switch (entry.action) {
    case "link":
      return t("history.linked", { skill, agent, mode });
    case "unlink":
      return t("history.unlinked", { skill, agent });
    case "install":
      return t("history.installed", { skill });
    case "delete":
      return t("history.deleted", { skill });
    case "batch_link":
      return t("history.batch_linked", { count: skill.split(",").length, agent });
    case "batch_unlink":
      return t("history.batch_unlinked", { count: skill.split(",").length, agent });
    default:
      return entry.action;
  }
}
</script>

<template>
  <div
    class="border-t px-4 py-2 flex items-center gap-3"
    style="border-color: var(--c-border); background: var(--c-surface);"
  >
    <span class="text-xs font-semibold shrink-0" style="color: var(--c-text);">
      {{ t('history.title') }}
    </span>

    <!-- Undo/Redo buttons -->
    <div class="flex gap-1 shrink-0">
      <button
        class="px-2 py-1 text-xs rounded border cursor-pointer hover:opacity-80 disabled:opacity-40 disabled:cursor-not-allowed"
        style="border-color: var(--c-border); color: var(--c-text);"
        :disabled="!historyStore.canUndo"
        @click="handleUndo"
        :title="t('history.undo')"
      >
        ↩ {{ t('history.undo') }}
      </button>
      <button
        class="px-2 py-1 text-xs rounded border cursor-pointer hover:opacity-80 disabled:opacity-40 disabled:cursor-not-allowed"
        style="border-color: var(--c-border); color: var(--c-text);"
        :disabled="!historyStore.canRedo"
        @click="handleRedo"
        :title="t('history.redo')"
      >
        ↪ {{ t('history.redo') }}
      </button>
    </div>

    <!-- History entries -->
    <div class="flex-1 overflow-x-auto">
      <div v-if="historyStore.entries.length === 0" class="text-xs" style="color: var(--c-text-secondary);">
        {{ t('history.empty') }}
      </div>
      <div v-else class="flex gap-2">
        <span
          v-for="entry in historyStore.entries.slice(-5).reverse()"
          :key="entry.id"
          class="text-xs shrink-0 px-2 py-0.5 rounded"
          :class="{ 'line-through opacity-50': entry.undone }"
          style="background: var(--c-bg); color: var(--c-text-secondary);"
          :title="getActionLabel(entry)"
        >
          {{ formatTime(entry.timestamp) }} {{ getActionLabel(entry) }}
        </span>
      </div>
    </div>
  </div>
</template>
