<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "../../stores/agents";
import type { Agent, SkillsTreeNode, SyncResult } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  agent: Agent;
  tree: SkillsTreeNode | null;
  loading: boolean;
  syncing: boolean;
  syncResult: SyncResult | null;
}>();

const { t } = useI18n();
const agentsStore = useAgentsStore();

const selectedSkills = ref<Set<string>>(new Set());
const expandedFolders = ref<Set<string>>(new Set());
const showDeleteConfirm = ref(false);
const toastMessage = ref("");
const toastType = ref<"success" | "error">("success");

watch(() => props.agent.id, () => {
  agentsStore.syncResult = null;
  selectedSkills.value.clear();
  expandedFolders.value.clear();
});

interface TreeItem {
  node: SkillsTreeNode;
  depth: number;
  relativePath: string;
  type: "folder" | "skill";
}

const flatItems = computed<TreeItem[]>(() => {
  if (!props.tree) return [];
  const items: TreeItem[] = [];
  flattenNode(props.tree, 0, items);
  return items;
});

function flattenNode(node: SkillsTreeNode, depth: number, items: TreeItem[]) {
  for (const child of node.children) {
    const parts = child.path.split(/[/\\]/);
    const skillsIdx = parts.indexOf("skills");
    const relativePath = skillsIdx >= 0 && skillsIdx + 1 < parts.length
      ? parts.slice(skillsIdx + 1).join("/")
      : child.name;

    const isSkill = child.skill_count > 0 && child.children.length === 0;
    items.push({ node: child, depth, relativePath, type: isSkill ? "skill" : "folder" });

    if (child.children.length > 0) {
      flattenNode(child, depth + 1, items);
    }
  }
}

function toggleFolder(path: string) {
  if (expandedFolders.value.has(path)) {
    expandedFolders.value.delete(path);
  } else {
    expandedFolders.value.add(path);
  }
}

function toggleSkill(path: string) {
  if (selectedSkills.value.has(path)) {
    selectedSkills.value.delete(path);
  } else {
    selectedSkills.value.add(path);
  }
}

function toggleFolderSkills(folderPath: string) {
  const children = flatItems.value.filter(
    (item) => item.type === "skill" && item.relativePath.startsWith(folderPath + "/")
  );
  const allSelected = children.every((c) => selectedSkills.value.has(c.relativePath));
  for (const child of children) {
    if (allSelected) {
      selectedSkills.value.delete(child.relativePath);
    } else {
      selectedSkills.value.add(child.relativePath);
    }
  }
}

function isFolderExpanded(path: string) {
  return expandedFolders.value.has(path);
}

function isSkillSelected(path: string) {
  return selectedSkills.value.has(path);
}

function isFolderFullySelected(folderPath: string) {
  const children = flatItems.value.filter(
    (item) => item.type === "skill" && item.relativePath.startsWith(folderPath + "/")
  );
  return children.length > 0 && children.every((c) => selectedSkills.value.has(c.relativePath));
}

function isFolderPartiallySelected(folderPath: string) {
  const children = flatItems.value.filter(
    (item) => item.type === "skill" && item.relativePath.startsWith(folderPath + "/")
  );
  return children.some((c) => selectedSkills.value.has(c.relativePath)) && !isFolderFullySelected(folderPath);
}

function getFolderSkillCount(folderPath: string) {
  return flatItems.value.filter(
    (item) => item.type === "skill" && item.relativePath.startsWith(folderPath + "/")
  ).length;
}

function getFolderSelectedCount(folderPath: string) {
  return flatItems.value.filter(
    (item) => item.type === "skill" && item.relativePath.startsWith(folderPath + "/") && selectedSkills.value.has(item.relativePath)
  ).length;
}

function selectAll() {
  for (const item of flatItems.value) {
    if (item.type === "skill") {
      selectedSkills.value.add(item.relativePath);
    }
  }
}

function deselectAll() {
  selectedSkills.value.clear();
}

function showToast(message: string, type: "success" | "error" = "success") {
  toastMessage.value = message;
  toastType.value = type;
  setTimeout(() => { toastMessage.value = ""; }, 3000);
}

async function syncSelected() {
  if (selectedSkills.value.size === 0) return;
  const folders = new Set<string>();
  for (const path of selectedSkills.value) {
    const parts = path.split("/");
    if (parts.length > 1) {
      folders.add(parts.slice(0, -1).join("/"));
    }
  }
  for (const folder of folders) {
    try {
      await agentsStore.syncCategoryToVab(props.agent.id, folder);
    } catch (e: unknown) {
      showToast(String(e), "error");
    }
  }
  const count = agentsStore.syncResult?.synced_count ?? 0;
  selectedSkills.value.clear();
  await agentsStore.getSkillsTree(props.agent.id);
  showToast(t('symlink.synced_count', { count }));
}

async function deleteSelected() {
  const syncedSkills = Array.from(selectedSkills.value).filter((path) => {
    const item = flatItems.value.find((i) => i.relativePath === path);
    return item?.node.synced;
  });
  if (syncedSkills.length === 0) return;

  const skillNames = syncedSkills.map((path) => {
    const parts = path.split("/");
    return parts[parts.length - 1];
  });

  try {
    const result = await agentsStore.removeSyncSkills(props.agent.id, skillNames);
    selectedSkills.value.clear();
    await agentsStore.getSkillsTree(props.agent.id);
    showDeleteConfirm.value = false;
    showToast(t('symlink.synced_count', { count: result.synced_count }));
  } catch (e: unknown) {
    showToast(String(e), "error");
  }
}

function getParentExpandKey(skillPath: string): string {
  const parts = skillPath.split("/");
  return parts.length > 1 ? parts.slice(0, -1).join("/") : "";
}

function isParentExpanded(skillPath: string): boolean {
  const parentKey = getParentExpandKey(skillPath);
  if (!parentKey) return true;
  return expandedFolders.value.has(parentKey);
}

function canDelete(): boolean {
  return Array.from(selectedSkills.value).some((path) => {
    const item = flatItems.value.find((i) => i.relativePath === path);
    return item?.node.synced;
  });
}
</script>

<template>
  <div
    class="rounded-lg border flex flex-col"
    style="background: var(--c-surface); border-color: var(--c-border); height: 100%;"
  >
    <!-- Header -->
    <div class="flex items-center justify-between p-3 border-b shrink-0" style="border-color: var(--c-border);">
      <div>
        <h3 class="text-sm font-semibold" style="color: var(--c-text);">{{ agent.name }}</h3>
        <p class="text-xs mt-0.5 truncate" style="color: var(--c-text-secondary);">{{ agent.skills_dir }}</p>
      </div>
    </div>

    <!-- Toolbar -->
    <div class="flex items-center gap-2 px-3 py-2 border-b text-xs shrink-0" style="border-color: var(--c-border); background: var(--c-surface-hover);">
      <button class="cursor-pointer transition-colors" style="color: var(--c-text-secondary);" @click="selectAll">
        {{ t('symlink.select_all') }}
      </button>
      <span style="color: var(--c-border);">|</span>
      <button class="cursor-pointer transition-colors" style="color: var(--c-text-secondary);" @click="deselectAll">
        {{ t('symlink.deselect_all') }}
      </button>
      <span v-if="selectedSkills.size > 0" class="ml-2" style="color: var(--c-primary);">
        {{ t('symlink.selected_count', { count: selectedSkills.size }) }}
      </span>
      <div class="ml-auto flex gap-1.5">
        <button
          v-if="selectedSkills.size > 0"
          class="text-xs px-2.5 py-1 rounded-md cursor-pointer transition-colors"
          style="background: var(--c-primary); color: white;"
          @click="syncSelected"
          :disabled="syncing"
        >
          {{ t('symlink.sync_selected', { count: selectedSkills.size }) }}
        </button>
        <button
          v-if="selectedSkills.size > 0"
          class="text-xs px-2.5 py-1 rounded-md cursor-pointer transition-colors"
          :style="{
            background: canDelete() ? 'var(--c-danger)' : 'var(--c-surface-hover)',
            color: canDelete() ? 'white' : 'var(--c-text-tertiary)',
            cursor: canDelete() ? 'pointer' : 'not-allowed',
          }"
          @click="canDelete() && (showDeleteConfirm = true)"
        >
          {{ t('symlink.delete_selected', { count: selectedSkills.size }) }}
        </button>
      </div>
    </div>

    <!-- Toast -->
    <div
      v-if="toastMessage"
      class="mx-3 mt-2 px-3 py-2 rounded-md text-xs"
      :style="{
        background: toastType === 'success' ? 'var(--c-success-light)' : 'var(--c-danger-light)',
        color: toastType === 'success' ? 'var(--c-success)' : 'var(--c-danger)',
      }"
    >
      {{ toastMessage }}
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 flex items-center justify-center text-sm" style="color: var(--c-text-secondary);">
      {{ t('app.loading') }}
    </div>

    <!-- Sync Result -->
    <div v-else-if="syncResult" class="p-3">
      <div
        class="p-2 rounded text-xs"
        :style="{
          background: syncResult.errors.length > 0 ? 'var(--c-warning-light)' : 'var(--c-success-light)',
          color: syncResult.errors.length > 0 ? 'var(--c-warning)' : 'var(--c-success)',
        }"
      >
        {{ t('symlink.synced_count', { count: syncResult.synced_count }) }}
        <span v-if="syncResult.errors.length > 0"> | {{ syncResult.errors.length }} {{ t('symlink.errors') }}</span>
      </div>
    </div>

    <!-- Empty -->
    <div v-else-if="flatItems.length === 0" class="flex-1 flex items-center justify-center text-sm" style="color: var(--c-text-secondary);">
      {{ t('symlink.no_skills') }}
    </div>

    <!-- Tree -->
    <div v-else class="flex-1 overflow-y-auto">
      <div v-for="item in flatItems" :key="item.node.path">
        <!-- Folder row -->
        <div
          v-if="item.type === 'folder'"
          class="flex items-center gap-2 px-3 py-1.5 text-xs cursor-pointer hover:bg-[var(--c-surface-hover)]"
          :style="{ paddingLeft: (12 + item.depth * 16) + 'px' }"
          @click="toggleFolder(item.relativePath)"
        >
          <span class="w-4 text-center" style="color: var(--c-text-secondary);">
            {{ isFolderExpanded(item.relativePath) ? '▼' : '▶' }}
          </span>
          <span style="color: var(--c-text-secondary);">📁</span>
          <span class="flex-1 truncate font-medium" style="color: var(--c-text);">{{ item.node.name }}</span>
          <span
            v-if="isFolderFullySelected(item.relativePath)"
            class="w-4 h-4 rounded border flex items-center justify-center text-[10px]"
            style="background: var(--c-primary); color: white; border-color: var(--c-primary);"
            @click.stop="toggleFolderSkills(item.relativePath)"
          >✓</span>
          <span
            v-else-if="isFolderPartiallySelected(item.relativePath)"
            class="w-4 h-4 rounded border flex items-center justify-center text-[10px]"
            style="background: var(--c-primary); color: white; border-color: var(--c-primary);"
            @click.stop="toggleFolderSkills(item.relativePath)"
          >—</span>
          <span
            v-else
            class="w-4 h-4 rounded border cursor-pointer"
            style="border-color: var(--c-border);"
            @click.stop="toggleFolderSkills(item.relativePath)"
          />
          <span class="text-[10px]" style="color: var(--c-text-secondary);">
            {{ getFolderSelectedCount(item.relativePath) }}/{{ getFolderSkillCount(item.relativePath) }}
          </span>
          <button
            class="text-[10px] px-1.5 py-0.5 rounded cursor-pointer transition-colors"
            style="background: var(--c-primary); color: white;"
            @click.stop="agentsStore.syncCategoryToVab(agent.id, item.relativePath)"
            :disabled="syncing"
          >↻</button>
        </div>

        <!-- Skill row (inside expanded folder) -->
        <div
          v-else-if="item.type === 'skill' && isParentExpanded(item.relativePath)"
          class="flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-[var(--c-surface-hover)]"
          :style="{ paddingLeft: (12 + item.depth * 16) + 'px' }"
          @click="toggleSkill(item.relativePath)"
        >
          <span class="w-4" />
          <span
            class="w-4 h-4 rounded border flex items-center justify-center text-[10px]"
            :style="{
              background: isSkillSelected(item.relativePath) ? 'var(--c-primary)' : 'transparent',
              color: isSkillSelected(item.relativePath) ? 'white' : 'transparent',
              borderColor: isSkillSelected(item.relativePath) ? 'var(--c-primary)' : 'var(--c-border)',
            }"
          >✓</span>
          <span v-if="item.node.is_source_link" style="color: var(--c-warning);" :title="t('symlink.linked_elsewhere') + ': ' + item.node.link_target">⚠</span>
          <span v-else style="color: var(--c-text-secondary);">📄</span>
          <span class="flex-1 truncate" style="color: var(--c-text);">{{ item.node.name }}</span>
          <span v-if="item.node.synced" class="text-[10px] px-1 rounded" style="background: var(--c-success-light); color: var(--c-success);">
            ✓ {{ t('symlink.status_synced') }}
          </span>
          <span v-else-if="item.node.is_source_link" class="text-[10px] truncate max-w-[120px]" style="color: var(--c-warning);" :title="item.node.link_target">
            → {{ item.node.link_target?.split(/[/\\]/).pop() }}
          </span>
        </div>

        <!-- Root-level skill (depth 0, no parent folder) -->
        <div
          v-else-if="item.type === 'skill' && item.depth === 0"
          class="flex items-center gap-2 px-3 py-1.5 text-xs cursor-pointer hover:bg-[var(--c-surface-hover)]"
          style="padding-left: 12px;"
          @click="toggleSkill(item.relativePath)"
        >
          <span class="w-4" />
          <span
            class="w-4 h-4 rounded border flex items-center justify-center text-[10px]"
            :style="{
              background: isSkillSelected(item.relativePath) ? 'var(--c-primary)' : 'transparent',
              color: isSkillSelected(item.relativePath) ? 'white' : 'transparent',
              borderColor: isSkillSelected(item.relativePath) ? 'var(--c-primary)' : 'var(--c-border)',
            }"
          >✓</span>
          <span v-if="item.node.is_source_link" style="color: var(--c-warning);" :title="t('symlink.linked_elsewhere') + ': ' + item.node.link_target">⚠</span>
          <span v-else style="color: var(--c-text-secondary);">📄</span>
          <span class="flex-1 truncate" style="color: var(--c-text);">{{ item.node.name }}</span>
          <span v-if="item.node.synced" class="text-[10px] px-1 rounded" style="background: var(--c-success-light); color: var(--c-success);">
            ✓ {{ t('symlink.status_synced') }}
          </span>
          <span v-else-if="item.node.is_source_link" class="text-[10px] truncate max-w-[120px]" style="color: var(--c-warning);" :title="item.node.link_target">
            → {{ item.node.link_target?.split(/[/\\]/).pop() }}
          </span>
        </div>
      </div>
    </div>

    <!-- Delete Confirm -->
    <ConfirmDialog
      v-if="showDeleteConfirm"
      :title="t('symlink.delete_selected', { count: selectedSkills.size })"
      :message="t('symlink.delete_confirm', { count: selectedSkills.size })"
      :confirm-text="t('symlink.delete_selected', { count: selectedSkills.size })"
      :danger="true"
      @confirm="deleteSelected"
      @cancel="showDeleteConfirm = false"
    />
  </div>
</template>
