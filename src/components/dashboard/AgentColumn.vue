<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { DashboardAgent } from "../../types";

const { t } = useI18n();

defineProps<{
  agent: DashboardAgent;
}>();

const dotColors = [
  "var(--dot-1)",
  "var(--dot-2)",
  "var(--dot-3)",
  "var(--dot-4)",
  "var(--dot-5)",
  "var(--dot-6)",
  "var(--dot-7)",
  "var(--dot-8)",
];

function getDotColor(index: number) {
  return dotColors[index % dotColors.length];
}
</script>

<template>
  <div
    class="flex-shrink-0 rounded-lg border p-3 min-w-[180px] max-w-[220px]"
    style="background: var(--c-surface); border-color: var(--c-border);"
  >
    <div class="flex items-center gap-2 mb-3">
      <span class="text-sm font-semibold" style="color: var(--c-text);">
        {{ agent.agent_name }}
      </span>
      <span
        class="text-xs px-1.5 py-0.5 rounded-full"
        style="background: var(--c-primary); color: white;"
      >
        {{ agent.skill_count }}
      </span>
    </div>

    <div v-if="agent.skills.length === 0" class="text-xs py-2" style="color: var(--c-text-secondary);">
      {{ t('dashboard.no_skills') }}
    </div>

    <div v-else class="space-y-1 overflow-y-auto" style="max-height: 220px;">
      <div
        v-for="(skill, idx) in agent.skills"
        :key="skill.skill_id"
        class="flex items-center gap-2 px-2 py-1 rounded text-xs"
        :style="{
          background: skill.shared_with.length > 0 ? 'var(--c-primary-light)' : 'transparent',
        }"
      >
        <span
          class="w-2 h-2 rounded-full shrink-0"
          :style="{ background: getDotColor(idx) }"
        />
        <span class="truncate flex-1" style="color: var(--c-text);">
          {{ skill.skill_name || skill.skill_id }}
        </span>
        <span
          v-if="skill.shared_with.length > 0"
          class="text-xs px-1 py-0 rounded shrink-0"
          style="background: var(--c-primary); color: white; font-size: 10px;"
        >
          {{ skill.shared_with.length + 1 }}
        </span>
      </div>
    </div>
  </div>
</template>
