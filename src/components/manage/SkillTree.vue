<script setup lang="ts">
import { ref, computed, nextTick, watch } from "vue";
import { useI18n } from "vue-i18n";
import { openPath } from "@tauri-apps/plugin-opener";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import type { Agent, Skill } from "../../types";
import type { TreeRoot, TreeSkillNode, NodeLinkState } from "../../types/tree";
import { buildSkillTree } from "../../types/tree";
import ConfirmDialog from "../common/ConfirmDialog.vue";
import SkillDetail from "./SkillDetail.vue";

const props = defineProps<{
  skills: Skill[]; // 已筛选/排序后的展示列表
  agents: Agent[];
  selectedIds: Set<string>;
  expandedSkillId?: string | null; // 用于 AgentMatrix 联动高亮
}>();

const emit = defineEmits<{
  (e: "toggle:select", skillId: string): void;
  (e: "open:detail", skillId: string): void;
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const toast = useToast();

const roots = computed<TreeRoot[]>(() => buildSkillTree(props.skills, props.agents));

const expandedRoots = ref<Set<string>>(
  new Set(props.agents.map((a) => a.id).concat("library"))
);

function toggleRoot(id: string) {
  const s = new Set(expandedRoots.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  expandedRoots.value = s;
}

// —— 状态图标/颜色（逐 source 派生口径）——
const STATE_META: Record<NodeLinkState, { icon: string; color: string; tip: string }> = {
  origin: { icon: "Package", color: "var(--c-text-secondary)", tip: "manage.status_origin" },
  synced: { icon: "Link2", color: "var(--c-success)", tip: "manage.status_synced" },
  linked_elsewhere: { icon: "Share2", color: "var(--c-warning)", tip: "manage.status_linked_elsewhere" },
  independent: { icon: "Folder", color: "var(--c-primary)", tip: "manage.status_independent" },
  independent_same: { icon: "Folder", color: "var(--c-text-secondary)", tip: "manage.status_independent_same" },
  independent_conflict: { icon: "FolderTree", color: "var(--c-warning)", tip: "manage.status_independent_conflict" },
  dangling: { icon: "CircleSlash", color: "var(--c-danger)", tip: "manage.status_dangling" },
  unlinked: { icon: "CircleDashed", color: "var(--c-text-secondary)", tip: "manage.status_unlinked" },
};

// —— 行内主操作（按 linkState 决定可见动作）——
async function doLink(node: TreeSkillNode) {
  try {
    await skillsStore.createLink(node.id, node.rootId);
    toast.show(t("skills.linked", { agent: rootName(node.rootId) }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}
async function doUnlink(node: TreeSkillNode) {
  try {
    await skillsStore.removeLink(node.id, node.rootId, node.path);
    toast.show(t("skills.unlinked", { agent: rootName(node.rootId) }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}
async function doSync(node: TreeSkillNode) {
  try {
    await skillsStore.syncToVibe(node.id, node.rootId, true, node.path);
    toast.show(t("manage.synced_to_vibe", { agent: rootName(node.rootId) }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}
async function doReplace(node: TreeSkillNode) {
  try {
    await skillsStore.syncToVibe(node.id, node.rootId, false, node.path);
    toast.show(t("manage.replaced_with_link", { agent: rootName(node.rootId) }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}
async function doRelink(node: TreeSkillNode) {
  try {
    await skillsStore.relink(node.id, node.rootId, node.path);
    toast.show(t("manage.relinked", { agent: rootName(node.rootId) }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

const showDelete = ref<{ node: TreeSkillNode } | null>(null);
async function confirmDelete() {
  if (!showDelete.value) return;
  try {
    await skillsStore.deleteSkill(showDelete.value.node.id);
    toast.show(t("skills.delete"), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    showDelete.value = null;
  }
}

async function reveal(node: TreeSkillNode) {
  try {
    await openPath(node.path);
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

function copyPath(node: TreeSkillNode) {
  const p = node.path;
  navigator.clipboard?.writeText(p).then(
    () => toast.show(t("manage.path_copied") || "路径已复制", "success"),
    () => toast.show(String(p), "info")
  );
}

function rootName(rootId: string): string {
  if (rootId === "library") return t("manage.library") || "技能库";
  return props.agents.find((a) => a.id === rootId)?.name ?? rootId;
}

function linkedByCount(root: TreeRoot, node: TreeSkillNode): number {
  if (root.kind !== "library") return 0;
  const map = (root as TreeRoot & { _linkedByMap?: Record<string, number> })._linkedByMap;
  return map?.[node.id] ?? 0;
}

// —— 树内拖拽链接（审计 P0-4：拖放前校验目标已存在实体）——
const dragSkillId = ref<string | null>(null);
function onDragStart(node: TreeSkillNode) {
  dragSkillId.value = node.id;
}
function onDropOnRoot(root: TreeRoot, ev: DragEvent) {
  ev.preventDefault();
  const skillId = dragSkillId.value;
  dragSkillId.value = null;
  if (!skillId || root.kind !== "agent") return;
  // 校验：目标根是否已存在该 skill 实体（真实文件夹或软链）
  const existing = root.children.find((c) => c.id === skillId);
  if (existing && (existing.isSymlink || existing.linkState.startsWith("independent"))) {
    toast.show(t("manage.already_exists_at_agent", { agent: root.label }) || "该 Agent 已存在此 skill", "warning");
    return;
  }
  doLink({ ...({ id: skillId, rootId: root.id } as TreeSkillNode) });
}

// —— 详情：行内展开（与列表模式 SkillRow 一致，避免抽屉遮挡与竞态；单列列表下仅下推兄弟节点，无网格留白问题）——
const expandedKey = ref<string | null>(null);
function toggleDetail(node: TreeSkillNode) {
  const next = expandedKey.value === node.nodeKey ? null : node.nodeKey;
  expandedKey.value = next;
  emit("open:detail", next ? node.id : "");
}
// AgentMatrix 联动：外部传入 expandedSkillId 时自动展开并滚动到该节点
watch(
  () => props.expandedSkillId,
  (id) => {
    if (!id) return;
    for (const root of roots.value) {
      const n = root.children.find((c) => c.id === id);
      if (n) {
        expandedKey.value = n.nodeKey;
        nextTick(() => {
          document
            .getElementById(`skill-${root.id}-${id}`)
            ?.scrollIntoView({ block: "nearest", behavior: "smooth" });
        });
        return;
      }
    }
  }
);

// AgentMatrix 联动高亮
const highlighted = computed(() => {
  if (!props.expandedSkillId) return null;
  return props.expandedSkillId;
});
</script>

<template>
  <div class="text-xs">
    <div v-for="root in roots" :key="root.id" class="mb-2">
      <!-- 根节点 -->
      <div
        class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer select-none"
        :style="{
          background: 'var(--c-surface)',
          border: '1px solid var(--c-border)',
        }"
        @click="toggleRoot(root.id)"
        @dragover="root.kind === 'agent' ? ($event.preventDefault()) : null"
        @drop="onDropOnRoot(root, $event)"
      >
        <ChevronRight class="text-[10px] transition-transform" :size="14" :style="{ transform: expandedRoots.has(root.id) ? 'rotate(90deg)' : 'rotate(0deg)', color: 'var(--c-text-secondary)' }" />
        <span class="text-xs font-semibold" style="color: var(--c-text);">{{ root.label }}</span>
        <span class="text-[10px] truncate" style="color: var(--c-text-secondary);">{{ root.dirPath }}</span>
        <span class="text-[10px] ml-auto shrink-0" style="color: var(--c-text-secondary);">
          {{ root.stats.total }} · {{ root.stats.synced }}<Link2 :size="12" class="inline" /> {{ root.stats.independent }}<Folder :size="12" class="inline" />
          <template v-if="root.stats.conflict"> · <span class="inline-flex items-center gap-0.5" style="color: var(--c-warning);"><TriangleAlert :size="12" /> {{ root.stats.conflict }}</span></template>
          <template v-if="root.stats.dangling"> · <span class="inline-flex items-center gap-0.5" style="color: var(--c-danger);"><CircleSlash :size="12" /> {{ root.stats.dangling }}</span></template>
        </span>
        <button
          class="text-[10px] px-1.5 py-0.5 rounded cursor-pointer shrink-0"
          style="border: 1px solid var(--c-border); background: var(--c-bg); color: var(--c-text-secondary);"
          @click.stop="reveal({ ...({ path: root.dirPath } as TreeSkillNode) })"
          :title="t('manage.reveal')"
        ><FolderOpen :size="14" /></button>
      </div>

      <!-- 子节点（单列表） -->
      <div v-if="expandedRoots.has(root.id)" class="ml-3 mt-1 space-y-1">
        <div
          v-for="node in root.children"
          :key="node.nodeKey"
          :id="`skill-${root.id}-${node.id}`"
        >
          <!-- 行 header：点击切换展开；只承载单行内容，保持行高不变，不嵌套详情 -->
          <div
            class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer group"
            :style="{
              background: selectedIds.has(node.id)
                ? 'var(--c-primary-light)'
                : highlighted === node.id
                  ? 'var(--c-primary-light)'
                  : 'transparent',
              border: highlighted === node.id ? '1px solid var(--c-primary)' : '1px solid transparent',
            }"
            draggable="true"
            @dragstart="onDragStart(node)"
            @click="toggleDetail(node)"
          >
            <input
              type="checkbox"
              :checked="selectedIds.has(node.id)"
              class="w-3.5 h-3.5 rounded cursor-pointer shrink-0"
              style="accent-color: var(--c-primary);"
              @click.stop="emit('toggle:select', node.id)"
            />
            <component :is="STATE_META[node.linkState].icon" class="text-sm shrink-0" :size="14" :style="{ color: STATE_META[node.linkState].color }" :title="t(STATE_META[node.linkState].tip)" />
            <TriangleAlert v-if="node.hasConflict" class="shrink-0" :size="14" style="color: var(--c-warning);" />
            <span class="text-xs truncate flex-1 min-w-0" style="color: var(--c-text);">{{ node.name }}</span>
            <span v-if="root.kind === 'library' && linkedByCount(root, node) > 0" class="text-[9px] shrink-0" style="color: var(--c-primary);">
              ← {{ linkedByCount(root, node) }} agent
            </span>

            <!-- 行内主操作 -->
            <div class="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 focus-within:opacity-100">
              <button v-if="node.linkState === 'independent'" class="text-[10px] px-1 rounded cursor-pointer inline-flex items-center" style="color: var(--c-primary);" :title="t('manage.btn_sync')" @click.stop="doSync(node)"><Plus :size="14" /></button>
              <button v-if="node.linkState === 'independent_same'" class="text-[10px] px-1 rounded cursor-pointer inline-flex items-center" style="color: var(--c-primary);" :title="t('manage.btn_replace')" @click.stop="doReplace(node)"><Link2 :size="14" /></button>
              <button v-if="node.linkState === 'synced'" class="text-[10px] px-1 rounded cursor-pointer inline-flex items-center" style="color: var(--c-text-secondary);" :title="t('skills.unlink')" @click.stop="doUnlink(node)"><Unlink :size="14" /></button>
              <button v-if="node.linkState === 'linked_elsewhere'" class="text-[10px] px-1 rounded cursor-pointer inline-flex items-center" :title="t('manage.relink')" @click.stop="doRelink(node)"><RefreshCw :size="14" /></button>
              <button class="text-[10px] px-1 rounded cursor-pointer inline-flex items-center" style="color: var(--c-text-secondary);" :title="t('manage.reveal')" @click.stop="reveal(node)"><FolderOpen :size="14" /></button>
              <button class="text-[10px] px-1 rounded cursor-pointer inline-flex items-center" style="color: var(--c-text-secondary);" :title="t('manage.copy_path')" @click.stop="copyPath(node)"><Copy :size="14" /></button>
              <button class="text-[10px] px-1 rounded cursor-pointer inline-flex items-center" style="color: var(--c-danger);" :title="t('skills.delete')" @click.stop="showDelete = { node }"><Trash2 :size="14" /></button>
            </div>
          </div>

          <!-- 行内展开详情：作为 header 行的兄弟节点（而非子节点），
               故点击详情内部按钮不会再冒泡到行、误触发收起 -->
          <div v-if="expandedKey === node.nodeKey" class="ml-6 my-1 border-l pl-3 py-1" style="border-color: var(--c-border);">
            <SkillDetail :skill="node.skill" :agents="agents" />
          </div>
        </div>
      </div>
    </div>


    <!-- 删除确认 -->
    <ConfirmDialog
      v-if="showDelete"
      :title="t('skills.delete_confirm_title')"
      :message="t('skills.delete_confirm', { name: showDelete.node.name })"
      :confirm-label="t('skills.delete')"
      @confirm="confirmDelete"
      @cancel="showDelete = null"
    />
  </div>
</template>
