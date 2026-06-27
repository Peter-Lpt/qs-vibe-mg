<script setup lang="ts">
import { useAgentsStore } from "../../stores/agents";
import { useSkillsStore } from "../../stores/skills";
import AgentCard from "./AgentCard.vue";

const agentsStore = useAgentsStore();
const skillsStore = useSkillsStore();
</script>

<template>
  <div>
    <h2 class="text-base font-semibold mb-3" style="color: var(--c-text);">
      Agents
      <span class="text-sm font-normal ml-1" style="color: var(--c-text-secondary);">
        ({{ agentsStore.agents.filter(a => a.detected).length }}/{{ agentsStore.agents.length }})
      </span>
    </h2>

    <div v-if="agentsStore.loading" class="text-sm" style="color: var(--c-text-secondary);">
      Loading...
    </div>

    <div v-else-if="agentsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ agentsStore.error }}
    </div>

    <div v-else class="flex flex-col gap-3">
      <AgentCard
        v-for="agent in agentsStore.agents"
        :key="agent.id"
        :agent="agent"
        :skills="skillsStore.skills"
      />
    </div>
  </div>
</template>
