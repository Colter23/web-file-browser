<script setup lang="ts">
import {nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerTab} from "../../class";
import type {TabContextMenuState, TabDropPlacement} from "./types.ts";
import Icon from "../Icon.vue";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import {useViewportMenuPosition} from "../../composables/useViewportMenuPosition.ts";
import {scrollHorizontallyWithWheel} from "../../utils/wheel.ts";

const props = defineProps<{
  tabs: ExplorerTab[];
  activeTabId: string;
  draggingTabId: string;
  dropTargetId: string;
  dropPlacement: TabDropPlacement | "";
  contextMenu: TabContextMenuState;
  contextTarget: ExplorerTab | null;
  canCloseTab: boolean;
  canCloseOtherTabs: boolean;
  canCloseRightTabs: boolean;
  canReopenClosedTab: boolean;
}>();

const tabButtonRefs = new Map<string, HTMLElement>();
const contextMenuRef = ref<HTMLElement | null>(null);
const {menuPosition: contextMenuPosition, placeMenu: placeContextMenu} = useViewportMenuPosition({menuRef: contextMenuRef});

const setTabButtonRef = (tabId: string, element: unknown) => {
  if (element instanceof HTMLElement) {
    tabButtonRefs.set(tabId, element);
  } else {
    tabButtonRefs.delete(tabId);
  }
}

const revealActiveTab = async () => {
  await nextTick();
  tabButtonRefs.get(props.activeTabId)?.scrollIntoView({block: "nearest", inline: "nearest"});
}

const {
  focusFirstMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef: contextMenuRef,
  onEscape: () => emit("close-context-menu")
});

const refreshContextMenu = async () => {
  await placeContextMenu({x: props.contextMenu.x, y: props.contextMenu.y});
  await focusFirstMenuButton();
}

useOutsidePointerDown({
  refs: [contextMenuRef],
  enabled: () => props.contextMenu.visible,
  onOutsidePointerDown: () => emit("close-context-menu")
});

watch(() => [props.activeTabId, props.tabs.length] as const, () => {
  void revealActiveTab();
}, {immediate: true});

watch(() => [props.contextMenu.visible, props.contextMenu.tabId, props.contextMenu.x, props.contextMenu.y] as const, ([visible]) => {
  if (visible) void refreshContextMenu();
}, {flush: "post"});

const handleWindowResize = () => {
  if (props.contextMenu.visible) void refreshContextMenu();
}

onMounted(() => {
  window.addEventListener("resize", handleWindowResize);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", handleWindowResize);
});

const emit = defineEmits<{
  (e: "new-tab"): void;
  (e: "activate-tab", tabId: string): void;
  (e: "close-tab", event: MouseEvent, tabId: string): void;
  (e: "tab-aux-click", event: MouseEvent, tabId: string): void;
  (e: "tab-context-menu", event: MouseEvent, tabId: string): void;
  (e: "tab-drag-start", event: DragEvent, tabId: string): void;
  (e: "tab-drag-over", event: DragEvent, tabId: string): void;
  (e: "tab-drag-leave", event: DragEvent, tabId: string): void;
  (e: "tab-drop", event: DragEvent, tabId: string): void;
  (e: "tab-drag-end"): void;
  (e: "duplicate-tab"): void;
  (e: "close-context-tab"): void;
  (e: "reopen-closed-tab"): void;
  (e: "close-other-tabs"): void;
  (e: "close-right-tabs"): void;
  (e: "close-context-menu"): void;
}>();
</script>

<template>
  <nav class="tab-strip" aria-label="目录标签">
    <div class="tab-scroll" @wheel="scrollHorizontallyWithWheel">
      <button
          v-for="tab in tabs"
          :key="tab.id"
          :ref="element => setTabButtonRef(tab.id, element)"
          class="tab-button"
          :class="{
            active: tab.id === activeTabId,
            dragging: draggingTabId === tab.id,
            dropBefore: dropTargetId === tab.id && dropPlacement === 'before',
            dropAfter: dropTargetId === tab.id && dropPlacement === 'after'
          }"
          :title="`${tab.path} · Ctrl+Tab 切换 · 中键关闭`"
          draggable="true"
          @click="emit('activate-tab', tab.id)"
          @auxclick="emit('tab-aux-click', $event, tab.id)"
          @contextmenu="emit('tab-context-menu', $event, tab.id)"
          @dragstart="emit('tab-drag-start', $event, tab.id)"
          @dragover="emit('tab-drag-over', $event, tab.id)"
          @dragleave="emit('tab-drag-leave', $event, tab.id)"
          @drop="emit('tab-drop', $event, tab.id)"
          @dragend="emit('tab-drag-end')">
        <icon icon="icon-folder-fill" />
        <span>{{ tab.title }}</span>
        <span class="tab-close" title="关闭标签页 (Ctrl+W)" @click="emit('close-tab', $event, tab.id)">
          <icon icon="icon-close" size="small" />
        </span>
      </button>
    </div>
    <button class="tab-add" title="新建标签页 (Ctrl+T)" @click="emit('new-tab')">
      <icon icon="icon-add" />
    </button>
  </nav>

  <div
      v-if="contextMenu.visible"
      ref="contextMenuRef"
      class="tab-context-menu"
      :style="{left: `${contextMenuPosition.x}px`, top: `${contextMenuPosition.y}px`}"
      role="menu"
      aria-label="标签页菜单"
      @click.stop
      @contextmenu.prevent
      @keydown="handleMenuKeyDown">
    <button role="menuitem" @click="emit('new-tab')">新建标签页</button>
    <button role="menuitem" :disabled="!canReopenClosedTab" @click="emit('reopen-closed-tab')">重新打开关闭的标签页</button>
    <button role="menuitem" :disabled="!contextTarget" @click="emit('duplicate-tab')">复制标签页</button>
    <div class="tab-context-separator"></div>
    <button role="menuitem" :disabled="!canCloseTab" @click="emit('close-context-tab')">关闭标签页</button>
    <button role="menuitem" :disabled="!canCloseOtherTabs" @click="emit('close-other-tabs')">关闭其他标签页</button>
    <button role="menuitem" :disabled="!canCloseRightTabs" @click="emit('close-right-tabs')">关闭右侧标签页</button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.tab-strip {
  @apply flex min-w-0 grow items-center gap-2 overflow-hidden rounded-xl border border-white bg-white/70 p-1 shadow-sm backdrop-blur;
}

.tab-scroll {
  @apply flex min-w-0 grow items-center gap-2 overflow-x-auto overflow-y-hidden;
  scrollbar-width: thin;
  scrollbar-color: rgb(203 213 225) transparent;
}

.tab-scroll::-webkit-scrollbar {
  height: 0.45rem;
}

.tab-scroll::-webkit-scrollbar-track {
  @apply bg-transparent;
}

.tab-scroll::-webkit-scrollbar-thumb {
  @apply rounded-full bg-slate-300;
}

.tab-button {
  @apply relative inline-flex h-9 min-w-32 max-w-52 shrink-0 items-center gap-2 rounded-lg border border-slate-200 bg-white px-3 text-sm text-slate-800 shadow-sm hover:bg-slate-50;
}

.tab-button.active {
  @apply border-blue-600 bg-blue-600 text-white;
}

.tab-button.dragging {
  @apply opacity-55;
}

.tab-button.dropBefore,
.tab-button.dropAfter {
  @apply bg-blue-50 ring-2 ring-blue-200;
}

.tab-button.active.dropBefore,
.tab-button.active.dropAfter {
  @apply bg-blue-600 ring-blue-200;
}

.tab-button.dropBefore::before,
.tab-button.dropAfter::after {
  content: "";
  @apply absolute bottom-1 top-1 w-0.5 rounded-full bg-blue-500;
}

.tab-button.dropBefore::before {
  @apply left-1;
}

.tab-button.dropAfter::after {
  @apply right-1;
}

.tab-button.active.dropBefore::before,
.tab-button.active.dropAfter::after {
  @apply bg-white;
}

.tab-button span:not(.tab-close) {
  @apply min-w-0 truncate;
}

.tab-close {
  @apply ml-auto inline-flex h-5 w-5 shrink-0 items-center justify-center rounded hover:bg-black/10;
}

.tab-add {
  @apply inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-white text-slate-700 shadow-sm hover:bg-blue-50;
}

.tab-context-menu {
  @apply fixed z-50 w-46 rounded-md border border-slate-200 bg-white py-1 text-sm shadow-xl;
}

.tab-context-menu button {
  @apply block h-8 w-full px-3 text-left text-slate-700 hover:bg-blue-50 disabled:text-slate-300 disabled:hover:bg-white;
}

.tab-context-menu button:focus-visible {
  @apply bg-blue-50 text-blue-700 outline-none ring-1 ring-inset ring-blue-300;
}

.tab-context-separator {
  @apply my-1 border-t border-slate-100;
}
</style>
