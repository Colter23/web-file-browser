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
  defaultEditMode: boolean;
}>();

const emit = defineEmits<{
  (e: "change-mode", mode: string): void;
  (e: "change-theme", theme: string): void;
  (e: "change-highlight", highlight: string): void;
  (e: "update:fontSize", value: number): void;
  (e: "update:tabSize", value: number): void;
  (e: "update:wrap", value: boolean): void;
  (e: "update:defaultEditMode", value: boolean): void;
}>();

const inputNumber = (event: Event) => {
  const input = event.target as HTMLInputElement | null;
  return Number(input?.value ?? 0);
}

const clampNumber = (value: number, min: number, max: number) => {
  if (!Number.isFinite(value)) return min;
  return Math.min(max, Math.max(min, Math.round(value)));
}

const updateFontSizeValue = (value: number) => emit("update:fontSize", clampNumber(value, 12, 28));

const updateTabSizeValue = (value: number) => emit("update:tabSize", clampNumber(value, 2, 8));

const updateFontSize = (event: Event) => updateFontSizeValue(inputNumber(event));

const updateTabSize = (event: Event) => updateTabSizeValue(inputNumber(event));

const stepFontSize = (step: number) => updateFontSizeValue(props.fontSize + step);

const stepTabSize = (step: number) => updateTabSizeValue(props.tabSize + step);

const themeGroups = computed(() => [
  {title: "跟随", items: props.themes.automatic},
  {title: "浅色", items: props.themes.light},
  {title: "深色", items: props.themes.dark}
].filter(group => group.items.length));

const menuLayerStyle = computed(() => {
  const anchor = props.anchor;
  if (!anchor || !props.activeMenu) return {};
  const width = props.activeMenu === "language" ? 208 : props.activeMenu === "settings" ? 288 : 224;
  const maxHeight = props.activeMenu === "settings" ? 360 : 320;
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
      <div class="setting-row">
        <div class="setting-copy">
          <span class="setting-label">字号</span>
          <span class="setting-hint">Ctrl + 滚轮缩放</span>
        </div>
        <div class="number-stepper" aria-label="字号">
          <button type="button" :disabled="fontSize <= 12" @click="stepFontSize(-1)">-</button>
          <input :value="fontSize" type="number" min="12" max="28" step="1" inputmode="numeric" @input="updateFontSize">
          <button type="button" :disabled="fontSize >= 28" @click="stepFontSize(1)">+</button>
        </div>
      </div>
      <div class="setting-row">
        <div class="setting-copy">
          <span class="setting-label">Tab 宽度</span>
          <span class="setting-hint">缩进空格数</span>
        </div>
        <div class="number-stepper" aria-label="Tab 宽度">
          <button type="button" :disabled="tabSize <= 2" @click="stepTabSize(-1)">-</button>
          <input :value="tabSize" type="number" min="2" max="8" step="1" inputmode="numeric" @input="updateTabSize">
          <button type="button" :disabled="tabSize >= 8" @click="stepTabSize(1)">+</button>
        </div>
      </div>
      <div class="setting-row switch-row">
        <div class="setting-copy">
          <span class="setting-label">自动换行</span>
          <span class="setting-hint">长行在窗口内折行</span>
        </div>
        <button
            type="button"
            class="switch-control"
            :class="{active: wrap}"
            role="switch"
            aria-label="自动换行"
            :aria-checked="wrap"
            @click="emit('update:wrap', !wrap)">
          <span></span>
        </button>
      </div>
      <div class="setting-row switch-row">
        <div class="setting-copy">
          <span class="setting-label">默认编辑模式</span>
          <span class="setting-hint">新打开文件直接可编辑</span>
        </div>
        <button
            type="button"
            class="switch-control"
            :class="{active: defaultEditMode}"
            role="switch"
            aria-label="默认编辑模式"
            :aria-checked="defaultEditMode"
            @click="emit('update:defaultEditMode', !defaultEditMode)">
          <span></span>
        </button>
      </div>
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
  @apply w-72 gap-1.5 p-2;
}

.editor-menu p {
  @apply px-2 pt-1 text-xs font-medium;
  color: var(--app-text-subtle);
}

.editor-menu > button {
  @apply flex h-8 items-center gap-2 rounded px-2 text-left;
  color: var(--app-text-muted);
}

.editor-menu > button:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.editor-menu > button.active {
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.editor-menu > button.active:hover {
  background: var(--app-accent, #2563eb);
}

.setting-row {
  @apply flex min-h-12 items-center justify-between gap-3 rounded-md px-2 py-1.5 text-sm;
  color: var(--app-text-muted);
}

.setting-row:hover {
  background: var(--app-control-hover);
}

.setting-copy {
  @apply flex min-w-0 flex-col gap-0.5;
}

.setting-label {
  @apply truncate font-medium;
  color: var(--app-text);
}

.setting-hint {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.number-stepper {
  @apply inline-flex h-8 shrink-0 items-center overflow-hidden rounded-md border;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
}

.number-stepper button {
  @apply inline-flex h-full w-7 items-center justify-center rounded-none border-0 p-0 text-sm shadow-none disabled:cursor-not-allowed disabled:opacity-35;
  background: transparent;
  color: var(--app-text-muted);
}

.number-stepper button:hover:not(:disabled) {
  background: var(--app-accent-hover);
  color: var(--app-accent);
}

.number-stepper input[type="number"] {
  @apply h-full w-11 border-x px-1 text-center text-sm outline-none;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-control-solid) 72%, transparent);
  color: var(--app-text);
  appearance: textfield;
}

.number-stepper input[type="number"]::-webkit-outer-spin-button,
.number-stepper input[type="number"]::-webkit-inner-spin-button {
  margin: 0;
  appearance: none;
}

.number-stepper:focus-within {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring);
}

.switch-control {
  @apply relative block h-6 w-11 shrink-0 rounded-full border p-0 shadow-none transition-colors;
  border-color: var(--app-border-soft);
  background: var(--app-control);
  box-sizing: border-box;
}

.switch-control:hover {
  background: var(--app-control-hover);
}

.switch-control.active {
  border-color: var(--app-accent);
  background: var(--app-accent);
}

.switch-control span {
  @apply absolute left-0.5 top-1/2 block h-5 w-5 rounded-full shadow-sm transition-transform;
  background: var(--app-panel-solid);
  transform: translateY(-50%);
}

.switch-control.active span {
  background: var(--app-accent-contrast);
  transform: translate(20px, -50%);
}
</style>
