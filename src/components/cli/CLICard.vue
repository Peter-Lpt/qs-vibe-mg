<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import { open } from "@tauri-apps/plugin-dialog";
import type { Agent } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  agent: Agent;
  skillCount: number;
}>();

const { t } = useI18n();
const agentsStore = useAgentsStore();
const showRemoveConfirm = ref(false);
const editing = ref(false);
const editName = ref(props.agent.name);
const editDir = ref(props.agent.skills_dir);

async function handleRemove() {
  try {
    await agentsStore.removeCustomAgent(props.agent.id);
    showRemoveConfirm.value = false;
  } catch (e: unknown) {
    alert(String(e));
  }
}

async function handleSaveEdit() {
  if (!editName.value.trim() || !editDir.value.trim()) return;
  try {
    await agentsStore.updateAgent(props.agent.id, {
      name: editName.value.trim(),
      skillsDir: editDir.value.trim(),
    });
    editing.value = false;
  } catch (e: unknown) {
    alert(String(e));
  }
}

function cancelEdit() {
  editName.value = props.agent.name;
  editDir.value = props.agent.skills_dir;
  editing.value = false;
}

async function pickDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("cli.pick_folder"),
    });
    if (selected) {
      editDir.value = selected;
    }
  } catch (e: unknown) {
    console.error("Failed to open directory picker:", e);
  }
}
</script>

<template>
  <div
    class="rounded-lg p-4 border transition-all"
    :style="{
      background: 'var(--c-surface)',
      borderColor: 'var(--c-border)',
      opacity: agent.detected ? 1 : 0.6,
    }"
  >
    <div class="flex items-center gap-2 mb-2">
      <span
        class="w-2.5 h-2.5 rounded-full shrink-0"
        :style="{ background: agent.detected ? 'var(--c-success)' : '#94a3b8' }"
      />
      <span class="text-sm font-semibold" style="color: var(--c-text);">
        {{ agent.name }}
      </span>
      <span
        v-if="!agent.auto_detected"
        class="text-xs px-1.5 py-0.5 rounded"
        style="background: var(--c-surface-hover); color: var(--c-text-secondary);"
      >
        {{ t('cli.custom') }}
      </span>
    </div>

    <div v-if="!editing">
      <p class="text-xs truncate mb-1" style="color: var(--c-text-secondary);">
        {{ agent.skills_dir }}
      </p>
      <div class="flex items-center justify-between">
        <span class="text-xs" style="color: var(--c-text-secondary);">
          {{ t('cli.skill_count', { count: skillCount }) }}
        </span>
        <div class="flex gap-1">
          <button
            class="text-xs px-1.5 py-0.5 rounded hover:opacity-80 cursor-pointer"
            style="color: var(--c-primary);"
            @click="editing = true"
          >
            {{ t('cli.edit') }}
          </button>
          <button
            v-if="!agent.auto_detected"
            class="text-xs px-1.5 py-0.5 rounded hover:opacity-80 cursor-pointer"
            style="color: var(--c-danger);"
            @click="showRemoveConfirm = true"
          >
            {{ t('cli.remove') }}
          </button>
        </div>
      </div>
    </div>

    <div v-else class="space-y-2">
      <input
        v-model="editName"
        class="w-full px-2 py-1 text-xs rounded border outline-none"
        style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
        :placeholder="t('cli.name')"
      />
      <div class="flex gap-1">
        <input
          v-model="editDir"
          class="flex-1 px-2 py-1 text-xs rounded border outline-none"
          style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
          :placeholder="t('cli.skills_dir')"
        />
        <button
          class="px-2 py-1 text-xs rounded border cursor-pointer hover:opacity-80 shrink-0"
          style="border-color: var(--c-border); color: var(--c-text);"
          @click="pickDirectory"
        >
          {{ t('cli.pick_folder') }}
        </button>
      </div>
      <div class="flex gap-1">
        <button
          class="text-xs px-2 py-1 rounded cursor-pointer hover:opacity-80"
          style="background: var(--c-primary); color: white;"
          @click="handleSaveEdit"
        >
          {{ t('settings.save') }}
        </button>
        <button
          class="text-xs px-2 py-1 rounded border cursor-pointer hover:opacity-80"
          style="border-color: var(--c-border); color: var(--c-text);"
          @click="cancelEdit"
        >
          {{ t('settings.cancel') }}
        </button>
      </div>
    </div>

    <ConfirmDialog
      v-if="showRemoveConfirm"
      :title="t('cli.remove')"
      :message="t('cli.remove_confirm', { name: agent.name })"
      :confirm-text="t('cli.remove')"
      :danger="true"
      @confirm="handleRemove"
      @cancel="showRemoveConfirm = false"
    />
  </div>
</template>
