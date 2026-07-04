<script setup lang="ts">
defineProps<{
  lines?: number;
}>();
</script>

<template>
  <div class="space-y-3 p-4 rounded-lg border" style="border-color: var(--c-border); background: var(--c-surface);">
    <div class="skeleton h-4 w-2/5" />
    <div class="skeleton h-3 w-4/5" />
    <div v-for="i in (lines ?? 2)" :key="i" class="skeleton h-3" :style="{ width: (90 - i * 10) + '%' }" />
  </div>
</template>
