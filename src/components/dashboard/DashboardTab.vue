<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import AgentColumn from "./AgentColumn.vue";
import RelationGraph from "./RelationGraph.vue";
import EmptyState from "../common/EmptyState.vue";
import SkeletonCard from "../common/SkeletonCard.vue";

const { t } = useI18n();
const skillsStore = useSkillsStore();

onMounted(async () => {
  if (!skillsStore.dashboardData) {
    await skillsStore.getDashboardData();
  }
});

async function refresh() {
  await skillsStore.getDashboardData();
}

const sortedAgents = computed(() => {
  if (!skillsStore.dashboardData) return [];
  return [...skillsStore.dashboardData.agents].sort((a, b) => b.skill_count - a.skill_count);
});
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-5">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t('dashboard.title') }}
      </h2>
      <button
        class="text-xs px-3 py-1.5 rounded-md border cursor-pointer btn-ghost"
        @click="refresh"
      >
        {{ t('dashboard.refresh') }}
      </button>
    </div>

    <div v-if="skillsStore.dashboardLoading" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <SkeletonCard v-for="i in 3" :key="i" />
    </div>

    <div v-else-if="skillsStore.error" class="text-sm py-8 text-center" style="color: var(--c-danger);">
      {{ skillsStore.error }}
    </div>

    <EmptyState
      v-else-if="!skillsStore.dashboardData"
      icon="📊"
      :title="t('dashboard.no_data')"
    />

    <template v-else>
      <div
        class="flex items-center gap-4 mb-5 px-4 py-2.5 rounded-lg text-xs"
        style="background: var(--c-surface); border: 1px solid var(--c-border);"
      >
        <span style="color: var(--c-text);">
          {{ t('dashboard.total_skills', { count: skillsStore.dashboardData.stats.total_skills }) }}
        </span>
        <span style="color: var(--c-text-secondary);">|</span>
        <span style="color: var(--c-primary);">
          {{ t('dashboard.shared_count', { count: skillsStore.dashboardData.stats.shared_count }) }}
        </span>
        <span style="color: var(--c-text-secondary);">|</span>
        <template v-for="(count, agentId) in skillsStore.dashboardData.stats.per_agent_count" :key="agentId">
          <span style="color: var(--c-text-secondary);">
            {{ t('dashboard.per_agent', { agent: agentId, count }) }}
          </span>
        </template>
      </div>

      <div class="flex gap-4 overflow-x-auto pb-3 mb-5" style="max-height: 360px;">
        <AgentColumn
          v-for="agent in sortedAgents"
          :key="agent.agent_id"
          :agent="agent"
        />
      </div>

      <RelationGraph
        v-if="skillsStore.dashboardData.shared_skills.length > 0"
        :agents="skillsStore.dashboardData.agents"
        :shared-skills="skillsStore.dashboardData.shared_skills"
      />
    </template>
  </div>
</template>
