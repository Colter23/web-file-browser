<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "./types.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import {useViewportMenuPosition} from "../../composables/useViewportMenuPosition.ts";

const props = defineProps<{
  background: boolean;
  x: number;
  y: number;
  canPaste: boolean;
  hasEntries: boolean;
  hasSelection: boolean;
  primaryEntry: ExplorerEntry | null;
  selectionCount: number;
  canViewImage: boolean;
  canEdit: boolean;
  canExtract: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "escape"): void;
  (e: "open"): void;
  (e: "open-new-tab"): void;
  (e: "view-image"): void;
  (e: "edit"): void;
  (e: "preview"): void;
  (e: "cut"): void;
  (e: "copy"): void;
  (e: "copy-path"): void;
  (e: "paste"): void;
  (e: "download"): void;
  (e: "archive"): void;
  (e: "extract"): void;
  (e: "rename"): void;
  (e: "delete"): void;
  (e: "properties"): void;
  (e: "create-file"): void;
  (e: "create-folder"): void;
  (e: "select-all"): void;
  (e: "invert-selection"): void;
  (e: "clear-selection"): void;
}>();

const menuRef = ref<HTMLElement | null>(null);
const {menuPosition, placeMenu} = useViewportMenuPosition({menuRef});

const isMultiSelect = computed(() => props.selectionCount > 1);

const contextLabel = (single: string, multiple: string) => {
  return isMultiSelect.value ? `${multiple}（${props.selectionCount} 项）` : single;
}

const {
  focusFirstMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef,
  onEscape: () => emit("escape")
});

const refreshMenu = async () => {
  await placeMenu({x: props.x, y: props.y});
  await focusFirstMenuButton();
}

useOutsidePointerDown({
  refs: [menuRef],
  onOutsidePointerDown: () => emit("close")
});

onMounted(() => {
  void refreshMenu();
  window.addEventListener("resize", refreshMenu);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", refreshMenu);
});

watch(() => [props.background, props.x, props.y, props.primaryEntry?.path, props.selectionCount] as const, () => {
  void refreshMenu();
}, {flush: "post"});
</script>

<template>
  <Teleport to="body">
    <div
        ref="menuRef"
        class="context-menu"
        :style="{left: `${menuPosition.x}px`, top: `${menuPosition.y}px`}"
        @click.stop
        @contextmenu.prevent.stop
        @keydown="handleMenuKeyDown">
      <template v-if="background">
        <button @click="emit('create-file')">新建文件</button>
        <button @click="emit('create-folder')">新建文件夹</button>
        <div class="context-separator"></div>
        <button :disabled="!canPaste" @click="emit('paste')">粘贴</button>
        <button @click="emit('copy-path')">复制当前路径</button>
        <div class="context-separator"></div>
        <button :disabled="!hasEntries" @click="emit('select-all')">全选</button>
        <button :disabled="!hasEntries" @click="emit('invert-selection')">反向选择</button>
        <button :disabled="!hasSelection" @click="emit('clear-selection')">取消选择</button>
      </template>

      <template v-else>
        <button @click="emit('open')">打开</button>
        <button :disabled="!primaryEntry || primaryEntry.type !== 'folder'" @click="emit('open-new-tab')">在新标签页中打开</button>
        <button :disabled="!canViewImage" @click="emit('view-image')">查看图片</button>
        <button :disabled="!canEdit" @click="emit('edit')">编辑</button>
        <button :disabled="!primaryEntry || primaryEntry.type !== 'file'" @click="emit('preview')">预览</button>
        <div class="context-separator"></div>
        <button :disabled="!selectionCount" @click="emit('cut')">{{ contextLabel("剪切", "剪切选中项") }}</button>
        <button :disabled="!selectionCount" @click="emit('copy')">{{ contextLabel("复制", "复制选中项") }}</button>
        <button :disabled="!selectionCount" @click="emit('copy-path')">{{ contextLabel("复制路径", "复制选中项路径") }}</button>
        <button :disabled="!canPaste" @click="emit('paste')">粘贴</button>
        <div class="context-separator"></div>
        <button :disabled="!primaryEntry || primaryEntry.type !== 'file'" @click="emit('download')">下载</button>
        <button :disabled="!selectionCount" @click="emit('archive')">{{ contextLabel("压缩", "压缩选中项") }}</button>
        <button :disabled="!canExtract" @click="emit('extract')">解压</button>
        <div class="context-separator"></div>
        <button :disabled="!primaryEntry || isMultiSelect" @click="emit('rename')">重命名</button>
        <button class="danger" :disabled="!primaryEntry" @click="emit('delete')">{{ contextLabel("删除", "删除选中项") }}</button>
        <div class="context-separator"></div>
        <button :disabled="!selectionCount" @click="emit('properties')">属性</button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.context-menu {
  @apply fixed z-50 w-44 rounded-md border border-slate-200 bg-white py-1 text-sm shadow-xl;
}

.context-menu button {
  @apply block h-8 w-full px-3 text-left text-slate-700 disabled:text-slate-300 disabled:hover:bg-white;
}

.context-menu button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.context-menu button:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.context-separator {
  @apply my-1 border-t border-slate-100;
}

.context-menu .danger {
  @apply text-red-600 hover:bg-red-50;
}
</style>
