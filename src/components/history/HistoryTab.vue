<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useHistoryStore } from "../../stores/history";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import ConfirmDialog from "../common/ConfirmDialog.vue";
import EmptyState from "../common/EmptyState.vue";
import SkeletonCard from "../common/SkeletonCard.vue";

const { t } = useI18n();
const historyStore = useHistoryStore();
const skillsStore = useSkillsStore();
const toast = useToast();

const currentPage = ref(1);
const pageSize = 20;

const actionTypes = [
  { value: "", labelKey: "history.filter_all" },
  { value: "install", labelKey: "history.filter_install" },
  { value: "delete", labelKey: "history.filter_delete" },
  { value: "link", labelKey: "history.filter_link" },
  { value: "unlink", labelKey: "history.filter_unlink" },
  { value: "batch_link", labelKey: "history.filter_batch_link" },
  { value: "batch_unlink", labelKey: "history.filter_batch_unlink" },
];

const totalPages = computed(() =>
  Math.max(1, Math.ceil(historyStore.filteredEntries.length / pageSize))
);

const pagedEntries = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  return historyStore.filteredEntries.slice(start, start + pageSize);
});

watch(
  [() => historyStore.searchQuery, () => historyStore.actionFilter],
  () => {
    currentPage.value = 1;
  }
);

// 当条目数减少时，确保当前页不超出范围
watch(
  () => historyStore.filteredEntries.length,
  () => {
    if (currentPage.value > totalPages.value) {
      currentPage.value = totalPages.value;
    }
  }
);

function formatTime(ts: string): string {
  try {
    const date = new Date(ts);
    const dateStr = date.toLocaleDateString();
    const timeStr = date.toLocaleTimeString();
    return `${dateStr} ${timeStr}`;
  } catch {
    return ts;
  }
}

function getActionLabel(entry: {
  action: string;
  skill_id: string;
  agent_id?: string;
  mode?: string;
  undone: boolean;
}): string {
  const skill = entry.skill_id;
  const agent = entry.agent_id || "";
  const mode = entry.mode || "symlink";

  switch (entry.action) {
    case "link":
      if (mode === "replace_with_library") {
        return t("history.replaced_with_library", { skill, agent });
      }
      return t("history.linked", { skill, agent, mode });
    case "unlink":
      return t("history.unlinked", { skill, agent });
    case "install":
      return t("history.installed", { skill });
    case "delete":
      return t("history.deleted", { skill });
    case "batch_link":
      return t("history.batch_linked", {
        count: skill.split(",").length,
        agent,
      });
    case "batch_unlink":
      return t("history.batch_unlinked", {
        count: skill.split(",").length,
        agent,
      });
    default:
      return entry.action;
  }
}

function getActionIcon(action: string): string {
  switch (action) {
    case "link":
      return "🔗";
    case "unlink":
      return "❌";
    case "install":
      return "📥";
    case "delete":
      return "🗑️";
    case "batch_link":
      return "🔗";
    case "batch_unlink":
      return "❌";
    default:
      return "📋";
  }
}

function getActionColor(action: string): string {
  switch (action) {
    case "link":
    case "batch_link":
      return "var(--c-primary)";
    case "unlink":
    case "batch_unlink":
      return "var(--c-warning)";
    case "install":
      return "var(--c-success)";
    case "delete":
      return "var(--c-danger)";
    default:
      return "var(--c-text-secondary)";
  }
}

function canUndoEntry(entry: { id: string; undone: boolean }): boolean {
  return !entry.undone && historyStore.latestUndoableId === entry.id;
}

function canRedoEntry(entry: { id: string; undone: boolean }): boolean {
  return entry.undone && historyStore.latestRedoableId === entry.id;
}

async function handleEntryUndo(entryId: string) {
  try {
    await historyStore.undoById(entryId);
    await skillsStore.fetchSkills();
    historyStore.updateUndoRedoState();
    toast.show(t("history.undo_success"), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleEntryRedo(entryId: string) {
  try {
    await historyStore.redoById(entryId);
    await skillsStore.fetchSkills();
    historyStore.updateUndoRedoState();
    toast.show(t("history.redo_success"), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

const showClearConfirm = ref(false);

async function handleClearHistory() {
  try {
    await historyStore.clearHistory();
    await skillsStore.fetchSkills();
    historyStore.updateUndoRedoState();
    showClearConfirm.value = false;
    toast.show(t("history.clear_success"), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}
</script>

<template>
  <div>
    <!-- 标题行 -->
    <div class="flex items-center justify-between mb-5">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t('history.tab_title') }}
      </h2>
      <button
        v-if="historyStore.entries.length > 0"
        class="flex items-center gap-1 px-3 py-1.5 text-xs rounded-md border cursor-pointer transition-colors hover:opacity-80"
        style="border-color: var(--c-border); color: var(--c-text-secondary); background: var(--c-surface);"
        @click="showClearConfirm = true"
      >
        🗑 {{ t('history.clear') }}
      </button>
    </div>

    <!-- 搜索+过滤工具栏 -->
    <div class="flex items-center gap-3 mb-4">
      <input
        v-model="historyStore.searchQuery"
        class="flex-1 px-3 py-1.5 text-xs rounded-md border outline-none transition-colors focus:border-[var(--c-primary)]"
        :placeholder="t('history.search_placeholder')"
        style="border-color: var(--c-border); background: var(--c-surface); color: var(--c-text);"
      />
      <div class="relative shrink-0">
        <select
          v-model="historyStore.actionFilter"
          class="appearance-none px-3 py-1.5 pr-8 text-xs rounded-md border outline-none cursor-pointer transition-colors"
          style="border-color: var(--c-border); background: var(--c-surface); color: var(--c-text); min-width: 110px;"
          @focus="($event.target as HTMLElement).style.borderColor = 'var(--c-primary)'"
          @blur="($event.target as HTMLElement).style.borderColor = 'var(--c-border)'"
        >
          <option
            v-for="at in actionTypes"
            :key="at.value"
            :value="at.value"
          >
            {{ t(at.labelKey) }}
          </option>
        </select>
        <svg
          class="absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none"
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          style="color: var(--c-text-secondary);"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </div>
    </div>

    <!-- 加载态 -->
    <div v-if="historyStore.loading" class="space-y-3">
      <SkeletonCard v-for="i in 3" :key="i" :lines="1" />
    </div>

    <!-- 空状态 -->
    <EmptyState
      v-else-if="historyStore.filteredEntries.length === 0"
      icon="🕐"
      :title="historyStore.searchQuery || historyStore.actionFilter ? t('history.no_results') : t('history.empty')"
    />

    <!-- 表格 -->
    <div v-else class="overflow-x-auto">
      <table class="w-full text-xs border-collapse">
        <thead>
          <tr
            class="text-left text-xs font-medium"
            style="color: var(--c-text-secondary); border-bottom: 1px solid var(--c-border);"
          >
            <th class="py-2 pr-3 font-medium">{{ t('history.col_time') }}</th>
            <th class="py-2 pr-3 font-medium">{{ t('history.col_action') }}</th>
            <th class="py-2 pr-3 font-medium">{{ t('history.col_detail') }}</th>
            <th class="py-2 pr-3 font-medium">{{ t('history.col_status') }}</th>
            <th class="py-2 font-medium text-right">{{ t('history.col_actions') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="entry in pagedEntries"
            :key="entry.id"
            class="border-b transition-colors hover:opacity-90"
            :style="{
              borderColor: 'var(--c-border)',
              opacity: entry.undone ? 0.55 : 1,
            }"
          >
            <td class="py-2.5 pr-3 whitespace-nowrap" style="color: var(--c-text-secondary);">
              {{ formatTime(entry.timestamp) }}
            </td>
            <td class="py-2.5 pr-3">
              <span
                class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium"
                :style="{ color: getActionColor(entry.action) }"
              >
                {{ getActionIcon(entry.action) }}
                <span>{{ t(`history.action_${entry.action}`) }}</span>
              </span>
            </td>
            <td class="py-2.5 pr-3" style="color: var(--c-text);">
              {{ getActionLabel(entry) }}
            </td>
            <td class="py-2.5 pr-3">
              <span
                class="inline-flex items-center px-1.5 py-0.5 rounded text-xs"
                :style="{
                  background: entry.undone
                    ? 'var(--c-danger-light)'
                    : 'var(--c-success-light)',
                  color: entry.undone ? 'var(--c-danger)' : 'var(--c-success)',
                }"
              >
                {{ entry.undone ? t('history.status_undone') : t('history.status_normal') }}
              </span>
            </td>
            <td class="py-2.5 text-right">
              <button
                v-if="canUndoEntry(entry)"
                class="px-2 py-1 text-xs rounded border cursor-pointer transition-colors hover:opacity-80"
                style="border-color: var(--c-border); color: var(--c-primary);"
                :title="t('history.undo_entry')"
                @click="handleEntryUndo(entry.id)"
              >
                ↩ {{ t('history.undo_entry') }}
              </button>
              <button
                v-else-if="canRedoEntry(entry)"
                class="px-2 py-1 text-xs rounded border cursor-pointer transition-colors hover:opacity-80"
                style="border-color: var(--c-border); color: var(--c-primary);"
                :title="t('history.redo_entry')"
                @click="handleEntryRedo(entry.id)"
              >
                ↪ {{ t('history.redo_entry') }}
              </button>
              <span
                v-else
                class="text-xs"
                style="color: var(--c-text-secondary);"
              >
                —
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 分页 -->
    <div
      v-if="totalPages > 1"
      class="flex items-center justify-center gap-3 mt-4"
    >
      <button
        class="px-2.5 py-1 text-xs rounded-md border cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-colors hover:opacity-80"
        style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
        :disabled="currentPage <= 1"
        @click="currentPage = Math.max(1, currentPage - 1)"
      >
        ← {{ t('history.prev_page') }}
      </button>
      <span class="text-xs" style="color: var(--c-text-secondary);">
        {{ t('history.page_info', { current: currentPage, total: totalPages }) }}
      </span>
      <button
        class="px-2.5 py-1 text-xs rounded-md border cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed transition-colors hover:opacity-80"
        style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
        :disabled="currentPage >= totalPages"
        @click="currentPage = Math.min(totalPages, currentPage + 1)"
      >
        {{ t('history.next_page') }} →
      </button>
    </div>

    <ConfirmDialog
      v-if="showClearConfirm"
      :title="t('history.clear')"
      :message="t('history.clear_confirm')"
      :confirm-text="t('history.clear_confirm_yes')"
      :danger="true"
      @confirm="handleClearHistory"
      @cancel="showClearConfirm = false"
    />
  </div>
</template>
