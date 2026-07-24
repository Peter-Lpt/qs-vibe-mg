<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { Agent } from "../../types";
import {
  computeCellView,
  computeColSelectableKeys,
  computeRowSelectableKeys,
  type BatchActionCard,
  type BatchRow,
} from "../../composables/useBatchCellActions";

const props = defineProps<{
  rows: BatchRow[];
  agents: Agent[];
  actionCard: BatchActionCard;
  selectedCells: ReadonlySet<string>;
}>();

const emit = defineEmits<{
  (e: "toggle-cell", skillId: string, agentId: string): void;
  (e: "toggle-row", skillId: string): void;
  (e: "toggle-col", agentId: string): void;
  (e: "select-all"): void;
  (e: "clear"): void;
  (e: "remove-skill", skillId: string): void;
  (e: "resolve-conflict", skillId: string): void;
}>();

const { t } = useI18n();

function cellOf(row: BatchRow, agent: Agent) {
  return computeCellView(row, agent, props.actionCard.mode, props.selectedCells, (k, p) =>
    t(k, p as Record<string, unknown>)
  );
}

function statusLabelOf(row: BatchRow, agentId: string): string {
  return row.statuses.find((s) => s.agent.id === agentId)?.statusLabel ?? "";
}

function rowKeys(row: BatchRow): string[] {
  return computeRowSelectableKeys(row, props.actionCard.mode, props.actionCard.cellScope);
}

function colKeys(agentId: string): string[] {
  return computeColSelectableKeys(props.rows, agentId, props.actionCard.mode, props.actionCard.cellScope);
}

function allChecked(keys: string[]): boolean {
  return keys.length > 0 && keys.every((k) => props.selectedCells.has(k));
}

function someChecked(keys: string[]): boolean {
  const n = keys.filter((k) => props.selectedCells.has(k)).length;
  return n > 0 && n < keys.length;
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="mb-2 flex items-center gap-2">
      <span class="text-[10px]" style="color: var(--c-text-secondary);">
        {{ t("manage.selected_scope_count", { selected: selectedCells.size, total: rows.length }) }}
      </span>
      <div class="ml-auto flex items-center gap-2">
        <button
          type="button"
          class="text-[10px] px-2 py-1 rounded cursor-pointer"
          style="color: var(--c-text-secondary); border: 1px solid var(--c-border);"
          @click="emit('select-all')"
        >
          {{ t("manage.batch_panel_select_all") }}
        </button>
        <button
          type="button"
          class="text-[10px] px-2 py-1 rounded cursor-pointer"
          style="color: var(--c-text-secondary); border: 1px solid var(--c-border);"
          @click="emit('clear')"
        >
          {{ t("manage.batch_panel_clear") }}
        </button>
      </div>
    </div>

    <div
      v-if="rows.length === 0"
      class="py-8 text-center text-xs"
      style="color: var(--c-text-secondary);"
    >
      {{ t("manage.batch_panel_no_skills") }}
    </div>
    <div
      v-else-if="agents.length === 0"
      class="py-8 text-center text-xs"
      style="color: var(--c-text-secondary);"
    >
      {{ t("manage.batch_panel_no_agents") }}
    </div>

    <div v-else class="min-h-0 flex-1 overflow-auto rounded border" style="border-color: var(--c-border);">
      <table class="w-full text-xs border-collapse">
        <thead>
          <tr>
            <th
              class="sticky left-0 top-0 z-20 px-2 py-2 text-left font-medium whitespace-nowrap"
              style="background: var(--c-surface); color: var(--c-text-secondary); min-width: 150px; border-bottom: 1px solid var(--c-border);"
            >
              {{ t("manage.title") }}
            </th>
            <th
              v-for="agent in agents"
              :key="agent.id"
              class="sticky top-0 z-10 px-2 py-2 text-center font-medium cursor-pointer select-none"
              style="background: var(--c-surface); color: var(--c-text-secondary); border-bottom: 1px solid var(--c-border);"
              :title="t('manage.batch_panel_col_tip')"
              @click="emit('toggle-col', agent.id)"
            >
              <div class="flex flex-col items-center gap-1">
                <span class="block max-w-[80px] truncate">{{ agent.name }}</span>
                <input
                  type="checkbox"
                  :checked="allChecked(colKeys(agent.id))"
                  :indeterminate="someChecked(colKeys(agent.id))"
                  class="w-3 h-3 rounded cursor-pointer"
                  style="accent-color: var(--c-primary);"
                  @click.stop="emit('toggle-col', agent.id)"
                />
              </div>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in rows" :key="row.skill.id" style="border-bottom: 1px solid var(--c-border);">
            <td
              class="sticky left-0 z-10 px-2 py-1.5 cursor-pointer select-none"
              style="background: var(--c-surface);"
              :title="t('manage.batch_panel_row_tip')"
              @click="emit('toggle-row', row.skill.id)"
            >
              <div class="flex items-center gap-2">
                <input
                  type="checkbox"
                  :checked="allChecked(rowKeys(row))"
                  :indeterminate="someChecked(rowKeys(row))"
                  class="w-3.5 h-3.5 rounded cursor-pointer shrink-0"
                  style="accent-color: var(--c-primary);"
                  @click.stop="emit('toggle-row', row.skill.id)"
                />
                <span class="max-w-[110px] truncate text-xs font-medium" style="color: var(--c-text);">
                  {{ row.skill.name || row.skill.id }}
                </span>
                <button
                  type="button"
                  class="shrink-0 rounded px-1 text-[10px] cursor-pointer"
                  style="color: var(--c-text-secondary);"
                  :title="t('manage.batch_panel_remove')"
                  @click.stop="emit('remove-skill', row.skill.id)"
                >
                  ✕
                </button>
              </div>
            </td>
            <td v-for="agent in agents" :key="agent.id" class="px-1 py-1.5 text-center">
              <button
                v-if="cellOf(row, agent).selectable"
                type="button"
                class="w-full min-w-[72px] rounded border px-1.5 py-1 text-left cursor-pointer transition-colors"
                :style="{
                  borderColor: cellOf(row, agent).checked
                    ? (cellOf(row, agent).color || 'var(--c-primary)')
                    : 'var(--c-border)',
                  background: cellOf(row, agent).checked ? 'var(--c-primary-light)' : 'transparent',
                }"
                @click="emit('toggle-cell', row.skill.id, agent.id)"
              >
                <div
                  class="text-[10px] font-medium leading-tight"
                  :style="{ color: cellOf(row, agent).checked ? (cellOf(row, agent).color || 'var(--c-primary)') : (cellOf(row, agent).color || 'var(--c-text)') }"
                >
                  {{ cellOf(row, agent).label }}
                </div>
                <div class="text-[9px] leading-tight" style="color: var(--c-text-tertiary);">
                  {{ statusLabelOf(row, agent.id) }}
                </div>
              </button>
              <button
                v-else-if="cellOf(row, agent).isConflict"
                type="button"
                class="w-full min-w-[72px] rounded border px-1.5 py-1 text-[10px] cursor-pointer transition-colors"
                style="color: var(--c-danger); background: var(--c-danger-light); border-color: var(--c-danger);"
                :title="t('manage.batch_panel_resolve_conflict_tip')"
                @click.stop="emit('resolve-conflict', row.skill.id)"
              >
                {{ t("manage.batch_panel_resolve_conflict") }}
              </button>
              <span
                v-else
                class="inline-block rounded px-1.5 py-1 text-[10px]"
                style="color: var(--c-text-secondary); border: 1px dashed var(--c-border);"
                :title="cellOf(row, agent).needsImport ? t('manage.batch_panel_needs_import_tip') : ''"
              >
                {{ cellOf(row, agent).label }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
