<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSkillsStore } from "../../stores/skills";
import { useAgentsStore } from "../../stores/agents";
import { useToast } from "../../composables/useToast";
import type { Skill, Agent, SkillSource } from "../../types";
import ConfirmDialog from "../common/ConfirmDialog.vue";

const props = defineProps<{
  skill: Skill;
  agents: Agent[];
  selected?: boolean;
}>();

const emit = defineEmits<{
  toggleSelect: [];
}>();

const { t } = useI18n();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();
const toast = useToast();

const expanded = ref(false);
const previewContent = ref("");
const previewLoading = ref(false);
const showPreview = ref(false);
const showDeleteConfirm = ref(false);
const showLinkMenu = ref(false);
const showUnlinkMenu = ref(false);
const resolvingConflict = ref<string | null>(null);

// 获取 vibe-lib source
const vibeSource = computed(() =>
  props.skill.sources.find((s) => s.from === "vibe-lib")
);

// 获取非 vibe-lib 的 sources
const agentSources = computed(() =>
  props.skill.sources.filter((s) => s.from !== "vibe-lib")
);

// 已通过 symlink 链接的 agent id 集合
const symlinkedAgentIds = computed(() =>
  new Set(
    props.skill.sources
      .filter((s) => s.from !== "vibe-lib" && s.is_symlink)
      .map((s) => s.from)
  )
);

// 可以建立链接的 agent（没有任何 source 的）
const linkableAgents = computed(() =>
  agentsStore.agents.filter(
    (a) => a.detected && !agentSources.value.some((s) => s.from === a.id)
  )
);

// 可以取消链接的 agent（已有 symlink 的）
const unlinkableAgents = computed(() =>
  agentsStore.agents.filter(
    (a) => a.detected && symlinkedAgentIds.value.has(a.id)
  )
);

// 需要重新链接的 agent（symlink 指向非 vibe-lib 位置的）
const relinkableAgents = computed(() =>
  agentSources.value.filter((s) => {
    if (!s.is_symlink || !s.symlink_target) return false;
    // 检查 symlink_target 是否指向 vibe-lib
    const vibeDir = vibeSource.value?.path;
    if (!vibeDir) return false;
    return !s.symlink_target.includes(vibeDir);
  })
);

// 获取 agent 名称
function getAgentName(agentId: string): string {
  if (agentId === "vibe-lib") return "Vibe 技能库";
  const agent = props.agents.find((a) => a.id === agentId);
  return agent?.name || agentId;
}

// 获取 agent 状态标签
function getAgentStatus(source: SkillSource): {
  icon: string;
  label: string;
  color: string;
} {
  if (source.from === "vibe-lib") {
    return { icon: "📦", label: t("manage.status_origin"), color: "var(--c-success)" };
  }

  if (!source.is_symlink) {
    // 独立副本
    if (vibeSource.value) {
      // 技能库有此 skill，检查内容是否一致
      if (source.content_hash === vibeSource.value.content_hash) {
        return { icon: "●", label: t("manage.status_independent_same"), color: "var(--c-text-secondary)" };
      }
      return { icon: "⚠", label: t("manage.status_independent_conflict"), color: "var(--c-warning)" };
    }
    return { icon: "●", label: t("manage.status_independent"), color: "var(--c-text-secondary)" };
  }

  // 是 symlink
  if (source.symlink_target) {
    // 检查是否指向 vibe-lib
    if (vibeSource.value?.path && source.symlink_target.includes(vibeSource.value.path)) {
      return { icon: "🔗", label: t("manage.status_synced"), color: "var(--c-primary)" };
    }
    // 指向其他位置
    return { icon: "🔗", label: t("manage.status_linked_elsewhere"), color: "var(--c-warning)" };
  }

  // 断链
  return { icon: "❌", label: t("manage.status_dangling"), color: "var(--c-danger)" };
}

async function toggleExpand() {
  expanded.value = !expanded.value;
}

async function togglePreview() {
  showPreview.value = !showPreview.value;
  if (showPreview.value && !previewContent.value) {
    previewLoading.value = true;
    try {
      previewContent.value = await skillsStore.previewSkill(props.skill.id);
    } catch {
      previewContent.value = "";
    } finally {
      previewLoading.value = false;
    }
  }
}

async function handleLink(agentId: string) {
  try {
    await skillsStore.createLink(props.skill.id, agentId);
    toast.show(t("skills.linked", { agent: getAgentName(agentId) }), "success");
    showLinkMenu.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleUnlink(agentId: string) {
  try {
    await skillsStore.removeLink(props.skill.id, agentId);
    toast.show(t("skills.unlinked", { agent: getAgentName(agentId) }), "success");
    showUnlinkMenu.value = false;
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleSyncToVibe(agentId: string) {
  resolvingConflict.value = agentId;
  try {
    await skillsStore.syncToVibe(props.skill.id, agentId);
    toast.show(t("manage.synced_to_vibe", { agent: getAgentName(agentId) }), "success");
  } catch (e: unknown) {
    const err = String(e);
    if (err.includes("Conflict")) {
      // 冲突，需要用户确认
      toast.show(t("manage.conflict_need_resolve"), "info");
    } else {
      toast.show(err, "error");
    }
  } finally {
    resolvingConflict.value = null;
  }
}

async function handleRelink(agentId: string) {
  try {
    await skillsStore.relink(props.skill.id, agentId);
    toast.show(t("manage.relinked", { agent: getAgentName(agentId) }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleRemoveDangling(source: SkillSource) {
  try {
    await skillsStore.removeLink(props.skill.id, source.from);
    toast.show(t("manage.dangling_removed", { agent: getAgentName(source.from) }), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

async function handleDelete() {
  try {
    await skillsStore.deleteSkill(props.skill.id);
    showDeleteConfirm.value = false;
    toast.show(t("skills.delete"), "success");
  } catch (e: unknown) {
    toast.show(String(e), "error");
  }
}

// 获取链接数/总数
function getSourceCount() {
  const linked = symlinkedAgentIds.value.size;
  const total = agentsStore.agents.filter((a) => a.detected).length;
  return `${linked}/${total}`;
}
</script>

<template>
  <div
    class="rounded-lg border transition-all"
    :style="{
      background: 'var(--c-surface)',
      borderColor: skill.has_conflict
        ? 'var(--c-warning)'
        : skill.has_dangling
          ? 'var(--c-danger)'
          : skill.is_duplicate
            ? 'var(--c-info)'
            : expanded
              ? 'var(--c-primary)'
              : 'var(--c-border)',
    }"
  >
    <!-- Collapsed header -->
    <div
      class="flex items-center gap-3 px-3 py-2.5 cursor-pointer select-none"
      @click="toggleExpand"
    >
      <!-- Checkbox -->
      <input
        type="checkbox"
        :checked="selected"
        class="w-4 h-4 rounded cursor-pointer shrink-0"
        style="accent-color: var(--c-primary);"
        @click.stop="emit('toggleSelect')"
      />

      <!-- Expand arrow -->
      <span
        class="w-4 text-center text-xs shrink-0 transition-transform"
        :style="{ color: 'var(--c-text-secondary)', transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)' }"
      >▶</span>

      <!-- Status icons -->
      <span v-if="skill.has_conflict" class="shrink-0" style="color: var(--c-warning);">⚠</span>
      <span v-else-if="skill.has_dangling" class="shrink-0" style="color: var(--c-danger);">❌</span>
      <span v-else-if="skill.is_duplicate" class="shrink-0" style="color: var(--c-info);">📋</span>

      <!-- Skill name -->
      <span class="text-sm font-medium truncate" style="color: var(--c-text);">
        {{ skill.name }}
      </span>

      <!-- Agent tags -->
      <div class="flex items-center gap-1 flex-wrap">
        <span
          v-for="source in agentSources.slice(0, 3)"
          :key="source.from"
          class="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
          :style="{
            background: source.is_symlink ? 'var(--c-primary-light)' : 'var(--c-surface-hover)',
            color: source.is_symlink ? 'var(--c-primary)' : 'var(--c-text-secondary)',
          }"
        >
          {{ source.is_symlink ? '🔗' : '●' }} {{ getAgentName(source.from) }}
        </span>
        <span
          v-if="agentSources.length > 3"
          class="text-[10px] px-1.5 py-0.5 rounded-full"
          style="background: var(--c-surface-hover); color: var(--c-text-secondary);"
        >
          +{{ agentSources.length - 3 }}
        </span>
      </div>

      <!-- Source count -->
      <span class="text-[11px] shrink-0" style="color: var(--c-text-secondary);">
        {{ getSourceCount() }}
      </span>

      <!-- Status badges -->
      <span
        v-if="skill.has_conflict"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-warning-light); color: var(--c-warning);"
      >
        {{ t("manage.status_conflict") }}
      </span>
      <span
        v-else-if="skill.has_dangling"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-danger-light); color: var(--c-danger);"
      >
        {{ t("manage.status_dangling") }}
      </span>
      <span
        v-else-if="skill.is_duplicate"
        class="text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0"
        style="background: var(--c-info-light); color: var(--c-info);"
      >
        {{ t("manage.status_duplicate") }}
      </span>

      <!-- Delete button -->
      <button
        class="w-5 h-5 flex items-center justify-center rounded cursor-pointer shrink-0 ml-auto transition-colors hover:bg-[var(--c-danger-light)]"
        style="color: var(--c-danger);"
        @click.stop="showDeleteConfirm = true"
        :title="t('skills.delete')"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        </svg>
      </button>
    </div>

    <!-- Expanded details -->
    <div v-if="expanded" class="px-3 pb-3 space-y-3">
      <!-- Agent status list -->
      <div class="space-y-2 pl-7">
        <div
          v-for="source in skill.sources"
          :key="source.from + source.path"
          class="flex items-center justify-between gap-2"
        >
          <div class="flex items-center gap-2">
            <span
              class="w-2 h-2 rounded-full shrink-0"
              :style="{ background: getAgentStatus(source).color }"
            />
            <span class="text-xs font-medium" style="color: var(--c-text);">
              {{ getAgentName(source.from) }}
            </span>
            <span class="text-[10px]" :style="{ color: getAgentStatus(source).color }">
              {{ getAgentStatus(source).icon }} {{ getAgentStatus(source).label }}
            </span>
            <span
              v-if="source.symlink_target && source.from !== 'vibe-lib'"
              class="text-[10px] truncate max-w-[200px]"
              style="color: var(--c-text-secondary);"
            >
              → {{ source.symlink_target.split(/[/\\]/).pop() }}
            </span>
          </div>

          <!-- Per-agent action buttons -->
          <div class="flex items-center gap-1">
            <!-- 同步到技能库 -->
            <button
              v-if="!source.is_symlink && source.from !== 'vibe-lib' && vibeSource"
              class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
              style="background: var(--c-primary); color: white;"
              :disabled="resolvingConflict === source.from"
              @click.stop="handleSyncToVibe(source.from)"
            >
              {{ resolvingConflict === source.from ? t("app.loading") : t("manage.sync_to_vibe") }}
            </button>

            <!-- 替换为链接 -->
            <button
              v-if="!source.is_symlink && source.from !== 'vibe-lib' && vibeSource && source.content_hash === vibeSource.content_hash"
              class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
              style="background: var(--c-surface-hover); color: var(--c-text-secondary); border: 1px solid var(--c-border);"
              @click.stop="handleSyncToVibe(source.from)"
            >
              {{ t("manage.replace_with_link") }}
            </button>

            <!-- 重新链接 -->
            <button
              v-if="source.is_symlink && source.from !== 'vibe-lib' && relinkableAgents.some(a => a.from === source.from)"
              class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
              style="background: var(--c-warning); color: white;"
              @click.stop="handleRelink(source.from)"
            >
              {{ t("manage.relink") }}
            </button>

            <!-- 清理断链 -->
            <button
              v-if="source.is_symlink && !source.symlink_target"
              class="text-[10px] px-2 py-1 rounded cursor-pointer transition-colors"
              style="background: var(--c-danger); color: white;"
              @click.stop="handleRemoveDangling(source)"
            >
              {{ t("manage.remove_dangling") }}
            </button>
          </div>
        </div>
      </div>

      <!-- Duplicate warning -->
      <div
        v-if="skill.is_duplicate"
        class="pl-7 rounded-md p-3 text-xs"
        style="background: var(--c-info-light); border: 1px solid var(--c-info);"
      >
        <div class="flex items-center gap-1.5 mb-2" style="color: var(--c-info);">
          <span>📋</span>
          <span class="font-medium">{{ t("manage.duplicate_warning") }}</span>
        </div>
        <p style="color: var(--c-text-secondary);">
          {{ t("manage.duplicate_description") }}
        </p>
      </div>

      <!-- Action buttons row -->
      <div class="pl-7 flex items-center gap-2 relative">
        <!-- Link to Agent -->
        <div v-if="linkableAgents.length > 0" class="relative">
          <button
            class="text-xs px-3 py-1.5 rounded-md cursor-pointer transition-colors"
            style="background: var(--c-primary); color: white;"
            @click.stop="showLinkMenu = !showLinkMenu"
          >
            {{ t("manage.link_to_agent") }} ▾
          </button>
          <div
            v-if="showLinkMenu"
            class="absolute top-full left-0 mt-1 rounded-md border shadow-lg z-10 py-1 min-w-[160px]"
            style="background: var(--c-surface); border-color: var(--c-border);"
          >
            <button
              v-for="agent in linkableAgents"
              :key="agent.id"
              class="block w-full text-left px-3 py-1.5 text-xs hover:bg-[var(--c-surface-hover)] cursor-pointer"
              style="color: var(--c-text);"
              @click.stop="handleLink(agent.id)"
            >
              {{ agent.name }}
            </button>
          </div>
        </div>

        <!-- Unlink from Agent -->
        <div v-if="unlinkableAgents.length > 0" class="relative">
          <button
            class="text-xs px-3 py-1.5 rounded-md cursor-pointer transition-colors"
            style="background: var(--c-surface-hover); color: var(--c-text-secondary); border: 1px solid var(--c-border);"
            @click.stop="showUnlinkMenu = !showUnlinkMenu"
          >
            {{ t("manage.remove_link") }} ▾
          </button>
          <div
            v-if="showUnlinkMenu"
            class="absolute top-full left-0 mt-1 rounded-md border shadow-lg z-10 py-1 min-w-[160px]"
            style="background: var(--c-surface); border-color: var(--c-border);"
          >
            <button
              v-for="agent in unlinkableAgents"
              :key="agent.id"
              class="block w-full text-left px-3 py-1.5 text-xs hover:bg-[var(--c-surface-hover)] cursor-pointer"
              style="color: var(--c-text);"
              @click.stop="handleUnlink(agent.id)"
            >
              {{ agent.name }}
            </button>
          </div>
        </div>

        <!-- Preview -->
        <button
          class="text-xs px-3 py-1.5 rounded-md cursor-pointer transition-colors"
          :style="{
            background: showPreview ? 'var(--c-primary-light)' : 'var(--c-surface-hover)',
            color: showPreview ? 'var(--c-primary)' : 'var(--c-text-secondary)',
            border: '1px solid ' + (showPreview ? 'var(--c-primary)' : 'var(--c-border)'),
          }"
          @click.stop="togglePreview"
        >
          {{ showPreview ? t("skills.hide_preview") : t("skills.preview") }}
        </button>
      </div>

      <!-- SKILL.md preview -->
      <div v-if="showPreview" class="pl-7">
        <div
          v-if="previewContent"
          class="rounded-md border p-3 text-xs max-h-[200px] overflow-y-auto prose prose-xs"
          style="background: var(--c-bg); border-color: var(--c-border); color: var(--c-text-secondary);"
          v-html="previewContent"
        />
        <div v-else-if="previewLoading" class="text-xs" style="color: var(--c-text-secondary);">
          {{ t("app.loading") }}
        </div>
      </div>
    </div>

    <!-- Delete Confirm -->
    <ConfirmDialog
      v-if="showDeleteConfirm"
      :title="t('skills.delete')"
      :message="t('skills.delete_confirm', { name: skill.name })"
      :confirm-text="t('skills.delete')"
      :danger="true"
      @confirm="handleDelete"
      @cancel="showDeleteConfirm = false"
    />
  </div>
</template>
