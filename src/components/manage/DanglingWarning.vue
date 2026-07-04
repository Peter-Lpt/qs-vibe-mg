<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { SkillSource } from "../../types";

const { t } = useI18n();

defineProps<{
  sources: SkillSource[];
}>();

const emit = defineEmits<{
  remove: [];
}>();
</script>

<template>
  <div
    class="rounded-md p-3 text-xs"
    style="background: var(--c-danger-light); border: 1px solid var(--c-danger);"
  >
    <div class="flex items-center gap-1.5 mb-2" style="color: var(--c-danger);">
      <span>❌</span>
      <span class="font-medium">{{ t("manage.dangling_warning") }}</span>
    </div>

    <div class="space-y-1 mb-3">
      <div
        v-for="source in sources.filter(s => s.is_symlink)"
        :key="source.from"
        class="flex items-center gap-2"
      >
        <span style="color: var(--c-text-secondary);">→</span>
        <span style="color: var(--c-text);">{{ source.from }}</span>
        <span style="color: var(--c-text-secondary);">→</span>
        <span class="line-through" style="color: var(--c-danger);">
          {{ source.symlink_target || "unknown" }}
        </span>
      </div>
    </div>

    <button
      class="text-[11px] px-2.5 py-1 rounded cursor-pointer transition-colors"
      style="background: var(--c-danger); color: white;"
      @click="emit('remove')"
    >
      {{ t("manage.remove_broken") }}
    </button>
  </div>
</template>
