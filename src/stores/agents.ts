import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Agent, SkillsTreeNode, SyncResult } from "../types";
import { useSkillsStore } from "./skills";

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
      syncResult.value = await invoke<SyncResult>("sync_agent_to_vibe", { agentId });
      await fetchAgents();
      await useSkillsStore().fetchSkills();
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
      syncResult.value = await invoke<SyncResult>("sync_category_to_vibe", {
        agentId,
        categoryPath,
      });
      await fetchAgents();
      await useSkillsStore().fetchSkills();
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
      await useSkillsStore().fetchSkills();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function removeSyncSkills(agentId: string, skillNames: string[]): Promise<SyncResult> {
    try {
      const result = await invoke<SyncResult>("remove_sync_skills", { agentId, skillNames });
      syncResult.value = result;
      await fetchAgents();
      await useSkillsStore().fetchSkills();
      return result;
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function setVabSkillsPath(newPath: string, migrate: boolean) {
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
    removeSyncSkills,
    setVabSkillsPath,
  };
});
