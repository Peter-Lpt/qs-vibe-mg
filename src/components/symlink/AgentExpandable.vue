<script setup lang="ts">
import type { Agent } from "../../types";

defineProps<{
  agent: Agent;
  selected: boolean;
}>();

const emit = defineEmits<{
  select: [agentId: string];
}>();
</script>

<template>
  <div
    class="rounded-lg border transition-all cursor-pointer"
    :style="{
      background: selected ? 'var(--c-primary-light)' : 'var(--c-surface)',
      borderColor: selected ? 'var(--c-primary)' : 'var(--c-border)',
    }"
    @click="emit('select', agent.id)"
  >
    <div class="flex items-center gap-2.5 px-3 py-2.5">
      <span
        class="w-2 h-2 rounded-full shrink-0"
        :style="{ background: agent.detected ? 'var(--c-success)' : '#94a3b8' }"
      />
      <span class="text-sm font-medium flex-1 truncate" style="color: var(--c-text);">
        {{ agent.name }}
      </span>
    </div>

    <p class="text-[11px] px-3 pb-2.5 truncate" style="color: var(--c-text-tertiary);">
      {{ agent.skills_dir }}
    </p>
  </div>
</template>
