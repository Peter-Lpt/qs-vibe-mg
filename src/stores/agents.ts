import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Agent } from "../types";
import { useSkillsStore } from "./skills";

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

  async function updateAgent(agentId: string, updates: { name?: string; skillsDir?: string }): Promise<Agent> {
    try {
      const agent = await invoke<Agent>("update_agent", {
        agentId,
        name: updates.name ?? null,
        skillsDir: updates.skillsDir ?? null,
      });
      await fetchAgents();
      await useSkillsStore().fetchSkills();
      return agent;
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function removeCustomAgent(agentId: string) {
    try {
      await invoke("remove_custom_agent", { agentId });
      await fetchAgents();
      await useSkillsStore().fetchSkills();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function setVibeSkillsPath(newPath: string, migrate: boolean) {
    try {
      await invoke("set_vibe_skills_path", { newPath, migrate });
      await fetchAgents();
      await useSkillsStore().fetchSkills();
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
    updateAgent,
    removeCustomAgent,
    setVibeSkillsPath,
  };
});
