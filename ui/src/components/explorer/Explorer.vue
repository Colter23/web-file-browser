<script setup lang="ts">
import {computed, nextTick, watch} from "vue";
import {useFileStore} from "../../store";
import {useDetailsColumns} from "../../composables/useDetailsColumns.ts";
import {useExplorerContextMenu} from "../../composables/useExplorerContextMenu.ts";
import {useExplorerEntryActions} from "../../composables/useExplorerEntryActions.ts";
import {useExplorerEntryDrag} from "../../composables/useExplorerEntryDrag.ts";
import {useExplorerFolderData} from "../../composables/useExplorerFolderData.ts";
import {useExplorerKeyboard} from "../../composables/useExplorerKeyboard.ts";
import {useExplorerLifecycle} from "../../composables/useExplorerLifecycle.ts";
import {useExplorerMarqueeSelection} from "../../composables/useExplorerMarqueeSelection.ts";
import {useExplorerPresentation} from "../../composables/useExplorerPresentation.ts";
import {useExplorerRename} from "../../composables/useExplorerRename.ts";
import {useExplorerSelection} from "../../composables/useExplorerSelection.ts";
import {useExplorerStatusText} from "../../composables/useExplorerStatusText.ts";
import {useExplorerThumbnails} from "../../composables/useExplorerThumbnails.ts";
import {useExplorerTypeahead} from "../../composables/useExplorerTypeahead.ts";
import {useExplorerItemRefs, useExplorerViewport} from "../../composables/useExplorerViewport.ts";
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
import ExplorerEmptyState from "./ExplorerEmptyState.vue";
import ExplorerStatusBar from "./ExplorerStatusBar.vue";
import ExplorerEntryItem from "./ExplorerEntryItem.vue";
import type {DetailsColumnKey, ExplorerEntry} from "./types.ts";

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
  applyViewShortcut?: (code: string) => boolean;
}>(), {
  filterText: "",
  dimmedPaths: () => [],
  canPaste: false
})

const fileStore = useFileStore();
let closeContextMenuHandler = () => {};
const closeContextMenu = () => closeContextMenuHandler();

const {
  viewportRef,
  itemRefs,
  focusViewport,
  getScrollTop,
  setScrollTop,
  currentColumns,
  currentPageStep,
  entryDomId,
  isViewportActive,
  viewportHeight,
  clearItemRefs
} = useExplorerViewport();

const {
  gridStyle: detailsGridStyle,
  startResize: startDetailsColumnResize,
  handleResizeMove: handleDetailsColumnResizeMove,
  finishResize: finishDetailsColumnResize,
  fitColumnToContent: fitDetailsColumnToContent
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
  selectedEntries,
  entryByPath,
  firstSelectedEntry,
  focusedOrSelectedEntry,
  isSelected,
  indexOfPath,
  setSelection,
  clearSelection,
  commitSelectionAnchor,
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
  currentColumns: () => currentColumns(entries.value),
  currentPageStep: columns => currentPageStep(entries.value, columns)
});

const hasMoreEntries = computed(() => Boolean(folderData.value.hasMore));

const {
  filterActive,
  emptyText,
  emptyHintText,
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

const thumbnailActive = computed(() => fileStore.viewMode === "icons" || fileStore.viewMode === "tiles");
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
  active: thumbnailActive,
  isImageFile
});

const {setItemRef} = useExplorerItemRefs({
  itemRefs,
  entryByPath,
  observeEntry: observeThumbnail,
  unobservePath: unobserveThumbnail
});

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
  itemRefs,
  viewportRef,
  isRenaming: () => Boolean(renamingPath.value),
  focusViewport,
  clearSelection,
  setSelection,
  commitSelectionAnchor,
  closeContextMenu
});

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
  currentFolder: () => fileStore.currentPath || "/",
  isSelected,
  isRenaming,
  setSelection,
  focusViewport,
  closeContextMenu,
  cancelMarqueeSelection: resetSelectionBox,
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

const {
  viewMode,
  viewModeClass,
  sortKey,
  sortOrder,
  itemSizeClass,
  changeSort,
  changeSortOrder,
  handleViewportWheel,
} = useExplorerPresentation({
  loading,
  markStale,
  loadFolder,
  captureSelectionSnapshot,
  restoreSelectionSnapshot,
  focusViewport,
  observePendingThumbnails,
  viewportHeight
});

watch(() => [fileStore.activeTabId, fileStore.currentPath] as const, async ([, path]) => {
  if (!path || fileStore.showEditor) return;
  if (isLoadedFor(path)) return;
  await loadFolder(path);
});

watch(thumbnailActive, async iconLike => {
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

const detailsColumnValue = (entry: ExplorerEntry, key: DetailsColumnKey) => {
  if (key === "name") return entry.name;
  if (key === "modified") return formatDate(entry.modified);
  if (key === "type") return entryTypeText(entry);
  return formatSize(entry.size);
}

const fitDetailsColumn = (key: DetailsColumnKey) => {
  if (viewMode.value !== "details") return;
  fitDetailsColumnToContent(key, {
    entries: entries.value,
    viewport: viewportRef.value,
    value: detailsColumnValue
  });
  focusViewport();
}

const selectPathForRename = async (path: string) => {
  const entry = entryByPath(path);
  if (!entry) return false;
  await selectPath(path);
  startRename(entry);
  return true;
}

const activateViewport = () => {
  closeContextMenu();
  focusViewport();
  ensureFocusAnchor();
}

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
  isViewportActive,
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
  moveFocus,
  applyViewShortcut: code => props.applyViewShortcut?.(code) ?? false
});

const handleAuxClick = (event: MouseEvent, entry: ExplorerEntry) => {
  if (event.button !== 1 || entry.type !== "folder") return;
  event.preventDefault();
  ensureEntrySelected(entry);
  openEntryInNewTab(entry);
}

useExplorerLifecycle({
  initialize: async () => {
    fileStore.ensureActiveTab();
    await loadFolder(fileStore.currentPath || "/");
  },
  handleKeyDown,
  closeContextMenu,
  handleSelectionMove,
  finishMarqueeSelection,
  resetSelectionBox,
  handleDetailsColumnResizeMove,
  finishDetailsColumnResize,
  stopMarqueeAutoScroll,
  resetTypeahead,
  disconnectThumbnailObserver,
  clearItemRefs,
  clearRenameInputRefs
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
  setSortKey: changeSort,
  setSortOrder: changeSortOrder,
  focus: focusViewport,
  getScrollTop,
  setScrollTop
})
</script>

<template>
  <section class="explorer-shell">
    <div
        ref="viewportRef"
        class="explorer-viewport"
        :class="[viewModeClass, itemSizeClass, {dropCurrent: dragState.overCurrentFolder}]"
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
          @resize-column="startDetailsColumnResize"
          @fit-column="fitDetailsColumn" />

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
            :icon-size="fileStore.iconSize"
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
        @close="closeContextMenu"
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

.explorer-viewport.view-details {
  @apply min-w-0;
}

.entry-surface {
  @apply min-h-full p-2;
}

.explorer-viewport.view-details .entry-surface {
  @apply flex w-max min-w-full flex-col gap-0 p-1;
  min-height: calc(100% - var(--details-header-height, 2.25rem));
}

.explorer-viewport.view-list .entry-surface {
  @apply grid auto-rows-[2rem] grid-cols-[repeat(auto-fill,minmax(14rem,1fr))] gap-x-3 gap-y-1 p-2;
}

.explorer-viewport.view-icons .entry-surface {
  @apply grid content-start gap-2 p-3;
  grid-template-columns: repeat(auto-fill, minmax(7.5rem, 1fr));
}

.explorer-viewport.view-icons.explorer-size-large .entry-surface {
  grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
}

.explorer-viewport.view-icons.explorer-size-small .entry-surface {
  grid-template-columns: repeat(auto-fill, minmax(6rem, 1fr));
}

.explorer-viewport.view-tiles .entry-surface {
  @apply grid content-start grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-2 p-3;
}

.load-more-row {
  @apply flex justify-center px-3 py-4;
}

.explorer-viewport.view-list .load-more-row,
.explorer-viewport.view-icons .load-more-row,
.explorer-viewport.view-tiles .load-more-row {
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
