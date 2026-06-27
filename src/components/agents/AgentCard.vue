<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import type { Agent, Skill } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  agent: Agent;
  skills: Skill[];
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const showRemoveConfirm = ref(false);

// 获取该 agent 已关联的 skills
const linkedSkills = () =>
  props.skills.filter((s) => s.linked_agents.includes(props.agent.id));

async function handleUnlink(skillId: string) {
  try {
    await skillsStore.removeLink(skillId, props.agent.id);
  } catch (e: unknown) {
    alert(String(e));
  }
}

async function handleRemove() {
  try {
    await agentsStore.removeCustomAgent(props.agent.id);
    showRemoveConfirm.value = false;
  } catch (e: unknown) {
    alert(String(e));
  }
}
</script>

<template>
  <div
    class="rounded-lg p-4 border"
    style="background: var(--c-bg); border-color: var(--c-border);"
  >
    <div class="flex items-center gap-2">
      <span
        class="w-2 h-2 rounded-full"
        :style="{ background: agent.detected ? 'var(--c-success)' : '#94a3b8' }"
      />
      <span class="text-sm font-semibold" style="color: var(--c-text);">
        {{ agent.name }}
      </span>
      <span
        class="text-xs"
        :style="{ color: agent.detected ? 'var(--c-success)' : '#94a3b8' }"
      >
        {{ agent.detected ? t('agents.detected') : t('agents.not_detected') }}
      </span>

      <!-- Remove button for custom agents -->
      <button
        v-if="!agent.auto_detected"
        class="ml-auto text-xs hover:opacity-70 cursor-pointer"
        style="color: var(--c-danger);"
        @click="showRemoveConfirm = true"
        :title="t('agents.remove')"
      >
        🗑
      </button>
    </div>

    <p class="text-xs mt-1 truncate" style="color: var(--c-text-secondary);">
      {{ agent.skills_dir }}
    </p>

    <!-- Linked skills -->
    <div v-if="agent.detected" class="flex flex-wrap gap-1.5 mt-3">
      <span
        v-for="skill in linkedSkills()"
        :key="skill.id"
        class="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full"
        style="background: #dbeafe; color: #1e40af;"
      >
        {{ skill.name }}
        <button
          class="ml-0.5 hover:opacity-70 cursor-pointer"
          style="color: #1e40af;"
          @click="handleUnlink(skill.id)"
          :title="t('skills.delete_link')"
        >
          ×
        </button>
      </span>
      <span
        v-if="linkedSkills().length === 0"
        class="text-xs"
        style="color: var(--c-text-secondary);"
      >
        {{ t('agents.no_links') }}
      </span>
    </div>

    <!-- Remove confirm -->
    <ConfirmDialog
      v-if="showRemoveConfirm"
      :title="t('agents.remove')"
      :message="t('agents.remove_confirm', { name: agent.name })"
      :confirm-text="t('agents.remove')"
      :danger="true"
      @confirm="handleRemove"
      @cancel="showRemoveConfirm = false"
    />
  </div>
</template>
