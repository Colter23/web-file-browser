<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "./types.ts";

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
const menuPosition = ref({x: props.x, y: props.y});
const viewportPadding = 8;

const isMultiSelect = computed(() => props.selectionCount > 1);

const contextLabel = (single: string, multiple: string) => {
  return isMultiSelect.value ? `${multiple}（${props.selectionCount} 项）` : single;
}

const contextMenuButtons = () => {
  const menu = menuRef.value;
  if (!menu) return [];
  return Array.from(menu.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"));
}

const focusContextMenuButton = (index: number) => {
  const buttons = contextMenuButtons();
  if (!buttons.length) return;
  const nextIndex = (index + buttons.length) % buttons.length;
  buttons[nextIndex]?.focus({preventScroll: true});
}

const focusFirstContextMenuButton = async () => {
  await nextTick();
  focusContextMenuButton(0);
}

const placeMenu = async () => {
  menuPosition.value = {x: props.x, y: props.y};
  await nextTick();
  const menu = menuRef.value;
  if (!menu) return;
  const rect = menu.getBoundingClientRect();
  const maxX = Math.max(viewportPadding, window.innerWidth - rect.width - viewportPadding);
  const maxY = Math.max(viewportPadding, window.innerHeight - rect.height - viewportPadding);
  menuPosition.value = {
    x: Math.min(Math.max(viewportPadding, props.x), maxX),
    y: Math.min(Math.max(viewportPadding, props.y), maxY)
  };
}

const refreshMenu = async () => {
  await placeMenu();
  await focusFirstContextMenuButton();
}

const moveContextMenuFocus = (direction: -1 | 1) => {
  const buttons = contextMenuButtons();
  if (!buttons.length) return;
  const currentIndex = buttons.findIndex(button => button === document.activeElement);
  focusContextMenuButton(currentIndex < 0 ? 0 : currentIndex + direction);
}

const handleKeyDown = (event: KeyboardEvent) => {
  if (event.key === "Escape") {
    event.preventDefault();
    emit("escape");
    return;
  }
  if (event.key === "ArrowDown") {
    event.preventDefault();
    moveContextMenuFocus(1);
    return;
  }
  if (event.key === "ArrowUp") {
    event.preventDefault();
    moveContextMenuFocus(-1);
    return;
  }
  if (event.key === "Home") {
    event.preventDefault();
    focusContextMenuButton(0);
    return;
  }
  if (event.key === "End") {
    event.preventDefault();
    focusContextMenuButton(contextMenuButtons().length - 1);
    return;
  }
  if (event.key === "Tab") {
    event.preventDefault();
    moveContextMenuFocus(event.shiftKey ? -1 : 1);
  }
}

onMounted(() => {
  void refreshMenu();
  window.addEventListener("resize", placeMenu);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", placeMenu);
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
        @keydown="handleKeyDown">
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
  @apply block h-8 w-full px-3 text-left text-slate-700 hover:bg-blue-50 disabled:text-slate-300 disabled:hover:bg-white;
}

.context-menu button:focus-visible {
  @apply bg-blue-50 text-blue-700 outline-none ring-1 ring-inset ring-blue-300;
}

.context-separator {
  @apply my-1 border-t border-slate-100;
}

.context-menu .danger {
  @apply text-red-600 hover:bg-red-50;
}
</style>
