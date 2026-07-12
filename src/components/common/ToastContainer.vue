<script setup lang="ts">
import { useToast } from "../../composables/useToast";

const { toasts, dismiss } = useToast();
</script>

<template>
  <Teleport to="body">
    <div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none" style="max-width: 360px;">
      <TransitionGroup
        enter-active-class="transition duration-200 ease-out"
        leave-active-class="transition duration-150 ease-in"
        enter-from-class="opacity-0 translate-x-10"
        enter-to-class="opacity-100 translate-x-0"
        leave-from-class="opacity-100 translate-x-0"
        leave-to-class="opacity-0 translate-x-10"
      >
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="px-4 py-2.5 rounded-lg text-xs shadow-md cursor-pointer pointer-events-auto transition-all"
          :style="{
            background: toast.type === 'success'
              ? 'var(--c-success-light)'
              : toast.type === 'error'
                ? 'var(--c-danger-light)'
                : 'var(--c-primary-light)',
            color: toast.type === 'success'
              ? 'var(--c-success)'
              : toast.type === 'error'
                ? 'var(--c-danger)'
                : 'var(--c-primary)',
            border: '1px solid',
            borderColor: toast.type === 'success'
              ? 'var(--c-success)'
              : toast.type === 'error'
                ? 'var(--c-danger)'
                : 'var(--c-primary)',
            opacity: 0.9,
          }"
          @click="dismiss(toast.id)"
        >
          {{ toast.message }}
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>
