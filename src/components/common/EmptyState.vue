<script setup lang="ts">
defineProps<{
  icon?: string; // lucide 全局组件名
  title: string;
  description?: string;
  actionLabel?: string;
}>();

defineEmits<{
  action: [];
}>();
</script>

<template>
  <div class="flex flex-col items-center py-16">
    <component v-if="icon" :is="icon" :size="40" class="mb-4" style="color: var(--c-text-secondary);" />
    <p class="text-sm font-medium mb-1" style="color: var(--c-text);">
      {{ title }}
    </p>
    <p v-if="description" class="text-xs mb-5 text-center max-w-xs" style="color: var(--c-text-secondary);">
      {{ description }}
    </p>
    <button
      v-if="actionLabel"
      class="text-xs px-4 py-2 rounded-md cursor-pointer font-medium btn-primary"
      @click="$emit('action')"
    >
      {{ actionLabel }}
    </button>
  </div>
</template>
