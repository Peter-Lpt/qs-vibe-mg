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

  /** 首次加载，显示 loading */
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

  /** 后台静默刷新，不显示 loading */
  async function refreshSkills() {
    try {
      skills.value = await invoke<Skill[]>("list_skills");
    } catch (e: unknown) {
      error.value = String(e);
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
    await invoke("create_link", { skillId, agentId });
    refreshSkills();
    useAgentsStore().fetchAgents();
  }

  async function removeLink(skillId: string, agentId: string) {
    await invoke("remove_link", { skillId, agentId });
    refreshSkills();
    useAgentsStore().fetchAgents();
  }

  async function installSkill(sourcePath: string): Promise<Skill> {
    const skill = await invoke<Skill>("install_skill", { sourcePath });
    refreshSkills();
    return skill;
  }

  async function deleteSkill(skillId: string) {
    await invoke("delete_skill", { skillId });
    refreshSkills();
    useAgentsStore().fetchAgents();
  }

  async function previewSkill(skillId: string): Promise<string> {
    return await invoke<string>("preview_skill", { skillId });
  }

  async function previewSkillAtPath(path: string): Promise<string> {
    return await invoke<string>("preview_skill_at_path", { path });
  }

  async function syncToVibe(skillId: string, agentId: string) {
    await invoke("sync_to_vibe", { skillId, agentId });
    refreshSkills();
    useAgentsStore().fetchAgents();
  }

  async function relink(skillId: string, agentId: string) {
    await invoke("relink", { skillId, agentId });
    refreshSkills();
    useAgentsStore().fetchAgents();
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
    refreshSkills,
    searchSkills,
    getDashboardData,
    fetchIssues,
    createLink,
    removeLink,
    installSkill,
    deleteSkill,
    previewSkill,
    previewSkillAtPath,
    syncToVibe,
    relink,
  };
});
