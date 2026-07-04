<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useEscapeKey } from "../../composables/useEscapeKey";

const { t } = useI18n();

useEscapeKey(() => emit("cancel"));

const props = defineProps<{
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
  disabled?: boolean;
  error?: string | null;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center"
      style="background: rgba(0, 0, 0, 0.5);"
      @click.self="emit('cancel')"
    >
      <div
        class="rounded-lg p-5 shadow-xl max-w-sm w-full mx-4"
        style="background: var(--c-surface); border: 1px solid var(--c-border);"
      >
        <h3 class="text-sm font-semibold mb-2" style="color: var(--c-text);">
          {{ title }}
        </h3>
        <p class="text-sm mb-4" style="color: var(--c-text-secondary);">
          {{ message }}
        </p>
        <div v-if="error" class="text-xs mb-3 px-2 py-1.5 rounded-md" style="background: var(--c-danger-light); color: var(--c-danger);">
          {{ error }}
        </div>
        <div class="flex justify-end gap-2">
          <button
            class="px-3 py-1.5 text-xs rounded-md border cursor-pointer hover:opacity-80"
            style="border-color: var(--c-border); color: var(--c-text);"
            :disabled="disabled"
            @click="emit('cancel')"
          >
            {{ cancelText || t('common.cancel') }}
          </button>
          <button
            class="px-3 py-1.5 text-xs rounded-md cursor-pointer hover:opacity-80 disabled:opacity-50 disabled:cursor-not-allowed"
            :style="{
              background: danger ? 'var(--c-danger)' : 'var(--c-primary)',
              color: 'white',
            }"
            :disabled="disabled"
            @click="emit('confirm')"
          >
            {{ confirmText || t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
