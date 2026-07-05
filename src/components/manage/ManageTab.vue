<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import SkillRow from "./SkillRow.vue";
import InstallDialog from "../skills/InstallDialog.vue";
import EmptyState from "../common/EmptyState.vue";
import SkeletonCard from "../common/SkeletonCard.vue";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();

const searchQuery = ref("");
const statusFilter = ref("");
const agentFilter = ref("");
const showInstall = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

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

  if (agentFilter.value) {
    list = list.filter((s) =>
      s.sources.some((src) => src.from === agentFilter.value)
    );
  }

  return list;
});

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
      <select
        v-model="agentFilter"
        class="appearance-none px-3 py-2 pr-8 text-xs rounded-md border outline-none cursor-pointer transition-colors"
        style="background: var(--c-surface); border-color: var(--c-border); color: var(--c-text); min-width: 120px;"
      >
        <option value="">{{ t("manage.agent_all") || "所有 Agent" }}</option>
        <option
          v-for="agent in agentsStore.agents.filter(a => a.detected)"
          :key="agent.id"
          :value="agent.id"
        >
          {{ agent.name }}
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
      <SkillRow
        v-for="skill in displaySkills"
        :key="skill.id"
        :skill="skill"
        :agents="agentsStore.agents"
      />
    </div>

    <!-- Install dialog -->
    <InstallDialog
      v-if="showInstall"
      @close="showInstall = false"
    />
  </div>
</template>
