import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Skill, DashboardData, SkillIssue } from "../types";
import { useAgentsStore } from "./agents";

export interface SyncActionResult {
  synced_count: number;
  errors: string[];
  warnings: string[];
}

export const useSkillsStore = defineStore("skills", () => {
  const skills = ref<Skill[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref("");
  const searchResults = ref<Skill[]>([]);
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

  /** 后台静默刷新，不显示 loading（P3：防抖，避免快速连续操作触发多次全量重取） */
  let refreshTimer: ReturnType<typeof setTimeout> | null = null;
  async function refreshSkills() {
    if (refreshTimer) clearTimeout(refreshTimer);
    refreshTimer = setTimeout(async () => {
      refreshTimer = null;
      try {
        skills.value = await invoke<Skill[]>("list_skills");
      } catch (e: unknown) {
        error.value = String(e);
      }
    }, 120);
  }

  /** 本地搜索：直接过滤已加载的 skills，去掉后端 search_skills 往返（P3） */
  function searchSkills(query: string) {
    searchQuery.value = query;
    const q = query.trim().toLowerCase();
    if (!q) {
      searchResults.value = [];
      return;
    }
    searchResults.value = skills.value.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        s.description.toLowerCase().includes(q) ||
        s.id.toLowerCase().includes(q)
    );
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

  async function createLink(skillId: string, agentId: string): Promise<string> {
    const result = await invoke<string>("create_link", { skillId, agentId });
    refreshSkills();
    useAgentsStore().fetchAgents();
    return result;
  }

  async function removeLink(skillId: string, agentId: string, sourcePath?: string) {
    await invoke("remove_link", { skillId, agentId, sourcePath: sourcePath ?? null });
    refreshSkills();
    useAgentsStore().fetchAgents();
  }

  async function removeAgentSkillCopy(skillId: string, agentId: string, sourcePath: string) {
    await invoke("remove_agent_skill_copy", { skillId, agentId, sourcePath });
    refreshSkills();
    useAgentsStore().fetchAgents();
  }

  async function installSkill(sourcePath: string): Promise<Skill> {
    const skill = await invoke<Skill>("install_skill", { sourcePath });
    // P3：install_skill 已返回最新 Skill，本地原地更新而非整表重取
    const i = skills.value.findIndex((s) => s.id === skill.id);
    if (i >= 0) skills.value[i] = skill;
    else skills.value.push(skill);
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

  async function syncToVibe(
    skillId: string,
    agentId: string,
    force = false,
    sourcePath?: string
  ): Promise<string> {
    const result = await invoke<string>("sync_to_vibe", { skillId, agentId, force, sourcePath: sourcePath ?? null });
    refreshSkills();
    useAgentsStore().fetchAgents();
    return result;
  }

  async function relink(skillId: string, agentId: string, sourcePath?: string): Promise<string> {
    const result = await invoke<string>("relink", { skillId, agentId, sourcePath: sourcePath ?? null });
    refreshSkills();
    useAgentsStore().fetchAgents();
    return result;
  }

  async function replaceWithLibrary(skillId: string, agentId: string, sourcePath?: string): Promise<string> {
    const result = await invoke<string>("replace_with_library", { skillId, agentId, sourcePath: sourcePath ?? null });
    refreshSkills();
    useAgentsStore().fetchAgents();
    return result;
  }

  async function batchSkillAction(
    skillId: string,
    agentIds: string[],
    action: string,
    silent = false
  ): Promise<SyncActionResult> {
    const result = await invoke<SyncActionResult>(
      "batch_skill_action",
      { skillId, agentIds, action }
    );
    if (!silent) {
      refreshSkills();
      useAgentsStore().fetchAgents();
    }
    return result;
  }

  return {
    skills,
    loading,
    error,
    searchQuery,
    searchResults,
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
    removeAgentSkillCopy,
    installSkill,
    deleteSkill,
    previewSkill,
    previewSkillAtPath,
    syncToVibe,
    relink,
    replaceWithLibrary,
    batchSkillAction,
  };
});
