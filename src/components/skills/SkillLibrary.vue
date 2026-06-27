<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import SkillCard from "./SkillCard.vue";
import InstallDialog from "./InstallDialog.vue";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();

const searchQuery = ref("");
const showInstall = ref(false);

const filteredSkills = () => {
  if (!searchQuery.value.trim()) return skillsStore.skills;
  const q = searchQuery.value.toLowerCase();
  return skillsStore.skills.filter(
    (s) =>
      s.name.toLowerCase().includes(q) ||
      s.description.toLowerCase().includes(q)
  );
};
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-base font-semibold" style="color: var(--c-text);">
        {{ t('skills.library') }}
        <span class="text-sm font-normal ml-1" style="color: var(--c-text-secondary);">
          ({{ skillsStore.skills.length }})
        </span>
      </h2>
      <div class="flex gap-2">
        <!-- Select all / deselect -->
        <button
          v-if="skillsStore.selectedIds.size > 0"
          class="text-xs px-2 py-1 rounded border cursor-pointer hover:opacity-80"
          style="border-color: var(--c-border); color: var(--c-text-secondary);"
          @click="skillsStore.deselectAll()"
        >
          {{ t('skills.deselect_all') }} ({{ skillsStore.selectedIds.size }})
        </button>
        <button
          v-else
          class="text-xs px-2 py-1 rounded border cursor-pointer hover:opacity-80"
          style="border-color: var(--c-border); color: var(--c-text-secondary);"
          @click="skillsStore.selectAll()"
        >
          {{ t('skills.select_all') }}
        </button>
        <!-- Install button -->
        <button
          class="text-xs px-3 py-1 rounded cursor-pointer hover:opacity-80"
          style="background: var(--c-primary); color: white;"
          @click="showInstall = true"
        >
          + {{ t('skills.install') }}
        </button>
      </div>
    </div>

    <!-- Search -->
    <div class="mb-3">
      <input
        v-model="searchQuery"
        :placeholder="t('skills.search')"
        class="w-full px-3 py-2 text-xs rounded-md border outline-none"
        style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text);"
      />
    </div>

    <!-- Batch actions bar -->
    <div
      v-if="skillsStore.selectedIds.size > 0"
      class="flex items-center gap-2 px-3 py-2 rounded-md mb-3 text-xs"
      style="background: var(--c-primary); color: white;"
    >
      <span>{{ t('skills.selected', { count: skillsStore.selectedIds.size }) }}</span>
      <div class="flex gap-1 ml-auto">
        <select
          v-for="agent in agentsStore.agents.filter(a => a.detected)"
          :key="agent.id"
          class="text-xs px-2 py-0.5 rounded border cursor-pointer"
          style="background: white; color: var(--c-text); border-color: var(--c-border);"
          @change="
            async (e) => {
              const target = e.target as HTMLSelectElement;
              if (target.value) {
                await skillsStore.batchLink(Array.from(skillsStore.selectedIds), target.value);
                skillsStore.deselectAll();
                target.value = '';
              }
            }
          "
        >
          <option value="">Link → {{ agent.name }}</option>
          <option :value="agent.id">{{ agent.name }}</option>
        </select>
      </div>
    </div>

    <div v-if="skillsStore.loading" class="text-sm" style="color: var(--c-text-secondary);">
      {{ t('app.loading') }}
    </div>

    <div v-else-if="skillsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ skillsStore.error }}
    </div>

    <div
      v-else-if="skillsStore.skills.length === 0"
      class="text-sm py-8 text-center"
      style="color: var(--c-text-secondary);"
    >
      {{ t('skills.no_skills') }}
      <p class="text-xs mt-1">{{ t('skills.no_skills_hint') }}</p>
    </div>

    <div v-else class="flex flex-col gap-3">
      <SkillCard
        v-for="skill in filteredSkills()"
        :key="skill.id"
        :skill="skill"
        :agents="agentsStore.agents"
      />
    </div>

    <!-- Install dialog -->
    <InstallDialog
      v-if="showInstall"
      @close="showInstall = false"
      @installed="skillsStore.fetchSkills()"
    />
  </div>
</template>
