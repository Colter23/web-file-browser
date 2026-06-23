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
        <icon icon="icon-edit-filling" color="#ffffff" />
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
        <icon icon="icon-file-common-filling" color="#475569" />
        <span>语言：{{ selectedModeName }}</span>
      </button>
      <button class="menu-button" :class="{active: activeMenu === 'theme'}" @click.stop="emit('toggle-menu', 'theme')">
        <icon icon="icon-setting" color="#475569" />
        <span>主题：{{ selectedThemeName }}</span>
      </button>
      <button class="icon-button" :class="{active: activeMenu === 'settings'}" title="编辑设置" @click.stop="emit('toggle-menu', 'settings')">
        <icon icon="icon-setting" color="#475569" />
      </button>
      <button class="icon-button" :disabled="loading" title="重新载入" @click.stop="emit('reload')">
        <icon icon="icon-refresh" color="#475569" />
      </button>
      <button class="save-button" :disabled="!canSave" title="保存 (Ctrl+S)" @click.stop="emit('save')">
        <icon icon="icon-save-fill" :color="canSave ? '#ffffff' : '#94a3b8'" />
        <span>{{ saving ? "保存中" : "保存" }}</span>
      </button>
      <button class="icon-button close-button" title="关闭 (Esc)" @click.stop="emit('close')">
        <icon icon="icon-close" color="#475569" />
      </button>
    </div>
  </header>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-titlebar {
  @apply relative z-20 flex h-12 shrink-0 items-center justify-between gap-3 border-b border-slate-200 bg-white/90 px-3 backdrop-blur;
}

.editor-file-head {
  @apply flex min-w-0 items-center gap-3;
}

.file-mark {
  @apply inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-blue-600 shadow-sm;
}

.file-title-block {
  @apply flex min-w-0 flex-col;
}

.file-title-line {
  @apply flex min-w-0 items-center gap-2;
}

.file-title {
  @apply min-w-0 truncate text-sm font-semibold text-slate-900;
}

.dirty-dot {
  @apply h-2 w-2 shrink-0 rounded-full bg-amber-400;
}

.file-path {
  @apply max-w-[42rem] truncate text-xs text-slate-500;
}

.editor-actions {
  @apply flex shrink-0 items-center gap-1;
}

.menu-button,
.icon-button,
.save-button {
  @apply inline-flex h-8 items-center justify-center rounded-md border border-slate-200 bg-white text-xs text-slate-700 shadow-sm hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-45 disabled:hover:bg-white;
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
  @apply border-blue-300 bg-blue-50 text-blue-700;
}

.save-button {
  @apply gap-1.5 border-blue-600 bg-blue-600 px-3 font-medium text-white hover:bg-blue-700 disabled:border-slate-200 disabled:bg-slate-100 disabled:text-slate-400;
}

.close-button {
  @apply hover:border-red-200 hover:bg-red-50;
}
</style>
