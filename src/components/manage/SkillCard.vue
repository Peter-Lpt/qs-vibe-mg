<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import { useSkillAgentStatus, cellBtnLabel } from "../../composables/useSkillAgentStatus";
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
const skillsStore = useSkillsStore();
const toast = useToast();
const actions = useSkillActions((k, p) => t(k, p as Record<string, unknown>));

const agentsRef = computed(() => props.agents);
const skillRef = computed(() => props.skill);
const { summary, syncedCount, totalCount, allAgentStatuses } =
  useSkillAgentStatus(skillRef, agentsRef, (k, p) => t(k, p as Record<string, unknown>));

const expandedLocal = ref(false);
const isExpanded = computed({
  get: () => props.expanded ?? expandedLocal.value,
  set: (v: boolean) => {
    if (props.expanded === undefined) expandedLocal.value = v;
    else emit("update:expanded", v);
  },
});

const showDeleteConfirm = ref(false);

const cardBorderColor = computed(() => {
  if (props.skill.has_conflict) return "var(--c-warning)";
  if (props.skill.has_dangling) return "var(--c-danger)";
  if (props.skill.is_duplicate) return "var(--c-info)";
  return "var(--c-border)";
});

const primaryActionLabel = computed(() =>
  summary.value.primaryAction !== "none"
    ? cellBtnLabel((k, p) => t(k, p as Record<string, unknown>), summary.value.primaryAction, primaryAgentName.value)
    : ""
);

const primaryAgentName = computed(() => {
  const status = allAgentStatuses.value.find(
    (s) => s.action === summary.value.primaryAction
  );
  return status?.agent.name ?? "";
});

async function handlePrimaryAction() {
  const status = allAgentStatuses.value.find(
    (s) => s.action === summary.value.primaryAction
  );
  if (!status) return;
  if (status.action === "sync_to_vibe" && props.skill.has_conflict) {
    isExpanded.value = true;
    toast.show(t("manage.conflict_use_resolution"), "warning");
    return;
  }
  try {
    await actions.runAgentAction(props.skill, status);
  } catch (e: unknown) {
    toast.show(String(e), "error");
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
  <!-- 展开时占满整行（grid-column:1/-1），消除同排卡片下方空白 -->
  <div
    class="rounded-lg border transition-all"
    :style="{
      background: selected ? 'var(--c-primary-light)' : 'var(--c-surface)',
      borderColor: selected ? 'var(--c-primary)' : isExpanded ? 'var(--c-primary)' : cardBorderColor,
      gridColumn: isExpanded ? '1 / -1' : undefined,
    }"
  >
    <!-- Summary header -->
    <div class="p-3">
      <div class="flex items-start gap-2">
        <input
          type="checkbox"
          :checked="selected"
          class="w-3.5 h-3.5 rounded cursor-pointer shrink-0 mt-0.5"
          style="accent-color: var(--c-primary);"
          @click.stop="emit('toggle:select', props.skill.id)"
        />
        <TriangleAlert v-if="skill.has_conflict" class="shrink-0 mt-0.5" :size="14" style="color: var(--c-warning);" />
        <CircleX v-else-if="skill.has_dangling" class="shrink-0 mt-0.5" :size="14" style="color: var(--c-danger);" />
        <Copy v-else-if="skill.is_duplicate" class="shrink-0 mt-0.5" :size="14" style="color: var(--c-info);" />

        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
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
          </div>
          <p class="text-[11px] mt-0.5 line-clamp-2" style="color: var(--c-text-secondary);">
            {{ skill.description || t("skills.no_skills_hint") }}
          </p>
        </div>

        <!-- 小删除图标（与列表行一致，不占用按钮） -->
        <button
          class="w-6 h-6 flex items-center justify-center rounded cursor-pointer transition-colors hover:bg-[var(--c-danger-light)] shrink-0"
          style="color: var(--c-danger);"
          @click.stop="showDeleteConfirm = true"
          :title="t('skills.delete')"
        >
          <Trash2 :size="14" />
        </button>

        <span class="text-[11px] shrink-0" style="color: var(--c-text-secondary);">
          {{ syncedCount }}/{{ totalCount }}
        </span>
      </div>

      <!-- Aggregate status -->
      <div class="flex flex-wrap items-center gap-x-3 gap-y-1 mt-2.5 text-[10px]" style="color: var(--c-text-secondary);">
        <span v-if="summary.synced > 0" style="color: var(--c-primary);">
          {{ t("manage.card_synced_count", { count: summary.synced }) }}
        </span>
        <span v-if="summary.needsAction > 0" style="color: var(--c-warning);">
          {{ t("manage.card_needs_action_count", { count: summary.needsAction }) }}
        </span>
        <span v-if="summary.unlinked > 0" style="color: var(--c-text-secondary);">
          {{ t("manage.card_unlinked_count", { count: summary.unlinked }) }}
        </span>
        <span v-if="summary.dangling > 0" style="color: var(--c-danger);">
          {{ t("manage.status_dangling") }} {{ summary.dangling }}
        </span>
        <span v-if="summary.synced > 0 && summary.needsAction === 0 && summary.unlinked === 0 && summary.dangling === 0" style="color: var(--c-success);">
          {{ t("manage.card_synced_all") }}
        </span>
      </div>

      <!-- Actions -->
      <div class="flex items-center gap-2 mt-2.5">
        <button
          class="text-[11px] px-2.5 py-1.5 rounded-md cursor-pointer transition-colors"
          :style="{
            background: 'var(--c-surface-hover)',
            color: 'var(--c-text)',
            border: '1px solid var(--c-border)',
          }"
          @click="isExpanded = !isExpanded"
        >
          {{ isExpanded ? (t("skills.hide_preview") || "收起") : t("manage.expand_detail") }}
        </button>
        <button
          v-if="summary.primaryAction !== 'none'"
          class="text-[11px] px-2.5 py-1.5 rounded-md cursor-pointer transition-colors shrink-0"
          :style="summary.primaryAction === 'unlink' || summary.primaryAction === 'remove_dangling'
            ? 'background: var(--c-surface-hover); color: var(--c-text-secondary); border: 1px solid var(--c-border);'
            : 'background: var(--c-primary); color: white;'"
          @click="handlePrimaryAction"
        >
          {{ primaryActionLabel }}
        </button>
        <button
          class="text-[11px] px-2.5 py-1.5 rounded-md cursor-pointer transition-colors shrink-0 hover:bg-[var(--c-danger-light)]"
          style="color: var(--c-danger); border: 1px solid var(--c-border);"
          :title="t('skills.delete')"
          @click="showDeleteConfirm = true"
        >
          {{ t("skills.delete") }}
        </button>
      </div>
    </div>

<!-- Expanded detail (shared SkillDetail, no longer nested SkillRow) -->
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
