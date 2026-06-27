import { defineStore } from "pinia";
import { ref } from "vue";
import type { TabId } from "../types";

export type ThemeMode = "system" | "light" | "dark";
export type Locale = "zh" | "en" | "zh-TW";

export const useAppStore = defineStore("app", () => {
  const theme = ref<ThemeMode>(
    (localStorage.getItem("vab-theme") as ThemeMode) || "system"
  );
  const locale = ref<Locale>(
    (localStorage.getItem("vab-locale") as Locale) || "zh"
  );
  const showSettings = ref(false);
  const resolvedTheme = ref<"light" | "dark">("light");
  const activeTab = ref<TabId>(
    (localStorage.getItem("vab-active-tab") as TabId) || "skills"
  );

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
    localStorage.setItem("vab-theme", mode);
    applyTheme(mode);
  }

  function setLocale(loc: Locale) {
    locale.value = loc;
    localStorage.setItem("vab-locale", loc);
  }

  function setActiveTab(tab: TabId) {
    activeTab.value = tab;
    localStorage.setItem("vab-active-tab", tab);
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

  return {
    theme,
    locale,
    showSettings,
    resolvedTheme,
    activeTab,
    setTheme,
    setLocale,
    setActiveTab,
    init,
  };
});
