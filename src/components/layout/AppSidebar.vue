<script setup lang="ts">
import { computed, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import type { Agent, SmartViewId, ViewId } from "../../types";
import { SMART_VIEWS, type SmartViewDef } from "../../composables/useSmartViews";
import type {
  FacetCounts,
  IssueFilter,
  LibraryScope,
  useManageFilters,
} from "../../components/manage/manageFilters";

type ManageFilterModel = ReturnType<typeof useManageFilters>;

const ISSUES: IssueFilter[] = ["conflict", "dangling", "duplicate"];
const LIBRARY_SCOPES: LibraryScope[] = ["missing_library", "library_only"];

const SIDEBAR_MIN = 180;
const SIDEBAR_MAX = 360;

const props = defineProps<{
  activeView: ViewId;
  counts: Record<SmartViewId, number>;
  filterModel: ManageFilterModel;
  agents: Agent[];
  facetCounts: FacetCounts;
}>();

const emit = defineEmits<{
  (e: "select", view: ViewId): void;
}>();

const { t } = useI18n();

// --- sidebar resize ---
const sidebarWidth = ref(208);
const resizing = ref(false);

function onResizeStart(e: MouseEvent) {
  e.preventDefault();
  resizing.value = true;
  document.addEventListener("mousemove", onResizeMove);
  document.addEventListener("mouseup", onResizeEnd);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
}

function onResizeMove(e: MouseEvent) {
  const clamped = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, e.clientX));
  sidebarWidth.value = clamped;
}

function onResizeEnd() {
  resizing.value = false;
  document.removeEventListener("mousemove", onResizeMove);
  document.removeEventListener("mouseup", onResizeEnd);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
}

onUnmounted(() => {
  document.removeEventListener("mousemove", onResizeMove);
  document.removeEventListener("mouseup", onResizeEnd);
});

// --- navigation ---
const libraryViews = computed(() => SMART_VIEWS.filter((v) => v.domain === "local"));
const pluginViews = computed(() => SMART_VIEWS.filter((v) => v.domain === "plugin"));

function isActive(view: SmartViewDef): boolean {
  return props.activeView === view.id;
}

function itemStyle(view: SmartViewDef) {
  if (isActive(view)) {
    return { background: "var(--c-primary-light)", color: "var(--c-primary)" };
  }
  return { color: "var(--c-text-secondary)" };
}

// --- filter section ---
const filterExpanded = ref(false);
const activeFilterCount = computed(() =>
  props.filterModel.issues.value.size +
  props.filterModel.libraryScope.value.size +
  props.filterModel.agentIds.value.size
);
</script>

<style scoped>
.sidebar-chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 10px;
  line-height: 18px;
  cursor: pointer;
  transition: all 0.15s ease;
  background: var(--c-surface-hover);
  color: var(--c-text-secondary);
  border: none;
}

.sidebar-chip:hover:not(:disabled) {
  background: var(--c-border);
}

.sidebar-chip:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.sidebar-chip--active {
  background: var(--c-primary);
  color: #fff;
}

.sidebar-chip--active:hover:not(:disabled) {
  background: var(--c-primary);
  opacity: 0.85;
}

.sidebar-chip--agent .sidebar-chip__dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--c-success);
  flex-shrink: 0;
}

.sidebar-chip--active .sidebar-chip__dot {
  background: rgba(255, 255, 255, 0.7);
}

.sidebar-chip__label {
  white-space: nowrap;
}

.sidebar-chip__count {
  font-size: 9px;
  opacity: 0.6;
}

.sidebar-chip--active .sidebar-chip__count {
  opacity: 0.8;
}
</style>

<template>
  <div
    class="relative flex h-full shrink-0"
    :style="{ width: sidebarWidth + 'px' }"
  >
    <nav
      class="flex h-full flex-1 flex-col gap-0.5 overflow-y-auto border-r px-3 py-4"
      style="background: var(--c-surface); border-color: var(--c-border);"
      aria-label="views"
    >
      <div class="px-2 pb-1.5 text-[10px] font-semibold uppercase tracking-wide" style="color: var(--c-text-tertiary);">
        {{ t("sidebar.skills_library") }}
      </div>

      <button
        v-for="view in libraryViews"
        :key="view.id"
        type="button"
        class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-xs cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
        :style="itemStyle(view)"
        :aria-current="isActive(view) ? 'page' : undefined"
        @click="emit('select', view.id)"
      >
        <component :is="view.icon" :size="14" class="shrink-0" />
        <span class="flex-1 truncate text-left">{{ t(view.labelKey) }}</span>
        <span
          v-if="view.showBadge && counts[view.id] > 0"
          class="min-w-4.5 rounded-full px-1 text-center text-[9px] leading-3.5"
          style="background: var(--c-warning); color: white;"
        >{{ counts[view.id] }}</span>
        <span v-else class="text-[10px]" style="color: var(--c-text-tertiary);">{{ counts[view.id] }}</span>
      </button>

      <div class="px-2 pb-1.5 pt-4 text-[10px] font-semibold uppercase tracking-wide" style="color: var(--c-text-tertiary);">
        {{ t("sidebar.section_source") }}
      </div>

      <button
        v-for="view in pluginViews"
        :key="view.id"
        type="button"
        class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-xs cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
        :style="
          isActive(view)
            ? { background: 'var(--c-plugin-light, rgba(139, 92, 246, 0.15))', color: 'var(--c-plugin, #8b5cf6)' }
            : { color: 'var(--c-text-secondary)' }
        "
        :aria-current="isActive(view) ? 'page' : undefined"
        @click="emit('select', view.id)"
      >
        <component :is="view.icon" :size="14" class="shrink-0" />
        <span class="flex-1 truncate text-left">{{ t(view.labelKey) }}</span>
        <span class="text-[10px]" style="color: var(--c-text-tertiary);">{{ counts[view.id] }}</span>
      </button>

      <!-- 筛选区（可折叠） -->
      <div class="my-2 border-t" style="border-color: var(--c-border-subtle);" />

      <button
        type="button"
        class="flex items-center gap-1.5 px-2 py-1 text-[10px] font-semibold uppercase tracking-wide cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)] rounded-md"
        style="color: var(--c-text-tertiary);"
        @click="filterExpanded = !filterExpanded"
      >
        <ChevronRight
          :size="11"
          class="transition-transform"
          :style="{ transform: filterExpanded ? 'rotate(90deg)' : 'rotate(0deg)' }"
        />
        {{ t("sidebar.filters") }}
        <span
          v-if="activeFilterCount > 0"
          class="ml-auto rounded-full px-1.5 text-[9px] leading-3.5"
          style="background: var(--c-primary); color: white;"
        >{{ activeFilterCount }}</span>
      </button>

      <template v-if="filterExpanded">
        <div class="px-2 pt-1 pb-1.5 space-y-1.5">
          <!-- 状态筛选（异常+范围） -->
          <div class="flex flex-wrap gap-1">
            <button
              v-for="issue in ISSUES"
              :key="issue"
              type="button"
              class="sidebar-chip"
              :class="{ 'sidebar-chip--active': filterModel.issues.value.has(issue) }"
              :disabled="facetCounts.issues[issue] === 0 && !filterModel.issues.value.has(issue)"
              @click="filterModel.toggleIssue(issue)"
            >
              <span class="sidebar-chip__label">{{ t(`manage.status_${issue}`) }}</span>
              <span class="sidebar-chip__count">{{ facetCounts.issues[issue] }}</span>
            </button>
            <button
              v-for="scope in LIBRARY_SCOPES"
              :key="scope"
              type="button"
              class="sidebar-chip"
              :class="{ 'sidebar-chip--active': filterModel.libraryScope.value.has(scope) }"
              :disabled="facetCounts.library[scope] === 0 && !filterModel.libraryScope.value.has(scope)"
              @click="filterModel.toggleLibraryScope(scope)"
            >
              <span class="sidebar-chip__label">{{ t(scope === 'missing_library' ? 'manage.quick_filter_missing_lib' : 'manage.quick_filter_only_lib') }}</span>
              <span class="sidebar-chip__count">{{ facetCounts.library[scope] }}</span>
            </button>
          </div>

          <!-- Agent 筛选 -->
          <div class="flex flex-wrap gap-1">
            <button
              v-for="agent in agents"
              :key="agent.id"
              type="button"
              class="sidebar-chip sidebar-chip--agent"
              :class="{ 'sidebar-chip--active': filterModel.agentIds.value.has(agent.id) }"
              @click="filterModel.toggleAgent(agent.id)"
            >
              <span class="sidebar-chip__dot" />
              <span class="sidebar-chip__label">{{ agent.name }}</span>
            </button>
            <span v-if="agents.length === 0" class="text-[10px] px-1" style="color: var(--c-text-tertiary);">
              {{ t("manage.no_agent_filter") }}
            </span>
          </div>
        </div>
      </template>

      <div class="flex-1" />

      <div class="my-2 border-t" style="border-color: var(--c-border-subtle);" />

      <button
        type="button"
        class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-xs cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
        :style="
          activeView === 'history'
            ? { background: 'var(--c-primary-light)', color: 'var(--c-primary)' }
            : { color: 'var(--c-text-secondary)' }
        "
        :aria-current="activeView === 'history' ? 'page' : undefined"
        @click="emit('select', 'history')"
      >
        <History :size="14" class="shrink-0" />
        <span class="flex-1 truncate text-left">{{ t("tabs.history") }}</span>
      </button>

      <button
        type="button"
        class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-xs cursor-pointer transition-colors hover:bg-[var(--c-surface-hover)]"
        :style="
          activeView === 'settings'
            ? { background: 'var(--c-primary-light)', color: 'var(--c-primary)' }
            : { color: 'var(--c-text-secondary)' }
        "
        :aria-current="activeView === 'settings' ? 'page' : undefined"
        @click="emit('select', 'settings')"
      >
        <Settings :size="14" class="shrink-0" />
        <span class="flex-1 truncate text-left">{{ t("app.settings") }}</span>
      </button>
    </nav>

    <!-- 拖拽手柄 -->
    <div
      class="absolute top-0 right-0 h-full w-1.5 cursor-col-resize z-10 group"
      :class="{ 'bg-[var(--c-primary)] opacity-30': resizing }"
      @mousedown="onResizeStart"
    >
      <div
        class="absolute inset-y-0 right-0 w-px transition-colors group-hover:bg-[var(--c-primary)] group-hover:opacity-40"
      />
    </div>
  </div>
</template>
