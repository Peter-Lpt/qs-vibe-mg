import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Skill, SkillIssue, SkillUpdateCheck } from "../types";
import { useAgentsStore } from "./agents";

export interface SyncActionResult {
  synced_count: number;
  errors: string[];
  warnings: string[];
}

// TTL 缓存：避免频繁检测更新
const UPDATE_CHECK_TTL_MS = 60 * 60 * 1000; // 60 分钟
const updateCheckCache = new Map<string, { timestamp: number; result: SkillUpdateCheck }>();

function getCachedCheck(skillId: string): SkillUpdateCheck | null {
  const cached = updateCheckCache.get(skillId);
  if (!cached) return null;
  const now = Date.now();
  if (now - cached.timestamp > UPDATE_CHECK_TTL_MS) {
    updateCheckCache.delete(skillId);
    return null;
  }
  return cached.result;
}

function setCachedCheck(skillId: string, result: SkillUpdateCheck) {
  updateCheckCache.set(skillId, { timestamp: Date.now(), result });
}

export const useSkillsStore = defineStore("skills", () => {
  const skills = ref<Skill[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const issues = ref<SkillIssue[]>([]);
  const issuesLoading = ref(false);
  const updateChecks = ref<Record<string, SkillUpdateCheck>>({});

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

  let refreshRequestId = 0;
  async function refreshSkills(): Promise<void> {
    const requestId = ++refreshRequestId;
    try {
      const nextSkills = await invoke<Skill[]>("list_skills");
      if (requestId !== refreshRequestId) return;
      skills.value = nextSkills;
      error.value = null;
    } catch (e: unknown) {
      if (requestId === refreshRequestId) error.value = String(e);
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

  async function detachKeepLocalCopy(skillId: string, agentId: string, sourcePath?: string) {
    await invoke("detach_keep_local_copy", { skillId, agentId, sourcePath: sourcePath ?? null });
    refreshSkills();
    useAgentsStore().fetchAgents();
  }

  async function removeAgentSkillCopy(skillId: string, agentId: string, sourcePath: string) {
    await invoke("remove_agent_skill_copy", { skillId, agentId, sourcePath });
    refreshSkills();
    useAgentsStore().fetchAgents();
  }

  async function installSkill(sourcePath: string, reference = false): Promise<Skill> {
    const skill = await invoke<Skill>("install_skill", { sourcePath, reference });
    const i = skills.value.findIndex((s) => s.id === skill.id);
    if (i >= 0) skills.value[i] = skill;
    else skills.value.push(skill);
    return skill;
  }

  async function installSkillFromSource(
    sourceMode: string,
    sourceValue: string,
    reference = false
  ): Promise<Skill> {
    const skill = await invoke<Skill>("install_skill_from_source", {
      sourceMode,
      sourceValue,
      reference,
    });
    const i = skills.value.findIndex((s) => s.id === skill.id);
    if (i >= 0) skills.value[i] = skill;
    else skills.value.push(skill);
    return skill;
  }

  async function updateSkill(skillId: string, force = false): Promise<Skill> {
    const skill = await invoke<Skill>("update_skill", { skillId, force });
    const i = skills.value.findIndex((s) => s.id === skill.id);
    if (i >= 0) skills.value[i] = skill;
    else skills.value.push(skill);
    return skill;
  }

  async function updatePluginSkillsFromMarketplace(marketplace: string): Promise<string[]> {
    const updatedIds = await invoke<string[]>("update_plugin_skills_from_marketplace", { marketplace });
    // 清除已更新 skill 的缓存
    for (const id of updatedIds) {
      updateCheckCache.delete(id);
    }
    await refreshSkills();
    return updatedIds;
  }

  // 检查插件更新（带 TTL 保护，进入页面时自动调用；复用 checkSkillUpdate 的缓存）
  async function checkPluginUpdates(skillIds: string[]): Promise<Record<string, { available: boolean; error?: string }>> {
    const results: Record<string, { available: boolean; error?: string }> = {};
    for (const id of skillIds) {
      try {
        const check = await checkSkillUpdate(id);
        results[id] = { available: check.available, error: check.error };
      } catch (e) {
        results[id] = { available: false, error: String(e) };
      }
    }
    return results;
  }

  async function checkSkillUpdate(skillId: string, force = false): Promise<SkillUpdateCheck> {
    // 检查缓存（除非强制刷新）
    if (!force) {
      const cached = getCachedCheck(skillId);
      if (cached) {
        updateChecks.value = { ...updateChecks.value, [skillId]: cached };
        return cached;
      }
    }
    const result = await invoke<SkillUpdateCheck>("check_skill_update", { skillId });
    setCachedCheck(skillId, result);
    updateChecks.value = { ...updateChecks.value, [skillId]: result };
    return result;
  }

  async function checkAllSkillUpdates(): Promise<SkillUpdateCheck[]> {
    const results = await invoke<SkillUpdateCheck[]>("check_all_skill_updates");
    // 结果统一写入 TTL 缓存，checkSkillUpdate 后续可命中
    for (const result of results) setCachedCheck(result.skill_id, result);
    updateChecks.value = Object.fromEntries(results.map((result) => [result.skill_id, result]));
    return results;
  }

  // 恢复上次落盘的更新检测结果（重启后无需重新联网即可看到历史状态）
  async function loadUpdateChecks() {
    const results = await invoke<SkillUpdateCheck[]>("load_update_checks");
    if (results.length > 0) {
      updateChecks.value = Object.fromEntries(results.map((result) => [result.skill_id, result]));
    }
  }

  // 更新异常：有 error 的检测结果
  const updateErrors = computed(() => {
    const errors: Record<string, string> = {};
    for (const [skillId, check] of Object.entries(updateChecks.value)) {
      if (check.error) {
        errors[skillId] = check.error;
      }
    }
    return errors;
  });

  // 更新异常数量
  const updateErrorCount = computed(() => Object.keys(updateErrors.value).length);

  // 监听异常数量变化，更新系统托盘菜单
  watch(updateErrorCount, async (count) => {
    try {
      await invoke("update_tray_menu", { errorCount: count });
    } catch {
      // 静默失败，不影响主流程
    }
  });

  async function deleteSkill(skillId: string) {
    await invoke("delete_library_skill", { skillId });
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
    const result = await invoke<string>("sync_to_vibe", {
      skillId,
      agentId,
      force,
      sourcePath: sourcePath ?? null,
    });
    refreshSkills();
    useAgentsStore().fetchAgents();
    return result;
  }

  async function relink(skillId: string, agentId: string, sourcePath?: string): Promise<string> {
    const result = await invoke<string>("relink", {
      skillId,
      agentId,
      sourcePath: sourcePath ?? null,
    });
    refreshSkills();
    useAgentsStore().fetchAgents();
    return result;
  }

  async function replaceWithLibrary(
    skillId: string,
    agentId: string,
    sourcePath?: string
  ): Promise<string> {
    const result = await invoke<string>("replace_with_library", {
      skillId,
      agentId,
      sourcePath: sourcePath ?? null,
    });
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
    const result = await invoke<SyncActionResult>("batch_skill_action", { skillId, agentIds, action });
    if (!silent) {
      refreshSkills();
      useAgentsStore().fetchAgents();
    }
    return result;
  }

  async function fetchPluginSkills(): Promise<Skill[]> {
    return invoke<Skill[]>("list_plugin_skills");
  }

  async function adoptPluginSkill(skillId: string): Promise<Skill> {
    const adoptedSkill = await invoke<Skill>("adopt_plugin_skill", { skillId });
    // 刷新 regular skills 列表以包含新认领的 skill
    await refreshSkills();
    return adoptedSkill;
  }

  return {
    skills,
    loading,
    error,
    issues,
    issuesLoading,
    fetchSkills,
    refreshSkills,
    fetchIssues,
    createLink,
    removeLink,
    detachKeepLocalCopy,
    removeAgentSkillCopy,
    installSkill,
    installSkillFromSource,
    updateSkill,
    updatePluginSkillsFromMarketplace,
    checkSkillUpdate,
    checkAllSkillUpdates,
    loadUpdateChecks,
    checkPluginUpdates,
    updateChecks,
    updateErrors,
    updateErrorCount,
    deleteSkill,
    previewSkill,
    previewSkillAtPath,
    syncToVibe,
    relink,
    replaceWithLibrary,
    batchSkillAction,
    fetchPluginSkills,
    adoptPluginSkill,
  };
});
