import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Agent } from "../types";

export const useAgentsStore = defineStore("agents", () => {
  const agents = ref<Agent[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchAgents() {
    loading.value = true;
    error.value = null;
    try {
      agents.value = await invoke<Agent[]>("list_agents");
    } catch (e: any) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  return { agents, loading, error, fetchAgents };
});
