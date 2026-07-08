<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import { useSkillAgentStatus, actionLabel, actionStyle } from "../../composables/useSkillAgentStatus";
import type { Skill, Agent } from "../../types";
import SkillRow from "./SkillRow.vue";

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

const cardBorderColor = computed(() => {
  if (props.skill.has_conflict) return "var(--c-warning)";
  if (props.skill.has_dangling) return "var(--c-danger)";
  if (props.skill.is_duplicate) return "var(--c-info)";
  return "var(--c-border)";
});

const primaryActionLabel = computed(() =>
  summary.value.primaryAction !== "none"
    ? actionLabel(t, summary.value.primaryAction)
    : ""
);

async function handlePrimaryAction() {
  const status = allAgentStatuses.value.find(
    (s) => s.action === summary.value.primaryAction
  );
  if (!status) return;
  try {
    switch (status.action) {
      case "link":
        await skillsStore.createLink(props.skill.id, status.agent.id);
        break;
      case "unlink":
        await skillsStore.removeLink(props.skill.id, status.agent.id);
        break;
      case "sync_to_vibe":
        await skillsStore.syncToVibe(props.skill.id, status.agent.id);
        break;
      case "replace_with_link":
        await skillsStore.syncToVibe(props.skill.id, status.agent.id);
        break;
      case "relink":
        await skillsStore.relink(props.skill.id, status.agent.id);
        break;
      case "remove_dangling":
        await skillsStore.removeLink(props.skill.id, status.agent.id);
        break;
    }
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
      borderColor: selected ? 'var(--c-primary)' : isExpanded ? 'var(--c-primary)' : cardBorderColor,
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
        <span v-if="skill.has_conflict" class="shrink-0 mt-0.5" style="color: var(--c-warning);">⚠</span>
        <span v-else-if="skill.has_dangling" class="shrink-0 mt-0.5" style="color: var(--c-danger);">❌</span>
        <span v-else-if="skill.is_duplicate" class="shrink-0 mt-0.5" style="color: var(--c-info);">📋</span>

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
          {{ isExpanded ? t("skills.hide_preview") || "收起" : t("manage.expand_detail") }}
        </button>
        <button
          v-if="summary.primaryAction !== 'none'"
          class="text-[11px] px-2.5 py-1.5 rounded-md cursor-pointer transition-colors shrink-0"
          :style="actionStyle(summary.primaryAction)"
          @click="handlePrimaryAction"
        >
          {{ primaryActionLabel }}
        </button>
      </div>
    </div>

    <!-- Expanded detail (reuse SkillRow) -->
    <div v-if="isExpanded" class="border-t" style="border-color: var(--c-border);">
      <SkillRow :skill="skill" :agents="agents" :expanded="true" />
    </div>
  </div>
</template>
