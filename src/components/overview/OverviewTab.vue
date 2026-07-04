<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import AgentOverviewCard from "./AgentOverviewCard.vue";
import EmptyState from "../common/EmptyState.vue";
import SkeletonCard from "../common/SkeletonCard.vue";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();

onMounted(async () => {
  if (skillsStore.skills.length === 0) {
    await skillsStore.fetchSkills();
  }
  if (skillsStore.issues.length === 0) {
    await skillsStore.fetchIssues();
  }
});

const detectedAgents = computed(() =>
  agentsStore.agents.filter((a) => a.detected)
);

const totalSkills = computed(() => skillsStore.skills.length);

const sharedSkills = computed(() =>
  skillsStore.skills.filter((s) =>
    s.sources.filter((src) => src.from !== "vibe-lib").length > 1
  )
);

const uniqueSkills = computed(() =>
  skillsStore.skills.filter((s) =>
    s.sources.filter((src) => src.from !== "vibe-lib").length === 1
  )
);

const issueSkills = computed(() =>
  skillsStore.skills.filter((s) => s.has_conflict || s.has_dangling)
);

// 关系矩阵数据
const matrixData = computed(() => {
  const agents = detectedAgents.value;
  const skills = skillsStore.skills.filter((s) =>
    s.sources.some((src) => src.from !== "vibe-lib")
  );

  return skills.map((skill) => ({
    skill,
    agentPresence: agents.map((agent) => ({
      agentId: agent.id,
      agentName: agent.name,
      hasSkill: skill.sources.some((src) => src.from === agent.id),
    })),
  }));
});

function handleAgentFilter(_agentId: string) {
  // 切换到 manage tab 并过滤
  // 由父组件处理
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-5">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t("overview.title") || "总览" }}
      </h2>
    </div>

    <!-- Stats bar -->
    <div
      class="flex items-center gap-4 mb-5 px-4 py-2.5 rounded-lg text-xs"
      style="background: var(--c-surface); border: 1px solid var(--c-border);"
    >
      <span style="color: var(--c-text);">
        {{ t("manage.total_skills") || "共" }} {{ totalSkills }} {{ t("manage.skill_count") || "个 skill" }}
      </span>
      <span style="color: var(--c-text-secondary);">|</span>
      <span style="color: var(--c-primary);">
        {{ sharedSkills.length }} {{ t("manage.linked_count") || "个共享" }}
      </span>
      <span style="color: var(--c-text-secondary);">|</span>
      <span style="color: var(--c-text-secondary);">
        {{ uniqueSkills.length }} {{ t("manage.status_unlinked") || "个独立" }}
      </span>
      <template v-if="issueSkills.length > 0">
        <span style="color: var(--c-text-secondary);">|</span>
        <span style="color: var(--c-warning);">
          {{ issueSkills.length }} {{ t("manage.conflict_count") || "个异常" }}
        </span>
      </template>
    </div>

    <!-- Agent overview cards -->
    <div v-if="agentsStore.loading" class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3 mb-5">
      <SkeletonCard v-for="i in 4" :key="i" />
    </div>

    <div v-else class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3 mb-5">
      <AgentOverviewCard
        v-for="agent in detectedAgents"
        :key="agent.id"
        :agent="agent"
        :skills="skillsStore.skills"
        @filter="handleAgentFilter"
      />
    </div>

    <!-- Relation matrix -->
    <div
      v-if="matrixData.length > 0"
      class="rounded-lg border overflow-hidden mb-5"
      style="background: var(--c-surface); border-color: var(--c-border);"
    >
      <div class="flex items-center justify-between px-3 py-2 border-b" style="border-color: var(--c-border);">
        <h3 class="text-xs font-semibold" style="color: var(--c-text);">
          {{ t("dashboard.relation_graph") || "关系矩阵" }}
        </h3>
        <span class="text-xs" style="color: var(--c-text-secondary);">
          {{ matrixData.length }} {{ t("dashboard.skills") || "skills" }}
        </span>
      </div>

      <div class="overflow-x-auto">
        <table class="w-full text-xs">
          <thead>
            <tr style="border-bottom: 1px solid var(--c-border);">
              <th
                class="px-3 py-2 text-left font-medium sticky left-0"
                style="background: var(--c-surface); color: var(--c-text-secondary); min-width: 140px;"
              >
                {{ t("dashboard.shared_skills") || "Skill" }}
              </th>
              <th
                v-for="agent in detectedAgents"
                :key="agent.id"
                class="px-2 py-2 text-center font-medium"
                style="color: var(--c-text-secondary); min-width: 60px;"
              >
                <div class="flex flex-col items-center gap-0.5">
                  <span class="truncate max-w-[60px]">{{ agent.name }}</span>
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(row, idx) in matrixData"
              :key="row.skill.id"
              :style="{
                background: idx % 2 === 0 ? 'transparent' : 'rgba(128,128,128,0.03)',
                borderBottom: '1px solid var(--c-border)',
              }"
            >
              <td
                class="px-3 py-1.5 sticky left-0"
                :style="{
                  background: idx % 2 === 0 ? 'var(--c-surface)' : 'rgba(128,128,128,0.03)',
                }"
              >
                <div class="flex items-center gap-1.5">
                  <span
                    v-if="row.skill.has_conflict"
                    class="w-1.5 h-1.5 rounded-full shrink-0"
                    style="background: var(--c-warning);"
                  />
                  <span
                    v-else-if="row.skill.has_dangling"
                    class="w-1.5 h-1.5 rounded-full shrink-0"
                    style="background: var(--c-danger);"
                  />
                  <span
                    v-else
                    class="w-1.5 h-1.5 rounded-full shrink-0"
                    style="background: var(--c-primary);"
                  />
                  <span class="truncate" style="color: var(--c-text);">
                    {{ row.skill.name }}
                  </span>
                </div>
              </td>
              <td
                v-for="presence in row.agentPresence"
                :key="presence.agentId"
                class="px-2 py-1.5 text-center"
              >
                <span
                  v-if="presence.hasSkill"
                  class="inline-block w-3 h-3 rounded-full"
                  style="background: var(--c-primary);"
                  :title="presence.agentName"
                />
                <span v-else class="inline-block w-3 h-3" />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Issues list -->
    <div
      v-if="issueSkills.length > 0"
      class="rounded-lg border overflow-hidden"
      style="background: var(--c-surface); border-color: var(--c-border);"
    >
      <div class="px-3 py-2 border-b" style="border-color: var(--c-border);">
        <h3 class="text-xs font-semibold" style="color: var(--c-text);">
          {{ t("overview.issues_title") || "异常检测" }}
        </h3>
      </div>
      <div class="divide-y" style="border-color: var(--c-border);">
        <div
          v-for="skill in issueSkills"
          :key="skill.id"
          class="px-3 py-2 flex items-center gap-2 text-xs"
        >
          <span v-if="skill.has_conflict" style="color: var(--c-warning);">⚠</span>
          <span v-else style="color: var(--c-danger);">❌</span>
          <span class="font-medium" style="color: var(--c-text);">{{ skill.name }}</span>
          <span style="color: var(--c-text-secondary);">—</span>
          <span style="color: var(--c-text-secondary);">
            {{
              skill.has_conflict
                ? t("manage.conflict_warning") || "同名 skill 有不同内容"
                : t("manage.dangling_warning") || "链接目标已删除"
            }}
          </span>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <EmptyState
      v-if="!agentsStore.loading && detectedAgents.length === 0"
      icon="🔧"
      :title="t('agents.empty_title') || '暂无 Agent'"
      :description="t('agents.empty_hint') || '请先添加 Agent'"
    />
  </div>
</template>
