<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Agent, Skill } from "../../types";

const { t } = useI18n();

const props = defineProps<{
  agent: Agent;
  skills: Skill[];
}>();

const emit = defineEmits<{
  filter: [agentId: string];
}>();

const skillCount = computed(() => {
  return props.skills.filter((s) =>
    s.sources.some((src) => src.from === props.agent.id)
  ).length;
});

const linkedCount = computed(() => {
  return props.skills.filter((s) =>
    s.sources.some(
      (src) => src.from === props.agent.id && src.is_symlink
    )
  ).length;
});

const conflictCount = computed(() => {
  return props.skills.filter(
    (s) =>
      s.has_conflict &&
      s.sources.some((src) => src.from === props.agent.id)
  ).length;
});
</script>

<template>
  <div
    class="rounded-lg border p-3 cursor-pointer transition-all hover:shadow-sm"
    style="background: var(--c-surface); border-color: var(--c-border);"
    @click="emit('filter', agent.id)"
  >
    <div class="flex items-center gap-2 mb-2">
      <span
        class="w-2 h-2 rounded-full shrink-0"
        :style="{ background: agent.detected ? 'var(--c-success)' : '#94a3b8' }"
      />
      <span class="text-sm font-medium" style="color: var(--c-text);">
        {{ agent.name }}
      </span>
    </div>

    <div class="flex items-center gap-3 text-[11px]">
      <span style="color: var(--c-text-secondary);">
        {{ skillCount }} {{ t("manage.skill_count") || "skills" }}
      </span>
      <span style="color: var(--c-primary);">
        {{ linkedCount }} {{ t("manage.linked_count") || "linked" }}
      </span>
      <span v-if="conflictCount > 0" style="color: var(--c-warning);">
        {{ conflictCount }} {{ t("manage.conflict_count") || "conflicts" }}
      </span>
    </div>
  </div>
</template>
