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
      class="modal-backdrop fixed inset-0 z-50 flex items-center justify-center p-4"
      @click.self="emit('cancel')"
    >
      <div
        class="modal-shell w-full max-w-sm"
      >
        <div class="modal-body">
          <h3 class="mb-2 text-[15px] font-semibold" style="color: var(--c-text);">{{ title }}</h3>
          <p class="text-[13px] leading-6" style="color: var(--c-text-secondary);">{{ message }}</p>
          <div v-if="error" class="mt-3 rounded-md px-3 py-2 text-xs" style="background: var(--c-danger-light); color: var(--c-danger);">{{ error }}</div>
        </div>
        <div class="modal-actions">
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
