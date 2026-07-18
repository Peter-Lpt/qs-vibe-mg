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
  { id: "manage", icon: "Link2", labelKey: "tabs.manage" },
  { id: "history", icon: "History", labelKey: "tabs.history" },
];

function selectTab(tab: TabId) {
  emit("update:modelValue", tab);
}
</script>

<template>
  <div
    class="flex items-center gap-1"
  >
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md cursor-pointer transition-all"
      :style="{
        background: modelValue === tab.id ? 'var(--c-primary-light)' : 'transparent',
        color: modelValue === tab.id ? 'var(--c-primary)' : 'var(--c-text-secondary)',
      }"
      @click="selectTab(tab.id)"
    >
      <component :is="tab.icon" :size="14" />
      <span>{{ t(tab.labelKey) }}</span>
    </button>
  </div>
</template>
