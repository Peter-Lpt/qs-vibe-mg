import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Skill, DashboardData, SkillIssue } from "../types";
import { useAgentsStore } from "./agents";

export const useSkillsStore = defineStore("skills", () => {
  const skills = ref<Skill[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref("");
  const searchResults = ref<Skill[]>([]);
  const searching = ref(false);
  const dashboardData = ref<DashboardData | null>(null);
  const dashboardLoading = ref(false);
  const issues = ref<SkillIssue[]>([]);
  const issuesLoading = ref(false);

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

  async function fetchIssues() {
    issuesLoading.value = true;
    try {
      issues.value = await invoke<SkillIssue[]>("detect_issues");
    } catch (e: unknown) {
      console.error("Failed to detect issues:", e);
    } finally {
      issuesLoading.value = false;
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

  return {
    skills,
    loading,
    error,
    searchQuery,
    searchResults,
    searching,
    dashboardData,
    dashboardLoading,
    issues,
    issuesLoading,
    fetchSkills,
    searchSkills,
    getDashboardData,
    fetchIssues,
    createLink,
    removeLink,
    installSkill,
    deleteSkill,
    previewSkill,
  };
});
