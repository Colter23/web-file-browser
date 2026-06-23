<script setup lang="ts">
import Icon from "../Icon.vue";
import type {EditorMenuName, EditorModeOption, EditorThemeGroups} from "./types.ts";

defineProps<{
  activeMenu: EditorMenuName;
  modes: EditorModeOption[];
  themes: EditorThemeGroups;
  currentMode: string;
  currentTheme: string;
  fontSize: number;
  tabSize: number;
  wrap: boolean;
}>();

const emit = defineEmits<{
  (e: "change-mode", mode: string): void;
  (e: "change-theme", theme: string): void;
  (e: "update:fontSize", value: number): void;
  (e: "update:tabSize", value: number): void;
  (e: "update:wrap", value: boolean): void;
}>();

const inputNumber = (event: Event) => {
  const input = event.target as HTMLInputElement | null;
  return Number(input?.value ?? 0);
}

const updateFontSize = (event: Event) => emit("update:fontSize", inputNumber(event));

const updateTabSize = (event: Event) => emit("update:tabSize", inputNumber(event));

const updateWrap = (event: Event) => {
  const input = event.target as HTMLInputElement | null;
  emit("update:wrap", Boolean(input?.checked));
}
</script>

<template>
  <div class="menu-layer" @click.stop>
    <div v-if="activeMenu === 'language'" class="editor-menu language-menu">
      <button
          v-for="mode in modes"
          :key="mode.key"
          :class="{active: currentMode === mode.key}"
          @click="emit('change-mode', mode.key)">
        <icon icon="file.code" />
        <span>{{ mode.name }}</span>
      </button>
    </div>

    <div v-if="activeMenu === 'theme'" class="editor-menu theme-menu">
      <p>浅色主题</p>
      <button
          v-for="theme in themes.light"
          :key="theme.key"
          :class="{active: currentTheme === theme.key}"
          @click="emit('change-theme', theme.key)">
        <span>{{ theme.name }}</span>
      </button>
      <p>深色主题</p>
      <button
          v-for="theme in themes.dark"
          :key="theme.key"
          :class="{active: currentTheme === theme.key}"
          @click="emit('change-theme', theme.key)">
        <span>{{ theme.name }}</span>
      </button>
    </div>

    <div v-if="activeMenu === 'settings'" class="editor-menu settings-menu">
      <label>
        <span>字号</span>
        <input :value="fontSize" type="number" min="12" max="28" step="1" @input="updateFontSize">
      </label>
      <label>
        <span>Tab 宽度</span>
        <input :value="tabSize" type="number" min="2" max="8" step="1" @input="updateTabSize">
      </label>
      <label class="check-row">
        <input :checked="wrap" type="checkbox" @change="updateWrap">
        <span>自动换行</span>
      </label>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.menu-layer {
  @apply absolute right-3 top-[5.25rem] z-30;
}

.editor-menu {
  @apply mt-2 flex max-h-80 min-w-44 flex-col gap-1 overflow-auto rounded-md border border-slate-200 bg-white p-1 text-sm shadow-2xl;
}

.language-menu {
  @apply w-52;
}

.theme-menu {
  @apply w-56;
}

.settings-menu {
  @apply w-56 gap-3 p-3;
}

.editor-menu p {
  @apply px-2 pt-1 text-xs font-medium text-slate-400;
}

.editor-menu button {
  @apply flex h-8 items-center gap-2 rounded px-2 text-left text-slate-700;
}

.editor-menu button:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.editor-menu button.active {
  @apply text-white;
  background: var(--app-accent, #2563eb);
}

.editor-menu button.active:hover {
  background: var(--app-accent, #2563eb);
}

.editor-menu label {
  @apply flex items-center justify-between gap-3 text-sm text-slate-600;
}

.editor-menu input[type="number"] {
  @apply h-8 w-20 rounded border border-slate-200 bg-white px-2 text-right text-slate-900 outline-none;
}

.editor-menu input[type="number"]:focus {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.check-row {
  @apply justify-start;
}

.check-row input {
  @apply h-4 w-4;
  accent-color: var(--app-accent, #2563eb);
}
</style>
