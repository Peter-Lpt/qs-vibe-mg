<script setup lang="ts">
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import SkillCard from "./SkillCard.vue";

const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
</script>

<template>
  <div>
    <h2 class="text-base font-semibold mb-3" style="color: var(--c-text);">
      Skills
      <span class="text-sm font-normal ml-1" style="color: var(--c-text-secondary);">
        ({{ skillsStore.skills.length }})
      </span>
    </h2>

    <div v-if="skillsStore.loading" class="text-sm" style="color: var(--c-text-secondary);">
      Loading...
    </div>

    <div v-else-if="skillsStore.error" class="text-sm" style="color: var(--c-danger);">
      {{ skillsStore.error }}
    </div>

    <div v-else-if="skillsStore.skills.length === 0" class="text-sm py-8 text-center" style="color: var(--c-text-secondary);">
      No skills found in any directory
    </div>

    <div v-else class="flex flex-col gap-3">
      <SkillCard
        v-for="skill in skillsStore.skills"
        :key="skill.id"
        :skill="skill"
        :agents="agentsStore.agents"
      />
    </div>
  </div>
</template>
