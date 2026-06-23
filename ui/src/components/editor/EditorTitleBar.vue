<script setup lang="ts">
import Icon from "../Icon.vue";
import type {EditorMenuName} from "./types.ts";

defineProps<{
  activeMenu: EditorMenuName;
  fileTitle: string;
  filePathText: string;
  dirty: boolean;
  selectedModeName: string;
  selectedThemeName: string;
  loading: boolean;
  saving: boolean;
  canSave: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle-menu", menu: EditorMenuName): void;
  (e: "reload"): void;
  (e: "save"): void;
  (e: "close"): void;
}>();
</script>

<template>
  <header class="editor-titlebar" @click.stop>
    <div class="editor-file-head">
      <div class="file-mark">
        <icon icon="action.edit" color="#ffffff" />
      </div>
      <div class="file-title-block">
        <div class="file-title-line">
          <span class="file-title">{{ fileTitle }}</span>
          <span v-if="dirty" class="dirty-dot"></span>
        </div>
        <span class="file-path" :title="filePathText">{{ filePathText }}</span>
      </div>
    </div>

    <div class="editor-actions">
      <button class="menu-button" :class="{active: activeMenu === 'language'}" @click.stop="emit('toggle-menu', 'language')">
        <icon icon="file.text" />
        <span>语言：{{ selectedModeName }}</span>
      </button>
      <button class="menu-button" :class="{active: activeMenu === 'theme'}" @click.stop="emit('toggle-menu', 'theme')">
        <icon icon="action.settings" />
        <span>主题：{{ selectedThemeName }}</span>
      </button>
      <button class="icon-button" :class="{active: activeMenu === 'settings'}" title="编辑设置" @click.stop="emit('toggle-menu', 'settings')">
        <icon icon="action.settings" />
      </button>
      <button class="icon-button" :disabled="loading" title="重新载入" @click.stop="emit('reload')">
        <icon icon="action.refresh" />
      </button>
      <button class="save-button" :disabled="!canSave" title="保存 (Ctrl+S)" @click.stop="emit('save')">
        <icon icon="action.save" :color="canSave ? '#ffffff' : '#94a3b8'" />
        <span>{{ saving ? "保存中" : "保存" }}</span>
      </button>
      <button class="icon-button close-button" title="关闭 (Esc)" @click.stop="emit('close')">
        <icon icon="action.close" />
      </button>
    </div>
  </header>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-titlebar {
  @apply relative z-20 flex h-12 shrink-0 items-center justify-between gap-3 border-b px-3 backdrop-blur;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 90%, transparent);
}

.editor-file-head {
  @apply flex min-w-0 items-center gap-3;
}

.file-mark {
  @apply inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md shadow-sm;
  background: var(--app-accent, #2563eb);
}

.file-title-block {
  @apply flex min-w-0 flex-col;
}

.file-title-line {
  @apply flex min-w-0 items-center gap-2;
}

.file-title {
  @apply min-w-0 truncate text-sm font-semibold;
  color: var(--app-text);
}

.dirty-dot {
  @apply h-2 w-2 shrink-0 rounded-full bg-amber-400;
}

.file-path {
  @apply max-w-[42rem] truncate text-xs;
  color: var(--app-text-subtle);
}

.editor-actions {
  @apply flex shrink-0 items-center gap-1;
}

.menu-button,
.icon-button,
.save-button {
  @apply inline-flex h-8 items-center justify-center rounded-md border text-xs shadow-sm disabled:cursor-not-allowed disabled:opacity-45;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.menu-button:disabled,
.icon-button:disabled,
.save-button:disabled {
  background: var(--app-control-solid);
}

.menu-button:hover:not(:disabled),
.icon-button:hover:not(:disabled),
.save-button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.menu-button {
  @apply max-w-40 gap-1.5 px-2;
}

.menu-button span {
  @apply min-w-0 truncate;
}

.icon-button {
  @apply w-8;
}

.menu-button.active,
.icon-button.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.save-button {
  @apply gap-1.5 px-3 font-medium text-white;
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent, #2563eb);
}

.save-button:disabled {
  border-color: var(--app-border-soft);
  background: var(--app-control);
  color: var(--app-text-disabled);
}

.save-button:hover:not(:disabled) {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 88%, black);
}

.close-button {
  @apply hover:border-red-200;
}

.close-button:hover:not(:disabled) {
  background: var(--app-danger-soft);
}
</style>
