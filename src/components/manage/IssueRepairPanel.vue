<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Agent, Skill } from "../../types";

const props = defineProps<{
  skills: Skill[];
  agents: Agent[];
}>();

const emit = defineEmits<{
  (e: "select-group", skillIds: string[], openBatch: boolean, repairContext: string): void;
}>();

const { t } = useI18n();

const detectedAgents = computed(() => props.agents.filter((agent) => agent.detected));

const groups = computed(() => {
  const agentIds = detectedAgents.value.map((agent) => agent.id);
  const data = [
    {
      id: "conflict",
      label: t("manage.repair_conflicts"),
      icon: "TriangleAlert",
      color: "var(--c-warning)",
      skills: props.skills.filter((skill) => skill.has_conflict),
      batch: false,
    },
    {
      id: "dangling",
      label: t("manage.repair_dangling"),
      icon: "CircleSlash",
      color: "var(--c-danger)",
      skills: props.skills.filter((skill) => skill.has_dangling),
      batch: true,
    },
    {
      id: "missing_lib",
      label: t("manage.repair_missing_lib"),
      icon: "Package",
      color: "var(--c-primary)",
      skills: props.skills.filter((skill) => {
        const hasLibrary = skill.sources.some((source) => source.from === "vibe-lib");
        const hasAgent = skill.sources.some((source) => source.from !== "vibe-lib" && !source.from.startsWith("project:"));
        const hasProject = skill.sources.some((source) => source.from.startsWith("project:"));
        return !hasLibrary && hasAgent && hasProject;
      }),
      batch: true,
    },
    {
      id: "uncovered",
      label: t("manage.repair_uncovered"),
      icon: "CircleDashed",
      color: "var(--c-text-secondary)",
      skills: props.skills.filter((skill) => {
        const hasLibrary = skill.sources.some((source) => source.from === "vibe-lib");
        if (!hasLibrary) return false;
        return agentIds.some((agentId) => !skill.sources.some((source) => source.from === agentId));
      }),
      batch: true,
    },
    {
      id: "only_agent",
      label: t("manage.repair_only_agent"),
      icon: "Folder",
      color: "var(--c-text-secondary)",
      skills: props.skills.filter((skill) => {
        const hasLibrary = skill.sources.some((source) => source.from === "vibe-lib");
        const hasAgent = skill.sources.some((source) => source.from !== "vibe-lib" && !source.from.startsWith("project:"));
        const hasProject = skill.sources.some((source) => source.from.startsWith("project:"));
        return !hasLibrary && hasAgent && !hasProject;
      }),
      batch: true,
    },
    {
      id: "only_project",
      label: t("manage.repair_only_project"),
      icon: "FileBox",
      color: "var(--c-text-secondary)",
      skills: props.skills.filter((skill) => {
        const hasLibrary = skill.sources.some((source) => source.from === "vibe-lib");
        const hasProject = skill.sources.some((source) => source.from.startsWith("project:"));
        const hasAgent = skill.sources.some((source) => source.from !== "vibe-lib" && !source.from.startsWith("project:"));
        return !hasLibrary && hasProject && !hasAgent;
      }),
      batch: false,
    },
  ];
  return data.filter((group) => group.skills.length > 0);
});
</script>

<template>
  <div v-if="groups.length > 0" class="mb-3 rounded-lg border p-3" style="background: var(--c-surface); border-color: var(--c-border);">
    <div class="flex items-center gap-2 mb-2">
      <CircleAlert :size="14" style="color: var(--c-warning);" />
      <span class="text-xs font-semibold" style="color: var(--c-text);">
        {{ t("manage.repair_title") }}
      </span>
      <span class="text-[10px]" style="color: var(--c-text-secondary);">
        {{ t("manage.repair_subtitle") }}
      </span>
    </div>
    <div class="grid gap-2" style="grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));">
      <button
        v-for="group in groups"
        :key="group.id"
        class="rounded-md border px-2.5 py-2 text-left cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
        style="border-color: var(--c-border); background: var(--c-bg);"
        @click="emit('select-group', group.skills.map((skill) => skill.id), group.batch, group.id)"
      >
        <div class="flex items-center gap-2">
          <component :is="group.icon" :size="14" :style="{ color: group.color }" />
          <span class="text-[11px] font-medium truncate" style="color: var(--c-text);">{{ group.label }}</span>
          <span class="text-[11px] ml-auto" :style="{ color: group.color }">{{ group.skills.length }}</span>
        </div>
      </button>
    </div>
  </div>
</template>
