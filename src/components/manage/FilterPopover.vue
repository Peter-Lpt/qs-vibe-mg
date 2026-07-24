<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { Agent } from "../../types";
import type {
  DomainScope,
  FacetCounts,
  IssueFilter,
  LibraryScope,
} from "./manageFilters";
import type { useManageFilters } from "./manageFilters";

type ManageFilterModel = ReturnType<typeof useManageFilters>;

const props = defineProps<{
  open: boolean;
  filterModel: ManageFilterModel;
  facetCounts: FacetCounts;
  agents: Agent[];
  defaultDomain: DomainScope;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { t } = useI18n();

const ISSUES: IssueFilter[] = ["conflict", "dangling", "duplicate"];
const LIBRARY_SCOPES: LibraryScope[] = ["missing_library", "library_only"];
const DOMAINS: DomainScope[] = ["local", "plugin"];

const agentQuery = ref("");
const filteredAgents = computed(() => {
  const q = agentQuery.value.trim().toLowerCase();
  if (!q) return props.agents;
  return props.agents.filter(
    (agent) =>
      agent.name.toLowerCase().includes(q) || agent.id.toLowerCase().includes(q)
  );
});

function toggleDomain(domain: DomainScope) {
  props.filterModel.domain.value =
    props.filterModel.domain.value === domain ? props.defaultDomain : domain;
}

function toggleMatchMode() {
  props.filterModel.agentMatch.value =
    props.filterModel.agentMatch.value === "any" ? "exclude" : "any";
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && props.open) {
    event.stopPropagation();
    emit("close");
  }
}

watch(
  () => props.open,
  (open) => {
    if (!open) agentQuery.value = "";
  }
);

onMounted(() => document.addEventListener("keydown", handleKeydown));
onUnmounted(() => document.removeEventListener("keydown", handleKeydown));
</script>

<template>
  <div
    v-if="open"
    class="absolute right-0 top-full z-30 mt-1 w-[min(340px,80vw)] rounded-lg border p-3 shadow-lg"
    style="background: var(--c-surface-raised); border-color: var(--c-border);"
    role="dialog"
    :aria-label="t('filter.trigger')"
    @click.stop
    @pointerdown.stop
  >
    <!-- 问题类型 -->
    <div class="mb-3">
      <div class="mb-1.5 text-[10px] font-semibold uppercase tracking-wide" style="color: var(--c-text-tertiary);">
        {{ t("filter.group_issue") }}
      </div>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="issue in ISSUES"
          :key="issue"
          type="button"
          class="filter-chip"
          :class="{ 'filter-chip-active': filterModel.issues.value.has(issue) }"
          :disabled="facetCounts.issues[issue] === 0 && !filterModel.issues.value.has(issue)"
          @click="filterModel.toggleIssue(issue)"
        >
          {{ t(`manage.status_${issue}`) }}
          <span class="filter-chip-count">{{ facetCounts.issues[issue] }}</span>
        </button>
      </div>
    </div>

    <!-- 来源范围 -->
    <div class="mb-3">
      <div class="mb-1.5 text-[10px] font-semibold uppercase tracking-wide" style="color: var(--c-text-tertiary);">
        {{ t("filter.group_library") }}
      </div>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="scope in LIBRARY_SCOPES"
          :key="scope"
          type="button"
          class="filter-chip"
          :class="{ 'filter-chip-active': filterModel.libraryScope.value.has(scope) }"
          :disabled="facetCounts.library[scope] === 0 && !filterModel.libraryScope.value.has(scope)"
          @click="filterModel.toggleLibraryScope(scope)"
        >
          {{ t(scope === "missing_library" ? "manage.quick_filter_missing_lib" : "manage.quick_filter_only_lib") }}
          <span class="filter-chip-count">{{ facetCounts.library[scope] }}</span>
        </button>
      </div>
    </div>

    <!-- 来源类型（domain 覆盖） -->
    <div class="mb-3">
      <div class="mb-1.5 text-[10px] font-semibold uppercase tracking-wide" style="color: var(--c-text-tertiary);">
        {{ t("filter.group_domain") }}
      </div>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="domain in DOMAINS"
          :key="domain"
          type="button"
          class="filter-chip"
          :class="{ 'filter-chip-active': filterModel.domain.value === domain && domain !== defaultDomain }"
          :disabled="domain === defaultDomain && filterModel.domain.value === defaultDomain"
          @click="toggleDomain(domain)"
        >
          {{ t(domain === "local" ? "filter.domain_local" : "filter.domain_plugin") }}
        </button>
      </div>
    </div>

    <!-- Agent 范围 -->
    <div>
      <div class="mb-1.5 flex items-center gap-2 text-[10px] font-semibold uppercase tracking-wide" style="color: var(--c-text-tertiary);">
        {{ t("filter.group_agent") }}
        <button
          v-if="agents.length > 1 && filterModel.agentIds.value.size > 0"
          type="button"
          class="normal-case cursor-pointer"
          style="color: var(--c-primary);"
          @click="toggleMatchMode"
        >
          {{ filterModel.agentMatch.value === "any" ? t("manage.filter_include") : t("manage.filter_exclude") }}
        </button>
      </div>
      <div v-if="agents.length === 0" class="text-[11px]" style="color: var(--c-text-tertiary);">
        {{ t("manage.no_agent_filter") }}
      </div>
      <template v-else>
        <div v-if="agents.length >= 6" class="relative mb-1.5">
          <input
            v-model="agentQuery"
            type="text"
            class="toolbar-control w-full rounded-md py-1.5 pl-2.5 pr-2 text-[11px] outline-none"
            :placeholder="t('skills.search')"
          />
        </div>
        <div class="flex flex-wrap gap-1.5" :class="{ 'max-h-36 overflow-y-auto': agents.length >= 6 }">
          <button
            v-for="agent in filteredAgents"
            :key="agent.id"
            type="button"
            class="filter-chip"
            :class="{ 'filter-chip-active': filterModel.agentIds.value.has(agent.id) }"
            :title="agent.skills_dir"
            @click="filterModel.toggleAgent(agent.id)"
          >
            <span class="h-1.5 w-1.5 rounded-full" style="background: var(--c-success);" />
            {{ agent.name }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
