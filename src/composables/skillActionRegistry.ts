export type AgentAction =
  | "none"
  | "sync_to_vibe"
  | "replace_with_link"
  | "relink"
  | "remove_dangling"
  | "link"
  | "unlink";

export type TFunc = (key: string, params?: Record<string, unknown>) => string;

export interface SkillActionDefinition {
  id: AgentAction;
  labelKey: string;
  successKey?: string;
  color: string;
  style: string;
  priority: number;
  mutatesLibrary: boolean;
  removesTarget: boolean;
}

export const SKILL_ACTIONS: Record<AgentAction, SkillActionDefinition> = {
  none: {
    id: "none",
    labelKey: "",
    color: "",
    style: "",
    priority: 99,
    mutatesLibrary: false,
    removesTarget: false,
  },
  sync_to_vibe: {
    id: "sync_to_vibe",
    labelKey: "manage.btn_sync",
    successKey: "manage.synced_to_vibe",
    color: "var(--c-primary)",
    style: "background: var(--c-primary); color: white;",
    priority: 1,
    mutatesLibrary: true,
    removesTarget: true,
  },
  relink: {
    id: "relink",
    labelKey: "manage.btn_relink",
    successKey: "manage.relinked",
    color: "var(--c-warning)",
    style: "background: var(--c-warning); color: white;",
    priority: 2,
    mutatesLibrary: false,
    removesTarget: false,
  },
  link: {
    id: "link",
    labelKey: "manage.btn_link",
    successKey: "skills.linked",
    color: "var(--c-primary)",
    style: "background: var(--c-primary); color: white;",
    priority: 3,
    mutatesLibrary: false,
    removesTarget: false,
  },
  replace_with_link: {
    id: "replace_with_link",
    labelKey: "manage.btn_replace",
    successKey: "manage.replaced_with_link",
    color: "var(--c-text)",
    style: "background: var(--c-surface-hover); color: var(--c-text); border: 1px solid var(--c-border);",
    priority: 4,
    mutatesLibrary: false,
    removesTarget: true,
  },
  remove_dangling: {
    id: "remove_dangling",
    labelKey: "manage.btn_clean",
    successKey: "manage.dangling_removed",
    color: "var(--c-danger)",
    style: "background: var(--c-danger); color: white;",
    priority: 5,
    mutatesLibrary: false,
    removesTarget: true,
  },
  unlink: {
    id: "unlink",
    labelKey: "manage.btn_unlink",
    successKey: "skills.unlinked",
    color: "var(--c-text-secondary)",
    style: "background: var(--c-surface-hover); color: var(--c-text-secondary); border: 1px solid var(--c-border);",
    priority: 6,
    mutatesLibrary: false,
    removesTarget: true,
  },
};

export const ACTION_PRIORITY: AgentAction[] = Object.values(SKILL_ACTIONS)
  .sort((a, b) => a.priority - b.priority)
  .map((action) => action.id);

export function actionLabel(t: TFunc, action: AgentAction): string {
  const labelKey = SKILL_ACTIONS[action]?.labelKey;
  return labelKey ? t(labelKey) : "";
}

export function cellBtnLabel(t: TFunc, action: AgentAction, agentName: string): string {
  if (action === "sync_to_vibe") {
    return t("manage.btn_sync_from", { agent: agentName });
  }
  return actionLabel(t, action);
}

export function actionStyle(action: AgentAction): string {
  return SKILL_ACTIONS[action]?.style ?? "";
}

export function actionColor(action: AgentAction): string {
  return SKILL_ACTIONS[action]?.color ?? "";
}

export function actionSuccessLabel(t: TFunc, action: AgentAction, agentName: string): string {
  const successKey = SKILL_ACTIONS[action]?.successKey;
  return successKey ? t(successKey, { agent: agentName }) : "";
}
