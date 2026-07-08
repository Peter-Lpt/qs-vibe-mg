<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Skill, Agent, SkillSource } from "../../types";

const props = defineProps<{
  skills: Skill[];
  agents: Agent[];
  expandedSkillId?: string | null;
}>();

const emit = defineEmits<{
  (e: "expand-skill", skillId: string): void;
  (e: "action", skillId: string, agentId: string): void;
}>();

const { t } = useI18n();

const detectedAgents = computed(() => props.agents.filter((a) => a.detected));

interface CellInfo {
  status: "origin" | "synced" | "independent" | "linked_elsewhere" | "dangling" | "unlinked";
  icon: string;
  color: string;
  source: SkillSource | null;
}

function getCell(skill: Skill, agent: Agent): CellInfo {
  const source = skill.sources.find((s) => s.from === agent.id);
  if (!source) return { status: "unlinked", icon: "○", color: "var(--c-text-secondary)", source: null };
  if (source.from === "vibe-lib") return { status: "origin", icon: "📦", color: "var(--c-success)", source };
  if (!source.is_symlink) return { status: "independent", icon: "●", color: "var(--c-text)", source };
  if (!source.symlink_target) return { status: "dangling", icon: "❌", color: "var(--c-danger)", source };
  const vibeLib = skill.sources.find((s) => s.from === "vibe-lib");
  if (vibeLib?.path && source.symlink_target.includes(vibeLib.path))
    return { status: "synced", icon: "●", color: "var(--c-primary)", source };
  return { status: "linked_elsewhere", icon: "⚠", color: "var(--c-warning)", source };
}

function handleCellClick(skill: Skill, agent: Agent, cell: CellInfo) {
  if (cell.status === "unlinked") return;
  emit("action", skill.id, agent.id);
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
              <span
                class="inline-block w-3 h-3 leading-3 rounded-full transition-colors"
                :style="{
                  background: getCell(skill, agent).color,
                  cursor: getCell(skill, agent).status !== 'unlinked' ? 'pointer' : 'default',
                }"
                :title="`${skill.name || skill.id} @ ${agent.name}: ${getCell(skill, agent).status}`"
              />
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="px-3 py-1.5 text-[10px] flex items-center gap-3" style="color: var(--c-text-secondary);">
      <span>● {{ t("manage.status_synced") }} ● {{ t("manage.real_file") }} ⚠ {{ t("manage.status_linked_elsewhere") }} ○ {{ t("manage.status_unlinked") }} ❌ {{ t("manage.status_dangling") }}</span>
    </div>
  </div>
</template>
