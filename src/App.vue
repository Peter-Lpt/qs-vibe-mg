<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAgentsStore } from "./stores/agents";
import { useSkillsStore } from "./stores/skills";
import { useHistoryStore } from "./stores/history";
import { useAppStore } from "./stores/app";
import { useToast } from "./composables/useToast";
import type { TabId } from "./types";
import AppLayout from "./components/layout/AppLayout.vue";
import TabBar from "./components/layout/TabBar.vue";
import ManageTab from "./components/manage/ManageTab.vue";
import HistoryTab from "./components/history/HistoryTab.vue";
import SettingsPage from "./components/settings/SettingsPage.vue";
import ToastContainer from "./components/common/ToastContainer.vue";

const { locale, t } = useI18n();
const agentsStore = useAgentsStore();
const skillsStore = useSkillsStore();
const historyStore = useHistoryStore();
const appStore = useAppStore();
const toast = useToast();
const appWindow = getCurrentWindow();

const tabs: TabId[] = ["manage", "history"];

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

function handleGlobalKeydown(e: KeyboardEvent) {
  // Ctrl+1-2: tab switch
  if (e.ctrlKey && !e.shiftKey && e.key >= "1" && e.key <= "2") {
    e.preventDefault();
    const idx = Number(e.key) - 1;
    if (idx < tabs.length) {
      appStore.activeTab = tabs[idx];
    }
    return;
  }

  // Ctrl+Z: undo (in history tab)
  if (e.ctrlKey && !e.shiftKey && e.key === "z" && appStore.activeTab === "history") {
    e.preventDefault();
    if (historyStore.canUndo && historyStore.latestUndoableId) {
      historyStore.undoById(historyStore.latestUndoableId);
      skillsStore.fetchSkills();
      historyStore.updateUndoRedoState();
      toast.show(t("history.undo_success"), "success");
    }
    return;
  }

  // Ctrl+Shift+Z: redo (in history tab)
  if (e.ctrlKey && e.shiftKey && e.key === "Z" && appStore.activeTab === "history") {
    e.preventDefault();
    if (historyStore.canRedo && historyStore.latestRedoableId) {
      historyStore.redoById(historyStore.latestRedoableId);
      skillsStore.fetchSkills();
      historyStore.updateUndoRedoState();
      toast.show(t("history.redo_success"), "success");
    }
    return;
  }
}

function isEditableTarget(target: EventTarget | null) {
  const el = target instanceof HTMLElement ? target : null;
  if (!el) return false;
  if (el.isContentEditable) return true;
  return Boolean(el.closest("input, textarea, select, [role='textbox']"));
}

function handleGlobalContextMenu(e: MouseEvent) {
  if (!isEditableTarget(e.target)) {
    e.preventDefault();
  }
}

let unlistenResize: (() => void) | null = null;
let unlistenScaleChange: (() => void) | null = null;

onMounted(async () => {
  appStore.init();
  locale.value = appStore.locale;
  await agentsStore.fetchAgents();
  await skillsStore.fetchSkills();
  await historyStore.fetchHistory();
  historyStore.updateUndoRedoState();
  document.addEventListener("keydown", handleGlobalKeydown);
  document.addEventListener("contextmenu", handleGlobalContextMenu);
  unlistenResize = await appWindow.onResized(() => {
    window.dispatchEvent(new Event("resize"));
  });
  unlistenScaleChange = await appWindow.onScaleChanged(() => {
    window.dispatchEvent(new Event("resize"));
  });
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleGlobalKeydown);
  document.removeEventListener("contextmenu", handleGlobalContextMenu);
  unlistenResize?.();
  unlistenScaleChange?.();
});
</script>

<template>
  <AppLayout>
    <template #tabs>
      <TabBar v-model="appStore.activeTab" />
    </template>

    <KeepAlive>
      <ManageTab v-if="appStore.activeTab === 'manage'" />
      <HistoryTab v-else-if="appStore.activeTab === 'history'" />
    </KeepAlive>
  </AppLayout>

  <SettingsPage v-if="appStore.showSettings" />
  <ToastContainer />
</template>
