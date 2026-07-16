import type { Skill, Agent, SkillSource } from "./index";
import { samePath } from "../composables/useSkillAgentStatus";

/**
 * 树节点链接状态（逐 source 派生，口径对齐 useSkillAgentStatus.ts）。
 * 注意：不直接复用 Skill 的 skill 级 has_conflict / has_dangling 布尔，
 * 而是基于该节点对应 source 的 symlink_target + content_hash 与 vibe-lib source 比较得出。
 */
export type NodeLinkState =
  | "origin" // library 自身（from === 'vibe-lib'）
  | "unlinked" // 该 agent 无任何来源（树中不会实际出现）
  | "independent" // 真实文件夹，库无同名
  | "independent_same" // 真实文件夹，库有同名且 hash 相同 → replace_with_link
  | "independent_conflict" // 真实文件夹，库有同名且 hash 不同 → sync_to_vibe
  | "dangling" // 软链但 symlink_target 不存在 → remove_dangling
  | "synced" // 软链指向库 vibe-lib 同名 → unlink
  | "linked_elsewhere"; // 软链指向其它 agent/目录 → relink

export interface TreeRoot {
  kind: "agent" | "library" | "project";
  id: string; // agent.id 或 'library'
  label: string;
  dirPath: string; // skills_dir 或 ~/.vibe-skills
  detected: boolean;
  children: TreeSkillNode[];
  /** 该根下被多少 agent 链接到库（仅 library 根有意义） */
  linkedByCount: number;
  stats: {
    total: number;
    synced: number;
    independent: number;
    conflict: number;
    dangling: number;
  };
}

export interface TreeSkillNode {
  kind: "skill";
  nodeKey: string; // `${rootId}/${skill.id}` —— 跨根唯一
  rootId: string;
  id: string; // Skill.id（= 文件夹名）
  name: string;
  dirName: string;
  path: string; // 该根下的目录路径（来自对应 source.path）
  isSymlink: boolean;
  symlinkTarget?: string;
  linkState: NodeLinkState;
  hasConflict: boolean;
  skill: Skill;
}

const STATE_PRIORITY: Record<NodeLinkState, number> = {
  dangling: 0,
  independent_conflict: 1,
  linked_elsewhere: 2,
  independent: 3,
  independent_same: 4,
  synced: 5,
  origin: 6,
  unlinked: 7,
};

function deriveLinkState(
  source: SkillSource,
  vibeSource: SkillSource | undefined
): NodeLinkState {
  if (!source.is_symlink) {
    if (vibeSource) {
      return source.content_hash === vibeSource.content_hash
        ? "independent_same"
        : "independent_conflict";
    }
    return "independent";
  }
  if (!source.symlink_target || source.content_hash === "") return "dangling";
  if (vibeSource?.path && samePath(source.symlink_target, vibeSource.path)) {
    return "synced";
  }
  return "linked_elsewhere";
}

/**
 * 由 Skill[] 纯前端派生来源树，无需改后端。
 * @param skills 已按筛选/排序处理后的展示列表
 * @param agents  检测到的 agents
 */
export function buildSkillTree(skills: Skill[], agents: Agent[]): TreeRoot[] {
  // 推导 library 根目录：取任一 vibe-lib source 的父目录
  let vibeDir = "";
  for (const s of skills) {
    const vs = s.sources.find((x) => x.from === "vibe-lib");
    if (vs) {
      vibeDir = vs.path.split(/[\\/]/).slice(0, -1).join("/");
      break;
    }
  }

  const roots: TreeRoot[] = [];

  // —— agent 根 ——
  for (const agent of agents) {
    const children: TreeSkillNode[] = [];
    for (const skill of skills) {
      const source = skill.sources.find((x) => x.from === agent.id);
      if (!source) continue;
      const vibeSource = skill.sources.find((x) => x.from === "vibe-lib");
      children.push({
        kind: "skill",
        nodeKey: `${agent.id}/${skill.id}`,
        rootId: agent.id,
        id: skill.id,
        name: skill.name || skill.id,
        dirName: skill.id,
        path: source.path,
        isSymlink: source.is_symlink,
        symlinkTarget: source.symlink_target,
        linkState: deriveLinkState(source, vibeSource),
        hasConflict: skill.has_conflict,
        skill,
      });
    }
    if (children.length === 0) continue;
    roots.push(buildRoot("agent", agent.id, agent.name, agent.skills_dir, agent.detected, children, 0));
  }

  const projectSources = new Map<string, { label: string; children: TreeSkillNode[] }>();
  for (const skill of skills) {
    for (const source of skill.sources.filter((x) => x.from.startsWith("project:") || x.source_kind === "project")) {
      const rootId = source.from;
      const label = source.from.replace(/^project:/, "") || "Project";
      const vibeSource = skill.sources.find((x) => x.from === "vibe-lib");
      if (!projectSources.has(rootId)) {
        projectSources.set(rootId, { label, children: [] });
      }
      projectSources.get(rootId)!.children.push({
        kind: "skill",
        nodeKey: `${rootId}/${skill.id}`,
        rootId,
        id: skill.id,
        name: skill.name || skill.id,
        dirName: skill.id,
        path: source.path,
        isSymlink: source.is_symlink,
        symlinkTarget: source.symlink_target,
        linkState: deriveLinkState(source, vibeSource),
        hasConflict: skill.has_conflict,
        skill,
      });
    }
  }
  for (const [rootId, project] of projectSources) {
    const dirPath = project.children[0]?.path.split(/[\\/]/).slice(0, -1).join("/") ?? "";
    roots.push(buildRoot("project", rootId, `Project · ${project.label}`, dirPath, true, project.children, 0));
  }

  // —— library 根 ——
  const libChildren: TreeSkillNode[] = [];
  for (const skill of skills) {
    const vs = skill.sources.find((x) => x.from === "vibe-lib");
    if (!vs) continue;
    // 统计被多少 agent 链接
    const linkedBy = skill.sources.filter(
      (x) => x.from !== "vibe-lib" && x.is_symlink && !!x.symlink_target
    ).length;
    libChildren.push({
      kind: "skill",
      nodeKey: `library/${skill.id}`,
      rootId: "library",
      id: skill.id,
      name: skill.name || skill.id,
      dirName: skill.id,
      path: vs.path,
      isSymlink: false,
      symlinkTarget: undefined,
      linkState: "origin",
      hasConflict: skill.has_conflict,
      skill,
    });
    // 把 linkedBy 计数累加到该库节点（用 stats 之外单独存，这里放进 root.linkedByCount）
    (libChildren[libChildren.length - 1] as TreeSkillNode & { _linkedBy?: number })._linkedBy =
      linkedBy;
  }
  if (libChildren.length > 0) {
    const libRoot = buildRoot("library", "library", "Library（技能库）", vibeDir, true, libChildren, 0);
    (libRoot as TreeRoot & { _linkedByMap?: Record<string, number> })._linkedByMap = {};
    for (const c of libChildren) {
      const lb = (c as TreeSkillNode & { _linkedBy?: number })._linkedBy ?? 0;
      (libRoot as TreeRoot & { _linkedByMap?: Record<string, number> })._linkedByMap![c.id] = lb;
    }
    libRoot.linkedByCount = libChildren.length;
    roots.push(libRoot);
  }

  // 排序：agent 根在前，library 根置后；根内子节点按「异常优先 + 名称」
  const rootOrder = { library: 0, agent: 1, project: 2 };
  roots.sort((a, b) => {
    if (a.kind !== b.kind) return rootOrder[a.kind] - rootOrder[b.kind];
    return a.label.localeCompare(b.label);
  });
  return roots;
}

function buildRoot(
  kind: "agent" | "library" | "project",
  id: string,
  label: string,
  dirPath: string,
  detected: boolean,
  children: TreeSkillNode[],
  _linkedBy: number
): TreeRoot {
  children.sort((a, b) => {
    const pa = STATE_PRIORITY[a.linkState];
    const pb = STATE_PRIORITY[b.linkState];
    if (pa !== pb) return pa - pb;
    return a.name.localeCompare(b.name);
  });
  const stats = {
    total: children.length,
    synced: children.filter((c) => c.linkState === "synced" || c.linkState === "linked_elsewhere")
      .length,
    independent: children.filter((c) =>
      c.linkState.startsWith("independent")
    ).length,
    conflict: children.filter((c) => c.hasConflict).length,
    dangling: children.filter((c) => c.linkState === "dangling").length,
  };
  return { kind, id, label, dirPath, detected, children, linkedByCount: _linkedBy, stats };
}
