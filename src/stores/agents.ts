import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Agent, SkillsTreeNode, SyncResult } from "../types";

export const useAgentsStore = defineStore("agents", () => {
  const agents = ref<Agent[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const skillsTree = ref<SkillsTreeNode | null>(null);
  const treeLoading = ref(false);
  const syncResult = ref<SyncResult | null>(null);
  const syncing = ref(false);

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

  async function getSkillsTree(agentId: string) {
    treeLoading.value = true;
    try {
      skillsTree.value = await invoke<SkillsTreeNode>("get_skills_tree", { agentId });
    } catch (e: unknown) {
      error.value = String(e);
    } finally {
      treeLoading.value = false;
    }
  }

  async function syncAgentToVab(agentId: string) {
    syncing.value = true;
    syncResult.value = null;
    try {
      syncResult.value = await invoke<SyncResult>("sync_agent_to_vab", { agentId });
      await fetchAgents();
    } catch (e: unknown) {
      throw new Error(String(e));
    } finally {
      syncing.value = false;
    }
  }

  async function syncCategoryToVab(agentId: string, categoryPath: string) {
    syncing.value = true;
    syncResult.value = null;
    try {
      syncResult.value = await invoke<SyncResult>("sync_category_to_vab", {
        agentId,
        categoryPath,
      });
      await fetchAgents();
    } catch (e: unknown) {
      throw new Error(String(e));
    } finally {
      syncing.value = false;
    }
  }

  async function removeSync(agentId: string, path?: string) {
    try {
      syncResult.value = null;
      await invoke("remove_sync", { agentId, path: path ?? null });
      await fetchAgents();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function setVabSkillsPath(newPath: string, migrate: boolean) {
    try {
      await invoke("set_vab_skills_path", { newPath, migrate });
      await fetchAgents();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  return {
    agents,
    loading,
    error,
    skillsTree,
    treeLoading,
    syncResult,
    syncing,
    fetchAgents,
    addCustomAgent,
    updateAgent,
    removeCustomAgent,
    getSkillsTree,
    syncAgentToVab,
    syncCategoryToVab,
    removeSync,
    setVabSkillsPath,
  };
});
