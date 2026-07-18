import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, TabId } from "../types";

export type ThemeMode = "system" | "light" | "dark";
export type Locale = "zh" | "en" | "zh-TW";

export const useAppStore = defineStore("app", () => {
  const theme = ref<ThemeMode>(
    (localStorage.getItem("vibe-theme") as ThemeMode) || "system"
  );
  const locale = ref<Locale>(
    (localStorage.getItem("vibe-locale") as Locale) || "zh"
  );
  const showSettings = ref(false);
  const resolvedTheme = ref<"light" | "dark">("light");
  const config = ref<AppConfig | null>(null);

  // 兼容旧值：将旧 tab id 映射到新 id
  const storedTab = localStorage.getItem("vibe-active-tab") as TabId | null;
  const initialTab: TabId =
    storedTab === "manage" || storedTab === "history"
      ? storedTab
      : storedTab === "overview" || storedTab === "symlink" || storedTab === "skills"
        ? "manage"
        : storedTab === "agents" || storedTab === "dashboard"
          ? "manage"
          : "manage";

  const activeTab = ref<TabId>(initialTab);

  function applyTheme(mode: ThemeMode) {
    const root = document.documentElement;
    root.classList.remove("light", "dark");

    if (mode === "system") {
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)"
      ).matches;
      const resolved = prefersDark ? "dark" : "light";
      root.classList.add(resolved);
      resolvedTheme.value = resolved;
    } else {
      root.classList.add(mode);
      resolvedTheme.value = mode;
    }
  }

  function setTheme(mode: ThemeMode) {
    theme.value = mode;
    localStorage.setItem("vibe-theme", mode);
    applyTheme(mode);
  }

  function setLocale(loc: Locale) {
    locale.value = loc;
    localStorage.setItem("vibe-locale", loc);
  }

  function setActiveTab(tab: TabId) {
    activeTab.value = tab;
    localStorage.setItem("vibe-active-tab", tab);
  }

  function init() {
    applyTheme(theme.value);

    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", () => {
        if (theme.value === "system") {
          applyTheme("system");
        }
      });
  }

  // 数据管理：导出/导入配置（后端调用统一收口到 store）
  async function exportData(): Promise<string> {
    return await invoke<string>("export_data");
  }

  async function readFileFromPath(path: string): Promise<string> {
    return await invoke<string>("read_file_from_path", { path });
  }

  async function writeFileToPath(path: string, content: string): Promise<void> {
    await invoke("write_file_to_path", { path, content });
  }

  async function importData(json: string): Promise<void> {
    await invoke("import_data", { json });
  }

  async function fetchConfig() {
    config.value = await invoke<AppConfig>("get_config");
  }

  async function updateProjectRoots(projectRoots: string[]) {
    config.value = await invoke<AppConfig>("update_config", {
      projectRoots,
    });
  }

  return {
    theme,
    locale,
    showSettings,
    resolvedTheme,
    config,
    activeTab,
    setTheme,
    setLocale,
    setActiveTab,
    init,
    exportData,
    readFileFromPath,
    writeFileToPath,
    importData,
    fetchConfig,
    updateProjectRoots,
  };
});
