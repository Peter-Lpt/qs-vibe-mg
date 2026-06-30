<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useAppStore } from "../../stores/app";

const { t } = useI18n();
const appStore = useAppStore();
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden" style="background: var(--c-bg);">
    <header
      class="flex items-center px-5 py-3 border-b shrink-0"
      style="border-color: var(--c-border); background: var(--c-surface);"
    >
      <div class="flex items-center gap-2.5">
        <div
          class="w-7 h-7 rounded-lg flex items-center justify-center text-sm font-bold"
          style="background: var(--c-primary); color: white;"
        >
          V
        </div>
        <h1 class="text-sm font-semibold" style="color: var(--c-text);">
          {{ t('app.title') }}
        </h1>
        <span
          class="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
          style="background: var(--c-primary-light); color: var(--c-primary);"
        >
          {{ t('app.version') }}
        </span>
      </div>

      <div class="ml-auto flex items-center gap-1">
        <button
          class="w-8 h-8 flex items-center justify-center rounded-lg cursor-pointer transition-colors"
          style="color: var(--c-text-secondary);"
          @click="appStore.setTheme(appStore.resolvedTheme === 'dark' ? 'light' : 'dark')"
          @mouseenter="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'var(--c-surface-hover)'"
          @mouseleave="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'transparent'"
        >
          {{ appStore.resolvedTheme === 'dark' ? '☀️' : '🌙' }}
        </button>
        <button
          class="w-8 h-8 flex items-center justify-center rounded-lg cursor-pointer transition-colors"
          style="color: var(--c-text-secondary);"
          @click="appStore.showSettings = true"
          :title="t('app.settings')"
          @mouseenter="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'var(--c-surface-hover)'"
          @mouseleave="(e: MouseEvent) => (e.target as HTMLElement).style.background = 'transparent'"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
        </button>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto p-6" style="background: var(--c-bg);">
      <slot />
    </div>

    <div class="shrink-0">
      <slot name="bottom" />
    </div>
  </div>
</template>
