<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Agent, Skill } from "../../types";
import { classifySkillSources } from "./manageFilters";

export type RepairGroupId = "conflict" | "dangling" | "missing_lib" | "linked_elsewhere";

const props = defineProps<{
  skills: Skill[];
  agents: Agent[];
  compact?: boolean;
}>();

const emit = defineEmits<{
  // 点击分组 = 应用对应筛选（显式动作，不自动全选、不自动弹批量）
  (e: "filter-group", group: RepairGroupId): void;
}>();

const { t } = useI18n();

// 四组谓词统一走 classifySkillSources（判定同源）：
// 插件/外部来源不再计入 Agent —— 与旧实现相比组计数会变化，属预期修正。
const groups = computed(() => {
  const data = [
    {
      id: "conflict" as const,
      label: t("manage.repair_conflicts"),
      icon: "TriangleAlert",
      color: "var(--c-warning)",
      skills: props.skills.filter((skill) => skill.has_conflict),
      actionable: true,
    },
    {
      id: "dangling" as const,
      label: t("manage.repair_dangling"),
      icon: "CircleSlash",
      color: "var(--c-danger)",
      skills: props.skills.filter((skill) => skill.has_dangling),
      actionable: true,
    },
    {
      id: "missing_lib" as const,
      label: t("manage.repair_missing_lib"),
      icon: "Package",
      color: "var(--c-primary)",
      skills: props.skills.filter((skill) => {
        const c = classifySkillSources(skill, props.agents);
        return !c.hasLibrary && c.hasAgent && c.hasProject;
      }),
      actionable: true,
    },
    {
      id: "linked_elsewhere" as const,
      label: t("manage.repair_linked_elsewhere"),
      icon: "Share2",
      color: "var(--c-warning)",
      skills: props.skills.filter((skill) => {
        const c = classifySkillSources(skill, props.agents);
        return c.hasLinkedElsewhere;
      }),
      actionable: true,
    },
  ];
  return data.filter((group) => group.skills.length > 0);
});

// 总数按唯一 skill id 去重（组间可重叠，不按组长度求和）
const totalIssueSkills = computed(() => {
  const ids = new Set<string>();
  for (const group of groups.value) {
    for (const skill of group.skills) ids.add(skill.id);
  }
  return ids.size;
});

function handleGroupClick(group: (typeof groups.value)[number]) {
  emit("filter-group", group.id);
}
</script>

<template>
  <div
    v-if="groups.length > 0"
    :class="props.compact ? 'rounded-lg border p-2.5' : 'workspace-panel'"
    :style="props.compact ? { borderColor: 'var(--c-border-subtle)', background: 'var(--c-bg)' } : undefined"
  >
    <div class="mb-2 flex items-center gap-2">
      <CircleAlert :size="14" style="color: var(--c-warning);" />
      <span class="text-xs font-semibold" style="color: var(--c-text-strong);">
        {{ t("manage.repair_title") }}
      </span>
      <span class="truncate text-[11px]" style="color: var(--c-text-secondary);">
        {{ totalIssueSkills }} {{ t("manage.repair_groups") }}
      </span>
    </div>
    <div
      class="grid grid-cols-1 gap-1.5 sm:grid-cols-2"
      :style="props.compact ? undefined : { gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))' }"
    >
      <button
        v-for="group in groups"
        :key="group.id"
        type="button"
        :class="props.compact ? 'rounded-md border px-2.5 py-2 text-left cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]' : 'agent-overview-card px-3 py-2.5 text-left cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]'"
        :style="props.compact ? { borderColor: 'var(--c-border-subtle)' } : undefined"
        @click="handleGroupClick(group)"
      >
        <div class="flex items-center gap-2">
          <component :is="group.icon" :size="13" :style="{ color: group.color }" />
          <span class="truncate text-[10px] font-medium" style="color: var(--c-text);">{{ group.label }}</span>
          <span class="ml-auto text-[11px]" :style="{ color: group.color }">{{ group.skills.length }}</span>
        </div>
      </button>
    </div>
  </div>
</template>
