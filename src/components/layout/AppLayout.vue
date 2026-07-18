<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStore } from "../../stores/app";

const { t } = useI18n();
const appStore = useAppStore();
const appWindow = getCurrentWindow();

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximizeWindow() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden" style="background: var(--c-bg);">
    <header
      class="app-shell-header flex items-center gap-5 px-5 py-3 shrink-0"
      data-tauri-drag-region
    >
      <div class="flex items-center gap-2.5 shrink-0" data-tauri-drag-region>
        <div
          class="brand-mark"
          data-tauri-drag-region
        >
          QS
        </div>
        <div class="min-w-0" data-tauri-drag-region>
          <h1 class="text-sm font-semibold leading-tight" style="color: var(--c-text-strong);">
            {{ t('app.workspace_title') }}
          </h1>
          <p class="text-[10px] leading-tight" style="color: var(--c-text-secondary);">
            {{ t('app.title') }}
          </p>
        </div>
      </div>

      <div class="flex-1 min-w-0">
        <slot name="tabs" />
      </div>

      <div class="ml-auto flex items-center gap-1">
        <button
          class="icon-button"
          @click="appStore.setTheme(appStore.resolvedTheme === 'dark' ? 'light' : 'dark')"
          :title="t('settings.theme')"
        >
          <Sun v-if="appStore.resolvedTheme === 'dark'" :size="16" />
          <Moon v-else :size="16" />
        </button>
        <button
          class="icon-button"
          @click="appStore.showSettings = true"
          :title="t('app.settings')"
        >
          <Settings :size="16" />
        </button>
        <div class="titlebar-divider" />
        <button class="window-button" :title="t('app.window_minimize')" @click="minimizeWindow">
          <Minus :size="14" />
        </button>
        <button class="window-button" :title="t('app.window_maximize')" @click="toggleMaximizeWindow">
          <Square :size="12" />
        </button>
        <button class="window-button window-button-close" :title="t('app.window_close')" @click="closeWindow">
          <X :size="15" />
        </button>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto px-5 py-4" style="background: var(--c-bg);">
      <slot />
    </div>
  </div>
</template>
