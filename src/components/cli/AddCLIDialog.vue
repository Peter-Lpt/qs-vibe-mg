<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import { open } from "@tauri-apps/plugin-dialog";

const { t } = useI18n();
const agentsStore = useAgentsStore();

const emit = defineEmits<{
  close: [];
  added: [];
}>();

const name = ref("");
const cliPath = ref("");
const skillsDir = ref("");
const adding = ref(false);
const addError = ref<string | null>(null);

function updateSkillsDir() {
  if (cliPath.value && !skillsDir.value) {
    const sep = cliPath.value.includes("\\") ? "\\" : "/";
    skillsDir.value = `${cliPath.value}${sep}skills`;
  }
}

async function pickDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("cli.pick_folder"),
    });
    if (selected) {
      cliPath.value = selected;
      updateSkillsDir();
    }
  } catch (e: unknown) {
    console.error("Failed to open directory picker:", e);
  }
}

async function handleAdd() {
  addError.value = null;
  if (!name.value.trim()) {
    addError.value = t("cli.name_required");
    return;
  }
  if (!skillsDir.value.trim()) {
    addError.value = t("cli.skills_dir_required");
    return;
  }

  adding.value = true;
  try {
    await agentsStore.addCustomAgent(name.value.trim(), skillsDir.value.trim());
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
      class="fixed inset-0 z-50 flex items-center justify-center"
      style="background: rgba(0, 0, 0, 0.5);"
      @click.self="emit('close')"
    >
      <div
        class="rounded-lg p-5 shadow-xl max-w-md w-full mx-4"
        style="background: var(--c-surface); border: 1px solid var(--c-border);"
      >
        <h3 class="text-sm font-semibold mb-4" style="color: var(--c-text);">
          {{ t('cli.add') }}
        </h3>

        <div class="space-y-3">
          <div>
            <label class="text-xs block mb-1" style="color: var(--c-text-secondary);">
              {{ t('cli.name') }} *
            </label>
            <input
              v-model="name"
              :placeholder="t('cli.name_placeholder')"
              class="w-full px-3 py-2 text-xs rounded-md border outline-none"
              style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
            />
            <p class="text-xs mt-0.5" style="color: var(--c-text-secondary);">
              {{ t('cli.name_hint') }}
            </p>
          </div>

          <div>
            <label class="text-xs block mb-1" style="color: var(--c-text-secondary);">
              {{ t('cli.path') }}
            </label>
            <div class="flex gap-2">
              <input
                v-model="cliPath"
                :placeholder="t('cli.path_placeholder')"
                class="flex-1 px-3 py-2 text-xs rounded-md border outline-none"
                style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
              />
              <button
                class="px-3 py-2 text-xs rounded-md border cursor-pointer hover:opacity-80"
                style="border-color: var(--c-border); color: var(--c-text);"
                @click="pickDirectory"
              >
                {{ t('cli.pick_folder') }}
              </button>
            </div>
          </div>

          <div>
            <label class="text-xs block mb-1" style="color: var(--c-text-secondary);">
              {{ t('cli.skills_dir') }} *
            </label>
            <input
              v-model="skillsDir"
              :placeholder="t('cli.skills_dir_placeholder')"
              class="w-full px-3 py-2 text-xs rounded-md border outline-none"
              style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
            />
            <p class="text-xs mt-0.5" style="color: var(--c-text-secondary);">
              {{ t('cli.skills_dir_hint') }}
            </p>
          </div>
        </div>

        <div v-if="addError" class="text-xs mt-3" style="color: var(--c-danger);">
          {{ addError }}
        </div>

        <div class="flex justify-end gap-2 mt-4">
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
