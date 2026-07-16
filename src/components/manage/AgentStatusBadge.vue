<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { SkillSource } from "../../types";

const { t } = useI18n();

const props = defineProps<{
  source: SkillSource;
  agents: { id: string; name: string }[];
}>();

const agentName = computed(() => {
  if (props.source.from === "vibe-lib") return "Vibe Library";
  const agent = props.agents.find((a) => a.id === props.source.from);
  return agent ? agent.name : props.source.from;
});

const statusLabel = computed(() => {
  if (props.source.is_symlink) {
    if (props.source.content_hash === "") {
      return t("manage.broken_symlink");
    }
    if (props.source.symlink_target) {
      const targetName = props.source.symlink_target.split(/[/\\]/).pop() || props.source.symlink_target;
      return t("manage.symlink_to", { target: targetName });
    }
    return t("manage.broken_symlink");
  }
  return t("manage.real_file");
});

const statusColor = computed(() => {
  if (props.source.is_symlink) {
    return "var(--c-primary)";
  }
  return "var(--c-text-secondary)";
});

const statusIcon = computed(() => {
  if (props.source.is_symlink) {
    return "Link2";
  }
  return "Circle";
});
</script>

<template>
  <div class="flex items-center gap-2 text-xs">
    <span
      class="w-2 h-2 rounded-full shrink-0"
      style="background: var(--c-success);"
    />
    <span class="font-medium" style="color: var(--c-text);">{{ agentName }}</span>
    <span class="flex items-center gap-1" :style="{ color: statusColor }">
      <component :is="statusIcon" :size="14" />
      <span>{{ statusLabel }}</span>
    </span>
  </div>
</template>
