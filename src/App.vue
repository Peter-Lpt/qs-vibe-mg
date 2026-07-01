<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useAgentsStore } from "./stores/agents";
import { useSkillsStore } from "./stores/skills";
import { useHistoryStore } from "./stores/history";
import { useAppStore } from "./stores/app";
import AppLayout from "./components/layout/AppLayout.vue";
import TabBar from "./components/layout/TabBar.vue";
import CLITab from "./components/cli/CLITab.vue";
import SkillList from "./components/skills/SkillList.vue";
import DashboardTab from "./components/dashboard/DashboardTab.vue";
import SymlinkTab from "./components/symlink/SymlinkTab.vue";
import HistoryTab from "./components/history/HistoryTab.vue";
import SettingsPage from "./components/settings/SettingsPage.vue";

const { locale } = useI18n();
const agentsStore = useAgentsStore();
const skillsStore = useSkillsStore();
const historyStore = useHistoryStore();
const appStore = useAppStore();

watch(
  () => appStore.locale,
  (newLocale) => {
    locale.value = newLocale;
  }
);

// 当技能列表发生变化时，同步刷新历史记录
watch(
  () => skillsStore.skills,
  () => {
    historyStore.fetchHistory();
  },
  { deep: true }
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
    <TabBar v-model="appStore.activeTab" />

    <CLITab v-if="appStore.activeTab === 'cli'" />
    <SkillList v-else-if="appStore.activeTab === 'skills'" />
    <DashboardTab v-else-if="appStore.activeTab === 'dashboard'" />
    <SymlinkTab v-else-if="appStore.activeTab === 'symlink'" />
    <HistoryTab v-else-if="appStore.activeTab === 'history'" />
  </AppLayout>

  <SettingsPage v-if="appStore.showSettings" />
</template>