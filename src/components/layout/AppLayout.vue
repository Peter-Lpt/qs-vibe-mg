<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useAppStore } from "../../stores/app";

const { t } = useI18n();
const appStore = useAppStore();
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden">
    <!-- Header -->
    <header
      class="flex items-center px-6 py-3 border-b shrink-0"
      style="border-color: var(--c-border); background: var(--c-surface);"
    >
      <h1 class="text-lg font-semibold" style="color: var(--c-text);">
        {{ t('app.title') }}
      </h1>
      <span
        class="ml-2 text-xs px-2 py-0.5 rounded-full"
        style="background: var(--c-primary); color: white;"
      >
        {{ t('app.version') }}
      </span>

      <div class="ml-auto flex items-center gap-2">
        <!-- Theme toggle -->
        <button
          class="text-sm px-2 py-1 rounded cursor-pointer hover:opacity-80"
          style="color: var(--c-text-secondary);"
          @click="appStore.setTheme(appStore.resolvedTheme === 'dark' ? 'light' : 'dark')"
          :title="appStore.resolvedTheme === 'dark' ? '☀️' : '🌙'"
        >
          {{ appStore.resolvedTheme === 'dark' ? '☀️' : '🌙' }}
        </button>

        <!-- Settings -->
        <button
          class="text-sm px-2 py-1 rounded cursor-pointer hover:opacity-80"
          style="color: var(--c-text-secondary);"
          @click="appStore.showSettings = true"
          :title="t('app.settings')"
        >
          ⚙
        </button>
      </div>
    </header>

    <!-- Content -->
    <div class="flex flex-1 overflow-hidden">
      <div class="w-1/2 border-r overflow-y-auto p-4" style="border-color: var(--c-border);">
        <slot name="left" />
      </div>
      <div class="w-1/2 overflow-y-auto p-4" style="background: var(--c-surface);">
        <slot name="right" />
      </div>
    </div>

    <!-- Bottom bar -->
    <div class="shrink-0">
      <slot name="bottom" />
    </div>
  </div>
</template>
