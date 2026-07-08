import { computed, type Ref } from "vue";
import type { Skill, Agent, SkillSource } from "../types";

export type AgentStatusType =
  | "origin"
  | "synced"
  | "linked_elsewhere"
  | "independent"
  | "dangling"
  | "unlinked";

export type AgentAction =
  | "none"
  | "sync_to_vibe"
  | "replace_with_link"
  | "relink"
  | "remove_dangling"
  | "link"
  | "unlink";

export interface AgentStatus {
  agent: Agent;
  source: SkillSource | null;
  status: AgentStatusType;
  action: AgentAction;
  statusLabel: string;
  statusColor: string;
  statusIcon: string;
}

export type TFunc = (key: string, params?: Record<string, unknown>) => string;

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

// Priority for the single "primary action" shown on a card: which action to
// surface when collapsing a skill into one button.
const ACTION_PRIORITY: AgentAction[] = [
  "sync_to_vibe",
  "relink",
  "link",
  "replace_with_link",
  "remove_dangling",
  "unlink",
  "none",
];

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
      const source = skill.value.sources.find((s) => s.from === agent.id);
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
          ...meta("dangling", t),
        });
        continue;
      }
      if (
        vibeSource.value?.path &&
        source.symlink_target.includes(vibeSource.value.path)
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
        s.status === "independent" ||
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
        s.status === "independent" ||
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

export function actionLabel(t: TFunc, action: AgentAction): string {
  switch (action) {
    case "link":
      return t("manage.btn_link");
    case "unlink":
      return t("manage.btn_unlink");
    case "sync_to_vibe":
      return t("manage.btn_sync");
    case "replace_with_link":
      return t("manage.btn_replace");
    case "relink":
      return t("manage.btn_relink");
    case "remove_dangling":
      return t("manage.btn_clean");
    default:
      return "";
  }
}

export function actionStyle(action: AgentAction): string {
  switch (action) {
    case "link":
      return "background: var(--c-primary); color: white;";
    case "sync_to_vibe":
      return "background: var(--c-primary); color: white;";
    case "replace_with_link":
      return "background: var(--c-surface-hover); color: var(--c-text); border: 1px solid var(--c-border);";
    case "relink":
      return "background: var(--c-warning); color: white;";
    case "remove_dangling":
      return "background: var(--c-danger); color: white;";
    case "unlink":
      return "background: var(--c-surface-hover); color: var(--c-text-secondary); border: 1px solid var(--c-border);";
    default:
      return "";
  }
}
