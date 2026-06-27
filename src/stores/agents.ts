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
    } catch (e: unknown) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function addCustomAgent(name: string, skillsDir: string): Promise<Agent> {
    try {
      const agent = await invoke<Agent>("add_custom_agent", { name, skillsDir });
      await fetchAgents();
      return agent;
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function removeCustomAgent(agentId: string) {
    try {
      await invoke("remove_custom_agent", { agentId });
      await fetchAgents();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  return {
    agents,
    loading,
    error,
    fetchAgents,
    addCustomAgent,
    removeCustomAgent,
  };
});
