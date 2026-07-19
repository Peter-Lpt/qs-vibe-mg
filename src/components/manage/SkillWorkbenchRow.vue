<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useSkillAgentStatus, type AgentStatus } from "../../composables/useSkillAgentStatus";
import type { Agent, Skill } from "../../types";
import { classifySkillSources } from "./manageFilters";
import SkillDetail from "./SkillDetail.vue";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
  selected?: boolean;
  expanded?: boolean;
  gridStyle?: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: "toggle:select", skillId: string): void;
  (e: "update:expanded", value: boolean): void;
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const expandedLocal = ref(false);
const isExpanded = computed(() => props.expanded ?? expandedLocal.value);
const skillRef = computed(() => props.skill);
const agentsRef = computed(() => props.agents);
const { allAgentStatuses } = useSkillAgentStatus(skillRef, agentsRef, (key, params) =>
  t(key, params as Record<string, unknown>)
);

const statusesByAgent = computed(() => new Map(allAgentStatuses.value.map((status) => [status.agent.id, status])));
const needsAttention = computed(() => {
  const sourceInfo = classifySkillSources(props.skill, props.agents);
  return props.skill.has_conflict || props.skill.has_dangling || sourceInfo.hasLinkedElsewhere;
});

function statusFor(agent: Agent): AgentStatus | undefined {
  return statusesByAgent.value.get(agent.id);
}

function toggleExpanded() {
  const next = !isExpanded.value;
  if (props.expanded === undefined) expandedLocal.value = next;
  emit("update:expanded", next);
}

function displayPath(path: string | undefined): string {
  return (path || "").replace(/[\\/]+/g, "/");
}

function statusLabel(status: AgentStatus | undefined): string {
  if (!status) return t("manage.status_unlinked");
  return status.statusLabel;
}
</script>

<template>
  <div class="workbench-row" :class="{ 'workbench-row-expanded': isExpanded, 'workbench-row-attention': needsAttention }">
    <div class="workbench-grid workbench-row-main" :class="{ 'workbench-row-selected': selected }" :style="gridStyle">
      <div class="workbench-skill-cell" @click="toggleExpanded">
        <input
          type="checkbox"
          :checked="selected"
          class="h-3.5 w-3.5 shrink-0 cursor-pointer rounded"
          style="accent-color: var(--c-primary);"
          @click.stop="emit('toggle:select', skill.id)"
        />
        <span class="workbench-severity" :class="needsAttention ? 'workbench-severity-warning' : 'workbench-severity-ok'" />
        <div class="min-w-0 flex-1">
          <div class="flex min-w-0 items-center gap-2">
            <span class="truncate text-[13px] font-semibold" style="color: var(--c-text-strong);">{{ skill.name || skill.id }}</span>
            <span v-if="skill.missing_name" class="workbench-badge workbench-badge-danger">{{ t("manage.missing_name") }}</span>
            <span v-if="skill.is_duplicate" class="workbench-badge workbench-badge-info">{{ t("manage.status_duplicate") }}</span>
            <span v-if="skillsStore.updateChecks[skill.id]?.available" class="workbench-badge workbench-badge-warning">{{ t("manage.skill_update_available") }}</span>
          </div>
          <div class="mt-0.5 flex min-w-0 items-center gap-2">
            <span class="truncate text-[11px]" style="color: var(--c-text-secondary);">{{ displayPath(skill.path) }}</span>
            <span class="shrink-0 text-[10px]" style="color: var(--c-text-tertiary);">{{ skill.sources.length }} {{ t("manage.sources_title") }}</span>
          </div>
        </div>
        <span v-if="needsAttention" class="workbench-badge workbench-badge-warning">{{ t("manage.group_needs_action") }}</span>
      </div>

      <button
        v-for="agent in agents"
        :key="agent.id"
        class="workbench-status-cell"
        type="button"
        :title="`${agent.name} · ${statusLabel(statusFor(agent))}`"
        @click="toggleExpanded"
      >
        <template v-if="statusFor(agent)">
          <span class="workbench-status-dot" :style="{ background: statusFor(agent)?.statusColor }" />
          <span class="truncate text-[11px]" :style="{ color: statusFor(agent)?.statusColor }">{{ statusLabel(statusFor(agent)) }}</span>
        </template>
        <span v-else class="text-[11px]" style="color: var(--c-text-tertiary);">—</span>
      </button>

    </div>

    <div v-if="isExpanded" class="workbench-detail">
      <SkillDetail :skill="skill" :agents="agents" embedded />
    </div>
  </div>
</template>
