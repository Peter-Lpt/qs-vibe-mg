<script setup lang="ts">
import { computed, ref } from "vue";
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

const props = defineProps<{
  activeView: ViewId;
  counts: Record<SmartViewId, number>;
  filterModel: ManageFilterModel;
  agents: Agent[];
  facetCounts: FacetCounts;
}>();

const emit = defineEmits<{
  (e: "select", view: ViewId): void;
  (e: "open-settings"): void;
}>();

const { t } = useI18n();

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

// 折叠状态
const filterExpanded = ref(false);
const activeFilterCount = computed(() =>
  props.filterModel.issues.value.size +
  props.filterModel.libraryScope.value.size +
  props.filterModel.agentIds.value.size
);
</script>

<template>
  <nav
    class="flex h-full w-52 shrink-0 flex-col gap-0.5 overflow-y-auto border-r px-3 py-4"
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
      <!-- 异常 -->
      <div class="px-2 pt-1.5 pb-1 text-[10px]" style="color: var(--c-text-tertiary);">
        {{ t("sidebar.issues") }}
      </div>
      <div class="flex flex-wrap gap-1 px-2 pb-1.5">
        <button
          v-for="issue in ISSUES"
          :key="issue"
          type="button"
          class="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] cursor-pointer transition-colors border"
          :style="
            filterModel.issues.value.has(issue)
              ? { background: 'var(--c-primary-light)', borderColor: 'var(--c-primary)', color: 'var(--c-primary)' }
              : { borderColor: 'var(--c-border)', color: 'var(--c-text-secondary)' }
          "
          :disabled="facetCounts.issues[issue] === 0 && !filterModel.issues.value.has(issue)"
          @click="filterModel.toggleIssue(issue)"
        >
          {{ t(`manage.status_${issue}`) }}
          <span class="text-[9px] opacity-70">{{ facetCounts.issues[issue] }}</span>
        </button>
      </div>

      <!-- 范围 -->
      <div class="px-2 pb-1 text-[10px]" style="color: var(--c-text-tertiary);">
        {{ t("sidebar.scope") }}
      </div>
      <div class="flex flex-wrap gap-1 px-2 pb-1.5">
        <button
          v-for="scope in LIBRARY_SCOPES"
          :key="scope"
          type="button"
          class="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] cursor-pointer transition-colors border"
          :style="
            filterModel.libraryScope.value.has(scope)
              ? { background: 'var(--c-primary-light)', borderColor: 'var(--c-primary)', color: 'var(--c-primary)' }
              : { borderColor: 'var(--c-border)', color: 'var(--c-text-secondary)' }
          "
          :disabled="facetCounts.library[scope] === 0 && !filterModel.libraryScope.value.has(scope)"
          @click="filterModel.toggleLibraryScope(scope)"
        >
          {{ t(scope === 'missing_library' ? 'manage.quick_filter_missing_lib' : 'manage.quick_filter_only_lib') }}
          <span class="text-[9px] opacity-70">{{ facetCounts.library[scope] }}</span>
        </button>
      </div>

      <!-- Agent -->
      <div class="px-2 pb-1 text-[10px]" style="color: var(--c-text-tertiary);">
        {{ t("sidebar.agents") }}
      </div>
      <div class="flex flex-wrap gap-1 px-2 pb-1.5">
        <button
          v-for="agent in agents"
          :key="agent.id"
          type="button"
          class="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] cursor-pointer transition-colors border"
          :style="
            filterModel.agentIds.value.has(agent.id)
              ? { background: 'var(--c-primary-light)', borderColor: 'var(--c-primary)', color: 'var(--c-primary)' }
              : { borderColor: 'var(--c-border)', color: 'var(--c-text-secondary)' }
          "
          @click="filterModel.toggleAgent(agent.id)"
        >
          <span class="h-1.5 w-1.5 rounded-full" style="background: var(--c-success);" />
          {{ agent.name }}
        </button>
        <div v-if="agents.length === 0" class="text-[10px] px-1" style="color: var(--c-text-tertiary);">
          {{ t("manage.no_agent_filter") }}
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
      style="color: var(--c-text-secondary);"
      @click="emit('open-settings')"
    >
      <Settings :size="14" class="shrink-0" />
      <span class="flex-1 truncate text-left">{{ t("app.settings") }}</span>
    </button>
  </nav>
</template>
