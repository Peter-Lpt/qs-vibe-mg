<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";

const { t } = useI18n();
const skillsStore = useSkillsStore();

const emit = defineEmits<{
  close: [];
  installed: [];
}>();

const sourcePath = ref("");
const installing = ref(false);
const installError = ref<string | null>(null);

async function handleInstall() {
  if (!sourcePath.value.trim()) {
    installError.value = "Please enter a source path";
    return;
  }

  installing.value = true;
  installError.value = null;

  try {
    await skillsStore.installSkill(sourcePath.value.trim());
    emit("installed");
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
          <input
            v-model="sourcePath"
            placeholder="/path/to/skill-directory"
            class="w-full px-3 py-2 text-xs rounded-md border outline-none"
            style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
            @keyup.enter="handleInstall"
          />
          <p class="text-xs mt-1" style="color: var(--c-text-secondary);">
            Directory must contain a SKILL.md file
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
