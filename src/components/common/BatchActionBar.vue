<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Agent } from "../../types";

const { t } = useI18n();

const props = defineProps<{
  selectedCount: number;
  agents: Agent[];
}>();

const emit = defineEmits<{
  linkToAgent: [agentId: string];
  unlinkFromAgent: [agentId: string];
  clearSelection: [];
}>();

const linkableAgents = computed(() => props.agents.filter((a) => a.detected));
</script>

<template>
  <Transition name="slide">
    <div
      v-if="selectedCount > 0"
      class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-3 px-4 py-2.5 rounded-xl shadow-lg"
      style="background: var(--c-surface); border: 1px solid var(--c-border);"
    >
      <span class="text-xs font-medium" style="color: var(--c-primary);">
        {{ t('skills.selected', { count: selectedCount }) }}
      </span>

      <span style="color: var(--c-border);">|</span>

      <div class="flex items-center gap-1.5">
        <select
          class="appearance-none px-2 py-1 text-xs rounded border outline-none cursor-pointer"
          style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text); max-width: 120px;"
          @change="(e: Event) => { const id = (e.target as HTMLSelectElement).value; if (id) { emit('linkToAgent', id); (e.target as HTMLSelectElement).value = ''; } }"
        >
          <option value="">{{ t('skills.link_to_agent') }}...</option>
          <option v-for="agent in linkableAgents" :key="agent.id" :value="agent.id">
            {{ agent.name }}
          </option>
        </select>

        <select
          class="appearance-none px-2 py-1 text-xs rounded border outline-none cursor-pointer"
          style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text); max-width: 120px;"
          @change="(e: Event) => { const id = (e.target as HTMLSelectElement).value; if (id) { emit('unlinkFromAgent', id); (e.target as HTMLSelectElement).value = ''; } }"
        >
          <option value="">{{ t('skills.unlink_hint', { agent: '' }).replace(' ', '') }}...</option>
          <option v-for="agent in linkableAgents" :key="agent.id" :value="agent.id">
            {{ agent.name }}
          </option>
        </select>
      </div>

      <button
        class="text-xs px-2 py-1 rounded cursor-pointer hover:opacity-70"
        style="color: var(--c-text-secondary);"
        @click="emit('clearSelection')"
      >
        {{ t('skills.deselect_all') }}
      </button>
    </div>
  </Transition>
</template>
