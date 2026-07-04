<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import type { Skill, Agent } from "../../types";
import SkillPreview from "./SkillPreview.vue";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = withDefaults(defineProps<{
  skill: Skill;
  agents: Agent[];
  selectable?: boolean;
}>(), {
  selectable: false,
});

const { t } = useI18n();
const skillsStore = useSkillsStore();
const toast = useToast();
const showPreview = ref(false);
const showDeleteConfirm = ref(false);
const isSelected = computed(() => skillsStore.selectedIds.has(props.skill.id));

async function handleDelete() {
  try {
    await skillsStore.deleteSkill(props.skill.id);
    showDeleteConfirm.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
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
  { bg: "var(--c-primary-light)", text: "var(--c-primary)" },
  { bg: "var(--c-purple-light)", text: "var(--c-purple)" },
  { bg: "var(--c-success-light)", text: "var(--c-success)" },
  { bg: "var(--c-warning-light)", text: "var(--c-warning)" },
  { bg: "var(--c-danger-light)", text: "var(--c-danger)" },
  { bg: "var(--c-cyan-light)", text: "var(--c-cyan)" },
];

function getTagColor(index: number) {
  return tagColors[index % tagColors.length];
}
</script>

<template>
  <div
    class="group rounded-lg p-3.5 border cursor-pointer card-hover"
    :style="{
      background: 'var(--c-surface)',
      borderColor: isSelected ? 'var(--c-primary)' : undefined,
      boxShadow: isSelected ? 'var(--shadow-sm)' : undefined,
    }"
    @click="selectable ? skillsStore.toggleSelect(skill.id) : (showPreview = true)"
  >
    <div class="flex items-start gap-2">
      <div v-if="selectable" class="pt-0.5 shrink-0">
        <div
          class="w-4 h-4 rounded border flex items-center justify-center text-[10px] transition-colors"
          :style="{
            background: isSelected ? 'var(--c-primary)' : 'transparent',
            color: isSelected ? 'white' : 'transparent',
            borderColor: isSelected ? 'var(--c-primary)' : 'var(--c-border)',
          }"
        >
          ✓
        </div>
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <h3
            class="text-sm font-medium truncate"
            style="color: var(--c-text);"
          >
            {{ skill.name }}
          </h3>
          <span
            v-if="skillsStore.hasUpdate(skill.id)"
            class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
            style="background: var(--c-amber-light); color: var(--c-amber);"
          >
            {{ t('skills.has_updates') }}
          </span>
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
        class="w-6 h-6 flex items-center justify-center rounded opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer shrink-0 icon-btn-danger"
        @click.stop="showDeleteConfirm = true"
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
