import { ref } from "vue";
import { openPath } from "@tauri-apps/plugin-opener";
import { marked } from "marked";
import { useSkillsStore } from "../stores/skills";
import { useToast } from "./useToast";
import type { Skill, SkillSource } from "../types";
import type { AgentStatus } from "./useSkillAgentStatus";
import { actionSuccessLabel } from "./skillActionRegistry";

export type SkillActionSource =
  | Pick<SkillSource, "path" | "from" | "is_symlink" | "symlink_target">
  | { path: string; from?: string; is_symlink?: boolean; symlink_target?: string };

export function useSkillActions(t: (key: string, params?: Record<string, unknown>) => string) {
  const skillsStore = useSkillsStore();
  const toast = useToast();
  const previewContent = ref("");
  const previewLoading = ref(false);

  async function loadPreview(skill: Skill, source?: SkillActionSource) {
    previewLoading.value = true;
    try {
      const md = source?.path
        ? await skillsStore.previewSkillAtPath(source.path)
        : await skillsStore.previewSkill(skill.id);
      previewContent.value = marked.parse(md) as string;
      return previewContent.value;
    } catch (e: unknown) {
      previewContent.value = "";
      toast.show(String(e), "error");
      return "";
    } finally {
      previewLoading.value = false;
    }
  }

  async function reveal(source: SkillActionSource) {
    try {
      await openPath(source.path);
    } catch (e: unknown) {
      toast.show(String(e), "error");
    }
  }

  function copyPath(source: SkillActionSource, target = false) {
    const value = target && source.symlink_target ? source.symlink_target : source.path;
    navigator.clipboard?.writeText(value).then(
      () => toast.show(t("manage.path_copied") || "Path copied", "success"),
      () => toast.show(value, "info")
    );
  }

  async function deleteLibrarySkill(skill: Skill) {
    await skillsStore.deleteSkill(skill.id);
    toast.show(t("skills.delete"), "success");
  }

  async function link(skill: Skill, agentId: string) {
    await skillsStore.createLink(skill.id, agentId);
  }

  async function unlink(skill: Skill, agentId: string, source?: SkillActionSource) {
    await skillsStore.removeLink(skill.id, agentId, source?.path);
  }

  async function syncToLibrary(skill: Skill, agentId: string, source?: SkillActionSource, force = true) {
    await skillsStore.syncToVibe(skill.id, agentId, force, source?.path);
  }

  async function relink(skill: Skill, agentId: string, source?: SkillActionSource) {
    await skillsStore.relink(skill.id, agentId, source?.path);
  }

  async function runAgentAction(skill: Skill, status: AgentStatus) {
    switch (status.action) {
      case "link":
        await skillsStore.createLink(skill.id, status.agent.id);
        break;
      case "unlink":
        await skillsStore.removeLink(skill.id, status.agent.id, status.source?.path);
        break;
      case "sync_to_vibe":
        await skillsStore.syncToVibe(skill.id, status.agent.id, true, status.source?.path);
        break;
      case "replace_with_link":
        await skillsStore.syncToVibe(skill.id, status.agent.id, false, status.source?.path);
        break;
      case "relink":
        await skillsStore.relink(skill.id, status.agent.id, status.source?.path);
        break;
      case "remove_dangling":
        await skillsStore.removeLink(skill.id, status.agent.id, status.source?.path);
        break;
      default:
        return;
    }
    const message = actionSuccessLabel(t, status.action, status.agent.name);
    if (message) toast.show(message, "success");
  }

  return {
    previewContent,
    previewLoading,
    loadPreview,
    reveal,
    copyPath,
    deleteLibrarySkill,
    link,
    unlink,
    syncToLibrary,
    relink,
    runAgentAction,
  };
}
