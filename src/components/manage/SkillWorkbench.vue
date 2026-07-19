<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Agent, Skill } from "../../types";
import { classifySkillSources } from "./manageFilters";
import SkillWorkbenchRow from "./SkillWorkbenchRow.vue";

const props = defineProps<{
  skills: Skill[];
  agents: Agent[];
  selectedIds: Set<string>;
  expandedSkillId?: string | null;
  allVisibleSelected?: boolean;
  partiallyVisibleSelected?: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle:select", skillId: string): void;
  (e: "toggle:all"): void;
  (e: "open:detail", skillId: string): void;
  (e: "request:add-agent"): void;
}>();

const { t } = useI18n();

const detectedAgents = computed(() => props.agents.filter((agent) => agent.detected));
const agentCount = computed(() => detectedAgents.value.length);
const layoutMode = computed<"empty" | "single" | "matrix" | "wide">(() => {
  if (agentCount.value === 0) return "empty";
  if (agentCount.value === 1) return "single";
  if (agentCount.value <= 4) return "matrix";
  return "wide";
});

const attentionSkills = computed(() => props.skills.filter((skill) => {
  const sourceInfo = classifySkillSources(skill, detectedAgents.value);
  return skill.has_conflict || skill.has_dangling || sourceInfo.hasLinkedElsewhere;
}));
const normalSkills = computed(() => props.skills.filter((skill) => !attentionSkills.value.includes(skill)));
const allVisibleSelected = computed(() => props.skills.length > 0 && props.skills.every((skill) => props.selectedIds.has(skill.id)));

const gridStyle = computed(() => ({
  gridTemplateColumns: `minmax(${layoutMode.value === "single" ? "220px" : "280px"}, 1.65fr) repeat(${agentCount.value}, minmax(${layoutMode.value === "wide" ? "150px" : "132px"}, 1fr)) minmax(92px, 0.5fr)`,
}));

function onExpanded(skillId: string, expanded: boolean) {
  emit("open:detail", expanded ? skillId : "");
}
</script>

<template>
  <section class="workbench-shell">
    <div class="workbench-heading">
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <input
            type="checkbox"
            :checked="props.allVisibleSelected ?? allVisibleSelected"
            :indeterminate="props.partiallyVisibleSelected ?? false"
            class="h-3.5 w-3.5 cursor-pointer rounded"
            style="accent-color: var(--c-primary);"
            :title="t('manage.workbench_select_filtered')"
            @change="emit('toggle:all')"
          />
          <h3 class="text-sm font-semibold" style="color: var(--c-text-strong);">{{ t("manage.workbench_title") }}</h3>
          <span class="workbench-count">{{ skills.length }}</span>
        </div>
        <p class="mt-0.5 text-[11px]" style="color: var(--c-text-secondary);">{{ t("manage.workbench_hint") }}</p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <span v-if="layoutMode === 'single'" class="workbench-mode-label">{{ t("manage.workbench_single_agent") }}</span>
        <span v-else-if="layoutMode === 'wide'" class="workbench-mode-label workbench-scroll-hint"><ArrowLeftRight :size="12" /> {{ t("manage.workbench_wide_agents", { count: agentCount }) }}</span>
        <span v-else-if="layoutMode === 'matrix'" class="workbench-mode-label">{{ t("manage.workbench_agents_count", { count: agentCount }) }}</span>
      </div>
    </div>

    <div v-if="layoutMode === 'empty'" class="workbench-empty">
      <UsersRound :size="22" style="color: var(--c-primary);" />
      <div>
        <div class="text-sm font-semibold" style="color: var(--c-text-strong);">{{ t("manage.workbench_no_agents") }}</div>
        <div class="mt-1 text-xs" style="color: var(--c-text-secondary);">{{ t("manage.workbench_no_agents_hint") }}</div>
      </div>
      <button class="workbench-primary-button" type="button" @click="emit('request:add-agent')">
        <Plus :size="14" />
        {{ t("agents.add") }}
      </button>
    </div>

    <div v-else class="workbench-scroll" :class="{ 'workbench-scroll-wide': layoutMode === 'wide' }">
      <div class="workbench-grid workbench-header-row" :style="gridStyle">
        <div class="workbench-header-cell workbench-header-skill">{{ t("manage.workbench_skill") }}</div>
        <div v-for="agent in detectedAgents" :key="agent.id" class="workbench-header-cell" :title="agent.skills_dir">
          <span class="workbench-agent-dot" :class="agent.detected ? 'workbench-agent-online' : ''" />
          <span class="truncate">{{ agent.name }}</span>
        </div>
        <div class="workbench-header-cell">{{ t("manage.workbench_action") }}</div>
      </div>

      <div v-if="attentionSkills.length > 0" class="workbench-group-label workbench-group-label-attention">
        <TriangleAlert :size="13" />
        {{ t("manage.group_needs_action") }}
        <span>{{ attentionSkills.length }}</span>
      </div>
      <SkillWorkbenchRow
        v-for="skill in attentionSkills"
        :key="`attention-${skill.id}`"
        :id="`skill-${skill.id}`"
        :skill="skill"
        :agents="detectedAgents"
        :selected="selectedIds.has(skill.id)"
        :expanded="expandedSkillId === skill.id"
        :grid-style="gridStyle"
        @toggle:select="emit('toggle:select', $event)"
        @update:expanded="onExpanded(skill.id, $event)"
      />

      <div v-if="normalSkills.length > 0" class="workbench-group-label workbench-group-label-normal">
        <CheckCircle2 :size="13" />
        {{ t("manage.group_normal") }}
        <span>{{ normalSkills.length }}</span>
      </div>
      <SkillWorkbenchRow
        v-for="skill in normalSkills"
        :key="`normal-${skill.id}`"
        :id="`skill-${skill.id}`"
        :skill="skill"
        :agents="detectedAgents"
        :selected="selectedIds.has(skill.id)"
        :expanded="expandedSkillId === skill.id"
        :grid-style="gridStyle"
        @toggle:select="emit('toggle:select', $event)"
        @update:expanded="onExpanded(skill.id, $event)"
      />
    </div>
  </section>
</template>
