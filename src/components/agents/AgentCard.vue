<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import { useToast } from "../../composables/useToast";
import { open } from "@tauri-apps/plugin-dialog";
import type { Agent } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  agent: Agent;
  skillCount: number;
}>();

const { t } = useI18n();
const agentsStore = useAgentsStore();
const toast = useToast();
const showRemoveConfirm = ref(false);
const editing = ref(false);
const editName = ref(props.agent.name);
const editDir = ref(props.agent.skills_dir);
const editDetectDir = ref(props.agent.detect_dir ?? "");
const editAdditionalScanDirs = ref((props.agent.additional_scan_dirs ?? []).join("\n"));
const isCommonAgent = computed(() => props.agent.kind === "common" || props.agent.id === "agents-shared" || props.agent.id === "agents-common");
const agentInitial = computed(() => props.agent.name.charAt(0).toUpperCase());
const directoryStatusLabel = computed(() => {
  if (isCommonAgent.value) return t("agents.common_source");
  return props.agent.detected ? t("agents.directory_exists") : t("agents.directory_missing");
});
const directoryStatusTitle = computed(() => {
  if (isCommonAgent.value) return t("agents.common_source_hint");
  if (props.agent.detect_dir) {
    return t("agents.directory_tool_hint", {
      detectDir: props.agent.detect_dir,
      skillsDir: props.agent.skills_dir,
    });
  }
  return props.agent.detected ? t("agents.directory_exists_hint") : t("agents.directory_missing_hint");
});

async function handleRemove() {
  try {
    await agentsStore.removeCustomAgent(props.agent.id);
    showRemoveConfirm.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleSaveEdit() {
  if (!editName.value.trim() || !editDir.value.trim()) return;
  try {
    await agentsStore.updateAgent(props.agent.id, {
      name: editName.value.trim(),
      skillsDir: editDir.value.trim(),
      detectDir: editDetectDir.value.trim(),
      additionalScanDirs: parseAdditionalScanDirs(editAdditionalScanDirs.value),
    });
    editing.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

function cancelEdit() {
  editName.value = props.agent.name;
  editDir.value = props.agent.skills_dir;
  editDetectDir.value = props.agent.detect_dir ?? "";
  editAdditionalScanDirs.value = (props.agent.additional_scan_dirs ?? []).join("\n");
  editing.value = false;
}

function parseAdditionalScanDirs(text: string) {
  return Array.from(
    new Set(
      text
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean),
    ),
  );
}

async function pickDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("agents.pick_folder"),
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
        {{ agentInitial }}
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
            {{ t('agents.custom') }}
          </span>
          <span
            class="text-[10px] px-1.5 py-0.5 rounded-full font-medium shrink-0"
            :style="{
              background: isCommonAgent ? 'var(--c-info-light)' : agent.detected ? 'var(--c-success-light)' : 'var(--c-danger-light)',
              color: isCommonAgent ? 'var(--c-info)' : agent.detected ? 'var(--c-success)' : 'var(--c-danger)',
            }"
            :title="directoryStatusTitle"
          >
            {{ directoryStatusLabel }}
          </span>
        </div>
        <p class="text-[11px] truncate mt-0.5" style="color: var(--c-text-tertiary);">
          {{ agent.skills_dir }}
        </p>
        <p v-if="agent.detect_dir" class="text-[11px] truncate mt-0.5" style="color: var(--c-text-tertiary);">
          {{ t('agents.detect_dir') }}: {{ agent.detect_dir }}
        </p>
        <p v-if="agent.additional_scan_dirs?.length" class="text-[11px] truncate mt-0.5" style="color: var(--c-text-secondary);">
          {{ t('agents.additional_scan_dirs') }}: {{ agent.additional_scan_dirs.length }}
        </p>
        <p
          v-if="isCommonAgent"
          class="text-[11px] mt-1 leading-snug"
          style="color: var(--c-text-secondary);"
        >
          {{ t('agents.common_source_hint') }}
        </p>
      </div>
    </div>

    <div v-if="!editing">
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium" style="color: var(--c-text-secondary);">
          {{ t('agents.skill_count', { count: skillCount }) }}
        </span>
        <div class="flex gap-1">
          <button
            class="text-xs px-2 py-1 rounded-md cursor-pointer hover:bg-[var(--c-primary-light)]"
            style="color: var(--c-primary);"
            @click="editing = true"
          >
            {{ t('agents.edit') }}
          </button>
          <button
            v-if="!agent.auto_detected"
            class="text-xs px-2 py-1 rounded-md cursor-pointer hover:bg-[var(--c-danger-light)]"
            style="color: var(--c-danger);"
            @click="showRemoveConfirm = true"
          >
            {{ t('agents.remove') }}
          </button>
        </div>
      </div>
    </div>

    <div v-else class="space-y-2">
      <input
        v-model="editName"
        class="w-full px-2.5 py-1.5 text-xs rounded-md border outline-none transition-colors"
        style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
        :placeholder="t('agents.name')"
      />
      <div class="flex gap-1.5">
        <input
          v-model="editDir"
          class="flex-1 px-2.5 py-1.5 text-xs rounded-md border outline-none transition-colors"
          style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
          :placeholder="t('agents.skills_dir')"
        />
        <button
          class="px-2.5 py-1.5 text-xs rounded-md border cursor-pointer shrink-0 btn-ghost"
          @click="pickDirectory"
        >
          {{ t('agents.pick_folder') }}
        </button>
      </div>
      <input
        v-model="editDetectDir"
        class="w-full px-2.5 py-1.5 text-xs rounded-md border outline-none transition-colors"
        style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
        :placeholder="t('agents.detect_dir')"
      />
      <textarea
        v-model="editAdditionalScanDirs"
        rows="3"
        class="w-full px-2.5 py-1.5 text-xs rounded-md border outline-none resize-none transition-colors"
        style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
        :placeholder="t('agents.additional_scan_dirs_placeholder')"
      />
      <p class="text-[11px] leading-snug" style="color: var(--c-text-secondary);">
        {{ t('agents.path_config_hint') }}
      </p>
      <div class="flex gap-1.5">
        <button
          class="text-xs px-3 py-1.5 rounded-md cursor-pointer font-medium btn-primary"
          @click="handleSaveEdit"
        >
          {{ t('settings.save') }}
        </button>
        <button
          class="text-xs px-3 py-1.5 rounded-md border cursor-pointer btn-ghost"
          @click="cancelEdit"
        >
          {{ t('settings.cancel') }}
        </button>
      </div>
    </div>

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
