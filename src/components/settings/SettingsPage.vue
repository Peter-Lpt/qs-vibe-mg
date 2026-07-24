<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAppStore, type Locale, type ThemeMode } from "../../stores/app";
import { useAgentsStore } from "../../stores/agents";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useToast } from "../../composables/useToast";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const { t } = useI18n();
const appStore = useAppStore();
const agentsStore = useAgentsStore();
const toast = useToast();

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
const pathError = ref<string | null>(null);
const savingProjectRoots = ref(false);
const projectRootsText = ref("");
const projectRootsLoaded = ref(false);
const projectRootsList = computed(() => parseProjectRoots(projectRootsText.value));
const showProjectRootsAdvanced = ref(false);

function handleThemeChange(mode: ThemeMode) {
  appStore.setTheme(mode);
}

function handleLocaleChange(loc: Locale) {
  appStore.setLocale(loc);
}

function handleCheckUpdate() {
  toast.show(t("settings.update_check_unavailable"), "info");
}

function parseProjectRoots(text: string) {
  return Array.from(
    new Set(
      text
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean),
    ),
  );
}

async function loadProjectRoots() {
  try {
    await appStore.fetchConfig();
    await appStore.fetchProjectRootSuggestions();
    projectRootsText.value = (appStore.config?.project_roots ?? []).join("\n");
  } catch (e: unknown) {
    console.error("Failed to load config:", e);
  } finally {
    projectRootsLoaded.value = true;
  }
}

async function saveProjectRoots() {
  savingProjectRoots.value = true;
  try {
    await appStore.updateProjectRoots(projectRootsList.value);
    toast.show(t("settings.project_roots_saved"), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    savingProjectRoots.value = false;
  }
}

async function addProjectRoot() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("settings.project_roots_pick"),
    });
    if (!selected) return;
    const next = new Set(projectRootsList.value);
    next.add(String(selected));
    projectRootsText.value = Array.from(next).join("\n");
  } catch (e: unknown) {
    console.error("Failed to open project root picker:", e);
  }
}

async function addSuggestedRoot(root: string) {
  const next = new Set(projectRootsList.value);
  next.add(root);
  projectRootsText.value = Array.from(next).join("\n");
}

function removeProjectRoot(root: string) {
  projectRootsText.value = projectRootsList.value.filter((item) => item !== root).join("\n");
}

function detectCurrentFolder(root: string) {
  const suggestion = appStore.projectRootSuggestions.find((item) => item.path === root);
  return suggestion?.is_current ?? false;
}

async function pickVibePath() {
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
  pathError.value = null;
  try {
    await agentsStore.setVibeSkillsPath(pendingPath.value, migrate);
    showMigrateConfirm.value = false;
    pendingPath.value = "";
  } catch (e: unknown) {
    pathError.value = String(e);
  } finally {
    savingPath.value = false;
  }
}

async function handleExport() {
  try {
    const json = await appStore.exportData();
    const filePath = await save({
      defaultPath: "vibe-config-backup.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (filePath) {
      await appStore.writeFileToPath(filePath, json);
      toast.show(t("settings.export_success"), "success");
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleImport() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (selected) {
      const content = await appStore.readFileFromPath(selected);
      await appStore.importData(content);
      toast.show(t("settings.import_success"), "success");
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

const projectRootsCount = computed(() => projectRootsList.value.length);

onMounted(() => {
  void loadProjectRoots();
});
</script>

<template>
  <div>
    <h2 class="text-base font-semibold mb-5" style="color: var(--c-text);">{{ t('settings.title') }}</h2>

    <div class="space-y-5">
      <!-- 版本 -->
      <section class="workspace-panel !p-3">
        <div class="flex items-center justify-between gap-3">
          <div>
            <div class="text-xs font-medium" style="color: var(--c-text);">{{ t('settings.version_info') }}</div>
            <div class="text-[11px] mt-1" style="color: var(--c-text-secondary);">{{ t('app.version') }}</div>
          </div>
          <button
            class="text-[11px] px-3 py-1.5 rounded-md cursor-pointer transition-colors"
            style="background: var(--c-surface); color: var(--c-text); border: 1px solid var(--c-border);"
            @click="handleCheckUpdate"
          >
            {{ t('settings.check_updates') }}
          </button>
        </div>
      </section>

      <!-- 主题 -->
      <div>
        <label class="text-xs font-medium block mb-2" style="color: var(--c-text);">{{ t('settings.theme') }}</label>
        <div class="flex gap-2">
          <button
            v-for="th in themes"
            :key="th.value"
            class="px-3 py-1.5 text-xs rounded-md border cursor-pointer transition-colors"
            :style="{
              background: appStore.theme === th.value ? 'var(--c-primary)' : 'var(--c-surface)',
              color: appStore.theme === th.value ? 'white' : 'var(--c-text)',
              borderColor: appStore.theme === th.value ? 'var(--c-primary)' : 'var(--c-border)',
            }"
            @click="handleThemeChange(th.value)"
          >
            {{ t(th.labelKey) }}
          </button>
        </div>
      </div>

      <!-- 语言 -->
      <div>
        <label class="text-xs font-medium block mb-2" style="color: var(--c-text);">{{ t('settings.language') }}</label>
        <div class="flex gap-2">
          <button
            v-for="loc in locales"
            :key="loc.value"
            class="px-3 py-1.5 text-xs rounded-md border cursor-pointer transition-colors"
            :style="{
              background: appStore.locale === loc.value ? 'var(--c-primary)' : 'var(--c-surface)',
              color: appStore.locale === loc.value ? 'white' : 'var(--c-text)',
              borderColor: appStore.locale === loc.value ? 'var(--c-primary)' : 'var(--c-border)',
            }"
            @click="handleLocaleChange(loc.value)"
          >
            {{ loc.label }}
          </button>
        </div>
      </div>

      <!-- 技能库路径 -->
      <div>
        <label class="text-xs font-medium block mb-2" style="color: var(--c-text);">{{ t('settings.vibe_skills_path') }}</label>
        <p class="text-[11px] mb-2" style="color: var(--c-text-secondary);">{{ t('settings.vibe_skills_path_hint') }}</p>
        <button
          class="w-full px-3 py-2 text-xs rounded-md border cursor-pointer hover:opacity-80 text-left transition-colors"
          style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
          @click="pickVibePath"
        >
          {{ t('settings.pick_vibe_path') }}
        </button>
      </div>

      <!-- 项目根目录 -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <label class="text-xs font-medium" style="color: var(--c-text);">{{ t('settings.project_roots') }}</label>
          <div class="flex gap-2">
            <button
              class="px-2.5 py-1 text-[11px] rounded-md border cursor-pointer hover:opacity-80 transition-colors"
              style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
              @click="addProjectRoot"
            >
              {{ t('settings.project_roots_pick') }}
            </button>
            <button
              class="px-2.5 py-1 text-[11px] rounded-md border cursor-pointer hover:opacity-80 transition-colors"
              style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
              @click="showProjectRootsAdvanced = !showProjectRootsAdvanced"
            >
              {{ showProjectRootsAdvanced ? t('settings.project_roots_advanced_hide') : t('settings.project_roots_advanced_show') }}
            </button>
          </div>
        </div>
        <p class="text-[11px] mb-2" style="color: var(--c-text-secondary);">{{ t('settings.project_roots_hint') }}</p>
        <div class="space-y-2">
          <div
            v-if="appStore.projectRootSuggestions.length > 0"
            class="rounded-md border p-2.5 space-y-2"
            style="border-color: var(--c-border); background: var(--c-surface);"
          >
            <div class="text-[11px] font-medium" style="color: var(--c-text-secondary);">{{ t('settings.project_roots_suggestions') }}</div>
            <div
              v-for="suggestion in appStore.projectRootSuggestions"
              :key="suggestion.path"
              class="flex items-center gap-2 justify-between text-[11px]"
            >
              <div class="min-w-0 flex-1">
                <div class="truncate" style="color: var(--c-text);">{{ suggestion.is_current ? t('settings.project_roots_current') : suggestion.path }}</div>
                <div class="truncate" style="color: var(--c-text-tertiary);">
                  <span v-if="suggestion.matched_dirs.length > 0">{{ suggestion.matched_dirs.join(', ') }}</span>
                  <span v-else>{{ t('settings.project_roots_suggestion_hint') }}</span>
                </div>
              </div>
              <button
                class="px-2 py-1 rounded-md border cursor-pointer hover:opacity-80 shrink-0 transition-colors"
                style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
                @click="addSuggestedRoot(suggestion.path)"
              >
                {{ t('settings.project_roots_add') }}
              </button>
            </div>
          </div>
          <div v-if="projectRootsList.length > 0" class="space-y-1.5">
            <div
              v-for="root in projectRootsList"
              :key="root"
              class="flex items-center justify-between gap-3 rounded-md border px-3 py-2 text-[11px]"
              style="border-color: var(--c-border); background: var(--c-surface); color: var(--c-text-secondary);"
            >
              <div class="min-w-0 flex-1">
                <div class="truncate" style="color: var(--c-text);">{{ root }}</div>
                <div class="truncate" style="color: var(--c-text-tertiary);">{{ detectCurrentFolder(root) ? t('settings.project_roots_current') : t('settings.project_roots_added') }}</div>
              </div>
              <button class="cursor-pointer hover:opacity-80 shrink-0 transition-colors" style="color: var(--c-danger);" :title="t('settings.project_roots_remove')" @click="removeProjectRoot(root)">&times;</button>
            </div>
          </div>
          <p v-else class="text-[11px]" style="color: var(--c-text-tertiary);">{{ t('settings.project_roots_empty') }}</p>
        </div>
        <details v-if="showProjectRootsAdvanced" class="mt-3">
          <summary class="text-[11px] cursor-pointer" style="color: var(--c-text-secondary);">{{ t('settings.project_roots_advanced_title') }}</summary>
          <textarea
            v-model="projectRootsText"
            rows="4"
            class="w-full mt-2 px-3 py-2 text-xs rounded-md border outline-none resize-none transition-colors"
            style="background: var(--c-surface); border-color: var(--c-border); color: var(--c-text);"
            :placeholder="t('settings.project_roots_placeholder')"
          />
        </details>
        <div class="flex items-center justify-between mt-2">
          <span class="text-[11px]" style="color: var(--c-text-tertiary);">{{ t('settings.project_roots_count', { count: projectRootsCount }) }}</span>
          <button
            class="px-3 py-1.5 text-[11px] rounded-md border cursor-pointer hover:opacity-80 transition-colors"
            style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
            :disabled="savingProjectRoots"
            @click="saveProjectRoots"
          >
            {{ savingProjectRoots ? t('app.loading') : t('settings.save_project_roots') }}
          </button>
        </div>
        <p v-if="!projectRootsLoaded" class="text-[11px] mt-1" style="color: var(--c-text-tertiary);">{{ t('settings.project_roots_loading') }}</p>
      </div>

      <!-- 数据管理 -->
      <div>
        <label class="text-xs font-medium block mb-2" style="color: var(--c-text);">{{ t('settings.data_management') }}</label>
        <div class="flex gap-2">
          <button
            class="flex-1 px-3 py-2 text-xs rounded-md border cursor-pointer hover:opacity-80 transition-colors"
            style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
            @click="handleExport"
          >
            {{ t('settings.export_data') }}
          </button>
          <button
            class="flex-1 px-3 py-2 text-xs rounded-md border cursor-pointer hover:opacity-80 transition-colors"
            style="border-color: var(--c-border); color: var(--c-text); background: var(--c-surface);"
            @click="handleImport"
          >
            {{ t('settings.import_data') }}
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-if="showMigrateConfirm"
      :title="t('settings.migrate_title')"
      :message="t('settings.migrate_confirm')"
      :confirm-text="savingPath ? t('app.loading') : t('settings.migrate_yes')"
      :cancel-text="t('settings.migrate_no')"
      :error="pathError"
      :disabled="savingPath"
      @confirm="handleMigrate(true)"
      @cancel="handleMigrate(false)"
    />
  </div>
</template>
