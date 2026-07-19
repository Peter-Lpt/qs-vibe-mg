<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import { open } from "@tauri-apps/plugin-dialog";
import { useEscapeKey } from "../../composables/useEscapeKey";

const { t } = useI18n();
const agentsStore = useAgentsStore();

const emit = defineEmits<{
  close: [];
  added: [];
}>();

useEscapeKey(() => emit("close"));

const name = ref("");
const agentPath = ref("");
const skillsDir = ref("");
const additionalScanDirsText = ref("");
const adding = ref(false);
const addError = ref<string | null>(null);

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

function updateSkillsDir() {
  if (agentPath.value && !skillsDir.value) {
    const sep = agentPath.value.includes("\\") ? "\\" : "/";
    skillsDir.value = `${agentPath.value}${sep}skills`;
  }
}

async function pickDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("agents.pick_folder"),
    });
    if (selected) {
      agentPath.value = selected;
      updateSkillsDir();
    }
  } catch (e: unknown) {
    console.error("Failed to open directory picker:", e);
  }
}

async function handleAdd() {
  addError.value = null;
  if (!name.value.trim()) {
    addError.value = t("agents.name_required");
    return;
  }
  if (!skillsDir.value.trim()) {
    addError.value = t("agents.skills_dir_required");
    return;
  }

  adding.value = true;
  try {
    await agentsStore.addCustomAgentWithOptions(
      name.value.trim(),
      skillsDir.value.trim(),
      agentPath.value.trim() || undefined,
      parseAdditionalScanDirs(additionalScanDirsText.value)
    );
    emit("added");
    emit("close");
  } catch (e: unknown) {
    addError.value = String(e);
  } finally {
    adding.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      class="modal-backdrop fixed inset-0 z-50 flex items-center justify-center p-4"
      @click.self="emit('close')"
    >
      <div
        class="modal-shell w-full max-w-md"
      >
        <div class="modal-header">
          <h3 class="text-[15px] font-semibold" style="color: var(--c-text);">{{ t('agents.add') }}</h3>
        </div>

        <div class="modal-body space-y-3">
          <div>
            <label class="text-xs block mb-1" style="color: var(--c-text-secondary);">
              {{ t('agents.name') }} *
            </label>
            <input
              v-model="name"
              :placeholder="t('agents.name_placeholder')"
              class="w-full px-3 py-2 text-xs rounded-md border outline-none"
              style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
            />
            <p class="text-xs mt-0.5" style="color: var(--c-text-secondary);">
              {{ t('agents.name_hint') }}
            </p>
          </div>

          <div>
            <label class="text-xs block mb-1" style="color: var(--c-text-secondary);">
              {{ t('agents.detect_dir') }}
            </label>
            <div class="flex gap-2">
              <input
                v-model="agentPath"
                :placeholder="t('agents.path_placeholder')"
                class="flex-1 px-3 py-2 text-xs rounded-md border outline-none"
                style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
              />
              <button
                class="px-3 py-2 text-xs rounded-md border cursor-pointer hover:opacity-80"
                style="border-color: var(--c-border); color: var(--c-text);"
                @click="pickDirectory"
              >
                {{ t('agents.pick_folder') }}
              </button>
            </div>
            <p class="text-xs mt-0.5" style="color: var(--c-text-secondary);">
              {{ t('agents.detect_dir_hint') }}
            </p>
          </div>

          <div>
            <label class="text-xs block mb-1" style="color: var(--c-text-secondary);">
              {{ t('agents.skills_dir') }} *
            </label>
            <input
              v-model="skillsDir"
              :placeholder="t('agents.skills_dir_placeholder')"
              class="w-full px-3 py-2 text-xs rounded-md border outline-none"
              style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
            />
            <p class="text-xs mt-0.5" style="color: var(--c-text-secondary);">
              {{ t('agents.skills_dir_hint') }}
            </p>
            <p class="text-xs mt-1 leading-snug" style="color: var(--c-text-secondary);">
              {{ t('agents.runtime_hint') }}
            </p>
          </div>

          <div>
            <label class="text-xs block mb-1" style="color: var(--c-text-secondary);">
              {{ t('agents.additional_scan_dirs') }}
            </label>
            <textarea
              v-model="additionalScanDirsText"
              rows="3"
              :placeholder="t('agents.additional_scan_dirs_placeholder')"
              class="w-full px-3 py-2 text-xs rounded-md border outline-none resize-none"
              style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
            />
            <p class="text-xs mt-0.5" style="color: var(--c-text-secondary);">
              {{ t('agents.additional_scan_dirs_hint') }}
            </p>
          </div>
        </div>

        <div v-if="addError" class="mt-3 rounded-md px-3 py-2 text-xs" style="background: var(--c-danger-light); color: var(--c-danger);">
          {{ addError }}
        </div>

        <div class="modal-actions">
          <button
            class="px-3 py-1.5 text-xs rounded-md border cursor-pointer hover:opacity-80"
            style="border-color: var(--c-border); color: var(--c-text);"
            @click="emit('close')"
            :disabled="adding"
          >
            {{ t('settings.cancel') }}
          </button>
          <button
            class="px-3 py-1.5 text-xs rounded-md cursor-pointer hover:opacity-80"
            style="background: var(--c-primary); color: white;"
            @click="handleAdd"
            :disabled="adding"
          >
            {{ adding ? t('app.loading') : t('settings.save') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
