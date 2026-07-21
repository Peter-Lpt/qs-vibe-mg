import { computed, type Ref } from "vue";
import type { Skill, Agent, SkillSource } from "../types";
import { ACTION_PRIORITY } from "./skillActionRegistry";
import { getPluginAgentId } from "../components/manage/manageFilters";
export type { AgentAction, TFunc } from "./skillActionRegistry";
export { actionLabel, actionStyle, cellBtnLabel } from "./skillActionRegistry";
import type { AgentAction, TFunc } from "./skillActionRegistry";

export type AgentStatusType =
  | "origin"
  | "synced"
  | "linked_elsewhere"
  | "independent"
  | "dangling"
  | "unlinked";

export interface AgentStatus {
  agent: Agent;
  source: SkillSource | null;
  status: AgentStatusType;
  action: AgentAction;
  statusLabel: string;
  statusColor: string;
  statusIcon: string;
}

const STATUS_META: Record<
  AgentStatusType,
  { label: string; color: string; icon: string }
> = {
  origin: {
    label: "manage.status_origin",
    color: "var(--c-success)",
    icon: "📦",
  },
  synced: {
    label: "manage.status_synced",
    color: "var(--c-primary)",
    icon: "🔗",
  },
  linked_elsewhere: {
    label: "manage.status_linked_elsewhere",
    color: "var(--c-warning)",
    icon: "🔗",
  },
  independent: {
    label: "manage.status_independent",
    color: "var(--c-text-secondary)",
    icon: "●",
  },
  dangling: {
    label: "manage.status_dangling",
    color: "var(--c-danger)",
    icon: "❌",
  },
  unlinked: {
    label: "manage.status_unlinked",
    color: "var(--c-text-secondary)",
    icon: "○",
  },
};

// 检查 source 是否属于指定的 agent（包括 plugin 来源）
function sourceBelongsToAgent(source: SkillSource, agentId: string): boolean {
  if (source.from === agentId) return true;
  const pluginAgentId = getPluginAgentId(source);
  return pluginAgentId === agentId;
}

export function useSkillAgentStatus(
  skill: Ref<Skill>,
  agents: Ref<Agent[]>,
  t: TFunc
) {
  const vibeSource = computed(() =>
    skill.value.sources.find((s) => s.from === "vibe-lib")
  );

  const allAgentStatuses = computed<AgentStatus[]>(() => {
    const detected = agents.value.filter((a) => a.detected);
    const result: AgentStatus[] = [];
    for (const agent of detected) {
      // 查找属于该 agent 的 source（包括 plugin 来源）
      const source = skill.value.sources.find((s) => sourceBelongsToAgent(s, agent.id));
      if (!source) {
        result.push({
          agent,
          source: null,
          status: "unlinked",
          action: vibeSource.value ? "link" : "none",
          ...meta("unlinked", t),
        });
        continue;
      }
      if (source.from === "vibe-lib") {
        result.push({
          agent,
          source,
          status: "origin",
          action: "none",
          ...meta("origin", t),
        });
        continue;
      }
      // Plugin 来源的 skill 显示为已同步，可从 plugin 同步到中心库
      if (source.source_kind === "marketplace" || source.from.startsWith("claude-plugin:") || source.from.startsWith("codex-plugin:")) {
        result.push({
          agent,
          source,
          status: "synced",
          action: vibeSource.value ? "none" : "sync_from_plugin",
          statusLabel: t("manage.status_plugin"),
          statusColor: "var(--c-plugin, #8b5cf6)",
          statusIcon: "🧩",
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
      if (!source.symlink_target || source.content_hash === "") {
        result.push({
          agent,
          source,
          status: "dangling",
          action: "remove_dangling",
          ...meta("dangling", t),
        });
        continue;
      }
      if (
        vibeSource.value?.path &&
        samePath(source.symlink_target, vibeSource.value.path)
      ) {
        result.push({
          agent,
          source,
          status: "synced",
          action: "unlink",
          ...meta("synced", t),
        });
      } else {
        result.push({
          agent,
          source,
          status: "linked_elsewhere",
          action: "relink",
          ...meta("linked_elsewhere", t),
        });
      }
    }
    return result;
  });

  interface StatusGroup {
    label: string;
    items: AgentStatus[];
    color: string;
  }

  const groupedStatuses = computed<StatusGroup[]>(() => {
    const groups: StatusGroup[] = [];

    const needsAction = allAgentStatuses.value.filter(
      (s) =>
        s.status === "dangling" ||
        s.status === "linked_elsewhere"
    );
    if (needsAction.length > 0) {
      groups.push({
        label: t("manage.group_needs_action"),
        items: needsAction,
        color: "var(--c-warning)",
      });
    }

    const optionalSync = allAgentStatuses.value.filter((s) => s.status === "independent");
    if (optionalSync.length > 0) {
      groups.push({
        label: t("manage.group_optional_sync"),
        items: optionalSync,
        color: "var(--c-text-secondary)",
      });
    }


    const normal = allAgentStatuses.value.filter(
      (s) => s.status === "origin" || s.status === "synced"
    );
    if (normal.length > 0) {
      groups.push({
        label: t("manage.group_normal"),
        items: normal,
        color: "var(--c-success)",
      });
    }

    const unlinked = allAgentStatuses.value.filter(
      (s) => s.status === "unlinked"
    );
    if (unlinked.length > 0) {
      groups.push({
        label: t("manage.group_unlinked"),
        items: unlinked,
        color: "var(--c-text-secondary)",
      });
    }

    return groups;
  });

  const syncedCount = computed(
    () =>
      allAgentStatuses.value.filter(
        (s) => s.status === "synced" || s.status === "origin"
      ).length
  );

  const totalCount = computed(() => allAgentStatuses.value.length);

  const summary = computed(() => {
    const statuses = allAgentStatuses.value;
    const needsAction = statuses.filter(
      (s) =>
        s.status === "dangling" ||
        s.status === "linked_elsewhere"
    );
    const unlinked = statuses.filter((s) => s.status === "unlinked");
    const dangling = statuses.filter((s) => s.status === "dangling");
    const priority = ACTION_PRIORITY.find((a) =>
      statuses.some((s) => s.action === a)
    );
    return {
      synced: syncedCount.value,
      total: totalCount.value,
      needsAction: needsAction.length,
      unlinked: unlinked.length,
      dangling: dangling.length,
      primaryAction: priority ?? "none",
    };
  });

  return {
    vibeSource,
    allAgentStatuses,
    groupedStatuses,
    syncedCount,
    totalCount,
    summary,
  };
}

function meta(type: AgentStatusType, t: TFunc) {
  const m = STATUS_META[type];
  return {
    statusLabel: t(m.label),
    statusColor: m.color,
    statusIcon: m.icon,
  };
}

export function samePath(a: string, b: string): boolean {
  return normalizePath(a) === normalizePath(b);
}

function normalizePath(path: string): string {
  return path.replace(/\\/g, "/").replace(/\/+$/, "");
}
