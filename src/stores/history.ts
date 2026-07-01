import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { HistoryEntry } from "../types";

export const useHistoryStore = defineStore("history", () => {
  const entries = ref<HistoryEntry[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref("");
  const actionFilter = ref("");
  const operationMessage = ref<{ type: "success" | "error"; text: string } | null>(null);

  async function fetchHistory() {
    loading.value = true;
    error.value = null;
    try {
      entries.value = await invoke<HistoryEntry[]>("get_history");
    } catch (e: unknown) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function undo() {
    try {
      await invoke("undo");
      await fetchHistory();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function redo() {
    try {
      await invoke("redo");
      await fetchHistory();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function undoById(id: string) {
    try {
      await invoke("undo_by_id", { id });
      await fetchHistory();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function redoById(id: string) {
    try {
      await invoke("redo_by_id", { id });
      await fetchHistory();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function clearHistory() {
    try {
      await invoke("clear_history");
      await fetchHistory();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  const canUndo = ref(false);
  const canRedo = ref(false);

  function updateUndoRedoState() {
    canUndo.value = entries.value.some((e) => !e.undone);
    canRedo.value = entries.value.some((e) => e.undone);
  }

  /** 最新可撤销条目的ID（堆栈顶） */
  const latestUndoableId = computed<string | null>(() => {
    const last = entries.value
      .slice()
      .reverse()
      .find((e) => !e.undone);
    return last?.id ?? null;
  });

  /** 最新可重做条目的ID（堆栈顶） */
  const latestRedoableId = computed<string | null>(() => {
    const last = entries.value
      .slice()
      .reverse()
      .find((e) => e.undone);
    return last?.id ?? null;
  });

  /** 按搜索和过滤条件筛选后的条目 */
  const filteredEntries = computed<HistoryEntry[]>(() => {
    let result = entries.value.slice().reverse(); // 最新的在最上面

    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter((e) => e.skill_id.toLowerCase().includes(q));
    }

    if (actionFilter.value) {
      result = result.filter((e) => e.action === actionFilter.value);
    }

    return result;
  });

  return {
    entries,
    loading,
    error,
    searchQuery,
    actionFilter,
    operationMessage,
    canUndo,
    canRedo,
    latestUndoableId,
    latestRedoableId,
    filteredEntries,
    fetchHistory,
    undo,
    redo,
    undoById,
    redoById,
    clearHistory,
    updateUndoRedoState,
  };
});