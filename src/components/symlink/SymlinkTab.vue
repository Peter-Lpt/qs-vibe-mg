<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import AgentExpandable from "./AgentExpandable.vue";
import SyncPreview from "./SyncPreview.vue";

const { t } = useI18n();
const agentsStore = useAgentsStore();

const selectedAgentId = ref<string | null>(null);

const selectedAgent = computed(() =>
  agentsStore.agents.find((a) => a.id === selectedAgentId.value)
);

function selectAgent(agentId: string) {
  selectedAgentId.value = agentId;
  agentsStore.getSkillsTree(agentId);
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t('symlink.title') }}
      </h2>
    </div>

    <div class="flex gap-4" style="min-height: 400px;">
      <div class="w-1/3">
        <div class="space-y-2">
          <AgentExpandable
            v-for="agent in agentsStore.agents.filter(a => a.detected)"
            :key="agent.id"
            :agent="agent"
            :selected="selectedAgentId === agent.id"
            @select="selectAgent"
          />
        </div>
      </div>

      <div class="flex-1">
        <SyncPreview
          v-if="selectedAgent"
          :agent="selectedAgent"
          :tree="agentsStore.skillsTree"
          :loading="agentsStore.treeLoading"
          :syncing="agentsStore.syncing"
          :sync-result="agentsStore.syncResult"
        />
        <div
          v-else
          class="flex items-center justify-center h-full rounded-lg border"
          style="border-color: var(--c-border); color: var(--c-text-secondary);"
        >
          <span class="text-sm">{{ t('symlink.select_agent') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
