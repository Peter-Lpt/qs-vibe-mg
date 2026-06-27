<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import { marked } from "marked";
import { useSkillsStore } from "../../stores/skills";
import type { Skill } from "../../types";

const props = defineProps<{
  skill: Skill;
}>();

const emit = defineEmits<{
  close: [];
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const content = ref<string>("");
const loading = ref(true);
const error = ref<string | null>(null);

const renderedHtml = computed(() => {
  if (!content.value) return "";
  try {
    return marked(content.value) as string;
  } catch {
    return content.value;
  }
});

onMounted(async () => {
  try {
    content.value = await skillsStore.previewSkill(props.skill.id);
  } catch (e: unknown) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center"
      style="background: rgba(0, 0, 0, 0.5);"
      @click.self="emit('close')"
    >
      <div
        class="rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] flex flex-col"
        style="background: var(--c-surface); border: 1px solid var(--c-border);"
      >
        <div class="flex items-center justify-between p-4 border-b" style="border-color: var(--c-border);">
          <div>
            <h3 class="text-sm font-semibold" style="color: var(--c-text);">
              {{ skill.name }}
            </h3>
            <p class="text-xs mt-0.5" style="color: var(--c-text-secondary);">
              {{ skill.path }}
            </p>
          </div>
          <button
            class="text-lg hover:opacity-70 cursor-pointer"
            style="color: var(--c-text-secondary);"
            @click="emit('close')"
          >
            ×
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-4">
          <div v-if="loading" class="text-sm text-center py-8" style="color: var(--c-text-secondary);">
            {{ t('app.loading') }}
          </div>
          <div v-else-if="error" class="text-sm text-center py-8" style="color: var(--c-danger);">
            {{ error }}
          </div>
          <div
            v-else
            class="prose prose-sm max-w-none"
            style="color: var(--c-text);"
            v-html="renderedHtml"
          />
        </div>

        <!-- Metadata footer -->
        <div class="p-3 border-t flex flex-wrap gap-2" style="border-color: var(--c-border);">
          <span
            v-if="skill.license"
            class="text-xs px-2 py-0.5 rounded"
            style="background: var(--c-surface-hover); color: var(--c-text-secondary);"
          >
            License: {{ skill.license }}
          </span>
          <span
            v-if="skill.has_scripts"
            class="text-xs px-2 py-0.5 rounded"
            style="background: #dbeafe; color: #1e40af;"
          >
            {{ t('skills.has_scripts') }}
          </span>
          <span
            v-if="skill.has_references"
            class="text-xs px-2 py-0.5 rounded"
            style="background: #f3e8ff; color: #7c3aed;"
          >
            {{ t('skills.has_references') }}
          </span>
          <span
            v-if="skill.has_assets"
            class="text-xs px-2 py-0.5 rounded"
            style="background: #fef3c7; color: #92400e;"
          >
            {{ t('skills.has_assets') }}
          </span>
        </div>
      </div>
    </div>
  </Teleport>
</template>
