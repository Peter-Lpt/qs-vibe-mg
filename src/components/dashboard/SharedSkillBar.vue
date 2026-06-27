<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { SharedSkillInfo } from "../../types";

const { t } = useI18n();

defineProps<{
  sharedSkills: SharedSkillInfo[];
}>();
</script>

<template>
  <div
    class="rounded-lg border p-3"
    style="background: var(--c-surface); border-color: var(--c-border);"
  >
    <h3 class="text-xs font-semibold mb-2" style="color: var(--c-text);">
      {{ t('dashboard.shared_skills') }}
    </h3>
    <div class="flex flex-wrap gap-2">
      <span
        v-for="skill in sharedSkills"
        :key="skill.skill_id"
        class="inline-flex items-center gap-1 text-xs px-2 py-1 rounded-md"
        style="background: rgba(59, 130, 246, 0.1); color: var(--c-primary);"
      >
        {{ skill.skill_name || skill.skill_id }}
        <span
          class="px-1 py-0 rounded text-white"
          style="background: var(--c-primary); font-size: 10px;"
        >
          {{ skill.agent_ids.length }}
        </span>
      </span>
    </div>
  </div>
</template>
