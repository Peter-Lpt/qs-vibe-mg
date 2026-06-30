<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import SkillCard from "./SkillCard.vue";
import InstallDialog from "./InstallDialog.vue";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();

const searchQuery = ref("");
const filterAgent = ref("");
const showInstall = ref(false);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

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
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t('skills.library') }}
        <span class="text-sm font-normal ml-1" style="color: var(--c-text-secondary);">
          ({{ displaySkills.length }}/{{ skillsStore.skills.length }})
        </span>
      </h2>
      <button
        class="text-xs px-3 py-1.5 rounded cursor-pointer hover:opacity-80"
        style="background: var(--c-primary); color: white;"
        @click="showInstall = true"
      >
        + {{ t('skills.install') }}
      </button>
    </div>

    <div class="flex gap-2 mb-3">
      <input
        v-model="searchQuery"
        :placeholder="t('skills.search')"
        class="flex-1 px-3 py-2 text-xs rounded-md border outline-none"
        style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
      />
      <select
        v-model="filterAgent"
        class="px-2 py-2 text-xs rounded-md border cursor-pointer outline-none shrink-0"
        style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text); min-width: 100px;"
      >
        <option value="">{{ t('skills.all_cli') }}</option>
        <option v-for="name in agentOptions" :key="name" :value="name">{{ name }}</option>
      </select>
    </div>

    <div v-if="skillsStore.loading || skillsStore.searching" class="text-sm" style="color: var(--c-text-secondary);">
      {{ t('app.loading') }}
    </div>

    <div v-else-if="skillsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ skillsStore.error }}
    </div>

    <div
      v-else-if="displaySkills.length === 0"
      class="text-sm py-8 text-center"
      style="color: var(--c-text-secondary);"
    >
      {{ t('skills.no_skills') }}
      <p class="text-xs mt-1">{{ t('skills.no_skills_hint') }}</p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <SkillCard
        v-for="skill in displaySkills"
        :key="skill.id"
        :skill="skill"
        :agents="agentsStore.agents"
      />
    </div>

    <InstallDialog
      v-if="showInstall"
      @close="showInstall = false"
      @installed="skillsStore.fetchSkills()"
    />
  </div>
</template>
