<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAgentsStore } from "./stores/agents";
import { useSkillsStore } from "./stores/skills";
import { useHistoryStore } from "./stores/history";
import { useAppStore } from "./stores/app";
import { useToast } from "./composables/useToast";
import { useSmartViews, viewToFilterPreset } from "./composables/useSmartViews";
import { useManageFilters } from "./components/manage/manageFilters";
import type { SmartViewId } from "./types";
import AppLayout from "./components/layout/AppLayout.vue";
import AppSidebar from "./components/layout/AppSidebar.vue";
import SkillsView from "./components/manage/SkillsView.vue";
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

// ── 筛选模型与视图计数（App 层持有，侧边栏与 SkillsView 共用同一数据源） ──
const skillList = computed(() => skillsStore.skills);
const detectedAgents = computed(() => agentsStore.agents.filter((a) => a.detected && a.enabled));
const skillsViewId = computed<SmartViewId>(() =>
  appStore.activeView === "history" ? appStore.lastSkillsView : appStore.activeView
);
const defaultDomain = computed(() => viewToFilterPreset(skillsViewId.value).domain);
const filterModel = useManageFilters(skillList, detectedAgents, defaultDomain);
const { viewCounts } = useSmartViews(skillList, detectedAgents, filterModel.state);

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
  // Ctrl/Cmd+1: 返回技能库（恢复上次子视图）
  if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === "1") {
    e.preventDefault();
    appStore.setActiveView(appStore.lastSkillsView);
    return;
  }
  // Ctrl/Cmd+2: 历史记录
  if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key === "2") {
    e.preventDefault();
    appStore.setActiveView("history");
    return;
  }

  // Ctrl+Z: undo（仅历史视图且非可编辑目标）
  if (
    (e.ctrlKey || e.metaKey) &&
    !e.shiftKey &&
    e.key === "z" &&
    appStore.activeView === "history" &&
    !isEditableTarget(e.target)
  ) {
    e.preventDefault();
    if (historyStore.canUndo && historyStore.latestUndoableId) {
      historyStore.undoById(historyStore.latestUndoableId);
      skillsStore.fetchSkills();
      historyStore.updateUndoRedoState();
      toast.show(t("history.undo_success"), "success");
    }
    return;
  }

  // Ctrl+Shift+Z: redo（仅历史视图且非可编辑目标）
  if (
    (e.ctrlKey || e.metaKey) &&
    e.shiftKey &&
    e.key === "Z" &&
    appStore.activeView === "history" &&
    !isEditableTarget(e.target)
  ) {
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
    <template #sidebar>
      <AppSidebar
        :active-view="appStore.activeView"
        :counts="viewCounts"
        @select="appStore.setActiveView"
        @open-settings="appStore.showSettings = true"
      />
    </template>

    <KeepAlive>
      <SkillsView
        v-if="appStore.activeView !== 'history'"
        :key="'skills'"
        :view="skillsViewId"
        :filter-model="filterModel"
      />
      <HistoryTab v-else :key="'history'" />
    </KeepAlive>
  </AppLayout>

  <SettingsPage v-if="appStore.showSettings" />
  <ToastContainer />
</template>
