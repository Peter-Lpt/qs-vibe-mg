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
  { bg: "#dbeafe", text: "#1e40af" },
  { bg: "#f3e8ff", text: "#7c3aed" },
  { bg: "#dcfce7", text: "#166534" },
  { bg: "#fef3c7", text: "#92400e" },
  { bg: "#ffe4e6", text: "#9f1239" },
  { bg: "#e0f2fe", text: "#075985" },
  { bg: "#f0fdf4", text: "#166534" },
];

function getTagColor(index: number) {
  return tagColors[index % tagColors.length];
}
</script>

<template>
  <div
    class="rounded-lg p-4 border transition-all cursor-pointer hover:shadow-sm"
    style="background: var(--c-surface); border-color: var(--c-border);"
    @click="showPreview = true"
  >
    <div class="flex items-start gap-2">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <h3
            class="text-sm font-semibold truncate"
            style="color: var(--c-text);"
          >
            {{ skill.name }}
          </h3>
          <span
            v-if="skill.license"
            class="text-xs px-1.5 py-0.5 rounded shrink-0"
            style="background: var(--c-surface-hover); color: var(--c-text-secondary);"
          >
            {{ skill.license }}
          </span>
        </div>
        <p class="text-xs mt-1 line-clamp-2" style="color: var(--c-text-secondary);">
          {{ skill.description || t('skills.none') }}
        </p>
      </div>

      <button
        class="text-xs px-1.5 py-0.5 rounded hover:opacity-80 cursor-pointer shrink-0"
        style="color: var(--c-danger);"
        @click.stop="showDeleteConfirm = true"
        :title="t('skills.delete')"
      >
        &#128465;
      </button>
    </div>

    <div class="flex flex-wrap gap-1 mt-2">
      <span
        v-for="(tag, idx) in agentTags()"
        :key="tag"
        class="text-xs px-1.5 py-0.5 rounded"
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
