<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAppStore, type Locale, type ThemeMode } from "../../stores/app";
import { useAgentsStore } from "../../stores/agents";
import { open } from "@tauri-apps/plugin-dialog";

const { t } = useI18n();
const appStore = useAppStore();
const agentsStore = useAgentsStore();

const themes: { value: ThemeMode; labelKey: string }[] = [
  { value: "system", labelKey: "settings.theme_system" },
  { value: "light", labelKey: "settings.theme_light" },
  { value: "dark", labelKey: "settings.theme_dark" },
];

const locales: { value: Locale; label: string }[] = [
  { value: "zh", label: "中文" },
  { value: "en", label: "English" },
  { value: "zh-TW", label: "繁體中文" },
];

const showMigrateConfirm = ref(false);
const pendingPath = ref("");
const savingPath = ref(false);

function handleThemeChange(mode: ThemeMode) {
  appStore.setTheme(mode);
}

function handleLocaleChange(loc: Locale) {
  appStore.setLocale(loc);
}

async function pickVabPath() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("settings.pick_vibe_path"),
    });
    if (selected) {
      pendingPath.value = selected;
      showMigrateConfirm.value = true;
    }
  } catch (e: unknown) {
    console.error("Failed to open directory picker:", e);
  }
}

async function handleMigrate(migrate: boolean) {
  savingPath.value = true;
  try {
    await agentsStore.setVabSkillsPath(pendingPath.value, migrate);
    showMigrateConfirm.value = false;
    pendingPath.value = "";
  } catch (e: unknown) {
    alert(String(e));
  } finally {
    savingPath.value = false;
  }
}
</script>

<template>
  <div
    class="fixed inset-0 z-40 flex items-center justify-center"
    style="background: rgba(0, 0, 0, 0.5);"
    @click.self="appStore.showSettings = false"
  >
    <div
      class="rounded-lg shadow-xl max-w-md w-full mx-4 max-h-[80vh] overflow-y-auto"
      style="background: var(--c-surface); border: 1px solid var(--c-border);"
    >
      <div class="flex items-center justify-between p-4 border-b" style="border-color: var(--c-border);">
        <h2 class="text-sm font-semibold" style="color: var(--c-text);">
          {{ t('settings.title') }}
        </h2>
        <button
          class="text-lg hover:opacity-70 cursor-pointer"
          style="color: var(--c-text-secondary);"
          @click="appStore.showSettings = false"
        >
          &times;
        </button>
      </div>

      <div class="p-4 space-y-4">
        <div>
          <label class="text-xs font-medium block mb-1.5" style="color: var(--c-text);">
            {{ t('settings.theme') }}
          </label>
          <div class="flex gap-2">
            <button
              v-for="th in themes"
              :key="th.value"
              class="px-3 py-1.5 text-xs rounded-md border cursor-pointer transition-opacity"
              :style="{
                background: appStore.theme === th.value ? 'var(--c-primary)' : 'transparent',
                color: appStore.theme === th.value ? 'white' : 'var(--c-text)',
                borderColor: appStore.theme === th.value ? 'var(--c-primary)' : 'var(--c-border)',
              }"
              @click="handleThemeChange(th.value)"
            >
              {{ t(th.labelKey) }}
            </button>
          </div>
        </div>

        <div>
          <label class="text-xs font-medium block mb-1.5" style="color: var(--c-text);">
            {{ t('settings.language') }}
          </label>
          <div class="flex gap-2">
            <button
              v-for="loc in locales"
              :key="loc.value"
              class="px-3 py-1.5 text-xs rounded-md border cursor-pointer transition-opacity"
              :style="{
                background: appStore.locale === loc.value ? 'var(--c-primary)' : 'transparent',
                color: appStore.locale === loc.value ? 'white' : 'var(--c-text)',
                borderColor: appStore.locale === loc.value ? 'var(--c-primary)' : 'var(--c-border)',
              }"
              @click="handleLocaleChange(loc.value)"
            >
              {{ loc.label }}
            </button>
          </div>
        </div>

        <div>
          <label class="text-xs font-medium block mb-1.5" style="color: var(--c-text);">
            {{ t('settings.vibe_skills_path') }}
          </label>
          <p class="text-xs mb-2" style="color: var(--c-text-secondary);">
            {{ t('settings.vibe_skills_path_hint') }}
          </p>
          <button
            class="w-full px-3 py-2 text-xs rounded-md border cursor-pointer hover:opacity-80 text-left"
            style="border-color: var(--c-border); color: var(--c-text); background: var(--c-bg);"
            @click="pickVabPath"
          >
            {{ t('settings.pick_vibe_path') }}
          </button>
        </div>
      </div>
    </div>

    <div
      v-if="showMigrateConfirm"
      class="fixed inset-0 z-50 flex items-center justify-center"
      style="background: rgba(0, 0, 0, 0.5);"
    >
      <div
        class="rounded-lg p-5 shadow-xl max-w-sm w-full mx-4"
        style="background: var(--c-surface); border: 1px solid var(--c-border);"
      >
        <h3 class="text-sm font-semibold mb-2" style="color: var(--c-text);">
          {{ t('settings.migrate_title') }}
        </h3>
        <p class="text-xs mb-4" style="color: var(--c-text-secondary);">
          {{ t('settings.migrate_confirm') }}
        </p>
        <div class="flex justify-end gap-2">
          <button
            class="px-3 py-1.5 text-xs rounded-md border cursor-pointer hover:opacity-80"
            style="border-color: var(--c-border); color: var(--c-text);"
            @click="handleMigrate(false)"
            :disabled="savingPath"
          >
            {{ t('settings.migrate_no') }}
          </button>
          <button
            class="px-3 py-1.5 text-xs rounded-md cursor-pointer hover:opacity-80"
            style="background: var(--c-primary); color: white;"
            @click="handleMigrate(true)"
            :disabled="savingPath"
          >
            {{ savingPath ? t('app.loading') : t('settings.migrate_yes') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
