<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { TabId } from "../../types";

const { t } = useI18n();

const props = defineProps<{
  modelValue: TabId;
}>();

const emit = defineEmits<{
  "update:modelValue": [tab: TabId];
}>();

const tabs: { id: TabId; icon: string; labelKey: string }[] = [
  { id: "cli", icon: "🔧", labelKey: "tabs.cli" },
  { id: "skills", icon: "📁", labelKey: "tabs.skills" },
  { id: "dashboard", icon: "📊", labelKey: "tabs.dashboard" },
  { id: "symlink", icon: "🔗", labelKey: "tabs.symlink" },
];

function selectTab(tab: TabId) {
  emit("update:modelValue", tab);
}
</script>

<template>
  <div
    class="flex items-center gap-1 px-4 py-2 border-b shrink-0"
    style="border-color: var(--c-border); background: var(--c-surface);"
  >
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-md cursor-pointer transition-all"
      :style="{
        background: modelValue === tab.id ? 'var(--c-primary)' : 'transparent',
        color: modelValue === tab.id ? 'white' : 'var(--c-text-secondary)',
      }"
      @click="selectTab(tab.id)"
    >
      <span>{{ tab.icon }}</span>
      <span>{{ t(tab.labelKey) }}</span>
    </button>
  </div>
</template>
