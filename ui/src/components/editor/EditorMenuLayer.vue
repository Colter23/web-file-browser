<script setup lang="ts">
import {computed} from "vue";
import Icon from "../Icon.vue";
import type {EditorHighlightOption, EditorMenuAnchor, EditorMenuName, EditorModeOption, EditorThemeGroups} from "./types.ts";

const props = defineProps<{
  activeMenu: EditorMenuName;
  anchor: EditorMenuAnchor | null;
  modes: EditorModeOption[];
  themes: EditorThemeGroups;
  highlights: EditorHighlightOption[];
  currentMode: string;
  currentTheme: string;
  currentHighlight: string;
  fontSize: number;
  tabSize: number;
  wrap: boolean;
}>();

const emit = defineEmits<{
  (e: "change-mode", mode: string): void;
  (e: "change-theme", theme: string): void;
  (e: "change-highlight", highlight: string): void;
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

const themeGroups = computed(() => [
  {title: "跟随", items: props.themes.automatic},
  {title: "浅色", items: props.themes.light},
  {title: "深色", items: props.themes.dark}
].filter(group => group.items.length));

const menuLayerStyle = computed(() => {
  const anchor = props.anchor;
  if (!anchor || !props.activeMenu) return {};
  const width = props.activeMenu === "language" ? 208 : 224;
  const maxHeight = props.activeMenu === "settings" ? 180 : 320;
  const padding = 8;
  const rawLeft = anchor.align === "start" ? anchor.left : anchor.right - width;
  const left = Math.min(Math.max(padding, rawLeft), Math.max(padding, window.innerWidth - width - padding));
  const top = Math.min(Math.max(padding, anchor.bottom + 6), Math.max(padding, window.innerHeight - maxHeight - padding));
  return {
    left: `${left}px`,
    top: `${top}px`
  };
});
</script>

<template>
  <Teleport to="body">
  <div v-if="activeMenu && anchor" class="menu-layer" :style="menuLayerStyle" data-editor-menu-layer @click.stop>
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
      <template v-for="group in themeGroups" :key="group.title">
        <p>{{ group.title }}</p>
        <button
            v-for="theme in group.items"
            :key="theme.key"
            :class="{active: currentTheme === theme.key}"
            @click="emit('change-theme', theme.key)">
          <icon :icon="theme.icon ?? 'action.appearance'" />
          <span>{{ theme.name }}</span>
        </button>
      </template>
    </div>

    <div v-if="activeMenu === 'highlight'" class="editor-menu highlight-menu">
      <button
          v-for="highlight in highlights"
          :key="highlight.key"
          :class="{active: currentHighlight === highlight.key}"
          @click="emit('change-highlight', highlight.key)">
        <icon :icon="highlight.icon ?? 'file.code'" />
        <span>{{ highlight.name }}</span>
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
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.menu-layer {
  @apply fixed z-50;
}

.editor-menu {
  @apply flex max-h-80 min-w-44 flex-col gap-1 overflow-auto rounded-md border p-1 text-sm shadow-2xl;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.language-menu {
  @apply w-52;
}

.theme-menu {
  @apply w-56;
}

.highlight-menu {
  @apply w-56;
}

.settings-menu {
  @apply w-56 gap-3 p-3;
}

.editor-menu p {
  @apply px-2 pt-1 text-xs font-medium;
  color: var(--app-text-subtle);
}

.editor-menu button {
  @apply flex h-8 items-center gap-2 rounded px-2 text-left;
  color: var(--app-text-muted);
}

.editor-menu button:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.editor-menu button.active {
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.editor-menu button.active:hover {
  background: var(--app-accent, #2563eb);
}

.editor-menu label {
  @apply flex items-center justify-between gap-3 text-sm;
  color: var(--app-text-muted);
}

.editor-menu input[type="number"] {
  @apply h-8 w-20 rounded border px-2 text-right outline-none;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text);
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
