<script setup lang="ts">
import { onMounted } from "vue";
import { useAgentsStore } from "./stores/agents";
import { useSkillsStore } from "./stores/skills";
import AppLayout from "./components/layout/AppLayout.vue";
import SkillLibrary from "./components/skills/SkillLibrary.vue";
import AgentPanel from "./components/agents/AgentPanel.vue";

const agentsStore = useAgentsStore();
const skillsStore = useSkillsStore();

onMounted(async () => {
  await agentsStore.fetchAgents();
  await skillsStore.fetchSkills();
});
</script>

<template>
  <AppLayout>
    <template #left>
      <SkillLibrary />
    </template>
    <template #right>
      <AgentPanel />
    </template>
  </AppLayout>
</template>
