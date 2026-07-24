<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { SmartViewId, ViewId } from "../../types";
import { SMART_VIEWS, type SmartViewDef } from "../../composables/useSmartViews";

const props = defineProps<{
  activeView: ViewId;
  counts: Record<SmartViewId, number>;
}>();

const emit = defineEmits<{
  (e: "select", view: ViewId): void;
  (e: "open-settings"): void;
}>();

const { t } = useI18n();

const libraryViews = computed(() => SMART_VIEWS.filter((v) => v.domain === "local"));
const pluginViews = computed(() => SMART_VIEWS.filter((v) => v.domain === "plugin"));

function isActive(view: SmartViewDef): boolean {
  return props.activeView === view.id;
}

function itemStyle(view: SmartViewDef) {
  if (isActive(view)) {
    return { background: "var(--c-primary-light)", color: "var(--c-primary)" };
  }
  return { color: "var(--c-text-secondary)" };
}
</script>

<template>
  <nav
    class="flex h-full w-52 shrink-0 flex-col gap-0.5 overflow-y-auto border-r px-3 py-4"
    style="background: var(--c-surface); border-color: var(--c-border);"
    aria-label="views"
  >
    <div class="px-2 pb-1.5 text-[10px] font-semibold uppercase tracking-wide" style="color: var(--c-text-tertiary);">
      {{ t("sidebar.skills_library") }}
    </div>

    <button
      v-for="view in libraryViews"
      :key="view.id"
      type="button"
      class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-xs cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
      :style="itemStyle(view)"
      :aria-current="isActive(view) ? 'page' : undefined"
      @click="emit('select', view.id)"
    >
      <component :is="view.icon" :size="14" class="shrink-0" />
      <span class="flex-1 truncate text-left">{{ t(view.labelKey) }}</span>
      <span
        v-if="view.showBadge && counts[view.id] > 0"
        class="min-w-4.5 rounded-full px-1 text-center text-[9px] leading-3.5"
        style="background: var(--c-warning); color: white;"
      >{{ counts[view.id] }}</span>
      <span v-else class="text-[10px]" style="color: var(--c-text-tertiary);">{{ counts[view.id] }}</span>
    </button>

    <div class="px-2 pb-1.5 pt-4 text-[10px] font-semibold uppercase tracking-wide" style="color: var(--c-text-tertiary);">
      {{ t("sidebar.section_source") }}
    </div>

    <button
      v-for="view in pluginViews"
      :key="view.id"
      type="button"
      class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-xs cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
      :style="
        isActive(view)
          ? { background: 'var(--c-plugin-light, rgba(139, 92, 246, 0.15))', color: 'var(--c-plugin, #8b5cf6)' }
          : { color: 'var(--c-text-secondary)' }
      "
      :aria-current="isActive(view) ? 'page' : undefined"
      @click="emit('select', view.id)"
    >
      <component :is="view.icon" :size="14" class="shrink-0" />
      <span class="flex-1 truncate text-left">{{ t(view.labelKey) }}</span>
      <span class="text-[10px]" style="color: var(--c-text-tertiary);">{{ counts[view.id] }}</span>
    </button>

    <div class="flex-1" />

    <div class="my-2 border-t" style="border-color: var(--c-border-subtle);" />

    <button
      type="button"
      class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-xs cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
      :style="
        activeView === 'history'
          ? { background: 'var(--c-primary-light)', color: 'var(--c-primary)' }
          : { color: 'var(--c-text-secondary)' }
      "
      :aria-current="activeView === 'history' ? 'page' : undefined"
      @click="emit('select', 'history')"
    >
      <History :size="14" class="shrink-0" />
      <span class="flex-1 truncate text-left">{{ t("tabs.history") }}</span>
    </button>

    <button
      type="button"
      class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-xs cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
      style="color: var(--c-text-secondary);"
      @click="emit('open-settings')"
    >
      <Settings :size="14" class="shrink-0" />
      <span class="flex-1 truncate text-left">{{ t("app.settings") }}</span>
    </button>
  </nav>
</template>
