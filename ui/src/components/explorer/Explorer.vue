<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ComponentPublicInstance} from "vue";
import type {DirSortKey, DirSortOrder} from "../../class.ts";
import {useFileStore} from "../../store";
import {useDetailsColumns} from "../../composables/useDetailsColumns.ts";
import {useExplorerContextMenu} from "../../composables/useExplorerContextMenu.ts";
import {useExplorerEntryActions} from "../../composables/useExplorerEntryActions.ts";
import {useExplorerEntryDrag} from "../../composables/useExplorerEntryDrag.ts";
import {useExplorerFolderData} from "../../composables/useExplorerFolderData.ts";
import {useExplorerKeyboard} from "../../composables/useExplorerKeyboard.ts";
import {useExplorerMarqueeSelection} from "../../composables/useExplorerMarqueeSelection.ts";
import {useExplorerRename} from "../../composables/useExplorerRename.ts";
import {useExplorerSelection} from "../../composables/useExplorerSelection.ts";
import {useExplorerStatusText} from "../../composables/useExplorerStatusText.ts";
import {useExplorerThumbnails} from "../../composables/useExplorerThumbnails.ts";
import {useExplorerTypeahead} from "../../composables/useExplorerTypeahead.ts";
import {useExplorerViewDensity} from "../../composables/useExplorerViewDensity.ts";
import {
  entryTypeText,
  fileEntryIcon,
  formatEntryDate as formatDate,
  formatEntrySize as formatSize,
  isExtractableArchiveEntry as canExtract,
  isImageEntry as isImageFile
} from "../../utils/file-entry.ts";
import DetailsHeader from "./DetailsHeader.vue";
import ExplorerContextMenu from "./ExplorerContextMenu.vue";
import ExplorerCommandRow from "./ExplorerCommandRow.vue";
import ExplorerEmptyState from "./ExplorerEmptyState.vue";
import ExplorerStatusBar from "./ExplorerStatusBar.vue";
import ExplorerEntryItem from "./ExplorerEntryItem.vue";
import type {ExplorerEntry} from "./types.ts";

type RenamePayload = {
  entry: ExplorerEntry;
  name: string;
}

type DropEntriesPayload = {
  entries: ExplorerEntry[];
  target: ExplorerEntry;
  action: "copy" | "move";
}

type DropToCurrentFolderPayload = {
  entries: ExplorerEntry[];
  action: "copy" | "move";
}

type ImageViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type CopyPathPayload = {
  paths: string[];
}

const emit = defineEmits<{
  (e: "rename", payload: RenamePayload): void;
  (e: "delete", entry: ExplorerEntry): void;
  (e: "download", entry: ExplorerEntry): void;
  (e: "archive", entry: ExplorerEntry): void;
  (e: "extract", entry: ExplorerEntry): void;
  (e: "properties", entries: ExplorerEntry[]): void;
  (e: "preview", entry: ExplorerEntry): void;
  (e: "open-image-viewer", payload: ImageViewerPayload): void;
  (e: "copy", entry: ExplorerEntry): void;
  (e: "cut", entry: ExplorerEntry): void;
  (e: "paste"): void;
  (e: "create-file"): void;
  (e: "create-folder"): void;
  (e: "drop-entries", payload: DropEntriesPayload): void;
  (e: "drop-to-current-folder", payload: DropToCurrentFolderPayload): void;
  (e: "open-new-tab", entry: ExplorerEntry): void;
  (e: "copy-path", payload: CopyPathPayload): void;
  (e: "selection-change", entries: ExplorerEntry[]): void;
  (e: "clear-filter"): void;
  (e: "scroll-change", scrollTop: number): void;
}>()

const props = withDefaults(defineProps<{
  filterText?: string;
  dimmedPaths?: string[];
  canPaste?: boolean;
}>(), {
  filterText: "",
  dimmedPaths: () => [],
  canPaste: false
})

const fileStore = useFileStore();
const viewportRef = ref<HTMLElement | null>(null);
const itemRefs = new Map<string, HTMLElement>();
let closeContextMenuHandler = () => {};
const closeContextMenu = () => closeContextMenuHandler();

const {
  gridStyle: detailsGridStyle,
  startResize: startDetailsColumnResize,
  handleResizeMove: handleDetailsColumnResizeMove,
  finishResize: finishDetailsColumnResize
} = useDetailsColumns();

const {
  folderData,
  loading,
  loadingMore,
  message,
  allEntries,
  filterKeyword,
  entries,
  loadFolder: loadFolderData,
  loadMore: loadMoreData,
  maybeLoadMoreOnScroll,
  isLoadedFor,
  markStale
} = useExplorerFolderData({
  filterText: () => props.filterText,
  viewportRef
});

const {
  selectedPaths,
  focusedPath,
  anchorPath,
  selectedEntries,
  entryByPath,
  firstSelectedEntry,
  focusedOrSelectedEntry,
  isSelected,
  indexOfPath,
  setSelection,
  clearSelection,
  ensureFocusAnchor,
  selectRange,
  selectEntry,
  toggleFocusedSelection,
  ensureEntrySelected,
  selectAllEntries,
  clearCurrentSelection,
  invertCurrentSelection,
  moveFocus,
  focusEntryByTypeahead,
  selectPath,
  selectPaths,
  captureSelectionSnapshot,
  restoreSelectionSnapshot
} = useExplorerSelection({
  entries,
  itemRefs,
  focusViewport: () => focusViewport(),
  closeContextMenu,
  currentColumns: () => currentColumns(),
  currentPageStep: columns => currentPageStep(columns)
});

const hasMoreEntries = computed(() => Boolean(folderData.value.hasMore));

const {
  filterActive,
  emptyText,
  emptyHintText,
  selectedCountText,
  totalCountText,
  folderStatusText,
  selectedStatusText
} = useExplorerStatusText({
  allEntries,
  entries,
  selectedEntries,
  filterKeyword,
  hasMore: hasMoreEntries
});

watch(selectedEntries, selected => {
  emit("selection-change", selected);
}, {immediate: true});

const viewMode = computed(() => fileStore.viewMode);
const isIconLikeMode = computed(() => fileStore.viewMode === "icons" || fileStore.viewMode === "tiles");
const sortKey = computed(() => fileStore.sortKey);
const sortOrder = computed(() => fileStore.sortOrder);
const sortOptions: {key: DirSortKey; label: string}[] = [
  {key: "name", label: "名称"},
  {key: "modified", label: "修改"},
  {key: "size", label: "大小"}
];

const itemSizeClass = computed(() => ({
  small: fileStore.iconSize === "small",
  medium: fileStore.iconSize === "medium",
  large: fileStore.iconSize === "large"
}));

const iconSizeText = computed(() => ({
  small: "小图标",
  medium: "中图标",
  large: "大图标"
}[fileStore.iconSize]));

const sortText = computed(() => {
  const keyText = sortOptions.find(option => option.key === fileStore.sortKey)?.label ?? "名称";
  const orderText = fileStore.sortOrder === "asc" ? "升序" : "降序";
  return `${keyText} ${orderText}`;
});

const nextSortOrder = computed<DirSortOrder>(() => fileStore.sortOrder === "asc" ? "desc" : "asc");

const imageEntries = computed(() => entries.value.filter(isImageFile));
const {
  shouldLoad: shouldLoadThumbnail,
  thumbnailUrl,
  handleError: handleThumbnailError,
  observe: observeThumbnail,
  observePending: observePendingThumbnails,
  unobserve: unobserveThumbnail,
  clearState: clearThumbnailState,
  disconnectObserver: disconnectThumbnailObserver
} = useExplorerThumbnails({
  entries,
  itemRefs,
  viewportRef,
  active: isIconLikeMode,
  isImageFile
});

const resolveElementRef = (element: Element | ComponentPublicInstance | null) => {
  if (element instanceof HTMLElement) return element;
  if (element && "$el" in element && element.$el instanceof HTMLElement) return element.$el;
  return null;
}

const setItemRef = (path: string, element: Element | ComponentPublicInstance | null) => {
  const target = resolveElementRef(element);
  const current = itemRefs.get(path);
  if (current && current !== target) unobserveThumbnail(path);
  if (target) {
    itemRefs.set(path, target);
    const entry = entryByPath(path);
    if (entry) observeThumbnail(entry, target);
  } else {
    itemRefs.delete(path);
  }
}

const focusViewport = () => {
  viewportRef.value?.focus({preventScroll: true});
}

const getScrollTop = () => viewportRef.value?.scrollTop ?? 0;

const setScrollTop = async (scrollTop: number) => {
  await nextTick();
  if (!viewportRef.value) return;
  viewportRef.value.scrollTop = Math.max(0, scrollTop);
}

const {
  reset: resetTypeahead,
  handleTypeahead
} = useExplorerTypeahead({
  entries,
  focusedPath,
  indexOfPath,
  focusEntry: focusEntryByTypeahead,
  closeContextMenu
});

const {
  selectionBox,
  resetSelectionBox,
  beginMarqueeSelection,
  handleSelectionMove,
  finishMarqueeSelection,
  stopAutoScroll: stopMarqueeAutoScroll
} = useExplorerMarqueeSelection({
  entries,
  selectedPaths,
  focusedPath,
  anchorPath,
  itemRefs,
  viewportRef,
  isRenaming: () => Boolean(renamingPath.value),
  focusViewport,
  clearSelection
});

const entryDomId = (path: string) => `explorer-entry-${encodeURIComponent(path).replace(/[^a-zA-Z0-9_-]/g, "-")}`;

const {
  renamingPath,
  renameDraft,
  renameSubmitting,
  setRenameInputRef,
  startRename,
  cancelRename,
  commitRename,
  isRenaming,
  resetRename,
  clearRenameInputRefs
} = useExplorerRename({
  entryByPath,
  ensureEntrySelected,
  closeContextMenu,
  focusViewport,
  submitRename: payload => emit("rename", payload)
});

const {
  canEditEntry,
  editEntry,
  openEntry,
  openEntryInNewTab,
  copySelectedPaths
} = useExplorerEntryActions({
  currentPath: () => fileStore.currentPath || "/",
  editableExtensions: () => fileStore.extensions,
  selectedEntries,
  imageEntries,
  isRenaming,
  requestEditorLeave: () => fileStore.requestEditorLeave(),
  openEditor: file => fileStore.openEditor(file),
  loadFolder: path => loadFolder(path),
  previewEntry: entry => emit("preview", entry),
  openImageViewer: payload => emit("open-image-viewer", payload),
  openNewTab: entry => emit("open-new-tab", entry),
  copyPath: payload => emit("copy-path", payload),
  closeContextMenu
});

const {
  dragState,
  dragHintText,
  isDragged,
  isDropTarget,
  beginEntryDrag,
  resetEntryDrag,
  dragOverEntry,
  dragLeaveEntry,
  dropOnEntry,
  dragOverCurrentFolder,
  dragLeaveCurrentFolder,
  dropOnCurrentFolder
} = useExplorerEntryDrag({
  selectedPaths,
  selectedEntries,
  itemRefs,
  viewportRef,
  isSelected,
  isRenaming,
  setSelection,
  focusViewport,
  closeContextMenu,
  dropEntries: (entries, target, action) => emit("drop-entries", {entries, target, action}),
  dropToCurrentFolder: (entries, action) => emit("drop-to-current-folder", {entries, action})
});

const loadFolder = async (path: string = fileStore.currentPath || "/") => {
  return loadFolderData(path, {
    resetBeforeLoad: () => {
      resetRename();
      resetTypeahead();
      resetSelectionBox();
      clearThumbnailState();
    },
    clearSelection,
    afterRender: observePendingThumbnails
  });
}

const loadMore = async () => {
  await loadMoreData({afterRender: observePendingThumbnails});
}

const handleViewportScroll = () => {
  emit("scroll-change", getScrollTop());
  maybeLoadMoreOnScroll({afterRender: observePendingThumbnails});
}

const changeSort = async (key: DirSortKey) => {
  if (loading.value) return;
  const snapshot = captureSelectionSnapshot();
  fileStore.setSort(key);
  markStale();
  if (await loadFolder(fileStore.currentPath || "/")) await restoreSelectionSnapshot(snapshot);
}

const changeSortOrder = async (order: DirSortOrder) => {
  if (loading.value || fileStore.sortOrder === order) return;
  const snapshot = captureSelectionSnapshot();
  fileStore.setSort(fileStore.sortKey, order);
  markStale();
  if (await loadFolder(fileStore.currentPath || "/")) await restoreSelectionSnapshot(snapshot);
}

watch(() => [fileStore.activeTabId, fileStore.currentPath] as const, async ([, path]) => {
  if (!path || fileStore.showEditor) return;
  if (isLoadedFor(path)) return;
  await loadFolder(path);
});

watch(isIconLikeMode, async iconLike => {
  resetTypeahead();
  if (!iconLike) {
    disconnectThumbnailObserver();
    return;
  }
  await nextTick();
  observePendingThumbnails();
});

watch(() => props.filterText, resetTypeahead);

const fileIcon = (entry: ExplorerEntry) => {
  return fileEntryIcon(entry, fileStore.extensions);
}

const isDimmed = (entry: ExplorerEntry) => props.dimmedPaths.includes(entry.path);

const currentColumns = () => {
  if (!viewportRef.value) return 1;
  const first = entries.value[0] ? itemRefs.get(entries.value[0].path) : null;
  if (!first) return 1;
  const firstTop = Math.round(first.getBoundingClientRect().top);
  let columns = 0;
  for (const entry of entries.value) {
    const element = itemRefs.get(entry.path);
    if (!element) break;
    if (Math.abs(Math.round(element.getBoundingClientRect().top) - firstTop) > 2) break;
    columns += 1;
  }
  return Math.max(1, columns);
}

const currentPageStep = (columns: number) => {
  const viewport = viewportRef.value;
  const first = entries.value[0] ? itemRefs.get(entries.value[0].path) : null;
  if (!viewport || !first) return Math.max(1, columns * 5);
  const rowHeight = Math.max(1, first.getBoundingClientRect().height);
  const visibleRows = Math.max(1, Math.floor(viewport.clientHeight / rowHeight) - 1);
  return Math.max(1, visibleRows * columns);
}

const selectPathForRename = async (path: string) => {
  const entry = entryByPath(path);
  if (!entry) return false;
  await selectPath(path);
  startRename(entry);
  return true;
}

const activateViewport = () => {
  focusViewport();
  ensureFocusAnchor();
}

const {
  setViewMode,
  handleViewportWheel,
  cycleIconSize
} = useExplorerViewDensity({
  focusViewport,
  observePendingThumbnails,
  viewportHeight: () => viewportRef.value?.clientHeight ?? 0
});

const primarySelected = () => firstSelectedEntry();


const {
  contextMenu,
  closeContextMenu: closeExplorerContextMenu,
  closeContextMenuAndFocus,
  openContextMenu,
  openBackgroundContextMenu,
  openKeyboardContextMenu,
  contextSelectionCount,
  primaryContextEntry,
  contextCanViewImage,
  contextCanEdit,
  contextCanExtract,
  openEntryFromContext,
  openContextEntryInNewTab,
  previewContextEntry,
  viewImageContextEntry,
  editContextEntry,
  downloadContextEntry,
  copyPathContextEntries,
  copyContextEntries,
  cutContextEntries,
  pasteIntoCurrentFolder,
  createFileFromContext,
  createFolderFromContext,
  selectAllFromContext,
  clearSelectionFromContext,
  invertSelectionFromContext,
  archiveContextEntries,
  extractContextEntry,
  renameContextEntry,
  deleteContextEntries,
  showContextProperties
} = useExplorerContextMenu({
  imageEntries,
  selectedPaths,
  selectedEntries,
  focusedPath,
  viewportRef,
  itemRefs,
  currentPath: () => fileStore.currentPath || "/",
  entryByPath,
  firstSelectedEntry,
  ensureEntrySelected,
  focusViewport,
  openEntry,
  openNewTab: openEntryInNewTab,
  editEntry,
  isImageFile,
  canEditEntry,
  canExtract,
  startRename,
  selectAllEntries,
  clearCurrentSelection,
  invertCurrentSelection,
  previewEntry: entry => emit("preview", entry),
  openImageViewer: payload => emit("open-image-viewer", payload),
  downloadEntry: entry => emit("download", entry),
  copyPath: payload => emit("copy-path", payload),
  copyEntry: entry => emit("copy", entry),
  cutEntry: entry => emit("cut", entry),
  paste: () => emit("paste"),
  createFile: () => emit("create-file"),
  createFolder: () => emit("create-folder"),
  archiveEntry: entry => emit("archive", entry),
  extractEntry: entry => emit("extract", entry),
  deleteEntry: entry => emit("delete", entry),
  showProperties: entries => emit("properties", entries)
});

closeContextMenuHandler = closeExplorerContextMenu;

const {handleKeyDown} = useExplorerKeyboard({
  isViewportActive: () => Boolean(viewportRef.value?.contains(document.activeElement)),
  isRenaming: () => Boolean(renamingPath.value),
  isContextMenuVisible: () => contextMenu.visible,
  isSelectionBoxActive: () => selectionBox.active,
  canPaste: () => props.canPaste,
  selectedEntries: () => selectedEntries.value,
  focusedOrSelectedEntry,
  firstSelectedEntry,
  cancelRename,
  commitRename,
  closeContextMenu,
  resetSelectionBox,
  clearSelection,
  openKeyboardContextMenu,
  showProperties: entries => emit("properties", entries),
  toggleFocusedSelection,
  selectRange,
  previewEntry: entry => emit("preview", entry),
  copySelectedPaths,
  copyEntry: entry => emit("copy", entry),
  cutEntry: entry => emit("cut", entry),
  paste: () => emit("paste"),
  selectAllEntries,
  invertCurrentSelection,
  openEntry,
  openEntryInNewTab,
  deleteEntry: entry => emit("delete", entry),
  startRename,
  handleTypeahead,
  moveFocus
});

const handleAuxClick = (event: MouseEvent, entry: ExplorerEntry) => {
  if (event.button !== 1 || entry.type !== "folder") return;
  event.preventDefault();
  ensureEntrySelected(entry);
  openEntryInNewTab(entry);
}

onMounted(async () => {
  fileStore.ensureActiveTab();
  await loadFolder(fileStore.currentPath || "/");
  window.addEventListener("click", closeContextMenu);
  window.addEventListener("keydown", handleKeyDown);
  window.addEventListener("mousemove", handleSelectionMove);
  window.addEventListener("mouseup", finishMarqueeSelection);
  window.addEventListener("pointermove", handleDetailsColumnResizeMove);
  window.addEventListener("pointerup", finishDetailsColumnResize);
  window.addEventListener("pointercancel", finishDetailsColumnResize);
});

onBeforeUnmount(() => {
  window.removeEventListener("click", closeContextMenu);
  window.removeEventListener("keydown", handleKeyDown);
  window.removeEventListener("mousemove", handleSelectionMove);
  window.removeEventListener("mouseup", finishMarqueeSelection);
  window.removeEventListener("pointermove", handleDetailsColumnResizeMove);
  window.removeEventListener("pointerup", finishDetailsColumnResize);
  window.removeEventListener("pointercancel", finishDetailsColumnResize);
  stopMarqueeAutoScroll();
  resetTypeahead();
  disconnectThumbnailObserver();
  itemRefs.clear();
  clearRenameInputRefs();
});

defineExpose({
  refresh: loadFolder,
  getSelectedEntry: primarySelected,
  getSelectedEntries: () => selectedEntries.value,
  getImageEntries: () => imageEntries.value,
  startRename: () => startRename(firstSelectedEntry()),
  selectPath,
  selectPaths,
  selectPathForRename,
  selectAllEntries,
  clearCurrentSelection,
  invertCurrentSelection,
  focus: focusViewport,
  getScrollTop,
  setScrollTop
})
</script>

<template>
  <section class="explorer-shell">
    <explorer-command-row
        :total-count-text="totalCountText"
        :selected-count-text="selectedCountText"
        :sort-text="sortText"
        :sort-options="sortOptions"
        :sort-key="sortKey"
        :sort-order="sortOrder"
        :next-sort-order="nextSortOrder"
        :view-mode="viewMode"
        :icon-size-text="iconSizeText"
        :loading="loading"
        @change-sort="changeSort"
        @change-sort-order="changeSortOrder"
        @set-view-mode="setViewMode"
        @cycle-icon-size="cycleIconSize" />

    <div
        ref="viewportRef"
        class="explorer-viewport"
        :class="[viewMode, itemSizeClass, {dropCurrent: dragState.overCurrentFolder}]"
        role="listbox"
        aria-label="文件列表"
        aria-multiselectable="true"
        :aria-busy="loading || loadingMore"
        :aria-activedescendant="focusedPath ? entryDomId(focusedPath) : undefined"
        tabindex="0"
        @focus="ensureFocusAnchor"
        @click="activateViewport"
        @mousedown="beginMarqueeSelection"
        @scroll="handleViewportScroll"
        @wheel="handleViewportWheel"
        @dragover="dragOverCurrentFolder"
        @dragleave="dragLeaveCurrentFolder"
        @drop="dropOnCurrentFolder"
        @contextmenu.prevent="openBackgroundContextMenu">
      <details-header
          v-if="viewMode === 'details'"
          :grid-style="detailsGridStyle"
          :loading="loading"
          :sort-key="sortKey"
          :sort-order="sortOrder"
          @change-sort="changeSort"
          @resize-column="startDetailsColumnResize" />

      <explorer-empty-state
          v-if="loading || message || !entries.length"
          :loading="loading"
          :message="message"
          :empty-text="emptyText"
          :empty-hint-text="emptyHintText"
          :filter-active="filterActive"
          @clear-filter="emit('clear-filter')" />

      <div v-else class="entry-surface">
        <explorer-entry-item
            v-for="entry in entries"
            :key="entry.path"
            :ref="element => setItemRef(entry.path, element)"
            :entry="entry"
            :entry-id="entryDomId(entry.path)"
            :view-mode="viewMode"
            :grid-style="detailsGridStyle"
            :selected="isSelected(entry.path)"
            :focused="focusedPath === entry.path"
            :image="isImageFile(entry)"
            :dimmed="isDimmed(entry)"
            :dragging="isDragged(entry)"
            :drop-target="isDropTarget(entry)"
            :renaming="isRenaming(entry)"
            :rename-draft="renameDraft"
            :rename-submitting="renameSubmitting"
            :thumbnail-visible="shouldLoadThumbnail(entry)"
            :thumbnail-src="thumbnailUrl(entry)"
            :icon="fileIcon(entry)"
            :type-text="entryTypeText(entry)"
            :modified-text="formatDate(entry.modified)"
            :size-text="formatSize(entry.size)"
            :tile-meta-text="`${formatDate(entry.modified)} · ${formatSize(entry.size)}`"
            @select="selectEntry(entry, $event)"
            @aux-click="handleAuxClick($event, entry)"
            @open="openEntry(entry)"
            @drag-start="beginEntryDrag($event, entry)"
            @drag-end="resetEntryDrag"
            @drag-over="dragOverEntry($event, entry)"
            @drag-leave="dragLeaveEntry($event, entry)"
            @drop="dropOnEntry($event, entry)"
            @context-menu="openContextMenu($event, entry)"
            @thumbnail-error="handleThumbnailError(entry)"
            @rename-input-ref="element => setRenameInputRef(entry.path, element)"
            @update:rename-draft="renameDraft = $event"
            @commit-rename="commitRename"
            @cancel-rename="cancelRename" />

        <div v-if="folderData.hasMore && !props.filterText.trim()" class="load-more-row">
          <button class="load-more-button" :disabled="loadingMore" @click.stop="loadMore">
            {{ loadingMore ? "正在加载..." : "加载更多" }}
          </button>
        </div>
      </div>

      <div
          v-if="selectionBox.active"
          class="selection-box"
          :style="{left: `${selectionBox.x}px`, top: `${selectionBox.y}px`, width: `${selectionBox.width}px`, height: `${selectionBox.height}px`}">
      </div>

      <div v-if="dragHintText" class="drag-hint" :class="{copy: dragState.copy}">
        {{ dragHintText }}
      </div>
    </div>

    <explorer-status-bar :folder-status-text="folderStatusText" :selected-status-text="selectedStatusText" />

    <explorer-context-menu
        v-if="contextMenu.visible"
        :background="contextMenu.background"
        :x="contextMenu.x"
        :y="contextMenu.y"
        :can-paste="props.canPaste"
        :has-entries="Boolean(entries.length)"
        :has-selection="Boolean(selectedPaths.length)"
        :primary-entry="primaryContextEntry"
        :selection-count="contextSelectionCount"
        :can-view-image="contextCanViewImage"
        :can-edit="contextCanEdit"
        :can-extract="contextCanExtract"
        @escape="closeContextMenuAndFocus"
        @open="openEntryFromContext"
        @open-new-tab="openContextEntryInNewTab"
        @view-image="viewImageContextEntry"
        @edit="editContextEntry"
        @preview="previewContextEntry"
        @cut="cutContextEntries"
        @copy="copyContextEntries"
        @copy-path="copyPathContextEntries"
        @paste="pasteIntoCurrentFolder"
        @download="downloadContextEntry"
        @archive="archiveContextEntries"
        @extract="extractContextEntry"
        @rename="renameContextEntry"
        @delete="deleteContextEntries"
        @properties="showContextProperties"
        @create-file="createFileFromContext"
        @create-folder="createFolderFromContext"
        @select-all="selectAllFromContext"
        @invert-selection="invertSelectionFromContext"
        @clear-selection="clearSelectionFromContext" />
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.explorer-shell {
  @apply relative flex h-full min-h-0 flex-col overflow-hidden bg-white;
}

.explorer-viewport {
  @apply relative grow overflow-auto outline-none select-none bg-white;
}

.explorer-viewport:focus-visible {
  @apply ring-2 ring-inset ring-blue-500;
}

.explorer-viewport.dropCurrent {
  @apply bg-blue-50/25 ring-2 ring-inset ring-blue-400;
}

.explorer-viewport.details {
  @apply min-w-0;
}

.entry-surface {
  @apply min-h-full p-2;
}

.details .entry-surface {
  @apply flex w-max min-w-full flex-col gap-0 p-1;
}

.list .entry-surface {
  @apply grid auto-rows-[2rem] grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-x-3 gap-y-1 p-2;
}

.icons .entry-surface {
  @apply grid content-start gap-2 p-3;
  grid-template-columns: repeat(auto-fill, minmax(7.5rem, 1fr));
}

.icons.large .entry-surface {
  grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
}

.icons.small .entry-surface {
  grid-template-columns: repeat(auto-fill, minmax(6rem, 1fr));
}

.tiles .entry-surface {
  @apply grid content-start grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-2 p-3;
}

.load-more-row {
  @apply flex justify-center px-3 py-4;
}

.list .load-more-row,
.icons .load-more-row,
.tiles .load-more-row {
  grid-column: 1 / -1;
}

.load-more-button {
  @apply h-8 rounded-md border border-slate-200 bg-white px-4 text-sm text-slate-600 shadow-sm hover:border-blue-300 hover:bg-blue-50 hover:text-blue-700 disabled:cursor-not-allowed disabled:border-slate-200 disabled:bg-slate-50 disabled:text-slate-400;
}

.selection-box {
  @apply pointer-events-none absolute z-20 border border-blue-500 bg-blue-500/15;
}

.drag-hint {
  @apply pointer-events-none sticky bottom-3 z-30 mx-auto mt-auto flex w-fit items-center rounded-md border border-blue-200 bg-white px-3 py-1.5 text-xs font-medium text-blue-700 shadow-lg;
}

.drag-hint.copy {
  @apply border-emerald-200 text-emerald-700;
}
</style>
