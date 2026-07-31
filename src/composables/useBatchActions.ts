import { computed, ref, type Ref } from "vue";
import type { Agent, Skill } from "../types";
import { getAgentStatuses } from "./useSkillAgentStatus";
import type { AgentAction, TFunc } from "./skillActionRegistry";
import { useSkillsStore } from "../stores/skills";
import { useAgentsStore } from "../stores/agents";

export interface BatchPair {
  skillId: string;
  skillName: string;
  agentId: string;
  agentName: string;
  action: AgentAction;
  statusType: string;
}

export interface BatchIntent {
  id: string;
  labelKey: string;
  icon: string;
  count: number;
  pairs: BatchPair[];
  actions: Record<string, BatchPair[]>;
  needsConfirm: boolean;
}

export interface BatchResultDetail {
  skillId: string;
  skillName: string;
  agentId: string;
  agentName: string;
  status: "success" | "error";
  message?: string;
}

export interface BatchResult {
  intentId: string;
  total: number;
  success: number;
  errors: number;
  details: BatchResultDetail[];
}

export function useBatchActions(
  selectedSkills: Ref<Skill[]>,
  agents: Ref<Agent[]>,
  t: TFunc
) {
  const skillsStore = useSkillsStore();
  const agentsStore = useAgentsStore();
  const operating = ref(false);
  const result = ref<BatchResult | null>(null);

  function getSkillName(skillId: string): string {
    return skillsStore.skills.find((s) => s.id === skillId)?.name || skillId;
  }
  function getAgentName(agentId: string): string {
    return agentsStore.agents.find((a) => a.id === agentId)?.name || agentId;
  }

  // 收集所有可操作的 (skill, agent, action) 对
  const allPairs = computed<BatchPair[]>(() => {
    const pairs: BatchPair[] = [];
    for (const skill of selectedSkills.value) {
      const statuses = getAgentStatuses(skill, agents.value, t);
      for (const st of statuses) {
        if (st.action !== "none") {
          pairs.push({
            skillId: skill.id,
            skillName: skill.name || skill.id,
            agentId: st.agent.id,
            agentName: st.agent.name,
            action: st.action,
            statusType: st.status,
          });
        }
      }
    }
    return pairs;
  });

  // 按意图分组
  const intents = computed<BatchIntent[]>(() => {
    const pairs = allPairs.value;
    const result: BatchIntent[] = [];

    // 修复问题：dangling + 错链
    const repair = pairs.filter(
      (p) => p.action === "remove_dangling" || p.action === "relink"
    );
    if (repair.length > 0) {
      result.push({
        id: "repair",
        labelKey: "batch.intent_repair",
        icon: "Wrench",
        count: repair.length,
        pairs: repair,
        actions: groupByAction(repair),
        needsConfirm: true,
      });
    }

    // 统一副本：independent 副本同步入库（sync_to_vibe）
    const unify = pairs.filter(
      (p) => p.action === "sync_to_vibe" && p.statusType === "independent"
    );
    if (unify.length > 0) {
      result.push({
        id: "unify",
        labelKey: "batch.intent_unify",
        icon: "Package",
        count: unify.length,
        pairs: unify,
        actions: groupByAction(unify),
        needsConfirm: true,
      });
    }

    // 链接到 Agent
    const link = pairs.filter((p) => p.action === "link");
    if (link.length > 0) {
      result.push({
        id: "link",
        labelKey: "batch.intent_link",
        icon: "Link2",
        count: link.length,
        pairs: link,
        actions: groupByAction(link),
        needsConfirm: false,
      });
    }

    // 断开链接
    const unlink = pairs.filter((p) => p.action === "unlink");
    if (unlink.length > 0) {
      result.push({
        id: "unlink",
        labelKey: "batch.intent_unlink",
        icon: "Unlink",
        count: unlink.length,
        pairs: unlink,
        actions: groupByAction(unlink),
        needsConfirm: true,
      });
    }

    // 导入插件
    const plugin = pairs.filter((p) => p.action === "sync_from_plugin");
    if (plugin.length > 0) {
      result.push({
        id: "plugin",
        labelKey: "batch.intent_plugin",
        icon: "Puzzle",
        count: plugin.length,
        pairs: plugin,
        actions: groupByAction(plugin),
        needsConfirm: true,
      });
    }

    return result;
  });

  // 执行某个意图
  async function execute(intent: BatchIntent): Promise<BatchResult> {
    operating.value = true;
    result.value = null;
    try {
      const details: BatchResultDetail[] = [];
    // 按 skillId 分组执行
    const bySkill = new Map<string, { action: AgentAction; agentIds: string[] }[]>();
    for (const pair of intent.pairs) {
      const existing = bySkill.get(pair.skillId);
      const group = existing?.find((g) => g.action === pair.action);
      if (group) {
        group.agentIds.push(pair.agentId);
      } else {
        if (!existing) bySkill.set(pair.skillId, []);
        bySkill.get(pair.skillId)!.push({
          action: pair.action,
          agentIds: [pair.agentId],
        });
      }
    }

    for (const [skillId, groups] of bySkill) {
      for (const group of groups) {
        try {
          const res = await skillsStore.batchSkillAction(
            skillId,
            group.agentIds,
            group.action,
            true
          );
          for (const agentId of group.agentIds) {
            const hasError = res.errors.some((e) => e.startsWith(agentId + ":"));
            details.push({
              skillId,
              skillName: getSkillName(skillId),
              agentId,
              agentName: getAgentName(agentId),
              status: hasError ? "error" : "success",
              message: hasError
                ? res.errors.find((e) => e.startsWith(agentId + ":"))?.slice(agentId.length + 2)
                : undefined,
            });
          }
        } catch (e) {
          for (const agentId of group.agentIds) {
            details.push({
              skillId,
              skillName: getSkillName(skillId),
              agentId,
              agentName: getAgentName(agentId),
              status: "error",
              message: String(e),
            });
          }
        }
      }
    }

    await skillsStore.refreshSkills();
    await agentsStore.fetchAgents();

    const batchResult: BatchResult = {
      intentId: intent.id,
      total: intent.pairs.length,
      success: details.filter((d) => d.status === "success").length,
      errors: details.filter((d) => d.status === "error").length,
      details,
    };
    result.value = batchResult;
    return batchResult;
    } finally {
      // 意外异常时也要复位 operating，避免 UI 永久禁用
      operating.value = false;
    }
  }

  function clearResult() {
    result.value = null;
  }

  return {
    allPairs,
    intents,
    operating,
    result,
    execute,
    clearResult,
  };
}

function groupByAction(pairs: BatchPair[]): Record<string, BatchPair[]> {
  const groups: Record<string, BatchPair[]> = {};
  for (const pair of pairs) {
    if (!groups[pair.action]) groups[pair.action] = [];
    groups[pair.action].push(pair);
  }
  return groups;
}
