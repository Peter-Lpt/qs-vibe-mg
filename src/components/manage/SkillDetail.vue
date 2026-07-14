<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import { useFileLogger } from "../../composables/useFileLogger";
import { marked } from "marked";
import { useSkillAgentStatus, actionStyle, cellBtnLabel } from "../../composables/useSkillAgentStatus";
import type { Skill, Agent, SkillSource } from "../../types";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const toast = useToast();
const logger = useFileLogger();

const agentsRef = computed(() => props.agents);
const skillRef = computed(() => props.skill);
const { allAgentStatuses, groupedStatuses, vibeSource } =
  useSkillAgentStatus(skillRef, agentsRef, (k, p) => t(k, p as Record<string, unknown>));

// ── per-agent 批量选择 ──
const selectedAgents = ref<Set<string>>(new Set());
const showBatchMenu = ref(false);
const batchOperating = ref(false);

const batchAvailableActions = computed(() => {
  const selected = allAgentStatuses.value.filter((s) =>
    selectedAgents.value.has(s.agent.id)
  );
  if (selected.length === 0) return [];

  const actions: { action: string; label: string; color: string }[] = [];
  const hasLink = selected.some((s) => s.action === "link");
  const hasSync = selected.some((s) => s.action === "sync_to_vibe");
  const hasReplace = selected.some((s) => s.action === "replace_with_link");
  const hasRelink = selected.some((s) => s.action === "relink");
  const hasUnlink = selected.some((s) => s.action === "unlink");
  const hasClean = selected.some((s) => s.action === "remove_dangling");

  if (hasLink) actions.push({ action: "link", label: t("manage.btn_link"), color: "var(--c-primary)" });
  if (hasSync) actions.push({ action: "sync_to_vibe", label: t("manage.btn_sync"), color: "var(--c-primary)" });
  if (hasReplace) actions.push({ action: "replace_with_link", label: t("manage.btn_replace"), color: "var(--c-text)" });
  if (hasRelink) actions.push({ action: "relink", label: t("manage.btn_relink"), color: "var(--c-warning)" });
  if (hasUnlink) actions.push({ action: "unlink", label: t("manage.btn_unlink"), color: "var(--c-text-secondary)" });
  if (hasClean) actions.push({ action: "remove_dangling", label: t("manage.btn_clean"), color: "var(--c-danger)" });

  return actions;
});

function toggleAgentSelect(agentId: string) {
  if (selectedAgents.value.has(agentId)) {
    selectedAgents.value.delete(agentId);
  } else {
    selectedAgents.value.add(agentId);
  }
}

function toggleSelectAllAgents() {
  const allIds = allAgentStatuses.value.map((s) => s.agent.id);
  if (selectedAgents.value.size === allIds.length) {
    selectedAgents.value.clear();
  } else {
    allIds.forEach((id) => selectedAgents.value.add(id));
  }
}

// ── per-agent 操作 ──
const resolvingConflict = ref<string | null>(null);

async function handleAction(status: ReturnType<typeof useSkillAgentStatus>["allAgentStatuses"]["value"][number]) {
  if (status.action === "none") return;

  try {
    switch (status.action) {
      case "link":
        await skillsStore.createLink(props.skill.id, status.agent.id);
        toast.show(t("skills.linked", { agent: status.agent.name }), "success");
        break;
      case "unlink":
        await skillsStore.removeLink(props.skill.id, status.agent.id);
        toast.show(t("skills.unlinked", { agent: status.agent.name }), "success");
        break;
      case "sync_to_vibe":
        resolvingConflict.value = status.agent.id;
        await skillsStore.syncToVibe(props.skill.id, status.agent.id);
        toast.show(t("manage.synced_to_vibe", { agent: status.agent.name }), "success");
        break;
      case "replace_with_link":
        await skillsStore.syncToVibe(props.skill.id, status.agent.id);
        toast.show(t("manage.replaced_with_link", { agent: status.agent.name }), "success");
        break;
      case "relink":
        await skillsStore.relink(props.skill.id, status.agent.id);
        toast.show(t("manage.relinked", { agent: status.agent.name }), "success");
        break;
      case "remove_dangling":
        await skillsStore.removeLink(props.skill.id, status.agent.id);
        toast.show(t("manage.dangling_removed", { agent: status.agent.name }), "success");
        break;
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    resolvingConflict.value = null;
  }
}

async function handleBatchAction(action: string) {
  const selected = allAgentStatuses.value.filter(
    (s) => selectedAgents.value.has(s.agent.id) && s.action === action
  );
  if (selected.length === 0) return;

  batchOperating.value = true;

  try {
    const agentIds = selected.map((s) => s.agent.id);
    const result = await skillsStore.batchSkillAction(props.skill.id, agentIds, action);

    if (result.errors.length > 0) {
      toast.show(t("manage.batch_result", { success: result.synced_count, error: result.errors.length }), "info");
    } else {
      toast.show(t("manage.batch_success", { count: result.synced_count }), "success");
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    batchOperating.value = false;
    selectedAgents.value.clear();
    showBatchMenu.value = false;
  }
}

// ── 冲突预览 ──
const conflictPreviewPath = ref<string | null>(null);
const conflictPreviewContent = ref("");
const conflictPreviewLoading = ref(false);

async function previewConflictPath(path: string) {
  if (conflictPreviewPath.value === path) {
    conflictPreviewPath.value = null;
    conflictPreviewContent.value = "";
    return;
  }

  conflictPreviewPath.value = path;
  conflictPreviewLoading.value = true;
  try {
    const md = await skillsStore.previewSkillAtPath(path);
    conflictPreviewContent.value = marked.parse(md) as string;
  } catch {
    conflictPreviewContent.value = "";
  } finally {
    conflictPreviewLoading.value = false;
  }
}

function getAgentNameFromPath(path: string): string {
  logger.debug(`[getAgentNameFromPath] input: ${path}`);
  const lowerPath = path.toLowerCase();
  if (lowerPath.includes(".claude/skills") || lowerPath.includes(".claude\\skills")) return "Claude";
  if (lowerPath.includes(".hermes/skills") || lowerPath.includes(".hermes\\skills")) return "Hermes";
  if (lowerPath.includes(".pi/agent/skills") || lowerPath.includes(".pi\\agent\\skills")) return "Pi";
  if (lowerPath.includes(".config/opencode/skills") || lowerPath.includes(".config\\opencode\\skills")) return "OpenCode";
  if (lowerPath.includes(".codex/skills") || lowerPath.includes(".codex\\skills")) return "Codex";
  if (lowerPath.includes(".config/mimocode/skills") || lowerPath.includes(".config\\mimocode\\skills")) return "MimoCode";
  if (lowerPath.includes(".agents/skills") || lowerPath.includes(".agents\\skills")) return "Agents";
  if (lowerPath.includes(".vibe-skills") || lowerPath.includes(".vibe_skills")) return "VibeLib";
  const match = path.match(/[/\\]\.?([^/\\]+)[/\\]skills/);
  if (match && match[1]) {
    logger.debug(`[getAgentNameFromPath] regex match: ${match[1]}`);
    return match[1].charAt(0).toUpperCase() + match[1].slice(1);
  }
  logger.debug("[getAgentNameFromPath] no match, returning empty");
  return "";
}

async function useThisVersion(source: SkillSource) {
  try {
    const agent = props.agents.find((a) => a.id === source.from);
    if (agent) {
      await skillsStore.syncToVibe(props.skill.id, agent.id);
      toast.show(t("manage.synced_to_vibe", { agent: agent.name }), "success");
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}
</script>

<template>
  <div>
    <!-- 冲突路径 -->
    <div v-if="skill.has_conflict" class="px-3 pb-3">
      <div class="mb-2 text-[10px] font-medium uppercase tracking-wide" style="color: var(--c-warning);">
        {{ t("manage.conflict_paths") }}
      </div>
      <div class="space-y-2">
        <div
          v-for="source in skill.sources"
          :key="source.path"
          class="rounded-md border p-2"
          style="background: var(--c-bg); border-color: var(--c-border);"
        >
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-[10px]" :style="{ color: source.is_symlink ? 'var(--c-primary)' : 'var(--c-text-secondary)' }">
                {{ source.is_symlink ? '🔗' : '📁' }}
              </span>
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-1.5">
                  <span class="text-[10px] truncate" style="color: var(--c-text-secondary);">
                    {{ source.from }}: {{ source.path.split(/[/\\]/).slice(-2, -1)[0] || source.path.split(/[/\\]/).pop() }}
                  </span>
                  <span class="text-[10px] shrink-0" :style="{ color: source.is_symlink ? 'var(--c-primary)' : 'var(--c-text-secondary)' }">
                    {{ source.is_symlink ? t("manage.symlink_to", { target: source.symlink_target?.split(/[/\\]/).pop() || '' }) : t("manage.real_file") }}
                  </span>
                </div>
                <div class="text-[10px] truncate mt-0.5" :title="source.path" style="color: var(--c-text-secondary); opacity: 0.7;">
                  {{ source.path }}
                </div>
              </div>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <button
                class="text-[10px] px-1.5 py-0.5 rounded cursor-pointer transition-colors"
                :style="{
                  background: conflictPreviewPath === source.path ? 'var(--c-primary-light)' : 'transparent',
                  color: conflictPreviewPath === source.path ? 'var(--c-primary)' : 'var(--c-text-secondary)',
                  border: '1px solid var(--c-border)',
                }"
                @click.stop="previewConflictPath(source.path)"
              >
                {{ t("manage.preview_path") }}
              </button>
              <button
                v-if="source.from !== 'vibe-lib'"
                class="text-[10px] px-1.5 py-0.5 rounded cursor-pointer transition-colors"
                style="background: var(--c-primary); color: white;"
                @click.stop="useThisVersion(source)"
              >
                {{ t("manage.use_this_version") }}
              </button>
            </div>
          </div>
          <div v-if="conflictPreviewPath === source.path" class="mt-2">
            <div
              v-if="conflictPreviewContent"
              class="markdown-body rounded border p-2 max-h-[200px] overflow-y-auto text-[11px]"
              style="background: var(--c-surface); border-color: var(--c-border);"
              v-html="conflictPreviewContent"
            />
            <div v-else-if="conflictPreviewLoading" class="text-[10px]" style="color: var(--c-text-secondary);">
              {{ t("app.loading") }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Per-agent 详情 -->
    <div class="px-3 pb-3">
      <div
        v-if="allAgentStatuses.length > 0"
        class="flex items-center gap-2 mb-2 pb-2 border-b"
        style="border-color: var(--c-border);"
      >
        <input
          type="checkbox"
          :checked="selectedAgents.size === allAgentStatuses.length && allAgentStatuses.length > 0"
          class="w-3.5 h-3.5 rounded cursor-pointer"
          style="accent-color: var(--c-primary);"
          @change="toggleSelectAllAgents"
        />
        <span class="text-[10px]" style="color: var(--c-text-secondary);">
          {{ t("manage.select_agents") }}
        </span>

        <div v-if="selectedAgents.size > 0" class="relative ml-auto">
          <button
            class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
            style="background: var(--c-primary); color: white;"
            :disabled="batchOperating || batchAvailableActions.length === 0"
            @click.stop="showBatchMenu = !showBatchMenu"
          >
            {{ t("manage.batch_apply") }} ({{ selectedAgents.size }}) ▾
          </button>
          <div
            v-if="showBatchMenu"
            class="absolute top-full right-0 mt-1 rounded-md border shadow-lg z-10 py-1 min-w-[140px]"
            style="background: var(--c-surface); border-color: var(--c-border);"
          >
            <button
              v-for="act in batchAvailableActions"
              :key="act.action"
              class="block w-full text-left px-3 py-1.5 text-xs hover:bg-[var(--c-surface-hover)] cursor-pointer"
              :style="{ color: act.color }"
              @click.stop="handleBatchAction(act.action)"
            >
              {{ act.label }}
            </button>
            <div v-if="batchAvailableActions.length === 0" class="px-3 py-1.5 text-xs" style="color: var(--c-text-secondary);">
              {{ t("manage.no_batch_actions") }}
            </div>
          </div>
        </div>
      </div>

      <div
        v-for="group in groupedStatuses"
        :key="group.label"
        class="mb-3 last:mb-0"
      >
        <div
          class="flex items-center gap-2 mb-1.5 text-[10px] font-medium uppercase tracking-wide"
          :style="{ color: group.color }"
        >
          <span class="w-1.5 h-1.5 rounded-full" :style="{ background: group.color }" />
          {{ group.label }} ({{ group.items.length }})
        </div>

        <div class="space-y-1">
          <div
            v-for="item in group.items"
            :key="item.agent.id"
            class="flex items-center gap-2 px-2 py-1.5 rounded"
            :style="{
              background: selectedAgents.has(item.agent.id) ? 'var(--c-primary-light)' : 'var(--c-bg)',
            }"
          >
            <!-- 左侧：flex-1 min-w-0，内联所有信息 -->
            <div class="flex items-center gap-2 min-w-0 flex-1">
              <input
                type="checkbox"
                :checked="selectedAgents.has(item.agent.id)"
                class="w-3.5 h-3.5 rounded cursor-pointer shrink-0"
                style="accent-color: var(--c-primary);"
                @click.stop="toggleAgentSelect(item.agent.id)"
              />
              <span class="text-[10px] shrink-0" :style="{ color: item.statusColor }">
                {{ item.statusIcon }}
              </span>
              <span class="text-xs font-medium truncate min-w-0" style="color: var(--c-text);">
                {{ item.agent.name }}
              </span>
              <span class="text-[10px] shrink-0" :style="{ color: item.statusColor }">
                {{ item.statusLabel }}
              </span>
              <!-- independent/conflict: 显示真实文件路径 -->
              <span
                v-if="item.status === 'independent' && item.source"
                class="text-[10px] truncate max-w-[150px] shrink-0 cursor-help"
                style="color: var(--c-text-secondary); opacity: 0.7;"
                :title="item.source.path"
              >
                {{ item.source.path }}
              </span>
              <!-- linked_elsewhere: 显示当前链接和期望链接 -->
              <template v-if="item.status === 'linked_elsewhere' && item.source?.symlink_target">
                <span
                  class="text-[10px] truncate flex-1 min-w-0 cursor-help"
                  style="color: var(--c-warning);"
                  :title="item.source.symlink_target"
                >
                  → {{ getAgentNameFromPath(item.source.symlink_target) || item.source.symlink_target.split(/[/\\]/).slice(-2, -1)[0] || '未知' }}
                </span>
                <span class="text-[10px] shrink-0" style="color: var(--c-text-secondary);">|</span>
                <span
                  class="text-[10px] shrink-0 cursor-help"
                  style="color: var(--c-success);"
                  :title="vibeSource?.path || ''"
                >
                  → 库
                </span>
              </template>
              <span
                v-else-if="item.source?.symlink_target && item.status !== 'unlinked'"
                class="text-[10px] truncate max-w-[150px] shrink-0"
                style="color: var(--c-text-secondary);"
              >
                → {{ item.source.symlink_target.split(/[/\\]/).pop() }}
              </span>
            </div>

            <!-- 右侧：操作按钮 -->
            <button
              v-if="item.action !== 'none'"
              class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors shrink-0 whitespace-nowrap"
              :style="actionStyle(item.action)"
              :disabled="resolvingConflict === item.agent.id"
              :title="item.action === 'sync_to_vibe' ? t('manage.sync_scope_tip', { skill: skill.name || skill.id, agent: item.agent.name }) : undefined"
              @click.stop="handleAction(item)"
            >
              {{ resolvingConflict === item.agent.id ? "..." : cellBtnLabel(t, item.action, item.agent.name) }}
            </button>
            <span
              v-else
              class="text-[10px] shrink-0"
              style="color: var(--c-text-secondary);"
            >
              —
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>