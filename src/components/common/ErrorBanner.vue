<script setup lang="ts">
defineProps<{
  message: string;
}>();

const emit = defineEmits<{
  dismiss: [];
}>();
</script>

<template>
  <div
    class="flex items-center gap-2 px-3 py-2 rounded-md text-xs mb-3"
    style="background: #fef2f2; border: 1px solid #fecaca; color: var(--c-danger);"
  >
    <span class="flex-1">{{ message }}</span>
    <button
      class="hover:opacity-70 cursor-pointer text-sm"
      style="color: var(--c-danger);"
      @click="emit('dismiss')"
    >
      ×
    </button>
  </div>
</template>
