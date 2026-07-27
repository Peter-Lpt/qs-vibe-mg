<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppStore } from "../../stores/app";

const { t } = useI18n();
const appStore = useAppStore();
const appWindow = getCurrentWindow();

// 关闭行为：ask | minimize_to_tray | close
const closeBehavior = ref<string>(
  localStorage.getItem("vibe-close-behavior") || "ask"
);
const showCloseDialog = ref(false);
const rememberChoice = ref(false);

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximizeWindow() {
  await appWindow.toggleMaximize();
}

async function hideWindow() {
  await appWindow.hide();
}

async function closeWindow() {
  await appWindow.close();
}

async function handleCloseClick() {
  const behavior = closeBehavior.value;
  if (behavior === "minimize_to_tray") {
    await hideWindow();
  } else if (behavior === "close") {
    await closeWindow();
  } else {
    // "ask" - 显示选择对话框
    showCloseDialog.value = true;
  }
}

async function handleDialogChoice(choice: "minimize_to_tray" | "close") {
  showCloseDialog.value = false;
  if (rememberChoice.value) {
    closeBehavior.value = choice;
    localStorage.setItem("vibe-close-behavior", choice);
  }
  if (choice === "minimize_to_tray") {
    await hideWindow();
  } else {
    await closeWindow();
  }
}

function cancelDialog() {
  showCloseDialog.value = false;
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
          <p class="text-[9px] leading-tight opacity-50" style="color: var(--c-text-secondary);">
            {{ t('app.title') }}
          </p>
        </div>
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
        <div class="titlebar-divider" />
        <button class="window-button" :title="t('app.window_minimize')" @click.stop="minimizeWindow">
          <Minus :size="14" />
        </button>
        <button class="window-button" :title="t('app.window_maximize')" @click.stop="toggleMaximizeWindow">
          <Square :size="12" />
        </button>
        <button class="window-button window-button-close" :title="t('app.window_close')" @click.stop="handleCloseClick">
          <X :size="15" />
        </button>
      </div>
    </header>

    <div class="flex flex-1 min-h-0">
      <slot name="sidebar" />
      <div class="app-shell-content flex-1 min-w-0 min-h-0 overflow-y-auto px-5 py-4" style="background: var(--c-bg);">
        <slot />
      </div>
    </div>

    <!-- 关闭行为选择对话框 -->
    <Teleport to="body">
      <div
        v-if="showCloseDialog"
        class="dialog-overlay"
        @click.self="cancelDialog"
      >
        <div class="dialog-content" style="background: var(--c-surface); border: 1px solid var(--c-border);">
          <h3 class="text-sm font-semibold mb-3" style="color: var(--c-text-strong);">
            {{ t('app.close_dialog_title') }}
          </h3>
          <p class="text-xs mb-4" style="color: var(--c-text-secondary);">
            {{ t('app.close_dialog_message') }}
          </p>
          <div class="flex flex-col gap-2 mb-4">
            <button
              class="dialog-button primary"
              @click="handleDialogChoice('minimize_to_tray')"
            >
              <Minimize2 :size="14" class="mr-2" />
              {{ t('app.minimize_to_tray') }}
            </button>
            <button
              class="dialog-button secondary"
              @click="handleDialogChoice('close')"
            >
              <X :size="14" class="mr-2" />
              {{ t('app.close_app') }}
            </button>
          </div>
          <label class="flex items-center gap-2 text-xs cursor-pointer" style="color: var(--c-text-secondary);">
            <input v-model="rememberChoice" type="checkbox" class="rounded" />
            {{ t('app.remember_choice') }}
          </label>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.dialog-content {
  border-radius: 12px;
  padding: 20px;
  min-width: 300px;
  max-width: 400px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.dialog-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.dialog-button.primary {
  background: var(--c-primary);
  color: white;
}

.dialog-button.primary:hover {
  opacity: 0.9;
}

.dialog-button.secondary {
  background: var(--c-surface-hover);
  color: var(--c-text);
  border: 1px solid var(--c-border);
}

.dialog-button.secondary:hover {
  background: var(--c-border);
}
</style>
