<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Skill, Agent, SkillSource } from "../../types";
import { samePath } from "../../composables/useSkillAgentStatus";

const props = defineProps<{
  skills: Skill[];
  agents: Agent[];
  expandedSkillId?: string | null;
}>();

const emit = defineEmits<{
  (e: "expand-skill", skillId: string): void;
}>();

const { t } = useI18n();

const detectedAgents = computed(() => props.agents.filter((a) => a.detected));

interface CellInfo {
  status: "origin" | "synced" | "independent" | "linked_elsewhere" | "dangling" | "unlinked";
  icon: string;
  color: string;
  label: string;
  source: SkillSource | null;
}

function getCell(skill: Skill, agent: Agent): CellInfo {
  const source = skill.sources.find((s) => s.from === agent.id);
  if (!source) {
    return {
      status: "unlinked",
      icon: "CircleDashed",
      color: "var(--c-text-secondary)",
      label: t("manage.status_unlinked"),
      source: null,
    };
  }
  if (source.from === "vibe-lib") {
    return {
      status: "origin",
      icon: "Package",
      color: "var(--c-success)",
      label: t("manage.status_origin"),
      source,
    };
  }
  if (!source.is_symlink) {
    return {
      status: "independent",
      icon: "Folder",
      color: skill.sources.some((s) => s.from === "vibe-lib") && source.content_hash !== skill.sources.find((s) => s.from === "vibe-lib")?.content_hash
        ? "var(--c-warning)"
        : "var(--c-text)",
      label: skill.sources.some((s) => s.from === "vibe-lib") && source.content_hash !== skill.sources.find((s) => s.from === "vibe-lib")?.content_hash
        ? t("manage.status_independent_conflict")
        : t("manage.status_independent"),
      source,
    };
  }
  if (!source.symlink_target || source.content_hash === "") {
    return {
      status: "dangling",
      icon: "CircleSlash",
      color: "var(--c-danger)",
      label: t("manage.status_dangling"),
      source,
    };
  }
  const vibeLib = skill.sources.find((s) => s.from === "vibe-lib");
  if (vibeLib?.path && samePath(source.symlink_target, vibeLib.path)) {
    return {
      status: "synced",
      icon: "Link2",
      color: "var(--c-primary)",
      label: t("manage.status_synced"),
      source,
    };
  }
  return {
    status: "linked_elsewhere",
    icon: "TriangleAlert",
    color: "var(--c-warning)",
    label: t("manage.status_linked_elsewhere"),
    source,
  };
}

function handleCellClick(skill: Skill, agent: Agent, cell: CellInfo) {
  void agent;
  void cell;
  emit("expand-skill", skill.id);
}

function cellTitle(skill: Skill, agent: Agent, cell: CellInfo): string {
  const parts = [
    `${skill.name || skill.id} @ ${agent.name}`,
    `${t("manage.matrix_status")}: ${cell.label}`,
    `${t("agents.skills_dir")}: ${agent.skills_dir}`,
  ];
  if (cell.source?.path) parts.push(`${t("manage.source_path")}: ${cell.source.path}`);
  if (cell.source?.symlink_target) parts.push(`${t("manage.symlink_target")}: ${cell.source.symlink_target}`);
  return parts.join("\n");
}
</script>

<template>
  <div
    class="rounded-lg border overflow-hidden"
    style="background: var(--c-surface); border-color: var(--c-border);"
  >
    <div class="px-3 py-2 border-b" style="border-color: var(--c-border);">
      <h3 class="text-xs font-semibold" style="color: var(--c-text);">
        {{ t("manage.agent_matrix") }}
        <span class="ml-1 font-normal" style="color: var(--c-text-secondary);">
          {{ t("manage.matrix_diagnostic_hint") }}
        </span>
      </h3>
    </div>

    <div class="overflow-x-auto">
      <table class="w-full text-xs">
        <thead>
          <tr style="border-bottom: 1px solid var(--c-border);">
            <th
              class="px-3 py-2 text-left font-medium sticky left-0"
              style="background: var(--c-surface); color: var(--c-text-secondary); min-width: 140px;"
            >
              {{ t("manage.status_all") || "Skill" }}
            </th>
            <th
              v-for="agent in detectedAgents"
              :key="agent.id"
              class="px-2 py-2 text-center font-medium"
              style="color: var(--c-text-secondary); min-width: 60px;"
            >
              <span class="truncate max-w-[60px] block">{{ agent.name }}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(skill, idx) in skills"
            :key="skill.id"
            :style="{
              background: expandedSkillId === skill.id
                ? 'var(--c-primary-light)'
                : idx % 2 === 0
                  ? 'transparent'
                  : 'rgba(128,128,128,0.03)',
              borderBottom: '1px solid var(--c-border)',
            }"
          >
            <td
              class="px-3 py-1.5 sticky left-0 cursor-pointer"
              :style="{
                background: expandedSkillId === skill.id
                  ? 'var(--c-primary-light)'
                  : idx % 2 === 0
                    ? 'var(--c-surface)'
                    : 'rgba(128,128,128,0.03)',
              }"
              @click="emit('expand-skill', skill.id)"
            >
              <div class="flex items-center gap-1.5">
                <span
                  v-if="skill.has_conflict"
                  class="w-1.5 h-1.5 rounded-full shrink-0"
                  style="background: var(--c-warning);"
                />
                <span
                  v-else-if="skill.has_dangling"
                  class="w-1.5 h-1.5 rounded-full shrink-0"
                  style="background: var(--c-danger);"
                />
                <span
                  v-else
                  class="w-1.5 h-1.5 rounded-full shrink-0"
                  style="background: var(--c-primary);"
                />
                <span class="truncate hover:underline" style="color: var(--c-text);">
                  {{ skill.name || skill.id }}
                </span>
              </div>
            </td>
            <td
              v-for="agent in detectedAgents"
              :key="agent.id"
              class="px-2 py-1.5 text-center cursor-pointer"
              @click="handleCellClick(skill, agent, getCell(skill, agent))"
            >
              <button
                class="inline-flex w-6 h-6 items-center justify-center rounded transition-colors"
                :style="{
                  color: getCell(skill, agent).color,
                  background: expandedSkillId === skill.id ? 'var(--c-primary-light)' : 'transparent',
                  border: getCell(skill, agent).status === 'unlinked' ? '1px dashed var(--c-border)' : '1px solid transparent',
                }"
                :title="cellTitle(skill, agent, getCell(skill, agent))"
                type="button"
              >
                <component :is="getCell(skill, agent).icon" :size="13" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="px-3 py-1.5 text-[10px] flex items-center gap-3" style="color: var(--c-text-secondary);">
      <span class="inline-flex items-center gap-1"><Link2 :size="12" /> {{ t("manage.status_synced") }}</span>
      <span class="inline-flex items-center gap-1"><Folder :size="12" /> {{ t("manage.status_independent") }}</span>
      <span class="inline-flex items-center gap-1"><TriangleAlert :size="12" /> {{ t("manage.status_linked_elsewhere") }}</span>
      <span class="inline-flex items-center gap-1"><CircleDashed :size="12" /> {{ t("manage.status_unlinked") }}</span>
      <span class="inline-flex items-center gap-1"><CircleSlash :size="12" /> {{ t("manage.status_dangling") }}</span>
    </div>
  </div>
</template>
