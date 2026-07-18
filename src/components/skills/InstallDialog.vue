<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { open } from "@tauri-apps/plugin-dialog";
import { useEscapeKey } from "../../composables/useEscapeKey";
import { useSkillsStore } from "../../stores/skills";

const { t } = useI18n();
const skillsStore = useSkillsStore();

const emit = defineEmits<{
  close: [];
}>();

useEscapeKey(() => emit("close"));

const sourceMode = ref<"folder" | "git" | "command">("folder");
const sourceValue = ref("");
const referenceInstall = ref(false);
const installing = ref(false);
const installError = ref<string | null>(null);

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
  if (!sourceValue.value.trim()) {
    installError.value = t("skills.source_path_required");
    return;
  }

  installing.value = true;
  installError.value = null;

  try {
    await skillsStore.installSkillFromSource(
      sourceMode.value,
      sourceValue.value.trim(),
      referenceInstall.value
    );
    emit("close");
  } catch (e: unknown) {
    installError.value = String(e);
  } finally {
    installing.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center"
      style="background: rgba(0, 0, 0, 0.5);"
      @click.self="emit('close')"
    >
      <div
        class="w-full max-w-[560px] mx-4 rounded-xl p-5 shadow-xl"
        style="background: var(--c-surface); border: 1px solid var(--c-border);"
      >
        <h3 class="mb-4 text-sm font-semibold" style="color: var(--c-text);">
          {{ t("skills.install") }}
        </h3>

        <div class="mb-4">
          <div class="mb-2 flex flex-wrap gap-2">
            <button
              v-for="option in modeOptions"
              :key="option.key"
              type="button"
              class="rounded-md border px-3 py-1.5 text-xs transition-colors"
              :class="sourceMode === option.key ? 'font-medium' : ''"
              :style="{
                background: sourceMode === option.key ? 'var(--c-primary)' : 'var(--c-surface)',
                borderColor: 'var(--c-border)',
                color: sourceMode === option.key ? 'white' : 'var(--c-text)',
              }"
              @click="sourceMode = option.key as 'folder' | 'git' | 'command'"
            >
              {{ option.label }}
            </button>
          </div>

          <label class="mb-1 block text-xs" style="color: var(--c-text-secondary);">
            {{ t("skills.install_source_label") }}
          </label>
          <div class="flex gap-2">
            <input
              v-model="sourceValue"
              :placeholder="currentPlaceholder"
              class="flex-1 rounded-md border px-3 py-2 text-xs outline-none transition-colors"
              style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
              @keyup.enter="handleInstall"
            />
            <button
              v-if="sourceMode === 'folder'"
              class="btn-ghost shrink-0 rounded-md border px-3 py-2 text-xs cursor-pointer"
              @click="pickFolder"
            >
              {{ t("skills.select_folder") }}
            </button>
          </div>
          <p class="mt-1 text-xs" style="color: var(--c-text-secondary);">
            {{ currentHint }}
          </p>
        </div>

        <label class="mb-2 flex items-center gap-2 text-xs" style="color: var(--c-text-secondary);">
          <input
            v-model="referenceInstall"
            type="checkbox"
            class="h-3.5 w-3.5 cursor-pointer rounded"
            style="accent-color: var(--c-primary);"
          />
          <span>{{ t("skills.install_reference") }}</span>
        </label>
        <p class="mb-4 text-[11px]" style="color: var(--c-text-secondary);">
          {{ t("skills.install_reference_hint") }}
        </p>

        <div v-if="installError" class="mb-3 text-xs" style="color: var(--c-danger);">
          {{ installError }}
        </div>

        <div class="flex justify-end gap-2">
          <button
            class="rounded-md border px-3 py-1.5 text-xs hover:opacity-80"
            style="border-color: var(--c-border); color: var(--c-text);"
            @click="emit('close')"
            :disabled="installing"
          >
            {{ t("settings.cancel") }}
          </button>
          <button
            class="rounded-md px-3 py-1.5 text-xs hover:opacity-80"
            style="background: var(--c-primary); color: white;"
            @click="handleInstall"
            :disabled="installing"
          >
            {{ installing ? t("app.loading") : t("skills.install") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
