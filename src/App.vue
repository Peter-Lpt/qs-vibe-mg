<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAgentsStore } from "./stores/agents";
import { useSkillsStore } from "./stores/skills";
import { useHistoryStore } from "./stores/history";
import { useAppStore } from "./stores/app";
import { useToast } from "./composables/useToast";
import { useSmartViews } from "./composables/useSmartViews";
import { useManageFilters } from "./components/manage/manageFilters";
import type { SmartViewId, Skill } from "./types";
import AppLayout from "./components/layout/AppLayout.vue";
import AppSidebar from "./components/layout/AppSidebar.vue";
import SkillsView from "./components/manage/SkillsView.vue";
import PluginSkillsView from "./components/manage/PluginSkillsView.vue";
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
  appStore.activeView === "history" || appStore.activeView === "settings"
    ? appStore.lastSkillsView
    : appStore.activeView
);
const updateChecks = computed(() => skillsStore.updateChecks);
const filterModel = useManageFilters(skillList, detectedAgents, updateChecks);
const { viewCounts } = useSmartViews(skillList, detectedAgents, filterModel.state);

// ── Plugin skills 计数 ──
const pluginSkillsCount = ref(0);
async function fetchPluginSkillsCount() {
  try {
    const pluginSkills: Skill[] = await skillsStore.fetchPluginSkills();
    pluginSkillsCount.value = pluginSkills.length;
  } catch {
    pluginSkillsCount.value = 0;
  }
}

// 合并视图计数（包含 plugin 计数）
const mergedViewCounts = computed(() => ({
  ...viewCounts.value,
  plugins: pluginSkillsCount.value,
}));

watch(
  () => appStore.locale,
  (newLocale) => {
    locale.value = newLocale;
  }
);

// 当技能列表发生变化时，同步刷新历史记录
// M12：浅 watch 数组引用 + length 即可覆盖 fetch/refresh（整体替换）与
// install push（长度变化）；深比较大数组每次变更开销大且无必要
watch(
  () => [skillsStore.skills, skillsStore.skills.length],
  () => {
    historyStore.fetchHistory();
  }
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
let unlistenTrayFilterErrors: (() => void) | null = null;

// 切换到异常筛选视图
async function switchToErrorFilter() {
  appStore.setActiveView("all");
  filterModel.clearFilters();
  filterModel.issues.value = new Set(["update_error"]);
  // 本会话尚未执行过检查时自动补一次，避免空转出"无结果"且无解释
  if (Object.keys(skillsStore.updateChecks).length === 0) {
    try {
      await skillsStore.checkAllSkillUpdates();
    } catch {
      // 检查失败时保持已筛选的视图，由空态提示兜底
    }
  }
}

onMounted(async () => {
  appStore.init();
  locale.value = appStore.locale;
  await appStore.fetchConfig();
  await skillsStore.loadUpdateChecks();
  // 启动后台静默检查更新（不阻塞 UI；失败静默，托盘/徽章空态由 U12 兜底）
  if (appStore.config?.ui?.auto_check_updates !== false) {
    skillsStore.checkAllSkillUpdates().catch(() => {});
  }
  await agentsStore.fetchAgents();
  await skillsStore.fetchSkills();
  await skillsStore.fetchIssues();
  await fetchPluginSkillsCount();
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
  // 监听系统托盘的「更新异常」菜单点击事件
  unlistenTrayFilterErrors = await appWindow.listen("tray-filter-update-errors", () => {
    switchToErrorFilter();
  });
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleGlobalKeydown);
  document.removeEventListener("contextmenu", handleGlobalContextMenu);
  unlistenResize?.();
  unlistenScaleChange?.();
  unlistenTrayFilterErrors?.();
});
</script>

<template>
  <AppLayout>
    <template #sidebar>
      <AppSidebar
        :active-view="appStore.activeView"
        :counts="mergedViewCounts"
        :filter-model="filterModel"
        :agents="detectedAgents"
        :facet-counts="filterModel.facetCounts.value"
        @select="appStore.setActiveView"
      />
    </template>

    <KeepAlive>
      <PluginSkillsView
        v-if="appStore.activeView === 'plugins'"
        :key="'plugin-skills'"
      />
      <SkillsView
        v-else-if="appStore.activeView !== 'history' && appStore.activeView !== 'settings'"
        :key="'skills'"
        :view="skillsViewId"
        :filter-model="filterModel"
      />
      <HistoryTab v-else-if="appStore.activeView === 'history'" :key="'history'" />
      <SettingsPage v-else :key="'settings'" />
    </KeepAlive>
  </AppLayout>
  <ToastContainer />
</template>
