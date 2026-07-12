<script setup lang="ts">
defineProps<{
  lines?: number;
}>();
</script>

<template>
  <div class="space-y-3 p-4 rounded-lg border" style="border-color: var(--c-border); background: var(--c-surface);">
    <div class="animate-pulse bg-[var(--c-surface-hover)] rounded-[var(--radius-sm)] h-4 w-2/5" />
    <div class="animate-pulse bg-[var(--c-surface-hover)] rounded-[var(--radius-sm)] h-3 w-4/5" />
    <div v-for="i in (lines ?? 2)" :key="i" class="animate-pulse bg-[var(--c-surface-hover)] rounded-[var(--radius-sm)] h-3" :style="{ width: (90 - i * 10) + '%' }" />
  </div>
</template>
