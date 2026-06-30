<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { DashboardAgent, DashboardSkill } from "../../types";

const { t } = useI18n();

const props = defineProps<{
  agents: DashboardAgent[];
  sharedSkills?: { skill_id: string; skill_name: string; agent_ids: string[] }[];
}>();

const agentList = computed(() => props.agents.filter((a) => a.agent_id !== "vibe-lib"));

const allSkills = computed(() => {
  const map = new Map<string, { skill: DashboardSkill; agentIds: string[] }>();
  for (const agent of agentList.value) {
    for (const skill of agent.skills) {
      const existing = map.get(skill.skill_id);
      if (existing) {
        if (!existing.agentIds.includes(agent.agent_id)) {
          existing.agentIds.push(agent.agent_id);
        }
      } else {
        map.set(skill.skill_id, { skill, agentIds: [agent.agent_id] });
      }
    }
  }
  return Array.from(map.values());
});

const sharedSkills = computed(() =>
  allSkills.value.filter((s) => s.agentIds.length > 1).sort((a, b) => b.agentIds.length - a.agentIds.length)
);

const uniqueSkills = computed(() => allSkills.value.filter((s) => s.agentIds.length === 1));

const agentColors = [
  "#6366f1", "#8b5cf6", "#06b6d4", "#10b981",
  "#f59e0b", "#ef4444", "#ec4899", "#14b8a6",
];

function getAgentColor(i: number) {
  return agentColors[i % agentColors.length];
}

function hasSkill(agentId: string, skillId: string): boolean {
  const agent = agentList.value.find((a) => a.agent_id === agentId);
  return agent?.skills.some((s) => s.skill_id === skillId) ?? false;
}
</script>

<template>
  <div class="rounded-lg border overflow-hidden" style="background: var(--c-surface); border-color: var(--c-border);">
    <div class="flex items-center justify-between px-3 py-2 border-b" style="border-color: var(--c-border);">
      <div class="flex items-center gap-3">
        <h3 class="text-xs font-semibold" style="color: var(--c-text);">{{ t('dashboard.relation_graph') }}</h3>
        <span class="text-xs" style="color: var(--c-text-secondary);">
          {{ allSkills.length }} {{ t('dashboard.skills') }}
          ({{ uniqueSkills.length }} {{ t('dashboard.unique') }}, {{ sharedSkills.length }} {{ t('dashboard.shared') }})
        </span>
      </div>
    </div>

    <div v-if="sharedSkills.length === 0" class="p-4 text-xs text-center" style="color: var(--c-text-secondary);">
      {{ t('dashboard.no_shared_skills') }}
    </div>

    <div v-else class="overflow-x-auto">
      <table class="w-full text-xs">
        <thead>
          <tr style="border-bottom: 1px solid var(--c-border);">
            <th class="px-3 py-2 text-left font-medium sticky left-0" style="background: var(--c-surface); color: var(--c-text-secondary); min-width: 140px;">
              {{ t('dashboard.shared_skills') }}
            </th>
            <th
              v-for="(agent, i) in agentList"
              :key="agent.agent_id"
              class="px-2 py-2 text-center font-medium"
              :style="{ color: getAgentColor(i), minWidth: '60px' }"
            >
              <div class="flex flex-col items-center gap-0.5">
                <span class="truncate max-w-[60px]">{{ agent.agent_name }}</span>
                <span class="text-[10px] opacity-60">{{ agent.skill_count }}</span>
              </div>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(skill, idx) in sharedSkills"
            :key="skill.skill.skill_id"
            class="hover:opacity-80 transition-opacity"
            :style="{
              background: idx % 2 === 0 ? 'transparent' : 'rgba(128,128,128,0.03)',
              borderBottom: '1px solid var(--c-border)',
            }"
          >
            <td class="px-3 py-1.5 sticky left-0" :style="{ background: idx % 2 === 0 ? 'var(--c-surface)' : 'rgba(128,128,128,0.03)' }">
              <div class="flex items-center gap-1.5">
                <span class="w-1.5 h-1.5 rounded-full shrink-0" style="background: #f59e0b;" />
                <span class="truncate" style="color: var(--c-text);">{{ skill.skill.skill_name || skill.skill.skill_id }}</span>
                <span
                  class="text-[10px] px-1 rounded shrink-0"
                  style="background: rgba(245, 158, 11, 0.15); color: #f59e0b;"
                >{{ skill.agentIds.length }}</span>
              </div>
            </td>
            <td
              v-for="(agent, ai) in agentList"
              :key="agent.agent_id"
              class="px-2 py-1.5 text-center"
            >
              <span
                v-if="hasSkill(agent.agent_id, skill.skill.skill_id)"
                class="inline-block w-3 h-3 rounded-full"
                :style="{ background: getAgentColor(ai) }"
                :title="agent.agent_name"
              />
              <span v-else class="inline-block w-3 h-3" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
