<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { useEscapeKey } from "../../composables/useEscapeKey";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();

const emit = defineEmits<{
  close: [];
}>();

useEscapeKey(() => emit("close"));

const sourceMode = ref<"folder" | "git" | "command">("folder");
const sourceValue = ref("");
const referenceInstall = ref(false);
const installing = ref(false);
const installError = ref<string | null>(null);

// 安装目标：勾选的 agent 在安装成功后自动链接
const targetAgents = computed(() =>
  agentsStore.agents.filter((a) => a.detected && a.enabled)
);
const selectedTargets = ref<Set<string>>(new Set());
const linking = ref(false);
const busy = computed(() => installing.value || linking.value);
const allTargetsSelected = computed(
  () => targetAgents.value.length > 0 && selectedTargets.value.size === targetAgents.value.length
);

function toggleTarget(agentId: string) {
  const next = new Set(selectedTargets.value);
  if (next.has(agentId)) next.delete(agentId);
  else next.add(agentId);
  selectedTargets.value = next;
}

function toggleAllTargets() {
  selectedTargets.value =
    allTargetsSelected.value
      ? new Set()
      : new Set(targetAgents.value.map((a) => a.id));
}

const modeOptions = computed(() => [
  { key: "folder", label: t("skills.install_mode_folder") },
  { key: "git", label: t("skills.install_mode_git") },
  { key: "command", label: t("skills.install_mode_command") },
]);

const currentPlaceholder = computed(() => {
  if (sourceMode.value === "git") return t("skills.install_git_placeholder");
  if (sourceMode.value === "command") return t("skills.install_command_placeholder");
  return t("skills.source_path_placeholder");
});

const currentHint = computed(() => {
  if (sourceMode.value === "git") return t("skills.install_git_hint");
  if (sourceMode.value === "command") return t("skills.install_command_hint");
  return t("skills.install_hint");
});

async function pickFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("skills.select_folder"),
    });
    if (selected) {
      sourceValue.value = selected;
    }
  } catch (e: unknown) {
    console.error("Failed to open folder picker:", e);
  }
}

async function handleInstall() {
  if (busy.value) return; // 防重复触发
  if (!sourceValue.value.trim()) {
    installError.value = t("skills.source_path_required");
    return;
  }

  installing.value = true;
  installError.value = null;

  try {
    const skill = await skillsStore.installSkillFromSource(
      sourceMode.value,
      sourceValue.value.trim(),
      referenceInstall.value
    );

    const targets = [...selectedTargets.value];
    if (targets.length > 0) {
      linking.value = true;
      const failed: string[] = [];
      for (const agentId of targets) {
        try {
          await skillsStore.createLink(skill.id, agentId, true);
        } catch {
          failed.push(agentsStore.agents.find((a) => a.id === agentId)?.name || agentId);
        }
      }
      await skillsStore.refreshSkills();
      await agentsStore.fetchAgents();
      if (failed.length > 0) {
        installError.value = `${t("skills.install_link_failed", { count: failed.length })}：${failed.join("、")}`;
        return; // 部分失败时留在弹窗展示，可关闭或重试
      }
    }
    emit("close");
  } catch (e: unknown) {
    installError.value = String(e);
  } finally {
    installing.value = false;
    linking.value = false;
  }
}

const submitLabel = computed(() => {
  if (linking.value) return t("skills.install_linking");
  if (!installing.value) return t("skills.install");
  return sourceMode.value === "command"
    ? t("skills.install_command_running")
    : t("app.loading");
});
</script>

<template>
  <Teleport to="body">
    <div
      class="modal-backdrop fixed inset-0 z-50 flex items-center justify-center p-4"
      @click.self="emit('close')"
    >
      <div
        class="modal-shell w-full max-w-[560px]"
      >
        <div class="modal-header">
          <h3 class="text-[15px] font-semibold" style="color: var(--c-text);">{{ t("skills.install") }}</h3>
        </div>

        <div class="modal-body">
          <div class="install-mode-tabs">
            <button
              v-for="option in modeOptions"
              :key="option.key"
              type="button"
              class="install-mode-tab"
              :class="sourceMode === option.key ? 'font-medium' : ''"
              :disabled="busy"
              :style="{
                background: sourceMode === option.key ? 'var(--c-primary)' : 'var(--c-surface)',
                borderColor: 'var(--c-border)',
                color: sourceMode === option.key ? 'white' : 'var(--c-text)',
                opacity: busy ? 0.6 : 1,
                cursor: busy ? 'not-allowed' : 'pointer',
              }"
              @click="sourceMode = option.key as 'folder' | 'git' | 'command'"
            >
              {{ option.label }}
            </button>
          </div>

          <label class="install-field-label">
            {{ t("skills.install_source_label") }}
          </label>
          <div class="install-source-row">
            <input
              v-model="sourceValue"
              :placeholder="currentPlaceholder"
              :disabled="busy"
              class="install-source-input"
              style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
              @keyup.enter="handleInstall"
            />
            <button
              v-if="sourceMode === 'folder'"
              class="install-browse-button"
              :disabled="busy"
              @click="pickFolder"
            >
              {{ t("skills.select_folder") }}
            </button>
          </div>
          <p class="install-field-hint">
            {{ currentHint }}
          </p>
          <label class="install-reference-option">
            <input
              v-model="referenceInstall"
              type="checkbox"
              :disabled="busy"
              class="h-3.5 w-3.5 cursor-pointer rounded"
              style="accent-color: var(--c-primary);"
            />
            <span>{{ t("skills.install_reference") }}</span>
          </label>
          <p class="install-reference-hint">
            {{ t("skills.install_reference_hint") }}
          </p>

          <div v-if="targetAgents.length > 0" class="install-target-section">
            <div class="install-target-header">
              <label class="install-field-label install-target-label">
                {{ t("skills.install_target_label") }}
              </label>
              <label class="install-target-toggle">
                <input
                  type="checkbox"
                  :checked="allTargetsSelected"
                  :disabled="busy"
                  class="h-3.5 w-3.5 cursor-pointer rounded"
                  style="accent-color: var(--c-primary);"
                  @change="toggleAllTargets"
                />
                <span>{{ t("skills.install_target_all") }}</span>
              </label>
            </div>
            <div class="install-target-list">
              <label
                v-for="agent in targetAgents"
                :key="agent.id"
                class="install-target-item"
              >
                <input
                  type="checkbox"
                  :checked="selectedTargets.has(agent.id)"
                  :disabled="busy"
                  class="h-3.5 w-3.5 cursor-pointer rounded shrink-0"
                  style="accent-color: var(--c-primary);"
                  @change="toggleTarget(agent.id)"
                />
                <span class="truncate">{{ agent.name }}</span>
              </label>
            </div>
            <p class="install-field-hint">
              {{ t("skills.install_target_hint") }}
            </p>
          </div>

          <div v-if="installError" class="install-error">
            {{ installError }}
          </div>
        </div>
        <div class="modal-actions install-actions">
          <button
            class="install-cancel-button"
            style="border-color: var(--c-border); color: var(--c-text);"
            @click="emit('close')"
            :disabled="busy"
          >
            {{ t("settings.cancel") }}
          </button>
          <button
            class="install-submit-button"
            style="background: var(--c-primary); color: white;"
            @click="handleInstall"
            :disabled="busy"
          >
            {{ submitLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
