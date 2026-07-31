<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useToast } from "../../composables/useToast";
import SkeletonCard from "../common/SkeletonCard.vue";

const { t } = useI18n();
const skillsStore = useSkillsStore();
const toast = useToast();

const pluginSkills = ref<typeof skillsStore.skills>([]);
const loading = ref(false);
const searchQuery = ref("");
const checkingUpdates = ref(false);
const updateResults = ref<Record<string, { available: boolean; error?: string }>>({});
const updatingMarketplace = ref<string | null>(null);

// 获取插件类型标签（Claude / Codex / 其他）
function getPluginTypeLabel(skill: typeof pluginSkills.value[0]): string {
  const source = skill.sources.find(
    (s) => s.from.startsWith("claude-plugin:") || s.from.startsWith("codex-plugin:")
  );
  if (!source) return "Plugin";
  if (source.from.startsWith("claude-plugin:")) return "Claude";
  if (source.from.startsWith("codex-plugin:")) return "Codex";
  return "Plugin";
}

// 检查 skill 是否已认领
function isAdopted(skill: typeof pluginSkills.value[0]): boolean {
  return skill.adopted === true;
}

// 按 plugin_source 分组，显示格式：[Type] plugin_name
interface PluginGroup {
  key: string;
  label: string;
  skills: typeof pluginSkills.value;
}

const pluginGroups = computed<PluginGroup[]>(() => {
  const groups = new Map<string, PluginGroup>();
  for (const skill of filteredSkills.value) {
    const pluginName = skill.plugin_source || "unknown";
    const pluginType = getPluginTypeLabel(skill);
    const key = `${pluginType}:${pluginName}`;
    const label = `[${pluginType}] ${pluginName}`;
    if (!groups.has(key)) {
      groups.set(key, { key, label, skills: [] });
    }
    groups.get(key)!.skills.push(skill);
  }
  return Array.from(groups.values());
});

const filteredSkills = computed(() => {
  if (!searchQuery.value.trim()) return pluginSkills.value;
  const query = searchQuery.value.trim().toLowerCase();
  return pluginSkills.value.filter(
    (s) =>
      s.name.toLowerCase().includes(query) ||
      s.id.toLowerCase().includes(query) ||
      s.description.toLowerCase().includes(query)
  );
});

const expandedGroups = ref<Set<string>>(new Set());

function toggleGroup(source: string) {
  const next = new Set(expandedGroups.value);
  if (next.has(source)) next.delete(source);
  else next.add(source);
  expandedGroups.value = next;
}

function isGroupExpanded(source: string): boolean {
  return expandedGroups.value.has(source);
}

async function fetchPluginSkills() {
  if (loading.value) return;
  loading.value = true;
  try {
    pluginSkills.value = await skillsStore.fetchPluginSkills();
    // 默认展开所有分组
    expandedGroups.value = new Set(pluginGroups.value.map((g) => g.key));
    // 自动检查已认领 plugin 的更新（使用 TTL 缓存）
    await autoCheckUpdates();
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    loading.value = false;
  }
}

// 自动检查更新（使用 TTL 缓存，进入页面时调用）
async function autoCheckUpdates() {
  const adoptedSkills = pluginSkills.value.filter((s) => s.adopted === true);
  if (adoptedSkills.length === 0) return;
  try {
    const results = await skillsStore.checkPluginUpdates(adoptedSkills.map((s) => s.id));
    updateResults.value = results;
  } catch {
    // 静默失败，不打扰用户
  }
}

async function handleAdopt(skillId: string) {
  try {
    await skillsStore.adoptPluginSkill(skillId);
    toast.show(t("plugins.adopt_success"), "success");
    // 从列表中移除已认领的 skill
    pluginSkills.value = pluginSkills.value.filter((s) => s.id !== skillId);
    // 清理空分组
    const nonEmptyGroups = new Set(pluginGroups.value.map((g) => g.key));
    expandedGroups.value = new Set([...expandedGroups.value].filter((k) => nonEmptyGroups.has(k)));
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

// 手动检查已认领的 plugin skills 是否有更新（强制刷新，忽略 TTL）
async function checkAllPluginUpdates() {
  checkingUpdates.value = true;
  try {
    const adoptedSkills = pluginSkills.value.filter((s) => s.adopted === true);
    const results: Record<string, { available: boolean; error?: string }> = {};
    for (const skill of adoptedSkills) {
      try {
        const check = await skillsStore.checkSkillUpdate(skill.id, true); // force = true
        results[skill.id] = { available: check.available, error: check.error };
      } catch (e) {
        results[skill.id] = { available: false, error: String(e) };
      }
    }
    updateResults.value = results;
    const availableCount = Object.values(results).filter((r) => r.available).length;
    if (availableCount > 0) {
      toast.show(t("plugins.updates_found", { count: availableCount }), "info");
    } else {
      toast.show(t("plugins.no_updates_found"), "success");
    }
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    checkingUpdates.value = false;
  }
}

// 获取 marketplace 名称
function getMarketplaceName(skill: typeof pluginSkills.value[0]): string | null {
  const source = skill.sources.find(
    (s) => s.from.startsWith("claude-plugin:") || s.from.startsWith("codex-plugin:")
  );
  if (!source) return null;
  if (source.from.startsWith("claude-plugin:")) return source.from.replace("claude-plugin:", "");
  if (source.from.startsWith("codex-plugin:")) return source.from.replace("codex-plugin:", "");
  return null;
}

// 更新同一 marketplace 的所有 plugin skills
async function handleUpdateMarketplace(marketplace: string) {
  updatingMarketplace.value = marketplace;
  try {
    const updatedIds = await skillsStore.updatePluginSkillsFromMarketplace(marketplace);
    toast.show(t("plugins.marketplace_updated", { count: updatedIds.length, marketplace }), "success");
    // 清除已更新的 skill 的更新状态
    const newResults = { ...updateResults.value };
    for (const id of updatedIds) {
      delete newResults[id];
    }
    updateResults.value = newResults;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  } finally {
    updatingMarketplace.value = null;
  }
}

onMounted(() => {
  fetchPluginSkills();
});
</script>

<template>
  <div class="space-y-4">
    <!-- 统计概览 -->
    <section class="workspace-panel !p-3">
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2">
          <Puzzle :size="16" style="color: var(--c-plugin, #8b5cf6);" />
          <h2 class="text-sm font-semibold" style="color: var(--c-text-strong);">
            {{ t("plugins.view_title") }}
          </h2>
        </div>
        <span
          class="rounded-full px-2 py-0.5 text-xs"
          style="background: var(--c-plugin-light, rgba(139, 92, 246, 0.15)); color: var(--c-plugin, #8b5cf6);"
        >
          {{ filteredSkills.length }}
        </span>
        <span class="text-[11px]" style="color: var(--c-text-secondary);">
          {{ t("plugins.view_hint") }}
        </span>
        <div class="ml-auto flex items-center gap-1">
          <button
            class="action-toolbar-icon disabled:opacity-50 disabled:cursor-not-allowed"
            :title="t('plugins.check_all_updates')"
            :disabled="loading || checkingUpdates"
            @click="checkAllPluginUpdates"
          >
            <CloudDownload :size="15" :class="{ 'animate-spin': checkingUpdates }" />
          </button>
          <button
            class="action-toolbar-icon disabled:opacity-50 disabled:cursor-not-allowed"
            :title="t('manage.refresh')"
            :disabled="loading"
            @click="fetchPluginSkills"
          >
            <RefreshCw :size="15" :class="{ 'animate-spin': loading }" />
          </button>
        </div>
      </div>
    </section>

    <!-- 搜索 -->
    <section class="workspace-panel !p-3">
      <div class="relative min-w-0">
        <Search
          :size="14"
          class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2"
          style="color: var(--c-text-secondary);"
        />
        <input
          v-model="searchQuery"
          :placeholder="t('skills.search')"
          class="toolbar-control w-full rounded-md py-2 pl-9 pr-9 text-xs outline-none transition-colors"
        />
        <button
          v-if="searchQuery"
          type="button"
          class="absolute right-2 top-1/2 inline-flex -translate-y-1/2 items-center justify-center rounded-full p-1"
          style="color: var(--c-text-secondary);"
          @click="searchQuery = ''"
        >
          <X :size="13" />
        </button>
      </div>
    </section>

    <!-- Loading -->
    <div v-if="loading" class="space-y-3">
      <SkeletonCard v-for="i in 3" :key="i" />
    </div>

    <!-- Empty -->
    <section v-else-if="filteredSkills.length === 0" class="manage-empty-state">
      <div class="manage-empty-icon">
        <Puzzle :size="28" style="color: var(--c-plugin, #8b5cf6);" />
      </div>
      <h3>{{ t("plugins.empty") }}</h3>
      <p>{{ t("plugins.view_hint") }}</p>
    </section>

    <!-- Plugin Groups -->
    <template v-else>
      <div v-for="group in pluginGroups" :key="group.key" class="plugin-group">
        <div
          class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 select-none hover:bg-[var(--c-surface-hover)]"
          @click="toggleGroup(group.key)"
        >
          <ChevronRight
            :size="13"
            class="transition-transform"
            :style="{
              color: 'var(--c-plugin, #8b5cf6)',
              transform: isGroupExpanded(group.key) ? 'rotate(90deg)' : 'rotate(0deg)',
            }"
          />
          <Puzzle :size="13" style="color: var(--c-plugin, #8b5cf6);" />
          <span class="text-[11px] font-semibold tracking-wide" style="color: var(--c-plugin, #8b5cf6);">
            {{ group.label }}
          </span>
          <span
            class="rounded-full px-1.5 text-[9px]"
            style="background: var(--c-plugin-light, rgba(139, 92, 246, 0.15)); color: var(--c-plugin, #8b5cf6);"
          >{{ group.skills.length }}</span>
        </div>
        <div v-if="isGroupExpanded(group.key)" class="mt-1.5 space-y-2">
          <div
            v-for="skill in group.skills"
            :key="skill.id"
            class="plugin-skill-card"
          >
            <div class="flex items-center gap-3 px-3.5 py-3">
              <Puzzle :size="14" style="color: var(--c-plugin, #8b5cf6);" />
              <span class="text-sm font-semibold truncate flex-1" style="color: var(--c-text-strong);">
                {{ skill.name || skill.id }}
              </span>
              <!-- Plugin 启用/禁用状态 -->
              <span
                v-if="skill.plugin_enabled === false"
                class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0 flex items-center gap-1"
                style="background: var(--c-danger-light, rgba(239, 68, 68, 0.15)); color: var(--c-danger, #ef4444);"
              >
                <XCircle :size="10" />
                {{ t("plugins.disabled") }}
              </span>
              <!-- 已认领状态 -->
              <span
                v-if="isAdopted(skill)"
                class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
                style="background: var(--c-success-light, rgba(34, 197, 94, 0.15)); color: var(--c-success);"
              >
                {{ t("plugins.adopted") }}
              </span>
              <span
                v-else
                class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
                style="background: var(--c-plugin-light, rgba(139, 92, 246, 0.15)); color: var(--c-plugin, #8b5cf6);"
              >
                {{ t("manage.status_plugin") }}
              </span>
              <!-- 更新可用状态 -->
              <span
                v-if="isAdopted(skill) && updateResults[skill.id]?.available"
                class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0 flex items-center gap-1"
                style="background: var(--c-warning-light, rgba(234, 179, 8, 0.15)); color: var(--c-warning, #eab308);"
              >
                <CloudDownload :size="10" />
                {{ t("plugins.update_available") }}
              </span>
              <button
                v-if="!isAdopted(skill)"
                class="h-6 px-2 flex items-center justify-center rounded cursor-pointer transition-colors text-[10px] font-medium"
                style="background: var(--c-plugin-light, rgba(139, 92, 246, 0.15)); color: var(--c-plugin, #8b5cf6);"
                :title="t('plugins.adopt_to_library')"
                @click.stop="handleAdopt(skill.id)"
              >
                <DownloadCloud :size="12" class="mr-1" />
                {{ t("plugins.adopt") }}
              </button>
              <!-- 更新按钮（已认领且有更新可用时显示） -->
              <button
                v-if="isAdopted(skill) && updateResults[skill.id]?.available && getMarketplaceName(skill)"
                class="h-6 px-2 flex items-center justify-center rounded cursor-pointer transition-colors text-[10px] font-medium"
                style="background: var(--c-warning-light, rgba(234, 179, 8, 0.15)); color: var(--c-warning, #eab308);"
                :title="t('plugins.update_marketplace')"
                :disabled="updatingMarketplace === getMarketplaceName(skill)"
                @click.stop="handleUpdateMarketplace(getMarketplaceName(skill)!)"
              >
                <RefreshCw :size="12" class="mr-1" :class="{ 'animate-spin': updatingMarketplace === getMarketplaceName(skill) }" />
                {{ t("plugins.update") }}
              </button>
            </div>
            <div v-if="skill.description" class="px-3.5 pb-3 text-[11px] line-clamp-2" style="color: var(--c-text-secondary);">
              {{ skill.description }}
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.plugin-skill-card {
  border-radius: 8px;
  border: 1px solid var(--c-plugin, #8b5cf6);
  background: var(--c-plugin-light, rgba(139, 92, 246, 0.05));
  transition: all 0.15s ease;
}

.plugin-skill-card:hover {
  box-shadow: 0 2px 8px rgba(139, 92, 246, 0.1);
}
</style>
