<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import { useFileLogger } from "../../composables/useFileLogger";
import { useSkillActions } from "../../composables/useSkillActions";
import { marked } from "marked";
import {
  useSkillAgentStatus,
  actionLabel,
  actionStyle,
  type AgentStatus,
} from "../../composables/useSkillAgentStatus";
import type { Skill, Agent, SkillSource } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const toast = useToast();
const logger = useFileLogger();
const actions = useSkillActions((k, p) => t(k, p as Record<string, unknown>));

const agentsRef = computed(() => props.agents);
const skillRef = computed(() => props.skill);
const { allAgentStatuses, groupedStatuses, vibeSource } =
  useSkillAgentStatus(skillRef, agentsRef, (k, p) => t(k, p as Record<string, unknown>));

// per-skill 批量选择
const selectedAgents = ref<Set<string>>(new Set());
const showBatchMenu = ref(false);
const batchOperating = ref(false);
const resolvingConflict = ref<string | null>(null);
const pendingOverwrite = ref<AgentStatus | null>(null);
const pendingPlanOverwrite = ref(false);
const selectedConflictPath = ref<string>("");
const resolvingPlan = ref(false);
const cleaningDanglingPath = ref<string | null>(null);

interface ConflictItem {
  source: SkillSource;
  content: string;
  loading: boolean;
}
const conflictItems = ref<ConflictItem[]>([]);

async function loadConflictSources() {
  if (!props.skill.has_conflict) {
    conflictItems.value = [];
    return;
  }
  conflictItems.value = props.skill.sources.map((s) => ({
    source: s,
    content: "",
    loading: true,
  }));
  selectedConflictPath.value =
    props.skill.sources.find((s) => s.from === "vibe-lib")?.path ||
    props.skill.sources[0]?.path ||
    "";
  await Promise.all(
    conflictItems.value.map(async (it) => {
      try {
        it.content = (marked.parse(
          await skillsStore.previewSkillAtPath(it.source.path)
        ) as string);
      } catch {
        it.content = "";
      } finally {
        it.loading = false;
      }
    })
  );
}

onMounted(loadConflictSources);
watch(
  () => props.skill.has_conflict,
  () => loadConflictSources()
);

const selectedConflictSource = computed(() =>
  props.skill.sources.find((s) => s.path === selectedConflictPath.value)
);

const selectedConflictAgent = computed(() => {
  const source = selectedConflictSource.value;
  if (!source || source.from === "vibe-lib") return null;
  return props.agents.find((a) => a.id === source.from) ?? null;
});

const sameContentSources = computed(() => {
  const selected = selectedConflictSource.value;
  if (!selected) return [];
  return props.skill.sources.filter(
    (s) =>
      s.from !== "vibe-lib" &&
      s.path !== selected.path &&
      s.content_hash !== "" &&
      s.content_hash === selected.content_hash
  );
});

const differentContentSources = computed(() => {
  const selected = selectedConflictSource.value;
  if (!selected) return [];
  return props.skill.sources.filter(
    (s) =>
      s.path !== selected.path &&
      s.content_hash !== "" &&
      s.content_hash !== selected.content_hash
  );
});

const differentAgentSources = computed(() =>
  differentContentSources.value.filter((s) => s.from !== "vibe-lib")
);

const agentConflictSources = computed(() =>
  props.skill.sources.filter((s) => s.from !== "vibe-lib" && !s.from.startsWith("project:"))
);

const isSingleAgentPathConflict = computed(() => {
  if (vibeSource.value || agentConflictSources.value.length < 2) return false;
  return new Set(agentConflictSources.value.map((s) => s.from)).size === 1;
});

const internalConflictRemovalSources = computed(() => {
  const selected = selectedConflictSource.value;
  if (!selected || !isSingleAgentPathConflict.value || selected.from === "vibe-lib") return [];
  return agentConflictSources.value.filter((s) => s.path !== selected.path);
});

const planWillOverwriteLibrary = computed(() => {
  const selected = selectedConflictSource.value;
  const library = vibeSource.value;
  return !!selected && selected.from !== "vibe-lib" && !!library && selected.content_hash !== library.content_hash;
});

const planWillReplaceDifferentCopies = computed(() => differentAgentSources.value.length > 0);

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
  if (hasSync && !props.skill.has_conflict) actions.push({ action: "sync_to_vibe", label: t("manage.btn_sync_from", { agent: "" }), color: "var(--c-primary)" });
  if (hasReplace) actions.push({ action: "replace_with_link", label: t("manage.btn_replace"), color: "var(--c-text)" });
  if (hasRelink) actions.push({ action: "relink", label: t("manage.btn_relink"), color: "var(--c-warning)" });
  if (hasUnlink) actions.push({ action: "unlink", label: t("manage.btn_unlink"), color: "var(--c-text-secondary)" });
  if (hasClean) actions.push({ action: "remove_dangling", label: t("manage.btn_clean"), color: "var(--c-danger)" });

  return actions;
});

const comparableSources = computed(() =>
  props.skill.sources.filter((source) => source.content_hash)
);

const latestSourcePath = computed(() => {
  let latestPath = "";
  let latestTime = 0;
  for (const source of props.skill.sources) {
    const time = source.modified_at ? Date.parse(source.modified_at) : 0;
    if (!Number.isNaN(time) && time > latestTime) {
      latestTime = time;
      latestPath = source.path;
    }
  }
  return latestPath;
});

function sameContentCount(source: SkillSource): number {
  if (!source.content_hash) return 0;
  return comparableSources.value.filter((s) => s.content_hash === source.content_hash).length;
}

function contentRelation(source: SkillSource): "unknown" | "same" | "different" {
  if (!source.content_hash || comparableSources.value.length < 2) return "unknown";
  return sameContentCount(source) === comparableSources.value.length ? "same" : "different";
}

function formatSourceTime(source: SkillSource): string {
  if (!source.modified_at) return "";
  try {
    return new Date(source.modified_at).toLocaleString();
  } catch {
    return source.modified_at;
  }
}

const sourceRows = computed(() =>
  props.skill.sources.map((source) => {
    const metadataUrl = props.skill.metadata?.repository || props.skill.metadata?.source || props.skill.metadata?.homepage;
    const inferredProvider = /github/i.test(`${metadataUrl || ""} ${source.path}`)
      ? "GitHub"
      : /gitee/i.test(`${metadataUrl || ""} ${source.path}`)
        ? "Gitee"
        : /gitlab/i.test(`${metadataUrl || ""} ${source.path}`)
          ? "GitLab"
          : "";
    const confidence = inferredProvider
        ? t("manage.source_confidence_inferred", { provider: inferredProvider })
        : t("manage.source_confidence_unknown");
    return {
      source,
      label: sourceLabel(source),
      kind: source.source_kind || (source.from === "vibe-lib" ? "library" : source.from.startsWith("project:") ? "project" : "agent"),
      confidence,
      dangling: source.is_symlink && (!source.symlink_target || source.content_hash === ""),
      isLatest: source.path === latestSourcePath.value,
      sameCount: sameContentCount(source),
      relation: contentRelation(source),
      modifiedAtLabel: formatSourceTime(source),
    };
  })
);

// 逐 agent 按钮文案：sync 显示「从 {agent} 同步」，其余复用 actionLabel
function cellBtnLabel(action: string, agentName: string): string {
  if (action === "sync_to_vibe") return t("manage.btn_sync_from", { agent: agentName });
  return actionLabel(t, action as never);
}

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

async function handleAction(status: ReturnType<typeof useSkillAgentStatus>["allAgentStatuses"]["value"][number]) {
  if (status.action === "none") return;

  if (
    status.action === "sync_to_vibe" &&
    !!vibeSource.value &&
    !!status.source &&
    status.source.content_hash !== vibeSource.value.content_hash
  ) {
    pendingOverwrite.value = status;
    return;
  }

  await runAction(status);
}

async function runAction(status: AgentStatus) {

  try {
    switch (status.action) {
      case "link":
        await skillsStore.createLink(props.skill.id, status.agent.id);
        toast.show(t("skills.linked", { agent: status.agent.name }), "success");
        break;
      case "unlink":
        await skillsStore.removeLink(props.skill.id, status.agent.id, status.source?.path);
        toast.show(t("skills.unlinked", { agent: status.agent.name }), "success");
        break;
      case "sync_to_vibe":
        resolvingConflict.value = status.agent.id;
        await skillsStore.syncToVibe(props.skill.id, status.agent.id, true, status.source?.path);
        toast.show(t("manage.synced_to_vibe", { agent: status.agent.name }), "success");
        break;
      case "replace_with_link":
        await skillsStore.syncToVibe(props.skill.id, status.agent.id, false, status.source?.path);
        toast.show(t("manage.replaced_with_link", { agent: status.agent.name }), "success");
        break;
      case "relink":
        await skillsStore.relink(props.skill.id, status.agent.id, status.source?.path);
        toast.show(t("manage.relinked", { agent: status.agent.name }), "success");
        break;
      case "remove_dangling":
        await skillsStore.removeLink(props.skill.id, status.agent.id, status.source?.path);
        toast.show(t("manage.dangling_removed", { agent: status.agent.name }), "success");
        break;
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    resolvingConflict.value = null;
  }
}

async function confirmOverwrite() {
  const status = pendingOverwrite.value;
  pendingOverwrite.value = null;
  pendingPlanOverwrite.value = false;
  if (status) await runAction(status);
}

function sourceLabel(source: SkillSource): string {
  if (source.from === "vibe-lib") return t("manage.library");
  return props.agents.find((a) => a.id === source.from)?.name ?? source.from;
}

function shortHash(source: SkillSource): string {
  return source.content_hash ? source.content_hash.slice(0, 8) : "—";
}

function sourceKindLabel(kind: string): string {
  if (kind === "library") return t("manage.source_kind_library");
  if (kind === "project") return t("manage.source_kind_project");
  if (kind === "external") return t("manage.source_kind_external");
  return t("manage.source_kind_agent");
}

function isProjectSource(source: SkillSource): boolean {
  return source.source_kind === "project" || source.from.startsWith("project:");
}

async function cleanDanglingSource(source: SkillSource) {
  if (source.from === "vibe-lib" || isProjectSource(source) || cleaningDanglingPath.value) return;
  cleaningDanglingPath.value = source.path;
  try {
    await skillsStore.removeLink(props.skill.id, source.from, source.path);
    toast.show(t("manage.dangling_removed", { agent: sourceLabel(source) }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    cleaningDanglingPath.value = null;
  }
}

async function executeConflictResolution() {
  const selected = selectedConflictSource.value;
  if (!selected || resolvingPlan.value) return;

  if ((planWillOverwriteLibrary.value || planWillReplaceDifferentCopies.value) && selected.from !== "vibe-lib") {
    const agent = selectedConflictAgent.value;
    if (agent) {
      pendingOverwrite.value = {
        agent,
        source: selected,
        status: "independent",
        action: "sync_to_vibe",
        statusLabel: t("manage.status_independent_conflict"),
        statusColor: "var(--c-warning)",
        statusIcon: "⚠",
      };
      pendingPlanOverwrite.value = true;
      return;
    }
  }

  if (planWillReplaceDifferentCopies.value && selected.from === "vibe-lib") {
    pendingOverwrite.value = {
      agent: {
        id: "vibe-lib",
        name: t("manage.library"),
        skills_dir: selected.path,
        kind: "external",
        detect_dir: undefined,
        tool_detected: true,
        detected: true,
        enabled: true,
        auto_detected: false,
        linked_skills: [],
      },
      source: selected,
      status: "origin",
      action: "sync_to_vibe",
      statusLabel: t("manage.status_origin"),
      statusColor: "var(--c-warning)",
      statusIcon: "⚠",
    };
    pendingPlanOverwrite.value = true;
    return;
  }

  await runConflictResolution(selected);
}

async function runConflictResolution(selected: SkillSource) {
  resolvingPlan.value = true;
  try {
    if (selected.from !== "vibe-lib") {
      const agent = props.agents.find((a) => a.id === selected.from);
      if (!agent) throw new Error(`Agent not found: ${selected.from}`);
      await skillsStore.syncToVibe(props.skill.id, agent.id, true, selected.path);
    }

    const sourcesToAlign = props.skill.sources.filter(
      (source) =>
        source.from !== "vibe-lib" &&
        source.path !== selected.path &&
        (!source.is_symlink || source.content_hash !== selected.content_hash)
    );

    for (const source of sourcesToAlign) {
      const agent = props.agents.find((a) => a.id === source.from);
      if (!agent) continue;
      await skillsStore.replaceWithLibrary(props.skill.id, agent.id, source.path);
    }

    toast.show(t("manage.conflict_resolve_success", { skill: props.skill.name || props.skill.id }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    resolvingPlan.value = false;
  }
}

async function keepSelectedAgentCopy() {
  const selected = selectedConflictSource.value;
  if (!selected || selected.from === "vibe-lib" || resolvingPlan.value) return;

  resolvingPlan.value = true;
  try {
    for (const source of internalConflictRemovalSources.value) {
      await skillsStore.removeAgentSkillCopy(props.skill.id, source.from, source.path);
    }
    toast.show(t("manage.internal_conflict_resolve_success", { skill: props.skill.name || props.skill.id }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    resolvingPlan.value = false;
  }
}

async function confirmConflictResolutionOverwrite() {
  const selected = selectedConflictSource.value;
  pendingOverwrite.value = null;
  pendingPlanOverwrite.value = false;
  if (selected) await runConflictResolution(selected);
}

async function confirmPendingOverwrite() {
  if (pendingPlanOverwrite.value) {
    await confirmConflictResolutionOverwrite();
    return;
  }
  await confirmOverwrite();
}

function cancelPendingOverwrite() {
  pendingOverwrite.value = null;
  pendingPlanOverwrite.value = false;
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

</script>

<template>
  <div class="px-3 pb-3">
    <div class="mb-3 rounded-md border p-2" style="background: var(--c-bg); border-color: var(--c-border);">
      <div class="text-[10px] font-medium uppercase tracking-wide mb-1.5" style="color: var(--c-text-secondary);">
        {{ t("manage.sources_title") }}
      </div>
      <div class="space-y-1">
        <div
          v-for="row in sourceRows"
          :key="row.source.path"
          class="flex items-center gap-2 rounded px-2 py-1"
          style="background: var(--c-surface);"
        >
          <component :is="row.source.is_symlink ? 'Link2' : row.kind === 'project' ? 'FileBox' : 'Folder'" :size="13" style="color: var(--c-text-secondary);" />
          <span class="text-[10px] font-medium shrink-0" style="color: var(--c-text);">
            {{ row.label }}
          </span>
          <span class="text-[9px] px-1.5 py-0.5 rounded shrink-0" style="background: var(--c-surface-hover); color: var(--c-text-secondary);">
            {{ sourceKindLabel(row.kind) }}
          </span>
          <span
            v-if="row.isLatest"
            class="text-[9px] px-1.5 py-0.5 rounded shrink-0"
            style="background: var(--c-primary-light); color: var(--c-primary);"
            :title="row.modifiedAtLabel"
          >
            {{ t("manage.source_latest") }}
          </span>
          <span
            v-if="row.relation !== 'unknown'"
            class="text-[9px] px-1.5 py-0.5 rounded shrink-0"
            :style="row.relation === 'same'
              ? 'background: var(--c-success-light); color: var(--c-success);'
              : 'background: var(--c-warning-light); color: var(--c-warning);'"
            :title="t('manage.source_same_count', { count: row.sameCount })"
          >
            {{ t(row.relation === "same" ? "manage.source_content_same" : "manage.source_content_different") }}
          </span>
          <span class="text-[10px] truncate min-w-0 flex-1" style="color: var(--c-text-secondary);" :title="row.source.path">
            {{ row.source.path }}
          </span>
          <span class="text-[9px] shrink-0" style="color: var(--c-text-secondary);">
            {{ shortHash(row.source) }}
          </span>
          <span class="text-[9px] shrink-0" style="color: var(--c-text-secondary);" :title="row.confidence">
            {{ row.confidence }}
          </span>
          <button
            v-if="row.dangling && !isProjectSource(row.source)"
            class="w-5 h-5 inline-flex items-center justify-center rounded cursor-pointer"
            style="color: var(--c-danger);"
            :disabled="cleaningDanglingPath === row.source.path"
            :title="t('manage.btn_clean')"
            @click.stop="cleanDanglingSource(row.source)"
          >
            <Link2Off v-if="cleaningDanglingPath !== row.source.path" :size="12" />
            <RefreshCw v-else :size="12" class="animate-spin" />
          </button>
          <button class="w-5 h-5 inline-flex items-center justify-center rounded cursor-pointer" style="color: var(--c-text-secondary);" :title="t('manage.reveal')" @click.stop="actions.reveal(row.source)">
            <FolderOpen :size="12" />
          </button>
          <button class="w-5 h-5 inline-flex items-center justify-center rounded cursor-pointer" style="color: var(--c-text-secondary);" :title="t('manage.copy_path')" @click.stop="actions.copyPath(row.source)">
            <Copy :size="12" />
          </button>
        </div>
      </div>
    </div>

    <!-- 冲突解决：先选择主版本，再预览影响，最后执行 -->
    <div v-if="skill.has_conflict" class="mb-3">
      <div class="flex items-center justify-between gap-2 mb-2">
        <div class="text-[10px] font-medium uppercase tracking-wide" style="color: var(--c-warning);">
          {{ t("manage.conflict_resolution") }}
        </div>
        <button
          v-if="!isSingleAgentPathConflict"
          class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
          :disabled="!selectedConflictSource || resolvingPlan"
          style="background: var(--c-primary); color: white;"
          @click.stop="executeConflictResolution"
        >
          {{ resolvingPlan ? "..." : t("manage.conflict_apply_plan") }}
        </button>
      </div>
      <div class="grid gap-2" style="grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));">
        <div
          v-for="it in conflictItems"
          :key="it.source.path"
          class="rounded-md border p-2 cursor-pointer transition-colors"
          :style="{
            background: selectedConflictPath === it.source.path ? 'var(--c-primary-light)' : 'var(--c-bg)',
            borderColor: selectedConflictPath === it.source.path ? 'var(--c-primary)' : 'var(--c-border)',
          }"
          @click="selectedConflictPath = it.source.path"
        >
          <div class="flex items-center justify-between gap-2 mb-1.5">
            <div class="flex items-center gap-2 min-w-0">
              <input
                type="radio"
                :checked="selectedConflictPath === it.source.path"
                class="w-3 h-3 shrink-0"
                style="accent-color: var(--c-primary);"
                @click.stop="selectedConflictPath = it.source.path"
              />
              <component :is="it.source.is_symlink ? 'Link2' : 'Folder'" class="shrink-0" :size="12" :style="{ color: it.source.is_symlink ? 'var(--c-primary)' : 'var(--c-text-secondary)' }" />
              <div class="min-w-0">
                <div class="text-[11px] font-medium truncate" style="color: var(--c-text);">
                  {{ sourceLabel(it.source) }}
                  <span v-if="it.source.from === 'vibe-lib'" class="ml-1 text-[9px]" style="color: var(--c-primary);">
                    {{ t("manage.current_library_version") }}
                  </span>
                  <span v-if="it.source.path === latestSourcePath" class="ml-1 text-[9px]" style="color: var(--c-primary);">
                    {{ t("manage.source_latest") }}
                  </span>
                </div>
                <div class="text-[10px] truncate" style="color: var(--c-text-secondary);" :title="it.source.path">
                  {{ it.source.path }}
                </div>
              </div>
            </div>
            <span class="text-[10px] shrink-0" style="color: var(--c-text-secondary);">
              {{ shortHash(it.source) }}
            </span>
          </div>
          <div v-if="it.loading" class="text-[10px]" style="color: var(--c-text-secondary);">
            {{ t("app.loading") }}
          </div>
          <div
            v-else-if="it.content"
            class="markdown-body rounded border p-2 max-h-[200px] overflow-y-auto text-[11px]"
            style="background: var(--c-surface); border-color: var(--c-border);"
            v-html="it.content"
          />
          <div v-else class="text-[10px]" style="color: var(--c-text-secondary);">—</div>
        </div>
      </div>

      <div
        v-if="selectedConflictSource"
        class="mt-2 rounded-md border p-2 text-[11px]"
        style="background: var(--c-surface); border-color: var(--c-border);"
      >
        <div class="font-medium mb-1.5" style="color: var(--c-text);">
          {{ t(isSingleAgentPathConflict ? "manage.internal_conflict_plan_title" : "manage.conflict_plan_title") }}
        </div>
        <div class="space-y-1" style="color: var(--c-text-secondary);">
          <template v-if="isSingleAgentPathConflict">
            <div>
              {{ t("manage.internal_conflict_keep_selected", { path: selectedConflictSource.path }) }}
            </div>
            <div>
              {{ t("manage.internal_conflict_remove_others", { count: internalConflictRemovalSources.length }) }}
            </div>
            <div style="color: var(--c-warning);">
              {{ t("manage.internal_conflict_no_library") }}
            </div>
            <div class="flex flex-wrap gap-2 pt-1">
              <button
                class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
                style="background: var(--c-primary); color: white;"
                :disabled="resolvingPlan || internalConflictRemovalSources.length === 0"
                @click.stop="keepSelectedAgentCopy"
              >
                {{ resolvingPlan ? "..." : t("manage.internal_conflict_keep_action") }}
              </button>
              <button
                class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
                style="border: 1px solid var(--c-border); color: var(--c-text-secondary); background: var(--c-bg);"
                :disabled="!selectedConflictSource || resolvingPlan"
                @click.stop="executeConflictResolution"
              >
                {{ t("manage.internal_conflict_import_action") }}
              </button>
            </div>
          </template>
          <template v-else>
            <div v-if="planWillOverwriteLibrary" style="color: var(--c-warning);">
              {{ t("manage.conflict_plan_overwrite_library", { source: sourceLabel(selectedConflictSource) }) }}
            </div>
            <div v-else>
              {{ t("manage.conflict_plan_keep_library") }}
            </div>
            <div v-if="selectedConflictSource.from !== 'vibe-lib'">
              {{ t("manage.conflict_plan_link_selected", { source: sourceLabel(selectedConflictSource) }) }}
            </div>
            <div>
              {{ t("manage.conflict_plan_align_same", { count: sameContentSources.length }) }}
            </div>
            <div>
              {{ t("manage.conflict_plan_replace_different", { count: differentAgentSources.length }) }}
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- 批量选择栏 -->
    <div
      v-if="allAgentStatuses.length > 0"
      class="flex items-center gap-2 mt-2 mb-3 pb-2 border-b"
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
          {{ t("manage.batch_apply") }} ({{ selectedAgents.size }})
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

    <!-- 逐 agent 分组 -->
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
          <div class="flex items-center gap-2 min-w-0 flex-1">
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
              class="text-[10px] truncate flex-1 min-w-0"
              style="color: var(--c-text-secondary);"
            >
              → {{ item.source.symlink_target.split(/[/\\]/).pop() }}
            </span>
          </div>

          <button
            v-if="item.action !== 'none'"
            class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors shrink-0 whitespace-nowrap"
            :style="actionStyle(item.action)"
            :disabled="resolvingConflict === item.agent.id"
            :title="item.action === 'sync_to_vibe' ? t('manage.sync_scope_tip', { skill: skill.name || skill.id, agent: item.agent.name }) : undefined"
            @click.stop="handleAction(item)"
          >
            {{ resolvingConflict === item.agent.id ? '...' : cellBtnLabel(item.action, item.agent.name) }}
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

    <ConfirmDialog
      v-if="pendingOverwrite"
      :title="t('manage.overwrite_confirm_title')"
      :message="t('manage.overwrite_confirm_message', {
        skill: skill.name || skill.id,
        agent: pendingOverwrite.agent.name,
      })"
      :confirm-text="t('manage.overwrite_confirm_action')"
      :danger="true"
      @confirm="confirmPendingOverwrite"
      @cancel="cancelPendingOverwrite"
    />
  </div>
</template>
