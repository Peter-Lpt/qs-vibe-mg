import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, ProjectRootSuggestion, SmartViewId, ViewId } from "../types";

export type ThemeMode = "system" | "light" | "dark";
export type Locale = "zh" | "en" | "zh-TW";

export const useAppStore = defineStore("app", () => {
  const theme = ref<ThemeMode>(
    (localStorage.getItem("vibe-theme") as ThemeMode) || "system"
  );
  const locale = ref<Locale>(
    (localStorage.getItem("vibe-locale") as Locale) || "zh"
  );
  const resolvedTheme = ref<"light" | "dark">("light");
  const config = ref<AppConfig | null>(null);
  const projectRootSuggestions = ref<ProjectRootSuggestion[]>([]);

  // ── 视图状态（v0.3：activeTab → activeView，统一 setter + 持久化） ──
  const VIEW_IDS: readonly ViewId[] = [
    "all",
    "attention",
    "linked",
    "unlinked",
    "plugins",
    "history",
    "settings",
  ];
  function isValidViewId(value: string | null): value is ViewId {
    return value !== null && (VIEW_IDS as readonly string[]).includes(value);
  }
  function isValidSmartViewId(value: string | null): value is SmartViewId {
    return isValidViewId(value) && value !== "history";
  }

  // 旧 key "vibe-active-tab" 一次性迁移：manage（含 legacy overview/symlink/skills/agents/dashboard）→ "all"；history → "history"
  const storedView = localStorage.getItem("vibe-active-view");
  const legacyTab = localStorage.getItem("vibe-active-tab");
  const initialView: ViewId = isValidViewId(storedView)
    ? storedView
    : legacyTab === "history"
      ? "history"
      : "all";

  const storedLastSkillsView = localStorage.getItem("vibe-last-skills-view");
  const activeView = ref<ViewId>(initialView);
  const lastSkillsView = ref<SmartViewId>(
    isValidSmartViewId(storedLastSkillsView) ? storedLastSkillsView : "all"
  );

  function setActiveView(view: ViewId) {
    activeView.value = view;
    if (view !== "history" && view !== "settings") {
      lastSkillsView.value = view as SmartViewId;
      localStorage.setItem("vibe-last-skills-view", view);
    }
    localStorage.setItem("vibe-active-view", view);
  }

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

  async function fetchProjectRootSuggestions() {
    projectRootSuggestions.value = await invoke<ProjectRootSuggestion[]>("suggest_project_roots");
  }

  async function updateProjectRoots(projectRoots: string[]) {
    config.value = await invoke<AppConfig>("update_config", {
      projectRoots,
    });
    await fetchProjectRootSuggestions();
  }

  return {
    theme,
    locale,
    resolvedTheme,
    config,
    projectRootSuggestions,
    activeView,
    lastSkillsView,
    setActiveView,
    setTheme,
    setLocale,
    init,
    exportData,
    readFileFromPath,
    writeFileToPath,
    importData,
    fetchConfig,
    fetchProjectRootSuggestions,
    updateProjectRoots,
  };
});
