<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useAppStore } from "../../stores/app";

const { t } = useI18n();
const appStore = useAppStore();
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden" style="background: var(--c-bg);">
    <header
      class="flex items-center gap-4 px-5 py-3 border-b shrink-0"
      style="border-color: var(--c-border); background: var(--c-surface);"
    >
      <div class="flex items-center gap-2.5">
        <div
          class="w-8 h-8 rounded-md flex items-center justify-center text-[11px] font-bold tracking-wide"
          style="background: var(--c-primary); color: white;"
        >
          QS
        </div>
        <div class="min-w-0">
          <h1 class="text-sm font-semibold leading-tight" style="color: var(--c-text);">
            {{ t('app.title') }}
          </h1>
          <p class="text-[10px] leading-tight" style="color: var(--c-text-secondary);">
            {{ t('app.subtitle') }}
          </p>
        </div>
      </div>

      <div class="flex-1 min-w-0">
        <slot name="tabs" />
      </div>

      <div class="ml-auto flex items-center gap-1">
        <button
          class="w-8 h-8 flex items-center justify-center rounded-md cursor-pointer bg-transparent text-[var(--c-text-secondary)] hover:bg-[var(--c-surface-hover)]"
          @click="appStore.setTheme(appStore.resolvedTheme === 'dark' ? 'light' : 'dark')"
          :title="t('settings.theme')"
        >
          <Sun v-if="appStore.resolvedTheme === 'dark'" :size="16" />
          <Moon v-else :size="16" />
        </button>
        <button
          class="w-8 h-8 flex items-center justify-center rounded-md cursor-pointer bg-transparent text-[var(--c-text-secondary)] hover:bg-[var(--c-surface-hover)]"
          @click="appStore.showSettings = true"
          :title="t('app.settings')"
        >
          <Settings :size="16" />
        </button>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto px-5 py-4" style="background: var(--c-bg);">
      <slot />
    </div>
  </div>
</template>
