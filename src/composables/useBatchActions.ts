import { computed, ref, type Ref } from "vue";
import type { Agent, Skill } from "../types";
import { useSkillAgentStatus } from "./useSkillAgentStatus";
import {
  ACTION_PRIORITY,
  actionLabel,
  actionColor,
  type AgentAction,
  type TFunc,
} from "./skillActionRegistry";
import { useSkillsStore } from "../stores/skills";
import { useAgentsStore } from "../stores/agents";

export interface BatchActionOption {
  action: AgentAction;
  label: string;
  color: string;
  count: number;
  pairs: [string, string][];
}

export interface BatchDetail {
  skillId: string;
  skillName: string;
  agentId: string;
  agentName: string;
  status: "success" | "error";
  message?: string;
}

export interface BatchResult {
  action: string;
  total: number;
  success: number;
  errors: number;
  details: BatchDetail[];
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

  const availableActions = computed<BatchActionOption[]>(() => {
    const actionMap = new Map<AgentAction, { count: number; pairs: [string, string][] }>();

    for (const skill of selectedSkills.value) {
      const skillRef = computed(() => skill);
      const { allAgentStatuses } = useSkillAgentStatus(skillRef, agents, t);
      for (const status of allAgentStatuses.value) {
        if (status.action !== "none") {
          const entry = actionMap.get(status.action) || { count: 0, pairs: [] };
          entry.count++;
          entry.pairs.push([skill.id, status.agent.id]);
          actionMap.set(status.action, entry);
        }
      }
    }

    return ACTION_PRIORITY.filter((a) => a !== "none" && actionMap.has(a)).map(
      (action) => ({
        action,
        label: actionLabel(t, action),
        color: actionColor(action),
        count: actionMap.get(action)!.count,
        pairs: actionMap.get(action)!.pairs,
      })
    );
  });

  function getSkillName(skillId: string): string {
    return skillsStore.skills.find((s) => s.id === skillId)?.name || skillId;
  }

  function getAgentName(agentId: string): string {
    return agentsStore.agents.find((a) => a.id === agentId)?.name || agentId;
  }

  async function execute(option: BatchActionOption): Promise<BatchResult> {
    operating.value = true;
    result.value = null;

    const bySkill = new Map<string, string[]>();
    for (const [skillId, agentId] of option.pairs) {
      const list = bySkill.get(skillId) || [];
      list.push(agentId);
      bySkill.set(skillId, list);
    }

    const details: BatchDetail[] = [];
    let totalSynced = 0;

    for (const [skillId, agentIds] of bySkill) {
      try {
        const res = await skillsStore.batchSkillAction(
          skillId,
          agentIds,
          option.action,
          true
        );
        totalSynced += res.synced_count;
        // 成功的 agent
        for (const agentId of agentIds) {
          const hasError = res.errors.some((e) => e.startsWith(agentId + ":"));
          if (!hasError) {
            details.push({
              skillId,
              skillName: getSkillName(skillId),
              agentId,
              agentName: getAgentName(agentId),
              status: "success",
            });
          }
        }
        // 失败的 agent
        for (const err of res.errors) {
          const ci = err.indexOf(": ");
          const agentId = ci >= 0 ? err.slice(0, ci) : "";
          const message = ci >= 0 ? err.slice(ci + 2) : err;
          details.push({
            skillId,
            skillName: getSkillName(skillId),
            agentId,
            agentName: agentId ? getAgentName(agentId) : "",
            status: "error",
            message,
          });
        }
      } catch (e) {
        for (const agentId of agentIds) {
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

    await skillsStore.refreshSkills();
    await agentsStore.fetchAgents();
    operating.value = false;

    const batchResult: BatchResult = {
      action: option.label,
      total: option.pairs.length,
      success: details.filter((d) => d.status === "success").length,
      errors: details.filter((d) => d.status === "error").length,
      details,
    };
    result.value = batchResult;
    return batchResult;
  }

  function clearResult() {
    result.value = null;
  }

  return {
    availableActions,
    operating,
    result,
    execute,
    clearResult,
  };
}
