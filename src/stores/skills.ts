import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Skill } from "../types";

export const useSkillsStore = defineStore("skills", () => {
  const skills = ref<Skill[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchSkills() {
    loading.value = true;
    error.value = null;
    try {
      skills.value = await invoke<Skill[]>("list_skills");
    } catch (e: any) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function createLink(skillId: string, agentId: string) {
    try {
      await invoke("create_link", { skillId, agentId });
      await fetchSkills();
    } catch (e: any) {
      throw new Error(String(e));
    }
  }

  async function removeLink(skillId: string, agentId: string) {
    try {
      await invoke("remove_link", { skillId, agentId });
      await fetchSkills();
    } catch (e: any) {
      throw new Error(String(e));
    }
  }

  return { skills, loading, error, fetchSkills, createLink, removeLink };
});
