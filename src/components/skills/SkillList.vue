<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { useToast } from "../../composables/useToast";
import SkillCard from "./SkillCard.vue";
import InstallDialog from "./InstallDialog.vue";
import EmptyState from "../common/EmptyState.vue";
import SkeletonCard from "../common/SkeletonCard.vue";
import BatchActionBar from "../common/BatchActionBar.vue";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const toast = useToast();

const searchQuery = ref("");
const filterAgent = ref("");
const showInstall = ref(false);
const selectMode = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

function handleSkillKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "f")) {
    e.preventDefault();
    searchInput.value?.focus();
  }
}

onMounted(() => document.addEventListener("keydown", handleSkillKeydown));
onUnmounted(() => document.removeEventListener("keydown", handleSkillKeydown));

const agentOptions = computed(() => {
  const tags = new Set<string>();
  for (const skill of skillsStore.skills) {
    for (const src of skill.sources) {
      if (src.from !== "vibe-lib") {
        const agent = agentsStore.agents.find(a => a.id === src.from);
        tags.add(agent ? agent.name : src.from);
      }
    }
  }
  return Array.from(tags).sort();
});

const displaySkills = computed(() => {
  let list = searchQuery.value.trim() ? skillsStore.searchResults : skillsStore.skills;
  if (filterAgent.value) {
    list = list.filter(s =>
      s.sources.some(src => {
        if (src.from === "vibe-lib") return false;
        const agent = agentsStore.agents.find(a => a.id === src.from);
        return agent ? agent.name === filterAgent.value : src.from === filterAgent.value;
      })
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

function toggleSelectMode() {
  selectMode.value = !selectMode.value;
  if (!selectMode.value) {
    skillsStore.deselectAll();
  }
}

async function handleBatchLink(agentId: string) {
  const ids = Array.from(skillsStore.selectedIds);
  if (ids.length === 0) return;
  try {
    const errors = await skillsStore.batchLink(ids, agentId);
    if (errors.length > 0) {
      toast.show(`${errors.length} errors`, "error");
    } else {
      toast.show(t("skills.linked", { agent: agentId }), "success");
    }
    skillsStore.deselectAll();
    selectMode.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleBatchUnlink(agentId: string) {
  const ids = Array.from(skillsStore.selectedIds);
  if (ids.length === 0) return;
  try {
    const errors = await skillsStore.batchUnlink(ids, agentId);
    if (errors.length > 0) {
      toast.show(`${errors.length} errors`, "error");
    } else {
      toast.show(t("skills.unlinked", { agent: agentId }), "success");
    }
    skillsStore.deselectAll();
    selectMode.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-5">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t('skills.library') }}
        <span class="text-sm font-normal ml-1.5" style="color: var(--c-text-secondary);">
          ({{ displaySkills.length }}/{{ skillsStore.skills.length }})
        </span>
      </h2>
      <div class="flex items-center gap-2">
        <button
          v-if="selectMode && skillsStore.selectedIds.size > 0"
          class="text-xs px-2 py-1.5 rounded-md cursor-pointer btn-ghost"
          @click="skillsStore.selectAll()"
        >
          {{ t('skills.select_all') }}
        </button>
        <button
          v-if="selectMode && skillsStore.selectedIds.size > 0"
          class="text-xs px-2 py-1.5 rounded-md cursor-pointer btn-ghost"
          @click="skillsStore.deselectAll()"
        >
          {{ t('skills.deselect_all') }}
        </button>
        <button
          class="text-xs px-2 py-1.5 rounded-md cursor-pointer"
          :style="{
            background: selectMode ? 'var(--c-primary-light)' : 'transparent',
            color: selectMode ? 'var(--c-primary)' : 'var(--c-text-secondary)',
            border: '1px solid',
            borderColor: selectMode ? 'var(--c-primary)' : 'var(--c-border)',
          }"
          @click="toggleSelectMode"
        >
          {{ selectMode ? t('skills.exit_select') : t('skills.enter_select') }}
        </button>
        <button
          class="text-xs px-3 py-1.5 rounded-md cursor-pointer btn-primary"
          @click="showInstall = true"
        >
          + {{ t('skills.install') }}
        </button>
      </div>
    </div>

    <div class="flex gap-3 mb-5">
      <input
        ref="searchInput"
        v-model="searchQuery"
        :placeholder="t('skills.search') + ' (Ctrl+K)'"
        class="flex-1 px-3 py-2 text-xs rounded-md border outline-none transition-colors"
        style="background: var(--c-surface); border-color: var(--c-border); color: var(--c-text);"
      />
      <div class="relative shrink-0">
        <select
          v-model="filterAgent"
          class="appearance-none px-3 py-2 pr-8 text-xs rounded-md border outline-none cursor-pointer transition-colors"
          style="background: var(--c-surface); border-color: var(--c-border); color: var(--c-text); min-width: 120px;"
          @focus="($event.target as HTMLElement).style.borderColor = 'var(--c-primary)'"
          @blur="($event.target as HTMLElement).style.borderColor = 'var(--c-border)'"
        >
          <option value="">{{ t('skills.all_agents') }}</option>
          <option v-for="name in agentOptions" :key="name" :value="name">{{ name }}</option>
        </select>
        <svg
          class="absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none"
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          style="color: var(--c-text-secondary);"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </div>
    </div>

    <div v-if="skillsStore.loading || skillsStore.searching" class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <SkeletonCard v-for="i in 4" :key="i" />
    </div>

    <div v-else-if="skillsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ skillsStore.error }}
    </div>

    <EmptyState
      v-else-if="displaySkills.length === 0"
      icon="📦"
      :title="t('skills.no_skills')"
      :description="t('skills.no_skills_hint')"
      :action-label="t('skills.install')"
      @action="showInstall = true"
    />

    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <SkillCard
        v-for="skill in displaySkills"
        :key="skill.id"
        :skill="skill"
        :agents="agentsStore.agents"
        :selectable="selectMode"
      />
    </div>

    <BatchActionBar
      :selected-count="skillsStore.selectedIds.size"
      :agents="agentsStore.agents"
      @link-to-agent="handleBatchLink"
      @unlink-from-agent="handleBatchUnlink"
      @clear-selection="skillsStore.deselectAll()"
    />

    <InstallDialog
      v-if="showInstall"
      @close="showInstall = false"
    />
  </div>
</template>
