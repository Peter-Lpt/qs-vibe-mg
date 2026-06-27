<script setup lang="ts">
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden">
    <!-- Header -->
    <header class="flex items-center px-6 py-3 border-b" style="border-color: var(--c-border); background: var(--c-surface);">
      <h1 class="text-lg font-semibold" style="color: var(--c-text);">VAB Skills Manager</h1>
      <span class="ml-2 text-xs px-2 py-0.5 rounded-full" style="background: var(--c-primary); color: white;">v0.0</span>
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
  </div>
</template>
