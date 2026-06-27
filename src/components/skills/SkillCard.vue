<script setup lang="ts">
import { ref } from "vue";
import { useSkillsStore } from "../../stores/skills";
import type { Skill, Agent } from "../../types";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
}>();

const skillsStore = useSkillsStore();
const showDropdown = ref(false);
const linking = ref(false);

async function handleLink(agentId: string) {
  linking.value = true;
  try {
    await skillsStore.createLink(props.skill.id, agentId);
  } catch (e: any) {
    alert(String(e));
  } finally {
    linking.value = false;
    showDropdown.value = false;
  }
}

const availableAgents = () =>
  props.agents.filter(
    (a) => a.detected && !props.skill.linked_agents.includes(a.id)
  );

const sourceLabel = (from: string) => {
  if (from === "vab-lib") return "Library";
  const agent = props.agents.find((a) => a.id === from);
  return agent ? agent.name : from;
};
</script>

<template>
  <div class="rounded-lg p-4 border" style="background: var(--c-surface); border-color: var(--c-border);">
    <div class="flex items-start justify-between">
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-semibold truncate" style="color: var(--c-text);">
          {{ skill.name }}
        </h3>
        <p class="text-xs mt-1 line-clamp-2" style="color: var(--c-text-secondary);">
          {{ skill.description || "No description" }}
        </p>
      </div>
    </div>

    <!-- Sources -->
    <div class="flex flex-wrap gap-1 mt-2">
      <span
        v-for="src in skill.sources"
        :key="src.from"
        class="text-xs px-1.5 py-0.5 rounded"
        :style="{
          background: src.from === 'vab-lib' ? '#dbeafe' : '#f3e8ff',
          color: src.from === 'vab-lib' ? '#1e40af' : '#7c3aed',
        }"
      >
        {{ sourceLabel(src.from) }}
      </span>
    </div>

    <!-- Linked agents -->
    <div class="flex flex-wrap gap-1.5 mt-3">
      <span
        v-for="agentId in skill.linked_agents"
        :key="agentId"
        class="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full"
        style="background: #dcfce7; color: #166534;"
      >
        {{ agentId }}
        <button
          class="ml-0.5 hover:opacity-70 cursor-pointer"
          style="color: #166534;"
          @click="skillsStore.removeLink(skill.id, agentId)"
          title="Remove link"
        >
          ×
        </button>
      </span>

      <!-- Link button -->
      <div class="relative" v-if="availableAgents().length > 0">
        <button
          class="text-xs px-2 py-0.5 rounded-full border cursor-pointer hover:opacity-80"
          style="border-color: var(--c-primary); color: var(--c-primary);"
          @click="showDropdown = !showDropdown"
          :disabled="linking"
        >
          + Link
        </button>
        <div
          v-if="showDropdown"
          class="absolute z-10 top-full left-0 mt-1 rounded-md shadow-lg border min-w-[140px]"
          style="background: var(--c-surface); border-color: var(--c-border);"
        >
          <button
            v-for="agent in availableAgents()"
            :key="agent.id"
            class="block w-full text-left px-3 py-1.5 text-xs hover:opacity-80 cursor-pointer"
            style="color: var(--c-text);"
            @click="handleLink(agent.id)"
          >
            {{ agent.name }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
