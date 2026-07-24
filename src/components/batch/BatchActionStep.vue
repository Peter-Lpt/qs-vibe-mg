<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  BATCH_ACTION_CARDS,
  REPAIR_PRESETS,
  type BatchActionCard,
  type RepairContext,
} from "../../composables/useBatchCellActions";

const props = defineProps<{
  modelValue: BatchActionCard;
  repairContext?: RepairContext | null;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", card: BatchActionCard): void;
}>();

const { t } = useI18n();

const repairInfo = computed(() => {
  if (!props.repairContext) return null;
  const preset = REPAIR_PRESETS[props.repairContext];
  return { label: t(preset.labelKey), hint: t(preset.hintKey) };
});

function choose(card: BatchActionCard) {
  emit("update:modelValue", card);
}
</script>

<template>
  <div class="space-y-2">
    <div
      v-if="repairInfo"
      class="flex items-center gap-2 rounded-md px-2.5 py-2 text-[10px]"
      style="background: var(--c-warning-light); color: var(--c-warning);"
    >
      <Wrench :size="12" class="shrink-0" />
      <span class="font-medium shrink-0">{{ repairInfo.label }}</span>
      <span style="color: var(--c-text-secondary);">{{ repairInfo.hint }}</span>
    </div>

    <button
      v-for="card in BATCH_ACTION_CARDS"
      :key="card.id"
      type="button"
      class="w-full rounded-lg border px-3 py-2.5 text-left cursor-pointer transition-colors"
      :style="
        modelValue.id === card.id
          ? { borderColor: 'var(--c-primary)', background: 'var(--c-primary-light)' }
          : { borderColor: 'var(--c-border)', background: 'var(--c-surface)' }
      "
      :aria-pressed="modelValue.id === card.id"
      @click="choose(card)"
    >
      <div class="flex items-center gap-2">
        <span
          class="flex h-3.5 w-3.5 items-center justify-center rounded-full border shrink-0"
          :style="{ borderColor: modelValue.id === card.id ? 'var(--c-primary)' : 'var(--c-border)' }"
        >
          <span
            v-if="modelValue.id === card.id"
            class="h-1.5 w-1.5 rounded-full"
            style="background: var(--c-primary);"
          />
        </span>
        <span
          class="text-xs font-semibold"
          :style="{ color: modelValue.id === card.id ? 'var(--c-primary)' : 'var(--c-text)' }"
        >
          {{ t(card.labelKey) }}
        </span>
      </div>
      <p class="mt-1 pl-5.5 text-[10px] leading-relaxed" style="color: var(--c-text-secondary);">
        {{ t(card.descKey) }}
      </p>
    </button>
  </div>
</template>
