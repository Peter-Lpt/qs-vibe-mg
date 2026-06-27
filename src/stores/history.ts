import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { HistoryEntry } from "../types";

export const useHistoryStore = defineStore("history", () => {
  const entries = ref<HistoryEntry[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

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

  const canUndo = ref(false);
  const canRedo = ref(false);

  function updateUndoRedoState() {
    canUndo.value = entries.value.some((e) => !e.undone);
    canRedo.value = entries.value.some((e) => e.undone);
  }

  return {
    entries,
    loading,
    error,
    canUndo,
    canRedo,
    fetchHistory,
    undo,
    redo,
    updateUndoRedoState,
  };
});
