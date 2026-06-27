<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "./stores/agents";
import { useSkillsStore } from "./stores/skills";
import { useHistoryStore } from "./stores/history";
import { useAppStore } from "./stores/app";
import AppLayout from "./components/layout/AppLayout.vue";
import SkillLibrary from "./components/skills/SkillLibrary.vue";
import AgentPanel from "./components/agents/AgentPanel.vue";
import HistoryBar from "./components/history/HistoryBar.vue";
import SettingsPage from "./components/settings/SettingsPage.vue";

const { locale } = useI18n();
const agentsStore = useAgentsStore();
const skillsStore = useSkillsStore();
const historyStore = useHistoryStore();
const appStore = useAppStore();

// Sync locale from app store to i18n
watch(
  () => appStore.locale,
  (newLocale) => {
    locale.value = newLocale;
  }
);

onMounted(async () => {
  appStore.init();
  locale.value = appStore.locale;
  await agentsStore.fetchAgents();
  await skillsStore.fetchSkills();
  await historyStore.fetchHistory();
  historyStore.updateUndoRedoState();
});
</script>

<template>
  <AppLayout>
    <template #left>
      <SkillLibrary />
    </template>
    <template #right>
      <AgentPanel />
    </template>
    <template #bottom>
      <HistoryBar />
    </template>
  </AppLayout>

  <SettingsPage v-if="appStore.showSettings" />
</template>
