<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import type { Skill, Agent } from "../../types";
import SkillPreview from "./SkillPreview.vue";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const showPreview = ref(false);
const showDeleteConfirm = ref(false);

async function handleDelete() {
  try {
    await skillsStore.deleteSkill(props.skill.id);
    showDeleteConfirm.value = false;
  } catch (e: unknown) {
    alert(String(e));
  }
}

const agentTags = () =>
  props.skill.sources
    .filter((s) => s.from !== "vibe-lib")
    .map((s) => {
      const agent = props.agents.find((a) => a.id === s.from);
      return agent ? agent.name : s.from;
    });

const tagColors = [
  { bg: "rgba(37, 99, 235, 0.1)", text: "#2563eb" },
  { bg: "rgba(124, 58, 237, 0.1)", text: "#7c3aed" },
  { bg: "rgba(22, 163, 74, 0.1)", text: "#16a34a" },
  { bg: "rgba(217, 119, 6, 0.1)", text: "#d97706" },
  { bg: "rgba(220, 38, 38, 0.1)", text: "#dc2626" },
  { bg: "rgba(8, 145, 178, 0.1)", text: "#0891b2" },
];

function getTagColor(index: number) {
  return tagColors[index % tagColors.length];
}
</script>

<template>
  <div
    class="group rounded-lg p-3.5 border transition-all cursor-pointer"
    style="background: var(--c-surface); border-color: var(--c-border);"
    @click="showPreview = true"
    @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--c-primary)'; (e.currentTarget as HTMLElement).style.boxShadow = 'var(--shadow-sm)'; }"
    @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--c-border)'; (e.currentTarget as HTMLElement).style.boxShadow = 'none'; }"
  >
    <div class="flex items-start gap-2">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <h3
            class="text-sm font-medium truncate"
            style="color: var(--c-text);"
          >
            {{ skill.name }}
          </h3>
          <span
            v-if="skill.license"
            class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
            style="background: var(--c-surface-hover); color: var(--c-text-secondary);"
          >
            {{ skill.license }}
          </span>
        </div>
        <p class="text-xs mt-1 line-clamp-2 leading-relaxed" style="color: var(--c-text-secondary);">
          {{ skill.description || t('skills.none') }}
        </p>
      </div>

      <button
        class="w-6 h-6 flex items-center justify-center rounded opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer shrink-0"
        style="color: var(--c-text-tertiary);"
        @click.stop="showDeleteConfirm = true"
        @mouseenter="(e: MouseEvent) => { (e.target as HTMLElement).style.color = 'var(--c-danger)'; (e.target as HTMLElement).style.background = 'var(--c-danger-light)'; }"
        @mouseleave="(e: MouseEvent) => { (e.target as HTMLElement).style.color = 'var(--c-text-tertiary)'; (e.target as HTMLElement).style.background = 'transparent'; }"
        :title="t('skills.delete')"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        </svg>
      </button>
    </div>

    <div class="flex flex-wrap gap-1 mt-2.5">
      <span
        v-for="(tag, idx) in agentTags()"
        :key="tag"
        class="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
        :style="{
          background: getTagColor(idx).bg,
          color: getTagColor(idx).text,
        }"
      >
        {{ tag }}
      </span>
    </div>

    <SkillPreview
      v-if="showPreview"
      :skill="skill"
      @close="showPreview = false"
    />

    <ConfirmDialog
      v-if="showDeleteConfirm"
      :title="t('skills.delete')"
      :message="t('skills.delete_confirm', { name: skill.name })"
      :confirm-text="t('skills.delete')"
      :danger="true"
      @confirm="handleDelete"
      @cancel="showDeleteConfirm = false"
    />
  </div>
</template>
