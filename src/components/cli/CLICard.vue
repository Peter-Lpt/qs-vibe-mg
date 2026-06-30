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
      opacity: agent.detected ? 1 : 0.5,
    }"
  >
    <div class="flex items-center gap-2.5 mb-2.5">
      <div
        class="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-semibold shrink-0"
        :style="{
          background: agent.detected ? 'var(--c-primary-light)' : 'var(--c-surface-hover)',
          color: agent.detected ? 'var(--c-primary)' : 'var(--c-text-tertiary)',
        }"
      >
        {{ agent.name.charAt(0).toUpperCase() }}
      </div>
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="text-sm font-medium truncate" style="color: var(--c-text);">
            {{ agent.name }}
          </span>
          <span
            v-if="!agent.auto_detected"
            class="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
            style="background: var(--c-warning-light); color: var(--c-warning);"
          >
            {{ t('cli.custom') }}
          </span>
        </div>
        <p class="text-[11px] truncate mt-0.5" style="color: var(--c-text-tertiary);">
          {{ agent.skills_dir }}
        </p>
      </div>
    </div>

    <div v-if="!editing">
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium" style="color: var(--c-text-secondary);">
          {{ t('cli.skill_count', { count: skillCount }) }}
        </span>
        <div class="flex gap-1">
          <button
            class="text-xs px-2 py-1 rounded-md cursor-pointer transition-colors"
            style="color: var(--c-primary);"
            @click="editing = true"
            @mouseenter="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'var(--c-primary-light)'"
            @mouseleave="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'transparent'"
          >
            {{ t('cli.edit') }}
          </button>
          <button
            v-if="!agent.auto_detected"
            class="text-xs px-2 py-1 rounded-md cursor-pointer transition-colors"
            style="color: var(--c-danger);"
            @click="showRemoveConfirm = true"
            @mouseenter="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'var(--c-danger-light)'"
            @mouseleave="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'transparent'"
          >
            {{ t('cli.remove') }}
          </button>
        </div>
      </div>
    </div>

    <div v-else class="space-y-2">
      <input
        v-model="editName"
        class="w-full px-2.5 py-1.5 text-xs rounded-md border outline-none transition-colors"
        style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
        :placeholder="t('cli.name')"
      />
      <div class="flex gap-1.5">
        <input
          v-model="editDir"
          class="flex-1 px-2.5 py-1.5 text-xs rounded-md border outline-none transition-colors"
          style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
          :placeholder="t('cli.skills_dir')"
        />
        <button
          class="px-2.5 py-1.5 text-xs rounded-md border cursor-pointer transition-colors shrink-0"
          style="border-color: var(--c-border); color: var(--c-text-secondary);"
          @click="pickDirectory"
          @mouseenter="(e: MouseEvent) => { (e.target as HTMLElement).style.background = 'var(--c-surface-hover)'; }"
          @mouseleave="(e: MouseEvent) => { (e.target as HTMLElement).style.background = 'transparent'; }"
        >
          {{ t('cli.pick_folder') }}
        </button>
      </div>
      <div class="flex gap-1.5">
        <button
          class="text-xs px-3 py-1.5 rounded-md cursor-pointer transition-colors font-medium"
          style="background: var(--c-primary); color: white;"
          @click="handleSaveEdit"
          @mouseenter="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'var(--c-primary-hover)'"
          @mouseleave="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'var(--c-primary)'"
        >
          {{ t('settings.save') }}
        </button>
        <button
          class="text-xs px-3 py-1.5 rounded-md border cursor-pointer transition-colors"
          style="border-color: var(--c-border); color: var(--c-text-secondary);"
          @click="cancelEdit"
          @mouseenter="(e: MouseEvent) => { (e.target as HTMLElement).style.background = 'var(--c-surface-hover)'; }"
          @mouseleave="(e: MouseEvent) => { (e.target as HTMLElement).style.background = 'transparent'; }"
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
