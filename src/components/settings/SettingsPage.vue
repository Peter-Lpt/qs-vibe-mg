<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAppStore, type Locale, type ThemeMode } from "../../stores/app";
import { useAgentsStore } from "../../stores/agents";

const { t } = useI18n();
const appStore = useAppStore();
const agentsStore = useAgentsStore();

const showAddAgent = ref(false);
const newAgentName = ref("");
const newAgentDir = ref("");
const addError = ref<string | null>(null);

const themes: { value: ThemeMode; labelKey: string }[] = [
  { value: "system", labelKey: "settings.theme_system" },
  { value: "light", labelKey: "settings.theme_light" },
  { value: "dark", labelKey: "settings.theme_dark" },
];

const locales: { value: Locale; label: string }[] = [
  { value: "zh", label: "中文" },
  { value: "en", label: "English" },
  { value: "zh-TW", label: "繁體中文" },
];

function handleThemeChange(mode: ThemeMode) {
  appStore.setTheme(mode);
}

function handleLocaleChange(loc: Locale) {
  appStore.setLocale(loc);
}

async function handleAddAgent() {
  addError.value = null;
  if (!newAgentName.value.trim() || !newAgentDir.value.trim()) {
    addError.value = "Name and path are required";
    return;
  }
  try {
    await agentsStore.addCustomAgent(newAgentName.value.trim(), newAgentDir.value.trim());
    newAgentName.value = "";
    newAgentDir.value = "";
    showAddAgent.value = false;
  } catch (e: unknown) {
    addError.value = String(e);
  }
}

async function handleRemoveAgent(agentId: string) {
  try {
    await agentsStore.removeCustomAgent(agentId);
  } catch (e: unknown) {
    alert(String(e));
  }
}
</script>

<template>
  <div
    class="fixed inset-0 z-40 flex items-center justify-center"
    style="background: rgba(0, 0, 0, 0.5);"
    @click.self="appStore.showSettings = false"
  >
    <div
      class="rounded-lg shadow-xl max-w-lg w-full mx-4 max-h-[80vh] overflow-y-auto"
      style="background: var(--c-surface); border: 1px solid var(--c-border);"
    >
      <div class="flex items-center justify-between p-4 border-b" style="border-color: var(--c-border);">
        <h2 class="text-sm font-semibold" style="color: var(--c-text);">
          {{ t('settings.title') }}
        </h2>
        <button
          class="text-lg hover:opacity-70 cursor-pointer"
          style="color: var(--c-text-secondary);"
          @click="appStore.showSettings = false"
        >
          ×
        </button>
      </div>

      <div class="p-4 space-y-4">
        <!-- Theme -->
        <div>
          <label class="text-xs font-medium block mb-1.5" style="color: var(--c-text);">
            {{ t('settings.theme') }}
          </label>
          <div class="flex gap-2">
            <button
              v-for="th in themes"
              :key="th.value"
              class="px-3 py-1.5 text-xs rounded-md border cursor-pointer transition-opacity"
              :style="{
                background: appStore.theme === th.value ? 'var(--c-primary)' : 'transparent',
                color: appStore.theme === th.value ? 'white' : 'var(--c-text)',
                borderColor: appStore.theme === th.value ? 'var(--c-primary)' : 'var(--c-border)',
              }"
              @click="handleThemeChange(th.value)"
            >
              {{ t(th.labelKey) }}
            </button>
          </div>
        </div>

        <!-- Language -->
        <div>
          <label class="text-xs font-medium block mb-1.5" style="color: var(--c-text);">
            {{ t('settings.language') }}
          </label>
          <div class="flex gap-2">
            <button
              v-for="loc in locales"
              :key="loc.value"
              class="px-3 py-1.5 text-xs rounded-md border cursor-pointer transition-opacity"
              :style="{
                background: appStore.locale === loc.value ? 'var(--c-primary)' : 'transparent',
                color: appStore.locale === loc.value ? 'white' : 'var(--c-text)',
                borderColor: appStore.locale === loc.value ? 'var(--c-primary)' : 'var(--c-border)',
              }"
              @click="handleLocaleChange(loc.value)"
            >
              {{ loc.label }}
            </button>
          </div>
        </div>

        <!-- Custom Agents -->
        <div>
          <div class="flex items-center justify-between mb-1.5">
            <label class="text-xs font-medium" style="color: var(--c-text);">
              {{ t('agents.add_custom') }}
            </label>
            <button
              class="text-xs px-2 py-0.5 rounded border cursor-pointer hover:opacity-80"
              style="border-color: var(--c-primary); color: var(--c-primary);"
              @click="showAddAgent = !showAddAgent"
            >
              {{ showAddAgent ? t('settings.cancel') : '+' }}
            </button>
          </div>

          <!-- Add agent form -->
          <div
            v-if="showAddAgent"
            class="p-3 rounded-md border mb-3 space-y-2"
            style="border-color: var(--c-border); background: var(--c-bg);"
          >
            <input
              v-model="newAgentName"
              :placeholder="t('settings.agent_name_placeholder')"
              class="w-full px-2 py-1.5 text-xs rounded border outline-none"
              style="background: var(--c-surface); border-color: var(--c-border); color: var(--c-text);"
            />
            <input
              v-model="newAgentDir"
              :placeholder="t('settings.skills_dir_placeholder')"
              class="w-full px-2 py-1.5 text-xs rounded border outline-none"
              style="background: var(--c-surface); border-color: var(--c-border); color: var(--c-text);"
            />
            <div v-if="addError" class="text-xs" style="color: var(--c-danger);">
              {{ addError }}
            </div>
            <button
              class="px-3 py-1.5 text-xs rounded cursor-pointer hover:opacity-80"
              style="background: var(--c-primary); color: white;"
              @click="handleAddAgent"
            >
              {{ t('settings.save') }}
            </button>
          </div>

          <!-- Custom agent list -->
          <div class="space-y-1.5">
            <div
              v-for="agent in agentsStore.agents.filter(a => !a.auto_detected)"
              :key="agent.id"
              class="flex items-center gap-2 px-2 py-1.5 rounded text-xs"
              style="background: var(--c-bg);"
            >
              <span class="flex-1 truncate" style="color: var(--c-text);">{{ agent.name }}</span>
              <span class="truncate text-xs" style="color: var(--c-text-secondary); max-width: 180px;">
                {{ agent.skills_dir }}
              </span>
              <button
                class="hover:opacity-70 cursor-pointer text-sm"
                style="color: var(--c-danger);"
                @click="handleRemoveAgent(agent.id)"
              >
                ×
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
