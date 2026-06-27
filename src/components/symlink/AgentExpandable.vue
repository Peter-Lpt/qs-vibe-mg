<script setup lang="ts">

import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import type { Agent } from "../../types";

const props = defineProps<{
  agent: Agent;
  selected: boolean;
}>();

const emit = defineEmits<{
  select: [agentId: string];
}>();

const { t } = useI18n();
const agentsStore = useAgentsStore();

async function handleSyncAll() {
  try {
    await agentsStore.syncAgentToVab(props.agent.id);
  } catch (e: unknown) {
    alert(String(e));
  }
}
</script>

<template>
  <div
    class="rounded-lg border transition-all cursor-pointer"
    :style="{
      background: selected ? 'rgba(59, 130, 246, 0.08)' : 'var(--c-surface)',
      borderColor: selected ? 'var(--c-primary)' : 'var(--c-border)',
    }"
    @click="emit('select', agent.id)"
  >
    <div class="flex items-center gap-2 px-3 py-2.5">
      <span
        class="w-2 h-2 rounded-full shrink-0"
        :style="{ background: agent.detected ? 'var(--c-success)' : '#94a3b8' }"
      />
      <span class="text-sm font-medium flex-1" style="color: var(--c-text);">
        {{ agent.name }}
      </span>
      <button
        class="text-xs px-2 py-1 rounded cursor-pointer hover:opacity-80"
        style="background: var(--c-primary); color: white;"
        @click.stop="handleSyncAll"
        :disabled="agentsStore.syncing"
      >
        {{ t('symlink.sync_all') }}
      </button>
    </div>

    <p class="text-xs px-3 pb-2 truncate" style="color: var(--c-text-secondary);">
      {{ agent.skills_dir }}
    </p>
  </div>
</template>
