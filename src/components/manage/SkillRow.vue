<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "../../composables/useToast";
import { useSkillAgentStatus } from "../../composables/useSkillAgentStatus";
import { useSkillActions } from "../../composables/useSkillActions";
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
const toast = useToast();
const actions = useSkillActions((k, p) => t(k, p as Record<string, unknown>));

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
const showAgentLegend = ref(false);
const hasLibrarySource = computed(() => props.skill.sources.some((s) => s.from === "vibe-lib"));

async function toggleExpand() {
  isExpanded.value = !isExpanded.value;
}

async function togglePreview() {
  showPreview.value = !showPreview.value;
  if (showPreview.value && !previewContent.value) {
    previewLoading.value = true;
    previewContent.value = await actions.loadPreview(props.skill);
    previewLoading.value = false;
  }
}

async function handleDelete() {
  try {
    await actions.deleteLibrarySkill(props.skill);
    showDeleteConfirm.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

function statusTip(item: (typeof allAgentStatuses.value)[number]): string {
  const parts = [`${item.agent.name}: ${item.statusLabel}`, item.agent.skills_dir];
  if (item.source?.path) parts.push(`${t("manage.source_path")}: ${item.source.path}`);
  if (item.source?.symlink_target) parts.push(`${t("manage.symlink_target")}: ${item.source.symlink_target}`);
  return parts.join("\n");
}
</script>

<template>
  <div
    class="skill-row-shell"
    :style="{
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
      class="flex items-center gap-3 px-3.5 py-3 cursor-pointer select-none"
      @click="toggleExpand"
    >
      <input
        type="checkbox"
        :checked="selected"
        class="w-3.5 h-3.5 rounded cursor-pointer shrink-0"
        style="accent-color: var(--c-primary);"
        @click.stop="emit('toggle:select', props.skill.id)"
      />
      <ChevronRight
        class="w-4 text-center shrink-0 transition-transform"
        :size="14"
        :style="{ color: 'var(--c-text-secondary)', transform: isExpanded ? 'rotate(90deg)' : 'rotate(0deg)' }"
      />

      <TriangleAlert v-if="skill.has_conflict" class="shrink-0" :size="14" style="color: var(--c-warning);" />
      <CircleX v-else-if="skill.has_dangling" class="shrink-0" :size="14" style="color: var(--c-danger);" />
      <Copy v-else-if="skill.is_duplicate" class="shrink-0" :size="14" style="color: var(--c-info);" />

      <span class="text-sm font-semibold truncate" style="color: var(--c-text-strong);">
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

      <!-- Agent 状态点 -->
      <span class="relative flex items-center gap-0.5 shrink-0 overflow-visible">
        <span
          v-for="item in allAgentStatuses"
          :key="item.agent.id"
          class="w-3 h-3 rounded-full shrink-0 inline-flex items-center justify-center text-[8px] font-medium cursor-help"
          :style="{ background: item.statusColor, color: item.status === 'unlinked' ? 'var(--c-text-secondary)' : 'white', border: item.status === 'unlinked' ? '1px solid var(--c-border)' : '0', boxShadow: item.status === 'unlinked' ? 'none' : '0 0 0 2px var(--c-surface)' }"
          :title="statusTip(item)"
          @mouseenter="showAgentLegend = true"
        />
        <button
          class="w-5 h-5 inline-flex items-center justify-center rounded cursor-help"
          style="color: var(--c-text-secondary);"
          :title="t('manage.agent_status_legend')"
          @mouseenter="showAgentLegend = true"
          @mouseleave="showAgentLegend = false"
          @click.stop="showAgentLegend = !showAgentLegend"
        >
          <CircleAlert :size="12" />
        </button>
        <div
          v-if="showAgentLegend"
          class="absolute right-0 top-6 z-20 w-64 rounded-md border p-2 shadow-lg"
          style="background: var(--c-surface-raised); border-color: var(--c-border);"
          @mouseenter="showAgentLegend = true"
          @mouseleave="showAgentLegend = false"
        >
          <div class="text-[10px] font-medium mb-1" style="color: var(--c-text);">
            {{ t("manage.agent_status_legend") }}
          </div>
          <div v-for="item in allAgentStatuses" :key="item.agent.id" class="flex items-start gap-2 py-0.5">
            <span class="w-2 h-2 rounded-full mt-1 shrink-0" :style="{ background: item.statusColor }" />
            <div class="min-w-0">
              <div class="text-[10px] truncate" style="color: var(--c-text);">
                {{ item.agent.name }} · {{ item.statusLabel }}
              </div>
              <div class="text-[9px] truncate" style="color: var(--c-text-secondary);" :title="item.source?.path || item.agent.skills_dir">
                {{ item.source?.path || item.agent.skills_dir }}
              </div>
            </div>
          </div>
        </div>
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
          <Eye :size="14" />
        </button>
        <button
          v-if="hasLibrarySource"
          class="w-6 h-6 flex items-center justify-center rounded cursor-pointer transition-colors hover:bg-[var(--c-danger-light)]"
          style="color: var(--c-danger);"
          @click.stop="showDeleteConfirm = true"
          :title="t('skills.delete_library')"
        >
          <Trash2 :size="14" />
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

<!-- Expanded per-agent detail (shared with card via SkillDetail) -->
    <div v-if="isExpanded" class="border-t" style="border-color: var(--c-border);">
      <SkillDetail :skill="skill" :agents="agents" />
    </div>

    <ConfirmDialog
      v-if="showDeleteConfirm"
      :title="t('skills.delete_library')"
      :message="t('skills.delete_library_confirm', { name: skill.name })"
      :confirm-text="t('skills.delete_library')"
      :danger="true"
      @confirm="handleDelete"
      @cancel="showDeleteConfirm = false"
    />
  </div>
</template>
