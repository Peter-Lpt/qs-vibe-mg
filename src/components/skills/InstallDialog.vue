<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { open } from "@tauri-apps/plugin-dialog";
import { useEscapeKey } from "../../composables/useEscapeKey";

const { t } = useI18n();
const skillsStore = useSkillsStore();

const emit = defineEmits<{
  close: [];
}>();

useEscapeKey(() => emit("close"));

const sourcePath = ref("");
const installing = ref(false);
const installError = ref<string | null>(null);

async function pickFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("skills.select_folder"),
    });
    if (selected) {
      sourcePath.value = selected;
    }
  } catch (e: unknown) {
    console.error("Failed to open folder picker:", e);
  }
}

async function handleInstall() {
  if (!sourcePath.value.trim()) {
    installError.value = t("skills.source_path_required");
    return;
  }

  installing.value = true;
  installError.value = null;

  try {
    await skillsStore.installSkill(sourcePath.value.trim());
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
        class="rounded-lg p-5 shadow-xl max-w-md w-full mx-4"
        style="background: var(--c-surface); border: 1px solid var(--c-border);"
      >
        <h3 class="text-sm font-semibold mb-3" style="color: var(--c-text);">
          {{ t('skills.install') }}
        </h3>

        <div class="mb-3">
          <label class="text-xs block mb-1" style="color: var(--c-text-secondary);">
            {{ t('settings.skills_dir') }}
          </label>
          <div class="flex gap-1.5">
            <input
              v-model="sourcePath"
              :placeholder="t('skills.source_path_placeholder')"
              class="flex-1 px-3 py-2 text-xs rounded-md border outline-none transition-colors"
              style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
              @keyup.enter="handleInstall"
            />
            <button
              class="px-2.5 py-2 text-xs rounded-md border cursor-pointer shrink-0 btn-ghost"
              @click="pickFolder"
            >
              📁
            </button>
          </div>
          <p class="text-xs mt-1" style="color: var(--c-text-secondary);">
            {{ t('skills.install_hint') }}
          </p>
        </div>

        <div v-if="installError" class="text-xs mb-3" style="color: var(--c-danger);">
          {{ installError }}
        </div>

        <div class="flex justify-end gap-2">
          <button
            class="px-3 py-1.5 text-xs rounded-md border cursor-pointer hover:opacity-80"
            style="border-color: var(--c-border); color: var(--c-text);"
            @click="emit('close')"
            :disabled="installing"
          >
            {{ t('settings.cancel') }}
          </button>
          <button
            class="px-3 py-1.5 text-xs rounded-md cursor-pointer hover:opacity-80"
            style="background: var(--c-primary); color: white;"
            @click="handleInstall"
            :disabled="installing"
          >
            {{ installing ? t('app.loading') : t('skills.install') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
