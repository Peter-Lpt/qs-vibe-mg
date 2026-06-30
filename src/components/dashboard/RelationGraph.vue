<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { DashboardAgent, DashboardSkill } from "../../types";

const { t } = useI18n();

const props = defineProps<{
  agents: DashboardAgent[];
}>();

const agentColors = [
  "#6366f1", "#8b5cf6", "#06b6d4", "#10b981",
  "#f59e0b", "#ef4444", "#ec4899", "#14b8a6",
];

function getAgentColor(i: number) {
  return agentColors[i % agentColors.length];
}

const panX = ref(0);
const panY = ref(0);
const zoom = ref(1);
const isPanning = ref(false);
const panStart = ref({ x: 0, y: 0 });
const panStartOffset = ref({ x: 0, y: 0 });
const dragNode = ref<string | null>(null);
const dragOffset = ref({ x: 0, y: 0 });
const hoveredNode = ref<string | null>(null);

interface GNode {
  id: string;
  x: number;
  y: number;
  type: "agent" | "skill";
  label: string;
  color: string;
  agentIds: string[];
  shared: boolean;
}

interface GEdge {
  x1: number; y1: number;
  x2: number; y2: number;
  color: string;
  shared: boolean;
  curved: boolean;
  cx: number; cy: number;
}

const nodes = ref<GNode[]>([]);
const nodeMap = ref<Record<string, GNode>>({});

const agentList = computed(() => props.agents.filter((a) => a.agent_id !== "vibe-lib"));

const allSkills = computed(() => {
  const map = new Map<string, { skill: DashboardSkill; agentIds: string[] }>();
  for (const agent of agentList.value) {
    for (const skill of agent.skills) {
      const existing = map.get(skill.skill_id);
      if (existing) {
        if (!existing.agentIds.includes(agent.agent_id)) {
          existing.agentIds.push(agent.agent_id);
        }
      } else {
        map.set(skill.skill_id, { skill, agentIds: [agent.agent_id] });
      }
    }
  }
  return Array.from(map.values());
});

function initLayout() {
  nodes.value = [];
  nodeMap.value = {};

  const list = agentList.value;
  const count = list.length;
  const cx = 450;
  const cy = 300;
  const agentR = 220;
  const agentPos: Record<string, { x: number; y: number }> = {};

  list.forEach((agent, i) => {
    const angle = (2 * Math.PI * i) / count - Math.PI / 2;
    const pos = { x: cx + agentR * Math.cos(angle), y: cy + agentR * Math.sin(angle) };
    agentPos[agent.agent_id] = pos;
    const n: GNode = {
      id: agent.agent_id, x: pos.x, y: pos.y, type: "agent",
      label: agent.agent_name, color: getAgentColor(i),
      agentIds: [agent.agent_id], shared: false,
    };
    nodes.value.push(n);
    nodeMap.value[agent.agent_id] = n;
  });

  const skillIdx: Record<string, number> = {};
  for (const { skill, agentIds } of allSkills.value) {
    const shared = agentIds.length > 1;
    let x: number, y: number;

    if (shared) {
      let sx = 0, sy = 0;
      for (const aid of agentIds) {
        const ap = agentPos[aid];
        if (ap) { sx += ap.x; sy += ap.y; }
      }
      sx /= agentIds.length;
      sy /= agentIds.length;
      const dx = cx - sx, dy = cy - sy;
      const d = Math.sqrt(dx * dx + dy * dy);
      if (d > 20) { sx += (dx / d) * 50; sy += (dy / d) * 50; }
      x = sx; y = sy;
    } else {
      const aid = agentIds[0];
      const ap = agentPos[aid];
      if (!ap) continue;
      const idx = skillIdx[aid] || 0;
      skillIdx[aid] = idx + 1;
      const baseAngle = Math.atan2(ap.y - cy, ap.x - cx);
      const jitter = ((idx * 137.508) % 360) * Math.PI / 180;
      const r = 65 + (idx % 5) * 12;
      x = ap.x + r * Math.cos(baseAngle + jitter);
      y = ap.y + r * Math.sin(baseAngle + jitter);
    }

    const n: GNode = {
      id: skill.skill_id, x, y, type: "skill",
      label: skill.skill_name || skill.skill_id,
      color: shared ? "#f59e0b" : getAgentColor(list.findIndex(a => a.agent_id === agentIds[0])),
      agentIds, shared,
    };
    nodes.value.push(n);
    nodeMap.value[skill.skill_id] = n;
  }
}

function buildEdges(): GEdge[] {
  const edges: GEdge[] = [];
  for (const node of nodes.value) {
    if (node.type !== "skill") continue;
    for (const aid of node.agentIds) {
      const an = nodeMap.value[aid];
      if (!an) continue;
      const shared = node.shared;
      // curve toward center for shared edges
      const mx = (node.x + an.x) / 2;
      const my = (node.y + an.y) / 2;
      const dx = 450 - mx, dy = 300 - my;
      const d = Math.sqrt(dx * dx + dy * dy);
      const cpx = mx + (d > 10 ? dx / d * 30 : 0);
      const cpy = my + (d > 10 ? dy / d * 30 : 0);
      edges.push({
        x1: node.x, y1: node.y, x2: an.x, y2: an.y,
        color: shared ? "#f59e0b" : node.color,
        shared, curved: shared, cx: cpx, cy: cpy,
      });
    }
  }
  return edges;
}

const graphEdges = computed(() => buildEdges());

onMounted(() => initLayout());
watch(() => props.agents, () => initLayout());

function getPos(id: string) {
  const n = nodeMap.value[id] || nodes.value.find(n => n.id === id);
  return n ? { x: n.x, y: n.y } : { x: 0, y: 0 };
}

function onSvgMouseDown(e: MouseEvent) {
  const target = (e.target as SVGElement).closest("[data-nid]");
  if (target) {
    const nid = target.getAttribute("data-nid")!;
    dragNode.value = nid;
    const pos = getPos(nid);
    dragOffset.value = { x: e.clientX - pos.x, y: e.clientY - pos.y };
    e.preventDefault();
    return;
  }
  isPanning.value = true;
  panStart.value = { x: e.clientX, y: e.clientY };
  panStartOffset.value = { x: panX.value, y: panY.value };
}

function onMouseMove(e: MouseEvent) {
  if (dragNode.value) {
    const n = nodeMap.value[dragNode.value] || nodes.value.find(n => n.id === dragNode.value);
    if (n) {
      n.x = (e.clientX - dragOffset.value.x - panX.value) / zoom.value;
      n.y = (e.clientY - dragOffset.value.y - panY.value) / zoom.value;
    }
    return;
  }
  if (isPanning.value) {
    panX.value = panStartOffset.value.x + (e.clientX - panStart.value.x);
    panY.value = panStartOffset.value.y + (e.clientY - panStart.value.y);
  }
}

function onMouseUp() { dragNode.value = null; isPanning.value = false; }

function onWheel(e: WheelEvent) {
  zoom.value = Math.min(Math.max(zoom.value * (e.deltaY > 0 ? 0.92 : 1.08), 0.15), 3);
  e.preventDefault();
}

onUnmounted(() => {
  document.removeEventListener("mousemove", onMouseMove);
  document.removeEventListener("mouseup", onMouseUp);
});

const viewBox = computed(() => {
  const w = 900 / zoom.value;
  const h = 600 / zoom.value;
  return `${-panX.value / zoom.value} ${-panY.value / zoom.value} ${w} ${h}`;
});

const uniqueCount = computed(() => allSkills.value.filter(s => s.agentIds.length === 1).length);
const sharedCount = computed(() => allSkills.value.filter(s => s.agentIds.length > 1).length);
</script>

<template>
  <div class="rounded-lg border overflow-hidden select-none" style="background: var(--c-surface); border-color: var(--c-border); cursor: grab;">
    <div class="flex items-center justify-between px-3 py-2 border-b" style="border-color: var(--c-border);">
      <div class="flex items-center gap-3">
        <h3 class="text-xs font-semibold" style="color: var(--c-text);">{{ t('dashboard.relation_graph') }}</h3>
        <span class="text-xs" style="color: var(--c-text-secondary);">
          {{ allSkills.length }} {{ t('dashboard.skills') }}
          ({{ uniqueCount }} {{ t('dashboard.unique') }}, {{ sharedCount }} {{ t('dashboard.shared') }})
        </span>
      </div>
      <span class="text-xs" style="color: var(--c-text-secondary);">{{ t('dashboard.drag_hint') }}</span>
    </div>

    <svg width="100%" height="450" :viewBox="viewBox"
      @mousedown="onSvgMouseDown" @mousemove="onMouseMove"
      @mouseup="onMouseUp" @mouseleave="onMouseUp" @wheel.prevent="onWheel"
      style="background: var(--c-bg);"
    >
      <defs>
        <filter id="glow">
          <feGaussianBlur stdDeviation="3" result="blur" />
          <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
        </filter>
        <filter id="shadow">
          <feDropShadow dx="0" dy="1" stdDeviation="2" flood-opacity="0.15" />
        </filter>
      </defs>

      <!-- Edges -->
      <template v-for="(edge, i) in graphEdges" :key="'e' + i">
        <line v-if="!edge.curved"
          :x1="edge.x1" :y1="edge.y1" :x2="edge.x2" :y2="edge.y2"
          :stroke="edge.color" :stroke-width="edge.shared ? 1.5 : 0.6"
          :stroke-opacity="edge.shared ? 0.4 : 0.15"
          :stroke-dasharray="edge.shared ? 'none' : '3,3'"
        />
        <path v-else
          :d="`M${edge.x1},${edge.y1} Q${edge.cx},${edge.cy} ${edge.x2},${edge.y2}`"
          fill="none" :stroke="edge.color" stroke-width="1.5"
          stroke-opacity="0.45" filter="url(#glow)"
        />
      </template>

      <!-- Skill nodes -->
      <g v-for="node in nodes.filter(n => n.type === 'skill')" :key="node.id"
        :data-nid="node.id" style="cursor: grab;"
        @mouseenter="hoveredNode = node.id" @mouseleave="hoveredNode = null"
      >
        <circle v-if="node.shared"
          :cx="node.x" :cy="node.y" r="7"
          fill="#f59e0b" fill-opacity="0.2" filter="url(#glow)"
        />
        <circle
          :cx="node.x" :cy="node.y" :r="node.shared ? 5 : 3"
          :fill="node.shared ? '#f59e0b' : node.color"
          :fill-opacity="node.shared ? 1 : 0.5"
          :stroke="hoveredNode === node.id ? '#fff' : (node.shared ? '#d97706' : 'none')"
          :stroke-width="hoveredNode === node.id ? 2 : (node.shared ? 1 : 0)"
        />
        <text v-if="node.shared || hoveredNode === node.id"
          :x="node.x" :y="node.y - 10" text-anchor="middle"
          fill="var(--c-text)" font-size="7" font-weight="500"
          style="pointer-events: none; text-shadow: 0 0 3px var(--c-bg);"
        >{{ node.label.length > 18 ? node.label.slice(0, 17) + '.' : node.label }}</text>
      </g>

      <!-- Agent nodes -->
      <g v-for="node in nodes.filter(n => n.type === 'agent')" :key="node.id"
        :data-nid="node.id" style="cursor: grab;"
        @mouseenter="hoveredNode = node.id" @mouseleave="hoveredNode = null"
      >
        <circle :cx="node.x" :cy="node.y" r="30"
          :fill="node.color" fill-opacity="0.06" filter="url(#shadow)"
        />
        <circle :cx="node.x" :cy="node.y" r="26"
          fill="var(--c-surface)" :stroke="node.color" stroke-width="2.5"
        />
        <circle :cx="node.x" :cy="node.y" r="22"
          :fill="node.color" fill-opacity="0.08"
        />
        <text :x="node.x" :y="node.y - 1" text-anchor="middle" dominant-baseline="middle"
          :fill="node.color" font-size="9" font-weight="700"
          style="pointer-events: none;"
        >{{ node.label.length > 10 ? node.label.slice(0, 9) + '.' : node.label }}</text>
        <text :x="node.x" :y="node.y + 12" text-anchor="middle"
          fill="var(--c-text-secondary)" font-size="8" style="pointer-events: none;"
        >{{ agents.find(a => a.agent_id === node.id)?.skill_count ?? 0 }}</text>
      </g>
    </svg>
  </div>
</template>
