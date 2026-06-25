<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerTab} from "../../class";
import type {TabContextMenuState, TabDropPlacement} from "./types.ts";
import type {ExplorerEntry, ExplorerEntryPathDropPayload} from "../explorer/types.ts";
import Icon from "../Icon.vue";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import {useViewportMenuPosition} from "../../composables/useViewportMenuPosition.ts";
import {hasInternalEntryDragData, readInternalEntryDragData} from "../../utils/internal-entry-drag.ts";
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
const tabStripRef = ref<HTMLElement | null>(null);
const tabScrollRef = ref<HTMLElement | null>(null);
const contextMenuRef = ref<HTMLElement | null>(null);
const tabStripWidth = ref(0);
const canScrollStart = ref(false);
const canScrollEnd = ref(false);
const entryDropTargetTabId = ref("");
const addButtonDropTarget = ref(false);
const {menuPosition: contextMenuPosition, placeMenu: placeContextMenu} = useViewportMenuPosition({menuRef: contextMenuRef});
let tabScrollResizeObserver: ResizeObserver | null = null;
let tabOverflowFrame = 0;
let tabOverflowRefreshTimer = 0;

const tabIdealWidth = 208;
const tabMinWidth = 116;
const tabGap = 8;
const tabAddWidth = 36;
const tabStripPaddingX = 8;

const tabWidth = computed(() => {
  const count = Math.max(1, props.tabs.length);
  if (!tabStripWidth.value) return tabIdealWidth;
  const availableWidth = Math.max(0, tabStripWidth.value - tabStripPaddingX - tabAddWidth - tabGap);
  const fittedWidth = (availableWidth - Math.max(0, count - 1) * tabGap) / count;
  return Math.round(Math.max(tabMinWidth, Math.min(tabIdealWidth, fittedWidth)));
});

const tabListStyle = computed(() => ({
  "--tab-width": `${tabWidth.value}px`
}));

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

const focusTabButton = async (tabId: string) => {
  await nextTick();
  const button = tabButtonRefs.get(tabId);
  button?.focus({preventScroll: true});
  button?.scrollIntoView({block: "nearest", inline: "nearest"});
}

const updateTabMetrics = () => {
  tabStripWidth.value = tabStripRef.value?.clientWidth ?? 0;
}

const updateTabOverflow = () => {
  updateTabMetrics();
  const scroll = tabScrollRef.value;
  if (!scroll) {
    canScrollStart.value = false;
    canScrollEnd.value = false;
    return;
  }
  const maxScrollLeft = Math.max(0, scroll.scrollWidth - scroll.clientWidth);
  const idealListWidth = props.tabs.length * tabIdealWidth + Math.max(0, props.tabs.length - 1) * tabGap;
  const canFitAtIdealWidth = idealListWidth <= scroll.clientWidth + 1;
  canScrollStart.value = !canFitAtIdealWidth && scroll.scrollLeft > 1;
  canScrollEnd.value = !canFitAtIdealWidth && maxScrollLeft > 1 && scroll.scrollLeft < maxScrollLeft - 1;
}

const scheduleTabOverflowUpdate = () => {
  if (tabOverflowFrame) window.cancelAnimationFrame(tabOverflowFrame);
  tabOverflowFrame = window.requestAnimationFrame(() => {
    tabOverflowFrame = 0;
    updateTabOverflow();
  });
}

const scheduleTabOverflowRefresh = () => {
  scheduleTabOverflowUpdate();
  if (tabOverflowRefreshTimer) window.clearTimeout(tabOverflowRefreshTimer);
  tabOverflowRefreshTimer = window.setTimeout(() => {
    tabOverflowRefreshTimer = 0;
    scheduleTabOverflowUpdate();
  }, 230);
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
  scheduleTabOverflowRefresh();
}, {immediate: true});

watch(() => props.tabs.map(tab => `${tab.id}:${tab.title}`).join("|"), async () => {
  await nextTick();
  scheduleTabOverflowRefresh();
}, {flush: "post", immediate: true});

watch(() => [props.contextMenu.visible, props.contextMenu.tabId, props.contextMenu.x, props.contextMenu.y] as const, ([visible]) => {
  if (visible) void refreshContextMenu();
}, {flush: "post"});

watch(() => props.contextMenu.visible, async (visible, wasVisible) => {
  if (visible || !wasVisible || !props.contextMenu.tabId) return;
  await focusTabButton(props.contextMenu.tabId);
});

const handleWindowResize = () => {
  scheduleTabOverflowUpdate();
  if (props.contextMenu.visible) void refreshContextMenu();
}

const handleTabScroll = () => {
  scheduleTabOverflowUpdate();
}

const handleTabWheel = (event: WheelEvent) => {
  scrollHorizontallyWithWheel(event);
  scheduleTabOverflowUpdate();
}

const clearEntryDropTarget = () => {
  if (entryDropTargetTabId.value) emit("entry-drag-leave-tab", entryDropTargetTabId.value);
  entryDropTargetTabId.value = "";
  addButtonDropTarget.value = false;
}

watch(() => props.activeTabId, () => {
  clearEntryDropTarget();
});

onMounted(() => {
  window.addEventListener("resize", handleWindowResize);
  document.addEventListener("dragend", clearEntryDropTarget, true);
  document.addEventListener("drop", clearEntryDropTarget, true);
  updateTabMetrics();
  if (typeof ResizeObserver !== "undefined") {
    tabScrollResizeObserver = new ResizeObserver(scheduleTabOverflowUpdate);
    if (tabStripRef.value) tabScrollResizeObserver.observe(tabStripRef.value);
    if (tabScrollRef.value) tabScrollResizeObserver.observe(tabScrollRef.value);
  }
  scheduleTabOverflowUpdate();
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", handleWindowResize);
  document.removeEventListener("dragend", clearEntryDropTarget, true);
  document.removeEventListener("drop", clearEntryDropTarget, true);
  tabScrollResizeObserver?.disconnect();
  if (tabOverflowFrame) window.cancelAnimationFrame(tabOverflowFrame);
  if (tabOverflowRefreshTimer) window.clearTimeout(tabOverflowRefreshTimer);
});

const emit = defineEmits<{
  (e: "new-tab"): void;
  (e: "activate-tab", tabId: string): void;
  (e: "close-tab", event: MouseEvent, tabId: string): void;
  (e: "tab-aux-click", event: MouseEvent, tabId: string): void;
  (e: "tab-context-menu", event: MouseEvent, tabId: string): void;
  (e: "tab-keyboard-context-menu", payload: {x: number; y: number; tabId: string}): void;
  (e: "tab-drag-start", event: DragEvent, tabId: string): void;
  (e: "tab-drag-over", event: DragEvent, tabId: string): void;
  (e: "tab-drag-leave", event: DragEvent, tabId: string): void;
  (e: "tab-drop", event: DragEvent, tabId: string): void;
  (e: "tab-drag-end"): void;
  (e: "drop-entries", payload: ExplorerEntryPathDropPayload): void;
  (e: "entry-drag-hover-tab", tabId: string): void;
  (e: "entry-drag-leave-tab", tabId: string): void;
  (e: "open-entry-new-tab", entry: ExplorerEntry): void;
  (e: "duplicate-tab"): void;
  (e: "close-context-tab"): void;
  (e: "reopen-closed-tab"): void;
  (e: "close-other-tabs"): void;
  (e: "close-right-tabs"): void;
  (e: "close-context-menu"): void;
}>();

const isCopyEntryDrop = (event: DragEvent) => Boolean(event.ctrlKey || event.metaKey);

const tabIndexById = (tabId: string) => props.tabs.findIndex(tab => tab.id === tabId);

const focusTabByOffset = async (tabId: string, offset: number) => {
  if (!props.tabs.length) return;
  const currentIndex = tabIndexById(tabId);
  const startIndex = currentIndex >= 0 ? currentIndex : 0;
  const nextIndex = Math.max(0, Math.min(props.tabs.length - 1, startIndex + offset));
  const nextTab = props.tabs[nextIndex];
  if (nextTab) await focusTabButton(nextTab.id);
}

const openKeyboardContextMenu = (tabId: string) => {
  const button = tabButtonRefs.get(tabId);
  const rect = button?.getBoundingClientRect();
  emit("tab-keyboard-context-menu", {
    x: rect ? rect.left + Math.min(rect.width - 8, 28) : window.innerWidth / 2,
    y: rect ? rect.bottom - 4 : window.innerHeight / 2,
    tabId
  });
}

const handleTabKeyDown = async (event: KeyboardEvent, tab: ExplorerTab) => {
  if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
    event.preventDefault();
    openKeyboardContextMenu(tab.id);
    return;
  }
  if (event.key === "ArrowRight") {
    event.preventDefault();
    await focusTabByOffset(tab.id, 1);
    return;
  }
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    await focusTabByOffset(tab.id, -1);
    return;
  }
  if (event.key === "Home") {
    event.preventDefault();
    const firstTab = props.tabs[0];
    if (firstTab) await focusTabButton(firstTab.id);
    return;
  }
  if (event.key === "End") {
    event.preventDefault();
    const lastTab = props.tabs[props.tabs.length - 1];
    if (lastTab) await focusTabButton(lastTab.id);
    return;
  }
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    emit("activate-tab", tab.id);
  }
}

const canAcceptEntryDrop = (event: DragEvent) => {
  if (props.draggingTabId) return false;
  return hasInternalEntryDragData(event.dataTransfer);
}

const handleTabDragOver = (event: DragEvent, tab: ExplorerTab) => {
  if (!canAcceptEntryDrop(event)) {
    emit("tab-drag-over", event, tab.id);
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  entryDropTargetTabId.value = tab.id;
  emit("entry-drag-hover-tab", tab.id);
  if (event.dataTransfer) event.dataTransfer.dropEffect = isCopyEntryDrop(event) ? "copy" : "move";
}

const handleTabDragLeave = (event: DragEvent, tab: ExplorerTab) => {
  if (entryDropTargetTabId.value !== tab.id) {
    emit("tab-drag-leave", event, tab.id);
    return;
  }
  const related = event.relatedTarget;
  if (related instanceof Node && event.currentTarget instanceof HTMLElement && event.currentTarget.contains(related)) return;
  entryDropTargetTabId.value = "";
  emit("entry-drag-leave-tab", tab.id);
}

const handleTabDrop = (event: DragEvent, tab: ExplorerTab) => {
  if (!canAcceptEntryDrop(event)) {
    emit("tab-drop", event, tab.id);
    return;
  }
  const entries = readInternalEntryDragData(event.dataTransfer);
  if (!entries.length) {
    entryDropTargetTabId.value = "";
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  entryDropTargetTabId.value = "";
  emit("entry-drag-leave-tab", tab.id);
  emit("drop-entries", {
    entries,
    target: {
      path: tab.path,
      name: tab.title
    },
    action: isCopyEntryDrop(event) ? "copy" : "move"
  });
}

const handleAddDragOver = (event: DragEvent) => {
  if (!canAcceptEntryDrop(event)) return;
  event.preventDefault();
  event.stopPropagation();
  addButtonDropTarget.value = true;
  if (event.dataTransfer) event.dataTransfer.dropEffect = "copy";
}

const handleAddDragLeave = (event: DragEvent) => {
  if (!addButtonDropTarget.value) return;
  const related = event.relatedTarget;
  if (related instanceof Node && event.currentTarget instanceof HTMLElement && event.currentTarget.contains(related)) return;
  addButtonDropTarget.value = false;
}

const handleAddDrop = (event: DragEvent) => {
  if (!canAcceptEntryDrop(event)) return;
  const [entry] = readInternalEntryDragData(event.dataTransfer);
  if (!entry) {
    addButtonDropTarget.value = false;
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  addButtonDropTarget.value = false;
  emit("open-entry-new-tab", entry);
}
</script>

<template>
  <nav ref="tabStripRef" class="tab-strip" aria-label="目录标签">
    <div
        class="tab-scroll-frame"
        :class="{canScrollStart, canScrollEnd}">
      <div
          ref="tabScrollRef"
          class="tab-scroll"
          @scroll="handleTabScroll"
          @wheel="handleTabWheel">
        <transition-group name="tab-motion" tag="div" class="tab-list" :style="tabListStyle">
          <button
              v-for="tab in tabs"
              :key="tab.id"
              :ref="element => setTabButtonRef(tab.id, element)"
              class="tab-button"
              :class="{
                active: tab.id === activeTabId,
                dragging: draggingTabId === tab.id,
                dropBefore: dropTargetId === tab.id && dropPlacement === 'before',
                dropAfter: dropTargetId === tab.id && dropPlacement === 'after',
                entryDropTarget: entryDropTargetTabId === tab.id
              }"
              :title="`${tab.path} · 中键关闭`"
              draggable="true"
              @click="emit('activate-tab', tab.id)"
              @mousedown.middle.prevent.stop="emit('tab-aux-click', $event, tab.id)"
              @auxclick="emit('tab-aux-click', $event, tab.id)"
              @contextmenu="emit('tab-context-menu', $event, tab.id)"
              @keydown="handleTabKeyDown($event, tab)"
              @dragstart="emit('tab-drag-start', $event, tab.id)"
              @dragover="handleTabDragOver($event, tab)"
              @dragleave="handleTabDragLeave($event, tab)"
              @drop="handleTabDrop($event, tab)"
              @dragend="emit('tab-drag-end')">
            <icon icon="file.folder" />
            <span>{{ tab.title }}</span>
            <span class="tab-close" title="关闭标签页 (Ctrl+W)" @click="emit('close-tab', $event, tab.id)">
              <icon icon="action.close" size="small" />
            </span>
          </button>
        </transition-group>
      </div>
    </div>
    <button
        :class="['tab-add', {entryDropTarget: addButtonDropTarget}]"
        title="新建标签页 (Ctrl+T)"
        @click="emit('new-tab')"
        @dragover="handleAddDragOver"
        @dragleave="handleAddDragLeave"
        @drop="handleAddDrop">
      <icon icon="action.add" />
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
  @apply flex h-full min-w-0 grow items-center gap-2 overflow-hidden rounded-xl border p-1 shadow-sm backdrop-blur;
  border-color: var(--app-border);
  background: var(--app-panel);
}

.tab-scroll-frame {
  @apply relative h-full min-w-0 overflow-hidden;
  flex: 0 1 auto;
  max-width: max(0px, calc(100% - 2.75rem));
}

.tab-scroll-frame::before,
.tab-scroll-frame::after {
  content: "";
  @apply pointer-events-none absolute bottom-0 top-0 z-10 w-7 opacity-0 transition-opacity duration-150;
}

.tab-scroll-frame::before {
  @apply left-0;
  background: linear-gradient(90deg, var(--app-panel) 0%, color-mix(in srgb, var(--app-panel) 72%, transparent) 42%, transparent 100%);
}

.tab-scroll-frame::after {
  @apply right-0;
  background: linear-gradient(270deg, var(--app-panel) 0%, color-mix(in srgb, var(--app-panel) 72%, transparent) 42%, transparent 100%);
}

.tab-scroll-frame.canScrollStart::before,
.tab-scroll-frame.canScrollEnd::after {
  @apply opacity-100;
}

.tab-scroll {
  @apply h-full min-w-0 overflow-x-auto overflow-y-hidden;
  overscroll-behavior-x: contain;
  scrollbar-width: none;
  scrollbar-gutter: stable;
}

.tab-scroll::-webkit-scrollbar {
  display: none;
}

.tab-list {
  @apply relative flex h-full items-center gap-2;
  width: max-content;
}

.tab-button {
  @apply relative inline-flex h-9 shrink-0 items-center gap-2 rounded-lg border px-3 text-sm shadow-sm;
  width: var(--tab-width, 10.5rem);
  min-width: var(--tab-width, 10.5rem);
  max-width: var(--tab-width, 10.5rem);
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text);
  transition:
      width 0.16s ease,
      min-width 0.16s ease,
      max-width 0.16s ease,
      opacity 0.16s ease,
      transform 0.18s cubic-bezier(0.2, 0.8, 0.2, 1),
      border-color 0.14s ease,
      background 0.14s ease,
      color 0.14s ease,
      box-shadow 0.14s ease;
}

.tab-button:hover {
  background: var(--app-control-hover);
}

.tab-button:focus-visible,
.tab-add:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.tab-button.active {
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.tab-button.dragging {
  @apply opacity-55;
}

.tab-button.dropBefore,
.tab-button.dropAfter {
  background: var(--app-accent-soft, #eff6ff);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.tab-button.entryDropTarget,
.tab-add.entryDropTarget {
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.tab-button.active.dropBefore,
.tab-button.active.dropAfter,
.tab-button.active.entryDropTarget {
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.tab-button.active:focus-visible {
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22)), inset 0 0 0 1px var(--app-accent-contrast);
}

.tab-button.dropBefore::before,
.tab-button.dropAfter::after {
  content: "";
  @apply absolute bottom-1 top-1 w-0.5 rounded-full;
  background: var(--app-accent, #2563eb);
}

.tab-button.dropBefore::before {
  @apply left-1;
}

.tab-button.dropAfter::after {
  @apply right-1;
}

.tab-button.active.dropBefore::before,
.tab-button.active.dropAfter::after {
  background: var(--app-accent-contrast);
}

.tab-button span:not(.tab-close) {
  @apply min-w-0 truncate;
}

.tab-close {
  @apply ml-auto inline-flex h-5 w-5 shrink-0 items-center justify-center rounded hover:bg-black/10;
}

.tab-add {
  @apply inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border shadow-sm;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
  transition: border-color 0.14s ease, background 0.14s ease, color 0.14s ease, transform 0.16s ease, box-shadow 0.14s ease;
}

.tab-add:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.tab-add:active {
  transform: scale(0.96);
}

.tab-motion-enter-active,
.tab-motion-leave-active {
  transition:
      opacity 0.16s ease,
      transform 0.18s cubic-bezier(0.2, 0.8, 0.2, 1);
}

.tab-motion-enter-from {
  opacity: 0;
  transform: translateY(6px) scale(0.96);
}

.tab-motion-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.96);
}

.tab-motion-leave-active {
  @apply absolute;
}

.tab-motion-move {
  transition: transform 0.2s cubic-bezier(0.2, 0.8, 0.2, 1);
}

.tab-context-menu {
  @apply fixed z-50 w-46 rounded-md border py-1 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.tab-context-menu button {
  @apply block h-8 w-full px-3 text-left;
  color: var(--app-text-muted);
}

.tab-context-menu button:disabled {
  color: var(--app-text-disabled);
}

.tab-context-menu button:disabled:hover {
  background: var(--app-panel-solid);
}

.tab-context-menu button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.tab-context-menu button:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.tab-context-separator {
  @apply my-1 border-t;
  border-color: var(--app-border-soft);
}
</style>
