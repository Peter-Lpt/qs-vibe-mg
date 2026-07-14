<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import { marked } from "marked";
import { useSkillAgentStatus } from "../../composables/useSkillAgentStatus";
import type { Skill, Agent } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";
import SkillDetail from "./SkillDetail.vue";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
  expanded?: boolean;
  selected?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:expanded", value: boolean): void;
  (e: "toggle:select", skillId: string): void;
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const toast = useToast();

const agentsRef = computed(() => props.agents);
const skillRef = computed(() => props.skill);
const { allAgentStatuses, syncedCount, totalCount } =
  useSkillAgentStatus(skillRef, agentsRef, (k, p) => t(k, p as Record<string, unknown>));

const expandedLocal = ref(false);
const isExpanded = computed({
  get: () => props.expanded ?? expandedLocal.value,
  set: (v: boolean) => {
    if (props.expanded === undefined) {
      expandedLocal.value = v;
    } else {
      emit("update:expanded", v);
    }
  },
});

const previewContent = ref("");
const previewLoading = ref(false);
const showPreview = ref(false);
const showDeleteConfirm = ref(false);

async function toggleExpand() {
  isExpanded.value = !isExpanded.value;
}

async function togglePreview() {
  showPreview.value = !showPreview.value;
  if (showPreview.value && !previewContent.value) {
    previewLoading.value = true;
    try {
      const md = await skillsStore.previewSkill(props.skill.id);
      previewContent.value = marked.parse(md) as string;
    } catch {
      previewContent.value = "";
    } finally {
      previewLoading.value = false;
    }
  }
}

async function handleDelete() {
  try {
    await skillsStore.deleteSkill(props.skill.id);
    showDeleteConfirm.value = false;
    toast.show(t("skills.delete"), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}
</script>

<template>
  <div
    class="rounded-lg border transition-all"
    :style="{
      background: selected ? 'var(--c-primary-light)' : 'var(--c-surface)',
      borderColor: skill.has_conflict
        ? 'var(--c-warning)'
        : skill.has_dangling
          ? 'var(--c-danger)'
          : skill.is_duplicate
            ? 'var(--c-info)'
            : selected
              ? 'var(--c-primary)'
              : isExpanded
                ? 'var(--c-primary)'
                : 'var(--c-border)',
    }"
  >
    <!-- Collapsed header -->
    <div
      class="flex items-center gap-3 px-3 py-2.5 cursor-pointer select-none"
      @click="toggleExpand"
    >
      <input
        type="checkbox"
        :checked="selected"
        class="w-3.5 h-3.5 rounded cursor-pointer shrink-0"
        style="accent-color: var(--c-primary);"
        @click.stop="emit('toggle:select', props.skill.id)"
      />
      <span
        class="w-4 text-center text-xs shrink-0 transition-transform"
        :style="{ color: 'var(--c-text-secondary)', transform: isExpanded ? 'rotate(90deg)' : 'rotate(0deg)' }"
      >▶</span>

      <span v-if="skill.has_conflict" class="shrink-0" style="color: var(--c-warning);">⚠</span>
      <span v-else-if="skill.has_dangling" class="shrink-0" style="color: var(--c-danger);">❌</span>
      <span v-else-if="skill.is_duplicate" class="shrink-0" style="color: var(--c-info);">📋</span>

      <span class="text-sm font-medium truncate" style="color: var(--c-text);">
        {{ skill.name || skill.id }}
      </span>

      <span
        v-if="skill.missing_name"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-danger-light); color: var(--c-danger);"
      >
        {{ t("manage.missing_name") }}
      </span>

      <span class="text-[11px] shrink-0" style="color: var(--c-text-secondary);">
        {{ syncedCount }}/{{ totalCount }}
      </span>

      <!-- Agent 迷你状态徽章 -->
      <span class="flex items-center gap-0.5 shrink-0 overflow-hidden">
        <span
          v-for="item in allAgentStatuses"
          :key="item.agent.id"
          class="w-2 h-2 rounded-full shrink-0"
          :style="{ background: item.statusColor, fontSize: '8px' }"
          :title="`${item.agent.name}: ${item.statusLabel}`"
        />
      </span>

      <span
        v-if="skill.has_conflict"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-warning-light); color: var(--c-warning);"
      >
        {{ t("manage.status_conflict") }}
      </span>
      <span
        v-else-if="skill.has_dangling"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-danger-light); color: var(--c-danger);"
      >
        {{ t("manage.status_dangling") }}
      </span>

      <div class="flex items-center gap-1 ml-auto shrink-0">
        <button
          class="w-6 h-6 flex items-center justify-center rounded cursor-pointer transition-colors"
          :style="{
            background: showPreview ? 'var(--c-primary-light)' : 'transparent',
            color: showPreview ? 'var(--c-primary)' : 'var(--c-text-secondary)',
          }"
          @click.stop="togglePreview"
          :title="t('skills.preview')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
            <circle cx="12" cy="12" r="3"/>
          </svg>
        </button>
        <button
          class="w-6 h-6 flex items-center justify-center rounded cursor-pointer transition-colors hover:bg-[var(--c-danger-light)]"
          style="color: var(--c-danger);"
          @click.stop="showDeleteConfirm = true"
          :title="t('skills.delete')"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
        </button>
      </div>
    </div>

    <div v-if="showPreview" class="px-3 pb-3">
      <div
        v-if="previewContent"
        class="markdown-body rounded-md border p-3 max-h-[300px] overflow-y-auto"
        style="background: var(--c-bg); border-color: var(--c-border);"
        v-html="previewContent"
      />
      <div v-else-if="previewLoading" class="text-xs" style="color: var(--c-text-secondary);">
        {{ t("app.loading") }}
      </div>
    </div>

    <!-- Expanded detail: 共用 SkillDetail -->
    <div v-if="isExpanded" class="border-t" style="border-color: var(--c-border);">
      <SkillDetail :skill="skill" :agents="agents" />
    </div>

    <ConfirmDialog
      v-if="showDeleteConfirm"
      :title="t('skills.delete')"
      :message="t('skills.delete_confirm', { name: skill.name })"
      :confirm-text="t('skills.delete')"
      :danger="true"
      @confirm="handleDelete"
      @cancel="showDeleteConfirm = false"
    />
  </div>
</template>