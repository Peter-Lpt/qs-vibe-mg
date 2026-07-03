import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Skill, DashboardData } from "../types";
import { useAgentsStore } from "./agents";

export const useSkillsStore = defineStore("skills", () => {
  const skills = ref<Skill[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const selectedIds = ref<Set<string>>(new Set());
  const searchQuery = ref("");
  const searchResults = ref<Skill[]>([]);
  const searching = ref(false);
  const dashboardData = ref<DashboardData | null>(null);
  const dashboardLoading = ref(false);

  async function fetchSkills() {
    loading.value = true;
    error.value = null;
    try {
      skills.value = await invoke<Skill[]>("list_skills");
    } catch (e: unknown) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function searchSkills(query: string) {
    searchQuery.value = query;
    if (!query.trim()) {
      searchResults.value = [];
      return;
    }
    searching.value = true;
    try {
      searchResults.value = await invoke<Skill[]>("search_skills", { query });
    } catch (e: unknown) {
      error.value = String(e);
    } finally {
      searching.value = false;
    }
  }

  async function getDashboardData() {
    dashboardLoading.value = true;
    try {
      dashboardData.value = await invoke<DashboardData>("get_dashboard_data");
    } catch (e: unknown) {
      error.value = String(e);
    } finally {
      dashboardLoading.value = false;
    }
  }

  async function createLink(skillId: string, agentId: string) {
    try {
      await invoke("create_link", { skillId, agentId });
      await fetchSkills();
      await useAgentsStore().fetchAgents();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function removeLink(skillId: string, agentId: string) {
    try {
      await invoke("remove_link", { skillId, agentId });
      await fetchSkills();
      await useAgentsStore().fetchAgents();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function installSkill(sourcePath: string): Promise<Skill> {
    try {
      const skill = await invoke<Skill>("install_skill", { sourcePath });
      await fetchSkills();
      return skill;
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function deleteSkill(skillId: string) {
    try {
      await invoke("delete_skill", { skillId });
      selectedIds.value.delete(skillId);
      await fetchSkills();
      await useAgentsStore().fetchAgents();
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function previewSkill(skillId: string): Promise<string> {
    try {
      return await invoke<string>("preview_skill", { skillId });
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function batchLink(skillIds: string[], agentId: string): Promise<string[]> {
    try {
      const errors = await invoke<string[]>("batch_link", { skillIds, agentId });
      await fetchSkills();
      await useAgentsStore().fetchAgents();
      return errors;
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  async function batchUnlink(skillIds: string[], agentId: string): Promise<string[]> {
    try {
      const errors = await invoke<string[]>("batch_unlink", { skillIds, agentId });
      await fetchSkills();
      await useAgentsStore().fetchAgents();
      return errors;
    } catch (e: unknown) {
      throw new Error(String(e));
    }
  }

  function toggleSelect(skillId: string) {
    if (selectedIds.value.has(skillId)) {
      selectedIds.value.delete(skillId);
    } else {
      selectedIds.value.add(skillId);
    }
  }

  function selectAll() {
    selectedIds.value = new Set(skills.value.map((s) => s.id));
  }

  function deselectAll() {
    selectedIds.value.clear();
  }

  return {
    skills,
    loading,
    error,
    selectedIds,
    searchQuery,
    searchResults,
    searching,
    dashboardData,
    dashboardLoading,
    fetchSkills,
    searchSkills,
    getDashboardData,
    createLink,
    removeLink,
    installSkill,
    deleteSkill,
    previewSkill,
    batchLink,
    batchUnlink,
    toggleSelect,
    selectAll,
    deselectAll,
  };
});
