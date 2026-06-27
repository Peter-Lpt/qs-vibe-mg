<script setup lang="ts">
import { useSkillsStore } from "../../stores/skills";
import type { Agent, Skill } from "../../types";

const props = defineProps<{
  agent: Agent;
  skills: Skill[];
}>();

const skillsStore = useSkillsStore();

// 获取该 agent 已关联的 skills
const linkedSkills = () =>
  props.skills.filter((s) => s.linked_agents.includes(props.agent.id));

async function handleUnlink(skillId: string) {
  try {
    await skillsStore.removeLink(skillId, props.agent.id);
  } catch (e: any) {
    alert(String(e));
  }
}
</script>

<template>
  <div class="rounded-lg p-4 border" style="background: var(--c-bg); border-color: var(--c-border);">
    <div class="flex items-center gap-2">
      <span
        class="w-2 h-2 rounded-full"
        :style="{ background: agent.detected ? 'var(--c-success)' : '#94a3b8' }"
      />
      <span class="text-sm font-semibold" style="color: var(--c-text);">
        {{ agent.name }}
      </span>
      <span class="text-xs ml-auto" :style="{ color: agent.detected ? 'var(--c-success)' : '#94a3b8' }">
        {{ agent.detected ? 'Detected' : 'Not installed' }}
      </span>
    </div>

    <p class="text-xs mt-1 truncate" style="color: var(--c-text-secondary);">
      {{ agent.skills_dir }}
    </p>

    <!-- Linked skills -->
    <div v-if="agent.detected" class="flex flex-wrap gap-1.5 mt-3">
      <span
        v-for="skill in linkedSkills()"
        :key="skill.id"
        class="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full"
        style="background: #dbeafe; color: #1e40af;"
      >
        {{ skill.name }}
        <button
          class="ml-0.5 hover:opacity-70 cursor-pointer"
          style="color: #1e40af;"
          @click="handleUnlink(skill.id)"
          title="Remove link"
        >
          ×
        </button>
      </span>
      <span v-if="linkedSkills().length === 0" class="text-xs" style="color: var(--c-text-secondary);">
        No linked skills
      </span>
    </div>
  </div>
</template>
