import { defineStore } from "pinia";
import { ref } from "vue";

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

  function init() {
    applyTheme(theme.value);

    // Listen for system theme changes
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
    setTheme,
    setLocale,
    init,
  };
});
