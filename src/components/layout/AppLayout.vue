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

async function handleTitlebarMouseDown(event: MouseEvent) {
  if (event.button !== 0) return;
  event.preventDefault();
  if (event.detail === 2) {
    await toggleMaximizeWindow();
    return;
  }
  await appWindow.startDragging();
}
</script>

<template>
  <div class="app-shell-root flex flex-col min-h-0">
    <header class="app-shell-header flex items-center gap-5 px-5 py-3 shrink-0">
      <div
        class="flex items-center gap-2.5 shrink-0 select-none"
        @mousedown="handleTitlebarMouseDown"
      >
        <div class="brand-mark">
          QS
        </div>
        <div class="min-w-0">
          <h1 class="text-sm font-semibold leading-tight" style="color: var(--c-text-strong);">
            {{ t('app.workspace_title') }}
          </h1>
          <p class="text-[10px] leading-tight" style="color: var(--c-text-secondary);">
            {{ t('app.title') }}
          </p>
        </div>
      </div>

      <div class="shrink-0 min-w-0">
        <slot name="tabs" />
      </div>

      <div
        class="titlebar-drag-spacer flex-1 self-stretch min-w-4"
        @mousedown="handleTitlebarMouseDown"
      />

      <div class="ml-auto flex items-center gap-1 shrink-0">
        <button
          class="icon-button"
          @click.stop="appStore.setTheme(appStore.resolvedTheme === 'dark' ? 'light' : 'dark')"
          :title="t('settings.theme')"
        >
          <Sun v-if="appStore.resolvedTheme === 'dark'" :size="16" />
          <Moon v-else :size="16" />
        </button>
        <button
          class="icon-button"
          @click.stop="appStore.showSettings = true"
          :title="t('app.settings')"
        >
          <Settings :size="16" />
        </button>
        <div class="titlebar-divider" />
        <button class="window-button" :title="t('app.window_minimize')" @click.stop="minimizeWindow">
          <Minus :size="14" />
        </button>
        <button class="window-button" :title="t('app.window_maximize')" @click.stop="toggleMaximizeWindow">
          <Square :size="12" />
        </button>
        <button class="window-button window-button-close" :title="t('app.window_close')" @click.stop="closeWindow">
          <X :size="15" />
        </button>
      </div>
    </header>

    <div class="app-shell-content flex-1 min-h-0 overflow-y-auto px-5 py-4" style="background: var(--c-bg);">
      <slot />
    </div>
  </div>
</template>
