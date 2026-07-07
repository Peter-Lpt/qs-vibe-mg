<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import { marked } from "marked";
import type { Skill, Agent, SkillSource } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const toast = useToast();

const expanded = ref(false);
const previewContent = ref("");
const previewLoading = ref(false);
const showPreview = ref(false);
const showDeleteConfirm = ref(false);
const resolvingConflict = ref<string | null>(null);

// 冲突路径预览
const conflictPreviewPath = ref<string | null>(null);
const conflictPreviewContent = ref("");
const conflictPreviewLoading = ref(false);

// per-skill 批量选择
const selectedAgents = ref<Set<string>>(new Set());
const showBatchMenu = ref(false);
const batchOperating = ref(false);

// 获取 vibe-lib source
const vibeSource = computed(() =>
  props.skill.sources.find((s) => s.from === "vibe-lib")
);

// 获取所有 detected agents 的状态
interface AgentStatus {
  agent: Agent;
  source: SkillSource | null;
  status: "origin" | "synced" | "linked_elsewhere" | "independent" | "dangling" | "unlinked";
  action: "none" | "sync_to_vibe" | "replace_with_link" | "relink" | "remove_dangling" | "link" | "unlink";
  statusLabel: string;
  statusColor: string;
  statusIcon: string;
}

const allAgentStatuses = computed<AgentStatus[]>(() => {
  const detected = props.agents.filter((a) => a.detected);
  const result: AgentStatus[] = [];

  for (const agent of detected) {
    const source = props.skill.sources.find((s) => s.from === agent.id);

    if (!source) {
      result.push({
        agent,
        source: null,
        status: "unlinked",
        action: vibeSource.value ? "link" : "none",
        statusLabel: t("manage.status_unlinked"),
        statusColor: "var(--c-text-secondary)",
        statusIcon: "○",
      });
      continue;
    }

    if (source.from === "vibe-lib") {
      result.push({
        agent,
        source,
        status: "origin",
        action: "none",
        statusLabel: t("manage.status_origin"),
        statusColor: "var(--c-success)",
        statusIcon: "📦",
      });
      continue;
    }

    if (!source.is_symlink) {
      if (vibeSource.value) {
        if (source.content_hash === vibeSource.value.content_hash) {
          result.push({
            agent,
            source,
            status: "independent",
            action: "replace_with_link",
            statusLabel: t("manage.status_independent_same"),
            statusColor: "var(--c-text-secondary)",
            statusIcon: "●",
          });
        } else {
          result.push({
            agent,
            source,
            status: "independent",
            action: "sync_to_vibe",
            statusLabel: t("manage.status_independent_conflict"),
            statusColor: "var(--c-warning)",
            statusIcon: "⚠",
          });
        }
      } else {
        result.push({
          agent,
          source,
          status: "independent",
          action: "sync_to_vibe",
          statusLabel: t("manage.status_independent"),
          statusColor: "var(--c-text-secondary)",
          statusIcon: "●",
        });
      }
      continue;
    }

    if (!source.symlink_target) {
      result.push({
        agent,
        source,
        status: "dangling",
        action: "remove_dangling",
        statusLabel: t("manage.status_dangling"),
        statusColor: "var(--c-danger)",
        statusIcon: "❌",
      });
      continue;
    }

    if (vibeSource.value?.path && source.symlink_target.includes(vibeSource.value.path)) {
      result.push({
        agent,
        source,
        status: "synced",
        action: "unlink",
        statusLabel: t("manage.status_synced"),
        statusColor: "var(--c-primary)",
        statusIcon: "🔗",
      });
    } else {
      result.push({
        agent,
        source,
        status: "linked_elsewhere",
        action: "relink",
        statusLabel: t("manage.status_linked_elsewhere"),
        statusColor: "var(--c-warning)",
        statusIcon: "🔗",
      });
    }
  }

  return result;
});

// 按状态分组
const groupedStatuses = computed(() => {
  const groups: { label: string; items: AgentStatus[]; color: string }[] = [];

  const needsAction = allAgentStatuses.value.filter(
    (s) => s.status === "independent" || s.status === "dangling" || s.status === "linked_elsewhere"
  );
  if (needsAction.length > 0) {
    groups.push({ label: t("manage.group_needs_action"), items: needsAction, color: "var(--c-warning)" });
  }

  const normal = allAgentStatuses.value.filter(
    (s) => s.status === "origin" || s.status === "synced"
  );
  if (normal.length > 0) {
    groups.push({ label: t("manage.group_normal"), items: normal, color: "var(--c-success)" });
  }

  const unlinked = allAgentStatuses.value.filter((s) => s.status === "unlinked");
  if (unlinked.length > 0) {
    groups.push({ label: t("manage.group_unlinked"), items: unlinked, color: "var(--c-text-secondary)" });
  }

  return groups;
});

// 批量：可执行的操作
const batchAvailableActions = computed(() => {
  const selected = allAgentStatuses.value.filter((s) =>
    selectedAgents.value.has(s.agent.id)
  );
  if (selected.length === 0) return [];

  const actions: { action: string; label: string; color: string }[] = [];

  // 检查可执行的操作
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

// 统计
const syncedCount = computed(() =>
  allAgentStatuses.value.filter((s) => s.status === "synced" || s.status === "origin").length
);
const totalCount = computed(() => allAgentStatuses.value.length);

// 切换 agent 选择
function toggleAgentSelect(agentId: string) {
  if (selectedAgents.value.has(agentId)) {
    selectedAgents.value.delete(agentId);
  } else {
    selectedAgents.value.add(agentId);
  }
}

// 全选/取消全选
function toggleSelectAllAgents() {
  const allIds = allAgentStatuses.value.map((s) => s.agent.id);
  if (selectedAgents.value.size === allIds.length) {
    selectedAgents.value.clear();
  } else {
    allIds.forEach((id) => selectedAgents.value.add(id));
  }
}

// 执行单个 agent 的操作
async function handleAction(status: AgentStatus) {
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

// 批量执行操作
async function handleBatchAction(action: string) {
  const selected = allAgentStatuses.value.filter(
    (s) => selectedAgents.value.has(s.agent.id) && s.action === action
  );
  if (selected.length === 0) return;

  batchOperating.value = true;

  try {
    const agentIds = selected.map((s) => s.agent.id);
    const result = await skillsStore.batchSkillAction(
      props.skill.id,
      agentIds,
      action
    );

    if (result.errors.length > 0) {
      toast.show(
        t("manage.batch_result", {
          success: result.synced_count,
          error: result.errors.length,
        }),
        "info"
      );
    } else {
      toast.show(
        t("manage.batch_success", { count: result.synced_count }),
        "success"
      );
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    batchOperating.value = false;
    selectedAgents.value.clear();
    showBatchMenu.value = false;
  }
}

function getActionButtonLabel(status: AgentStatus): string {
  switch (status.action) {
    case "link": return t("manage.btn_link");
    case "unlink": return t("manage.btn_unlink");
    case "sync_to_vibe": return t("manage.btn_sync");
    case "replace_with_link": return t("manage.btn_replace");
    case "relink": return t("manage.btn_relink");
    case "remove_dangling": return t("manage.btn_clean");
    default: return "";
  }
}

function getActionButtonStyle(status: AgentStatus): string {
  switch (status.action) {
    case "link": return "background: var(--c-primary); color: white;";
    case "sync_to_vibe": return "background: var(--c-primary); color: white;";
    case "replace_with_link": return "background: var(--c-surface-hover); color: var(--c-text); border: 1px solid var(--c-border);";
    case "relink": return "background: var(--c-warning); color: white;";
    case "remove_dangling": return "background: var(--c-danger); color: white;";
    case "unlink": return "background: var(--c-surface-hover); color: var(--c-text-secondary); border: 1px solid var(--c-border);";
    default: return "";
  }
}

async function toggleExpand() {
  expanded.value = !expanded.value;
  if (!expanded.value) {
    selectedAgents.value.clear();
  }
}

async function togglePreview() {
  showPreview.value = !showPreview.value;
  if (showPreview.value && !previewContent.value) {
    previewLoading.value = true;
    try {
      const md = await skillsStore.previewSkill(props.skill.id);
      previewContent.value = marked.parse(md) as string;
    } catch {
      previewContent.value = "";
    } finally {
      previewLoading.value = false;
    }
  }
}

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

// 从路径中提取 agent 名称
function getAgentNameFromPath(path: string): string {
  const lowerPath = path.toLowerCase();
  if (lowerPath.includes(".claude/skills") || lowerPath.includes(".claude\\skills")) return "Claude";
  if (lowerPath.includes(".hermes/skills") || lowerPath.includes(".hermes\\skills")) return "Hermes";
  if (lowerPath.includes(".pi/agent/skills") || lowerPath.includes(".pi\\agent\\skills")) return "Pi";
  if (lowerPath.includes(".config/opencode/skills") || lowerPath.includes(".config\\opencode\\skills")) return "OpenCode";
  if (lowerPath.includes(".codex/skills") || lowerPath.includes(".codex\\skills")) return "Codex";
  if (lowerPath.includes(".config/mimocode/skills") || lowerPath.includes(".config\\mimocode\\skills")) return "MimoCode";
  if (lowerPath.includes(".agents/skills") || lowerPath.includes(".agents\\skills")) return "Agents";
  // 尝试从路径中提取常见的 agent 目录名
  const match = path.match(/[/\\]\.?([^/\\]+)[/\\]skills/);
  if (match && match[1]) {
    return match[1].charAt(0).toUpperCase() + match[1].slice(1);
  }
  return "";
}

async function useThisVersion(source: SkillSource) {
  try {
    // 找到对应的 agent id
    const agent = props.agents.find(a => a.id === source.from);
    if (agent) {
      await skillsStore.syncToVibe(props.skill.id, agent.id);
      toast.show(t("manage.synced_to_vibe"), "success");
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
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
          : skill.is_duplicate
            ? 'var(--c-info)'
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

      <!-- Status icons -->
      <span v-if="skill.has_conflict" class="shrink-0" style="color: var(--c-warning);">⚠</span>
      <span v-else-if="skill.has_dangling" class="shrink-0" style="color: var(--c-danger);">❌</span>
      <span v-else-if="skill.is_duplicate" class="shrink-0" style="color: var(--c-info);">📋</span>

      <!-- Skill name (显示文件夹名或 name，如果 name 为空则显示 id) -->
      <span class="text-sm font-medium truncate" style="color: var(--c-text);">
        {{ skill.name || skill.id }}
      </span>

      <!-- Missing name badge -->
      <span
        v-if="skill.missing_name"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-danger-light); color: var(--c-danger);"
      >
        {{ t("manage.missing_name") }}
      </span>

      <!-- Sync count -->
      <span class="text-[11px] shrink-0" style="color: var(--c-text-secondary);">
        {{ syncedCount }}/{{ totalCount }}
      </span>

      <!-- Status badges -->
      <span
        v-if="skill.has_conflict"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-warning-light); color: var(--c-warning);"
      >
        {{ t("manage.status_conflict") }}
      </span>
      <span
        v-else-if="skill.has_dangling"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-danger-light); color: var(--c-danger);"
      >
        {{ t("manage.status_dangling") }}
      </span>

      <!-- Preview & Delete buttons -->
      <div class="flex items-center gap-1 ml-auto shrink-0">
        <button
          class="w-6 h-6 flex items-center justify-center rounded cursor-pointer transition-colors"
          :style="{
            background: showPreview ? 'var(--c-primary-light)' : 'transparent',
            color: showPreview ? 'var(--c-primary)' : 'var(--c-text-secondary)',
          }"
          @click.stop="togglePreview"
          :title="t('skills.preview')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
            <circle cx="12" cy="12" r="3"/>
          </svg>
        </button>
        <button
          class="w-6 h-6 flex items-center justify-center rounded cursor-pointer transition-colors hover:bg-[var(--c-danger-light)]"
          style="color: var(--c-danger);"
          @click.stop="showDeleteConfirm = true"
          :title="t('skills.delete')"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- SKILL.md preview -->
    <div v-if="showPreview" class="px-3 pb-3">
      <div
        v-if="previewContent"
        class="markdown-body rounded-md border p-3 max-h-[300px] overflow-y-auto"
        style="background: var(--c-bg); border-color: var(--c-border);"
        v-html="previewContent"
      />
      <div v-else-if="previewLoading" class="text-xs" style="color: var(--c-text-secondary);">
        {{ t("app.loading") }}
      </div>
    </div>

    <!-- Conflict paths list -->
    <div v-if="skill.has_conflict && expanded" class="px-3 pb-3">
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
              <span class="text-[10px] truncate" style="color: var(--c-text-secondary);">
                {{ source.from }}: {{ source.path.split(/[/\\]/).slice(-2, -1)[0] || source.path.split(/[/\\]/).pop() }}
              </span>
              <span class="text-[10px] shrink-0" :style="{ color: source.is_symlink ? 'var(--c-primary)' : 'var(--c-text-secondary)' }">
                {{ source.is_symlink ? t("manage.symlink_to", { target: source.symlink_target?.split(/[/\\]/).pop() || '' }) : t("manage.real_file") }}
              </span>
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
          <!-- Conflict path preview -->
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

    <!-- Expanded agent matrix -->
    <div v-if="expanded" class="px-3 pb-3">
      <!-- Select all + batch bar -->
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

        <!-- Batch action dropdown -->
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

      <!-- Grouped agent rows -->
      <div
        v-for="group in groupedStatuses"
        :key="group.label"
        class="mb-3 last:mb-0"
      >
        <!-- Group header -->
        <div
          class="flex items-center gap-2 mb-1.5 text-[10px] font-medium uppercase tracking-wide"
          :style="{ color: group.color }"
        >
          <span class="w-1.5 h-1.5 rounded-full" :style="{ background: group.color }" />
          {{ group.label }} ({{ group.items.length }})
        </div>

        <!-- Agent rows -->
        <div class="space-y-1 pl-3.5">
          <div
            v-for="item in group.items"
            :key="item.agent.id"
            class="flex items-center justify-between gap-2 px-2 py-1.5 rounded"
            :style="{
              background: selectedAgents.has(item.agent.id) ? 'var(--c-primary-light)' : 'var(--c-bg)',
            }"
          >
            <!-- Left: checkbox + agent name + status -->
            <div class="flex items-center gap-2 min-w-0">
              <input
                type="checkbox"
                :checked="selectedAgents.has(item.agent.id)"
                class="w-3.5 h-3.5 rounded cursor-pointer shrink-0"
                style="accent-color: var(--c-primary);"
                @click.stop="toggleAgentSelect(item.agent.id)"
              />
              <span class="text-[10px]" :style="{ color: item.statusColor }">
                {{ item.statusIcon }}
              </span>
              <span class="text-xs font-medium truncate" style="color: var(--c-text);">
                {{ item.agent.name }}
              </span>
              <span class="text-[10px] shrink-0" :style="{ color: item.statusColor }">
                {{ item.statusLabel }}
              </span>
              <!-- linked_elsewhere: 显示当前链接和期望链接 -->
              <template v-if="item.status === 'linked_elsewhere' && item.source?.symlink_target">
                <span
                  class="text-[10px] truncate max-w-[120px] shrink-0 cursor-help"
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
                  应→ vibe-lib
                </span>
              </template>
              <!-- synced: 显示链接目标 -->
              <span
                v-else-if="item.source?.symlink_target && item.status !== 'unlinked'"
                class="text-[10px] truncate max-w-[150px] shrink-0"
                style="color: var(--c-text-secondary);"
              >
                → {{ item.source.symlink_target.split(/[/\\]/).pop() }}
              </span>
            </div>

            <!-- Right: action button -->
            <button
              v-if="item.action !== 'none'"
              class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors shrink-0 whitespace-nowrap"
              :style="getActionButtonStyle(item)"
              :disabled="resolvingConflict === item.agent.id"
              @click.stop="handleAction(item)"
            >
              {{ resolvingConflict === item.agent.id ? "..." : getActionButtonLabel(item) }}
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
