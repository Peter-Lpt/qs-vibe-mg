<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { useToast } from "../../composables/useToast";
import type { Skill, Agent } from "../../types";
import AgentStatusBadge from "./AgentStatusBadge.vue";
import ConflictWarning from "./ConflictWarning.vue";
import DanglingWarning from "./DanglingWarning.vue";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const toast = useToast();

const expanded = ref(false);
const previewContent = ref("");
const previewLoading = ref(false);
const showLinkMenu = ref(false);
const showUnlinkMenu = ref(false);
const showDeleteConfirm = ref(false);

// 获取非 vibe-lib 的 sources
const agentSources = computed(() =>
  props.skill.sources.filter((s) => s.from !== "vibe-lib")
);

// 获取所有可用的 agent（用于 Link 操作）
const linkableAgents = computed(() => {
  const linkedIds = new Set(
    props.skill.sources
      .filter((s) => s.from !== "vibe-lib")
      .map((s) => s.from)
  );
  return agentsStore.agents.filter(
    (a) => a.detected && !linkedIds.has(a.id)
  );
});

// 获取已链接的 agents（用于 Unlink 操作）
const unlinkableAgents = computed(() => {
  const linkedIds = new Set(
    props.skill.sources
      .filter((s) => s.from !== "vibe-lib")
      .map((s) => s.from)
  );
  return agentsStore.agents.filter(
    (a) => a.detected && linkedIds.has(a.id)
  );
});

// 操作按钮：根据状态决定显示哪种
const actionType = computed(() => {
  if (props.skill.has_dangling) return "dangling";
  if (props.skill.has_conflict) return "conflict";
  if (linkableAgents.value.length > 0) return "link";
  if (unlinkableAgents.value.length > 0) return "unlink";
  return "none";
});

async function toggleExpand() {
  expanded.value = !expanded.value;
  if (expanded.value && !previewContent.value) {
    previewLoading.value = true;
    try {
      previewContent.value = await skillsStore.previewSkill(props.skill.id);
    } catch {
      previewContent.value = "";
    } finally {
      previewLoading.value = false;
    }
  }
}

async function handleLink(agentId: string) {
  try {
    await skillsStore.createLink(props.skill.id, agentId);
    toast.show(t("skills.linked", { agent: agentId }), "success");
    showLinkMenu.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleUnlink(agentId: string) {
  try {
    await skillsStore.removeLink(props.skill.id, agentId);
    toast.show(t("skills.unlinked", { agent: agentId }), "success");
    showUnlinkMenu.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleRemoveBroken() {
  // 移除所有断链的 sources
  for (const source of props.skill.sources.filter(
    (s) => s.is_symlink && s.from !== "vibe-lib"
  )) {
    try {
      await skillsStore.removeLink(props.skill.id, source.from);
    } catch {
      // 忽略单个失败
    }
  }
  toast.show(t("manage.remove_broken"), "success");
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

function getSourceCount() {
  const count = agentSources.value.length;
  const total = agentsStore.agents.filter((a) => a.detected).length;
  return `${count}/${total}`;
}
</script>

<template>
  <div
    class="rounded-lg border transition-all"
    :style="{
      background: 'var(--c-surface)',
      borderColor: skill.has_conflict
        ? 'var(--c-warning)'
        : skill.has_dangling
          ? 'var(--c-danger)'
          : expanded
            ? 'var(--c-primary)'
            : 'var(--c-border)',
    }"
  >
    <!-- Collapsed header -->
    <div
      class="flex items-center gap-3 px-3 py-2.5 cursor-pointer select-none"
      @click="toggleExpand"
    >
      <!-- Expand arrow -->
      <span
        class="w-4 text-center text-xs shrink-0 transition-transform"
        :style="{ color: 'var(--c-text-secondary)', transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)' }"
      >▶</span>

      <!-- Status icon -->
      <span v-if="skill.has_conflict" class="shrink-0" style="color: var(--c-warning);">⚠</span>
      <span v-else-if="skill.has_dangling" class="shrink-0" style="color: var(--c-danger);">❌</span>

      <!-- Skill name -->
      <span class="text-sm font-medium truncate" style="color: var(--c-text);">
        {{ skill.name }}
      </span>

      <!-- Agent tags -->
      <div class="flex items-center gap-1 flex-wrap">
        <span
          v-for="source in agentSources"
          :key="source.from"
          class="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
          style="background: var(--c-primary-light); color: var(--c-primary);"
        >
          {{ agents.find(a => a.id === source.from)?.name || source.from }}
        </span>
      </div>

      <!-- Source count -->
      <span class="text-[11px] shrink-0" style="color: var(--c-text-secondary);">
        {{ getSourceCount() }}
      </span>

      <!-- Status badge -->
      <span
        v-if="skill.has_conflict"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-warning-light); color: var(--c-warning);"
      >
        {{ t("status_conflict") || "冲突" }}
      </span>
      <span
        v-else-if="skill.has_dangling"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-danger-light); color: var(--c-danger);"
      >
        {{ t("status_dangling") || "断链" }}
      </span>

      <!-- Delete button -->
      <button
        class="w-5 h-5 flex items-center justify-center rounded opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer shrink-0 ml-auto"
        style="color: var(--c-danger);"
        @click.stop="showDeleteConfirm = true"
        :title="t('skills.delete')"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        </svg>
      </button>
    </div>

    <!-- Expanded details -->
    <div v-if="expanded" class="px-3 pb-3 space-y-3">
      <!-- Agent sources list -->
      <div class="space-y-1.5 pl-7">
        <AgentStatusBadge
          v-for="source in skill.sources"
          :key="source.from + source.path"
          :source="source"
          :agents="agents"
        />
      </div>

      <!-- Conflict warning -->
      <div v-if="skill.has_conflict" class="pl-7">
        <ConflictWarning
          :sources="skill.sources.filter(s => s.from !== 'vibe-lib')"
          @resolve="(from) => handleLink(from)"
        />
      </div>

      <!-- Dangling warning -->
      <div v-if="skill.has_dangling" class="pl-7">
        <DanglingWarning
          :sources="skill.sources"
          @remove="handleRemoveBroken"
        />
      </div>

      <!-- Action buttons -->
      <div class="pl-7 flex items-center gap-2 relative">
        <!-- Link to Agent -->
        <div v-if="actionType === 'link'" class="relative">
          <button
            class="text-xs px-3 py-1.5 rounded-md cursor-pointer transition-colors"
            style="background: var(--c-primary); color: white;"
            @click.stop="showLinkMenu = !showLinkMenu"
          >
            {{ t("manage.link_to_agent") || "Link to Agent" }} ▾
          </button>
          <div
            v-if="showLinkMenu"
            class="absolute top-full left-0 mt-1 rounded-md border shadow-lg z-10 py-1 min-w-[160px]"
            style="background: var(--c-surface); border-color: var(--c-border);"
          >
            <button
              v-for="agent in linkableAgents"
              :key="agent.id"
              class="block w-full text-left px-3 py-1.5 text-xs hover:bg-[var(--c-surface-hover)] cursor-pointer"
              style="color: var(--c-text);"
              @click.stop="handleLink(agent.id)"
            >
              {{ agent.name }}
            </button>
            <div v-if="linkableAgents.length === 0" class="px-3 py-1.5 text-xs" style="color: var(--c-text-secondary);">
              {{ t("manage.no_agents_available") || "无可用 Agent" }}
            </div>
          </div>
        </div>

        <!-- Remove Link -->
        <div v-if="actionType === 'unlink'" class="relative">
          <button
            class="text-xs px-3 py-1.5 rounded-md cursor-pointer transition-colors"
            style="background: var(--c-surface-hover); color: var(--c-text-secondary); border: 1px solid var(--c-border);"
            @click.stop="showUnlinkMenu = !showUnlinkMenu"
          >
            {{ t("manage.remove_link") || "Remove Link" }} ▾
          </button>
          <div
            v-if="showUnlinkMenu"
            class="absolute top-full left-0 mt-1 rounded-md border shadow-lg z-10 py-1 min-w-[160px]"
            style="background: var(--c-surface); border-color: var(--c-border);"
          >
            <button
              v-for="agent in unlinkableAgents"
              :key="agent.id"
              class="block w-full text-left px-3 py-1.5 text-xs hover:bg-[var(--c-surface-hover)] cursor-pointer"
              style="color: var(--c-text);"
              @click.stop="handleUnlink(agent.id)"
            >
              {{ agent.name }}
            </button>
          </div>
        </div>

        <!-- Preview -->
        <button
          class="text-xs px-3 py-1.5 rounded-md cursor-pointer transition-colors"
          style="background: var(--c-surface-hover); color: var(--c-text-secondary); border: 1px solid var(--c-border);"
          @click.stop
        >
          {{ t("skills.preview") || "Preview" }}
        </button>
      </div>

      <!-- SKILL.md preview -->
      <div v-if="previewContent" class="pl-7">
        <div
          class="rounded-md border p-3 text-xs max-h-[200px] overflow-y-auto prose prose-xs"
          style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text-secondary);"
          v-html="previewContent"
        />
      </div>
      <div v-else-if="previewLoading" class="pl-7 text-xs" style="color: var(--c-text-secondary);">
        {{ t("app.loading") }}
      </div>
    </div>

    <!-- Delete Confirm -->
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
