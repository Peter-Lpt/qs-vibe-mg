<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import { useSkillsStore } from "../../stores/skills";
import AgentCard from "./AgentCard.vue";
import AddAgentDialog from "./AddAgentDialog.vue";
import EmptyState from "../common/EmptyState.vue";
import SkeletonCard from "../common/SkeletonCard.vue";

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
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-5">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t('agents.title') }}
        <span class="text-sm font-normal ml-1.5" style="color: var(--c-text-secondary);">
          ({{ agentsStore.agents.length }})
        </span>
      </h2>
      <button
        class="text-xs px-3 py-1.5 rounded-md cursor-pointer btn-primary"
        @click="showAddDialog = true"
      >
        + {{ t('agents.add') }}
      </button>
    </div>

    <div v-if="agentsStore.loading" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <SkeletonCard v-for="i in 3" :key="i" />
    </div>

    <div v-else-if="agentsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ agentsStore.error }}
    </div>

    <EmptyState
      v-else-if="agentsStore.agents.length === 0"
      icon="🤖"
      :title="t('agents.empty_title')"
      :description="t('agents.empty_hint')"
      :action-label="'+ ' + t('agents.add')"
      @action="showAddDialog = true"
    />

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <AgentCard
        v-for="agent in agentsStore.agents"
        :key="agent.id"
        :agent="agent"
        :skill-count="getSkillCount(agent.id)"
      />
    </div>

    <AddAgentDialog
      v-if="showAddDialog"
      @close="showAddDialog = false"
      @added="handleAdded"
    />
  </div>
</template>
