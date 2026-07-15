<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { SkillSource } from "../../types";

const { t } = useI18n();

defineProps<{
  sources: SkillSource[];
}>();

const emit = defineEmits<{
  resolve: [sourceFrom: string];
}>();
</script>

<template>
  <div
    class="rounded-md p-3 text-xs"
    style="background: var(--c-warning-light); border: 1px solid var(--c-warning);"
  >
    <div class="flex items-center gap-1.5 mb-2" style="color: var(--c-warning);">
      <TriangleAlert :size="14" />
      <span class="font-medium">{{ t("manage.conflict_warning") }}</span>
    </div>

    <div class="space-y-2 mb-3">
      <div
        v-for="source in sources"
        :key="source.from"
        class="flex items-start gap-2"
      >
        <span style="color: var(--c-text-secondary);">-</span>
        <div>
          <span class="font-medium" style="color: var(--c-text);">{{ source.name }}</span>
          <span class="ml-1" style="color: var(--c-text-secondary);">({{ source.from }})</span>
          <p v-if="source.description" class="mt-0.5" style="color: var(--c-text-secondary);">
            {{ source.description }}
          </p>
        </div>
      </div>
    </div>

    <div class="flex gap-2">
      <button
        v-for="source in sources"
        :key="source.from"
        class="text-[11px] px-2.5 py-1 rounded cursor-pointer transition-colors"
        style="background: var(--c-surface); border: 1px solid var(--c-border); color: var(--c-text);"
        @click="emit('resolve', source.from)"
      >
        {{ t("manage.use_version", { agent: source.from }) }}
      </button>
    </div>
  </div>
</template>
