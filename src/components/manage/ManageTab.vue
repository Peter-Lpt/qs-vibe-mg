<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { useToast } from "../../composables/useToast";
import SkillRow from "./SkillRow.vue";
import InstallDialog from "../skills/InstallDialog.vue";
import EmptyState from "../common/EmptyState.vue";
import SkeletonCard from "../common/SkeletonCard.vue";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const toast = useToast();

const searchQuery = ref("");
const statusFilter = ref("");
const showInstall = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

// 批量选择
const selectedSkills = ref<Set<string>>(new Set());
const showBatchLinkMenu = ref(false);
const showBatchUnlinkMenu = ref(false);
const batchOperating = ref(false);

function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "f")) {
    e.preventDefault();
    searchInput.value?.focus();
  }
}

onMounted(() => document.addEventListener("keydown", handleKeydown));
onUnmounted(() => document.removeEventListener("keydown", handleKeydown));

const statusOptions = computed(() => [
  { value: "", label: t("manage.status_all") || "全部" },
  { value: "conflict", label: t("manage.status_conflict") || "有冲突" },
  { value: "dangling", label: t("manage.status_dangling") || "有断链" },
  { value: "independent", label: t("manage.status_independent") || "需要同步" },
  { value: "unlinked", label: t("manage.status_unlinked") || "未链接" },
  { value: "linked", label: t("manage.status_linked") || "已链接" },
  { value: "duplicate", label: t("manage.status_duplicate") || "重复" },
]);

const displaySkills = computed(() => {
  let list = searchQuery.value.trim()
    ? skillsStore.searchResults
    : skillsStore.skills;

  if (statusFilter.value) {
    list = list.filter((s) => {
      switch (statusFilter.value) {
        case "conflict":
          return s.has_conflict;
        case "dangling":
          return s.has_dangling;
        case "duplicate":
          return s.is_duplicate;
        case "independent":
          // 有独立副本（非 symlink）的 agent
          return s.sources.some(
            (src) => src.from !== "vibe-lib" && !src.is_symlink
          );
        case "unlinked":
          return !s.sources.some(
            (src) => src.from !== "vibe-lib" && src.is_symlink
          );
        case "linked":
          return s.sources.some(
            (src) => src.from !== "vibe-lib" && src.is_symlink
          );
        default:
          return true;
      }
    });
  }

  return list;
});

// 是否全选
const isAllSelected = computed(() => {
  if (displaySkills.value.length === 0) return false;
  return displaySkills.value.every((s) => selectedSkills.value.has(s.id));
});

// 切换全选
function toggleSelectAll() {
  if (isAllSelected.value) {
    selectedSkills.value.clear();
  } else {
    displaySkills.value.forEach((s) => selectedSkills.value.add(s.id));
  }
}

// 切换单个选择
function toggleSelect(skillId: string) {
  if (selectedSkills.value.has(skillId)) {
    selectedSkills.value.delete(skillId);
  } else {
    selectedSkills.value.add(skillId);
  }
}

// 可以批量链接的 agent（选中的 skill 都没有 symlink 到该 agent，且都在技能库中）
const batchLinkableAgents = computed(() => {
  const selected = displaySkills.value.filter((s) =>
    selectedSkills.value.has(s.id)
  );
  if (selected.length === 0) return [];

  // 所有选中的 skill 都必须在技能库中
  const allInVibeLib = selected.every((s) =>
    s.sources.some((src) => src.from === "vibe-lib")
  );
  if (!allInVibeLib) return [];

  return agentsStore.agents.filter((a) => {
    if (!a.detected) return false;
    // 选中的 skill 都没有 symlink 到这个 agent
    return selected.every(
      (s) => !s.sources.some((src) => src.from === a.id && src.is_symlink)
    );
  });
});

// 可以批量取消链接的 agent（选中的 skill 都有 symlink 到该 agent）
const batchUnlinkableAgents = computed(() => {
  const selected = displaySkills.value.filter((s) =>
    selectedSkills.value.has(s.id)
  );
  if (selected.length === 0) return [];

  return agentsStore.agents.filter((a) => {
    if (!a.detected) return false;
    // 选中的 skill 都有 symlink 到这个 agent
    return selected.every((s) =>
      s.sources.some((src) => src.from === a.id && src.is_symlink)
    );
  });
});

// 批量同步到技能库
async function handleBatchSyncToVibe() {
  const selected = displaySkills.value.filter(
    (s) =>
      selectedSkills.value.has(s.id) &&
      s.sources.some((src) => src.from !== "vibe-lib" && !src.is_symlink)
  );

  if (selected.length === 0) {
    toast.show(t("manage.no_syncable_skills"), "info");
    return;
  }

  batchOperating.value = true;
  let successCount = 0;
  let errorCount = 0;

  for (const skill of selected) {
    // 找到有独立副本的 agent
    const independentSource = skill.sources.find(
      (src) => src.from !== "vibe-lib" && !src.is_symlink
    );
    if (independentSource) {
      try {
        await skillsStore.syncToVibe(skill.id, independentSource.from);
        successCount++;
      } catch {
        errorCount++;
      }
    }
  }

  batchOperating.value = false;
  selectedSkills.value.clear();

  if (errorCount > 0) {
    toast.show(
      t("manage.batch_sync_result", { success: successCount, error: errorCount }),
      "info"
    );
  } else {
    toast.show(t("manage.batch_sync_success", { count: successCount }), "success");
  }
}

// 批量链接到 Agent
async function handleBatchLink(agentId: string) {
  batchOperating.value = true;
  let successCount = 0;
  let errorCount = 0;

  for (const skillId of selectedSkills.value) {
    try {
      await skillsStore.createLink(skillId, agentId);
      successCount++;
    } catch {
      errorCount++;
    }
  }

  batchOperating.value = false;
  selectedSkills.value.clear();
  showBatchLinkMenu.value = false;

  if (errorCount > 0) {
    toast.show(
      t("manage.batch_link_result", { success: successCount, error: errorCount }),
      "info"
    );
  } else {
    toast.show(t("manage.batch_link_success", { count: successCount }), "success");
  }
}

// 批量取消链接
async function handleBatchUnlink(agentId: string) {
  batchOperating.value = true;
  let successCount = 0;
  let errorCount = 0;

  for (const skillId of selectedSkills.value) {
    try {
      await skillsStore.removeLink(skillId, agentId);
      successCount++;
    } catch {
      errorCount++;
    }
  }

  batchOperating.value = false;
  selectedSkills.value.clear();
  showBatchUnlinkMenu.value = false;

  if (errorCount > 0) {
    toast.show(
      t("manage.batch_unlink_result", { success: successCount, error: errorCount }),
      "info"
    );
  } else {
    toast.show(t("manage.batch_unlink_success", { count: successCount }), "success");
  }
}

// 批量清理断链
async function handleBatchCleanDangling() {
  const selected = displaySkills.value.filter(
    (s) =>
      selectedSkills.value.has(s.id) &&
      s.sources.some((src) => src.is_symlink && !src.symlink_target)
  );

  if (selected.length === 0) {
    toast.show(t("manage.no_dangling_skills"), "info");
    return;
  }

  batchOperating.value = true;
  let successCount = 0;

  for (const skill of selected) {
    const danglingSources = skill.sources.filter(
      (src) => src.is_symlink && !src.symlink_target
    );
    for (const source of danglingSources) {
      try {
        await skillsStore.removeLink(skill.id, source.from);
        successCount++;
      } catch {
        // 忽略单个失败
      }
    }
  }

  batchOperating.value = false;
  selectedSkills.value.clear();

  toast.show(t("manage.batch_clean_success", { count: successCount }), "success");
}

watch(searchQuery, (val) => {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    skillsStore.searchSkills(val);
  }, 300);
});
</script>

<template>
  <div>
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t("manage.title") || "软连接管理" }}
        <span class="text-sm font-normal ml-1.5" style="color: var(--c-text-secondary);">
          ({{ displaySkills.length }}/{{ skillsStore.skills.length }})
        </span>
      </h2>
      <button
        class="text-xs px-3 py-1.5 rounded-md cursor-pointer btn-primary"
        @click="showInstall = true"
      >
        + {{ t("skills.install") }}
      </button>
    </div>

    <!-- Filters -->
    <div class="flex gap-3 mb-4">
      <select
        v-model="statusFilter"
        class="appearance-none px-3 py-2 pr-8 text-xs rounded-md border outline-none cursor-pointer transition-colors"
        style="background: var(--c-surface); border-color: var(--c-border); color: var(--c-text); min-width: 120px;"
      >
        <option v-for="opt in statusOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
      <input
        ref="searchInput"
        v-model="searchQuery"
        :placeholder="t('skills.search') + ' (Ctrl+K)'"
        class="flex-1 px-3 py-2 text-xs rounded-md border outline-none transition-colors"
        style="background: var(--c-surface); border-color: var(--c-border); color: var(--c-text);"
      />
    </div>

    <!-- Batch action bar -->
    <div
      v-if="selectedSkills.size > 0"
      class="flex items-center gap-3 px-3 py-2 mb-4 rounded-md"
      style="background: var(--c-primary-light); border: 1px solid var(--c-primary);"
    >
      <span class="text-xs font-medium" style="color: var(--c-primary);">
        {{ t("manage.selected_count", { count: selectedSkills.size }) }}
      </span>

      <button
        class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
        style="background: var(--c-primary); color: white;"
        :disabled="batchOperating"
        @click="handleBatchSyncToVibe"
      >
        {{ t("manage.batch_sync") }}
      </button>

      <div class="relative">
        <button
          class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
          style="background: var(--c-surface); color: var(--c-text); border: 1px solid var(--c-border);"
          :disabled="batchOperating || batchLinkableAgents.length === 0"
          @click="showBatchLinkMenu = !showBatchLinkMenu"
        >
          {{ t("manage.batch_link") }} ▾
        </button>
        <div
          v-if="showBatchLinkMenu"
          class="absolute top-full left-0 mt-1 rounded-md border shadow-lg z-10 py-1 min-w-[160px]"
          style="background: var(--c-surface); border-color: var(--c-border);"
        >
          <button
            v-for="agent in batchLinkableAgents"
            :key="agent.id"
            class="block w-full text-left px-3 py-1.5 text-xs hover:bg-[var(--c-surface-hover)] cursor-pointer"
            style="color: var(--c-text);"
            @click="handleBatchLink(agent.id)"
          >
            {{ agent.name }}
          </button>
        </div>
      </div>

      <div class="relative">
        <button
          class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
          style="background: var(--c-surface); color: var(--c-text); border: 1px solid var(--c-border);"
          :disabled="batchOperating || batchUnlinkableAgents.length === 0"
          @click="showBatchUnlinkMenu = !showBatchUnlinkMenu"
        >
          {{ t("manage.batch_unlink") }} ▾
        </button>
        <div
          v-if="showBatchUnlinkMenu"
          class="absolute top-full left-0 mt-1 rounded-md border shadow-lg z-10 py-1 min-w-[160px]"
          style="background: var(--c-surface); border-color: var(--c-border);"
        >
          <button
            v-for="agent in batchUnlinkableAgents"
            :key="agent.id"
            class="block w-full text-left px-3 py-1.5 text-xs hover:bg-[var(--c-surface-hover)] cursor-pointer"
            style="color: var(--c-text);"
            @click="handleBatchUnlink(agent.id)"
          >
            {{ agent.name }}
          </button>
        </div>
      </div>

      <button
        class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
        style="background: var(--c-danger); color: white;"
        :disabled="batchOperating"
        @click="handleBatchCleanDangling"
      >
        {{ t("manage.batch_clean_dangling") }}
      </button>
    </div>

    <!-- Loading -->
    <div v-if="skillsStore.loading" class="space-y-3">
      <SkeletonCard v-for="i in 4" :key="i" />
    </div>

    <!-- Error -->
    <div v-else-if="skillsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ skillsStore.error }}
    </div>

    <!-- Empty -->
    <EmptyState
      v-else-if="displaySkills.length === 0"
      icon="📦"
      :title="t('skills.no_skills')"
      :description="t('skills.no_skills_hint')"
      :action-label="t('skills.install')"
      @action="showInstall = true"
    />

    <!-- Skill list -->
    <div v-else class="space-y-2">
      <!-- Select all header -->
      <div class="flex items-center gap-3 px-3 py-2 text-xs" style="color: var(--c-text-secondary);">
        <input
          type="checkbox"
          :checked="isAllSelected"
          class="w-4 h-4 rounded cursor-pointer"
          style="accent-color: var(--c-primary);"
          @change="toggleSelectAll"
        />
        <span>{{ t("manage.select_all") }}</span>
      </div>

      <SkillRow
        v-for="skill in displaySkills"
        :key="skill.id"
        :skill="skill"
        :agents="agentsStore.agents"
        :selected="selectedSkills.has(skill.id)"
        @toggle-select="toggleSelect(skill.id)"
      />
    </div>

    <!-- Install dialog -->
    <InstallDialog
      v-if="showInstall"
      @close="showInstall = false"
    />
  </div>
</template>
