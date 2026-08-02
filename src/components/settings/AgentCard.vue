<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import type { Agent } from "../../types";

const props = defineProps<{
  agent: Agent;
  editing: boolean;
}>();

const emit = defineEmits<{
  (e: "edit"): void;
  (e: "cancel-edit"): void;
  (e: "save", updates: { name: string; skillsDir: string; detectDir: string | null }): void;
  (e: "toggle-enabled"): void;
  (e: "remove"): void;
}>();

const { t } = useI18n();

// ── 编辑表单（本地状态，保存时一次性提交给父级） ──
const editName = ref("");
const editPath = ref("");
const editDetectDir = ref("");

function startEditing() {
  editName.value = props.agent.name;
  editPath.value = props.agent.skills_dir;
  editDetectDir.value = props.agent.detect_dir || "";
  emit("edit");
}

async function pickDir(target: "path" | "detect") {
  try {
    const selected = await open({ directory: true, multiple: false, title: t("settings.agent_pick_dir") });
    if (!selected) return;
    const path = String(Array.isArray(selected) ? selected[0] : selected).replace(/\\/g, "/");
    if (target === "path") editPath.value = path;
    else editDetectDir.value = path;
  } catch (e: unknown) {
    console.error(e);
  }
}

function save() {
  if (!editName.value.trim() || !editPath.value.trim()) return;
  emit("save", {
    name: editName.value.trim(),
    skillsDir: editPath.value.trim(),
    detectDir: editDetectDir.value.trim() || null,
  });
}

// ── 展示辅助 ──
const statusInfo = computed(() => {
  if (props.agent.detected) {
    return { text: t("settings.agent_detected"), background: "var(--c-success-light)", color: "var(--c-success)" };
  }
  if (props.agent.tool_detected) {
    return { text: t("settings.agent_tool_only"), background: "var(--c-warning-light)", color: "var(--c-warning)" };
  }
  return { text: t("settings.agent_not_detected"), background: "var(--c-surface-hover)", color: "var(--c-text-secondary)" };
});

const badge = computed(() => {
  const letter = (props.agent.name || props.agent.id).trim().charAt(0).toUpperCase() || "?";
  const color = !props.agent.detected
    ? "var(--c-text-tertiary)"
    : props.agent.enabled
      ? "var(--c-success)"
      : "var(--c-warning)";
  const background = !props.agent.detected
    ? "var(--c-surface-hover)"
    : props.agent.enabled
      ? "var(--c-success-light)"
      : "var(--c-warning-light)";
  return { letter, color, background };
});

const dirTitle = computed(() => {
  const parts = [props.agent.skills_dir];
  if (props.agent.detect_dir) parts.push(`${t("settings.agent_detect_dir")}: ${props.agent.detect_dir}`);
  return parts.join("\n");
});
</script>

<template>
  <div class="agent-row" :class="{ 'agent-row--dim': !agent.enabled }">
    <!-- 查看行（表格行） -->
    <div
      class="agent-table-grid px-3 py-1.5"
      :style="{ background: editing ? 'var(--c-primary-light)' : 'transparent' }"
    >
      <!-- Agent -->
      <div class="flex items-center gap-2 min-w-0">
        <span class="agent-badge" :style="{ background: badge.background, color: badge.color }">
          {{ badge.letter }}
        </span>
        <span class="text-[11px] font-medium truncate" style="color: var(--c-text);" :title="agent.name">
          {{ agent.name }}
        </span>
        <span
          v-if="!agent.auto_detected"
          class="agent-custom-badge"
        >{{ t('agents.custom') }}</span>
      </div>

      <!-- 状态 -->
      <div class="flex items-center gap-1 justify-start">
        <span
          class="agent-state-badge"
          :style="{ background: statusInfo.background, color: statusInfo.color }"
        >{{ statusInfo.text }}</span>
        <span
          v-if="!agent.enabled"
          class="agent-state-badge"
          style="background: var(--c-surface-hover); color: var(--c-text-secondary);"
        >{{ t('settings.agent_disabled_badge') }}</span>
      </div>

      <!-- 技能目录（detect_dir 放 tooltip） -->
      <div class="min-w-0">
        <div
          class="truncate font-mono text-[10px]"
          style="color: var(--c-text-secondary);"
          :title="dirTitle"
        >{{ agent.skills_dir }}</div>
      </div>

      <!-- 操作 -->
      <div class="flex items-center gap-1 justify-end shrink-0">
        <button
          type="button"
          role="switch"
          class="agent-switch"
          :class="{ on: agent.enabled }"
          :aria-checked="agent.enabled"
          :title="agent.enabled ? t('settings.agent_disable') : t('settings.agent_enable')"
          @click="emit('toggle-enabled')"
        >
          <span class="agent-switch__thumb" />
        </button>
        <button
          class="agent-icon-btn"
          style="color: var(--c-primary);"
          :title="t('agents.edit')"
          :disabled="editing"
          @click="startEditing"
        ><Wrench :size="12" /></button>
        <button
          v-if="!agent.auto_detected"
          class="agent-icon-btn"
          style="color: var(--c-danger);"
          :title="t('agents.remove')"
          @click="emit('remove')"
        ><Trash2 :size="12" /></button>
      </div>
    </div>

    <!-- 编辑面板（行下展开） -->
    <Transition
      enter-active-class="transition-all duration-200 ease-out"
      leave-active-class="transition-all duration-200 ease-in"
      enter-from-class="opacity-0 max-h-0 overflow-hidden"
      enter-to-class="opacity-100 max-h-64 overflow-hidden"
      leave-from-class="opacity-100 max-h-64 overflow-hidden"
      leave-to-class="opacity-0 max-h-0 overflow-hidden"
    >
      <div v-if="editing" class="border-t px-3 py-2.5 space-y-1.5" style="border-color: var(--c-border);">
        <input
          v-model="editName"
          class="w-full px-2.5 py-1.5 text-[11px] rounded-md border outline-none transition-colors"
          style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
          :placeholder="t('settings.agent_name_placeholder')"
          @keydown.enter="save"
          @keydown.escape="emit('cancel-edit')"
        />
        <div class="flex gap-1.5">
          <input
            v-model="editPath"
            class="flex-1 px-2.5 py-1.5 text-[11px] rounded-md border outline-none transition-colors"
            style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
            :placeholder="t('settings.agent_path_placeholder')"
            @keydown.enter="save"
            @keydown.escape="emit('cancel-edit')"
          />
          <button
            class="px-2 py-1 text-[11px] rounded-md border cursor-pointer shrink-0 hover:opacity-80 transition-colors"
            style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
            @click="pickDir('path')"
          >{{ t('settings.agent_pick_dir') }}</button>
        </div>
        <div class="flex gap-1.5">
          <input
            v-model="editDetectDir"
            class="flex-1 px-2.5 py-1.5 text-[11px] rounded-md border outline-none transition-colors"
            style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
            :placeholder="t('settings.agent_detect_dir_placeholder')"
            @keydown.enter="save"
            @keydown.escape="emit('cancel-edit')"
          />
          <button
            class="px-2 py-1 text-[11px] rounded-md border cursor-pointer shrink-0 hover:opacity-80 transition-colors"
            style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
            @click="pickDir('detect')"
          >{{ t('settings.agent_pick_dir') }}</button>
        </div>
        <div class="flex gap-1.5">
          <button
            class="text-[11px] px-2.5 py-1 rounded-md cursor-pointer font-medium"
            style="background: var(--c-primary); color: white;"
            @click="save"
          >{{ t('settings.save') }}</button>
          <button
            class="text-[11px] px-2.5 py-1 rounded-md border cursor-pointer hover:opacity-80 transition-colors"
            style="border-color: var(--c-border); color: var(--c-text);"
            @click="emit('cancel-edit')"
          >{{ t('settings.cancel') }}</button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.agent-row {
  border-bottom: 1px solid var(--c-border-subtle);
}

.agent-row:last-child {
  border-bottom: none;
}

.agent-row--dim {
  opacity: 0.55;
}

.agent-badge {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 600;
  flex-shrink: 0;
  user-select: none;
}

.agent-state-badge {
  font-size: 9px;
  padding: 1px 7px;
  border-radius: 999px;
  font-weight: 500;
  white-space: nowrap;
}

.agent-custom-badge {
  font-size: 9px;
  padding: 1px 7px;
  border-radius: 999px;
  font-weight: 500;
  white-space: nowrap;
  background: rgba(14, 165, 233, 0.12);
  color: #0284c7;
}

.agent-icon-btn {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.agent-icon-btn:hover {
  background: var(--c-surface-hover);
}

.agent-icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.agent-switch {
  position: relative;
  width: 32px;
  height: 17px;
  border-radius: 999px;
  background: var(--c-surface-hover);
  border: 1px solid var(--c-border);
  cursor: pointer;
  transition: background 0.2s ease, border-color 0.2s ease;
}

.agent-switch.on {
  background: var(--c-success);
  border-color: var(--c-success);
}

.agent-switch__thumb {
  position: absolute;
  top: 1px;
  left: 1px;
  width: 13px;
  height: 13px;
  border-radius: 50%;
  background: white;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  transition: transform 0.2s ease;
}

.agent-switch.on .agent-switch__thumb {
  transform: translateX(15px);
}
</style>
