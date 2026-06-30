<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import { useSkillsStore } from "../../stores/skills";
import CLICard from "./CLICard.vue";
import AddCLIDialog from "./AddCLIDialog.vue";

const { t } = useI18n();
const agentsStore = useAgentsStore();
const skillsStore = useSkillsStore();
const showAddDialog = ref(false);

function getSkillCount(agentId: string): number {
  return skillsStore.skills.filter(
    (s) => s.sources.some((src) => src.from === agentId)
  ).length;
}

function handleAdded() {
  showAddDialog.value = false;
  agentsStore.fetchAgents();
  skillsStore.fetchSkills();
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-5">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t('cli.title') }}
        <span class="text-sm font-normal ml-1.5" style="color: var(--c-text-secondary);">
          ({{ agentsStore.agents.length }})
        </span>
      </h2>
      <button
        class="text-xs px-3 py-1.5 rounded-md cursor-pointer transition-colors"
        style="background: var(--c-primary); color: white;"
        @click="showAddDialog = true"
        @mouseenter="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'var(--c-primary-hover)'"
        @mouseleave="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'var(--c-primary)'"
      >
        + {{ t('cli.add') }}
      </button>
    </div>

    <div v-if="agentsStore.loading" class="text-sm" style="color: var(--c-text-secondary);">
      {{ t('app.loading') }}
    </div>

    <div v-else-if="agentsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ agentsStore.error }}
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <CLICard
        v-for="agent in agentsStore.agents"
        :key="agent.id"
        :agent="agent"
        :skill-count="getSkillCount(agent.id)"
      />
    </div>

    <AddCLIDialog
      v-if="showAddDialog"
      @close="showAddDialog = false"
      @added="handleAdded"
    />
  </div>
</template>
