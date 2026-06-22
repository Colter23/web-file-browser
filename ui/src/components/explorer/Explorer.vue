<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch} from "vue";
import type {ComponentPublicInstance} from "vue";
import type {DirSortKey, DirSortOrder, ExplorerIconSize, ExplorerViewMode, FolderData, FolderQueryParams} from "../../class.ts";
import {useFileStore} from "../../store";
import {getFolderData} from "../../network/file-api.ts";
import {useDetailsColumns} from "../../composables/useDetailsColumns.ts";
import {useExplorerEntryDrag} from "../../composables/useExplorerEntryDrag.ts";
import {useExplorerMarqueeSelection} from "../../composables/useExplorerMarqueeSelection.ts";
import {useExplorerSelection} from "../../composables/useExplorerSelection.ts";
import {useExplorerThumbnails} from "../../composables/useExplorerThumbnails.ts";
import {useExplorerTypeahead} from "../../composables/useExplorerTypeahead.ts";
import {
  entryTypeText,
  fileEntryIcon,
  formatEntryDate as formatDate,
  formatEntrySize as formatSize,
  isEditableEntry,
  isExtractableArchiveEntry as canExtract,
  isImageEntry as isImageFile
} from "../../utils/file-entry.ts";
import DetailsHeader from "./DetailsHeader.vue";
import ExplorerContextMenu from "./ExplorerContextMenu.vue";
import ExplorerCommandRow from "./ExplorerCommandRow.vue";
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

type ViewDensityStep = {
  mode: ExplorerViewMode;
  iconSize: ExplorerIconSize;
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
const folderData = ref<FolderData>({ path: "/", folder: [], file: [] });
const loading = ref(false);
const loadingMore = ref(false);
const message = ref("");
const loadedSignature = ref("");
const viewportRef = ref<HTMLElement | null>(null);
const itemRefs = new Map<string, HTMLElement>();
const renameInputRefs = new Map<string, HTMLInputElement>();
const pageSize = 200;
const contextMenu = reactive({visible: false, x: 0, y: 0, targetPath: "", background: false});
const closeContextMenu = () => {
  contextMenu.visible = false;
}
const renamingPath = ref("");
const renameDraft = ref("");
const renameSubmitting = ref(false);
const viewWheelStepThreshold = 80;
const autoLoadMoreDistance = 360;
let viewWheelDelta = 0;

const {
  gridStyle: detailsGridStyle,
  startResize: startDetailsColumnResize,
  handleResizeMove: handleDetailsColumnResizeMove,
  finishResize: finishDetailsColumnResize
} = useDetailsColumns();

const viewDensitySteps: ViewDensityStep[] = [
  {mode: "details", iconSize: "small"},
  {mode: "list", iconSize: "small"},
  {mode: "tiles", iconSize: "medium"},
  {mode: "icons", iconSize: "small"},
  {mode: "icons", iconSize: "medium"},
  {mode: "icons", iconSize: "large"}
];

const normalizeFolderData = (data: FolderData): FolderData => ({
  path: data.path || "/",
  folder: data.folder ?? [],
  file: data.file ?? [],
  folderTotal: data.folderTotal,
  fileTotal: data.fileTotal,
  offset: data.offset,
  limit: data.limit,
  hasMore: data.hasMore
})

const allEntries = computed<ExplorerEntry[]>(() => [
  ...(folderData.value.folder ?? []).map(folder => ({
    type: "folder" as const,
    name: folder.name,
    path: folder.path,
    modified: folder.modified
  })),
  ...(folderData.value.file ?? []).map(file => ({
    type: "file" as const,
    name: file.name,
    path: file.path,
    modified: file.modified,
    size: file.size,
    extension: file.extension,
    file
  }))
]);

const filterKeyword = computed(() => props.filterText.trim());

const entries = computed<ExplorerEntry[]>(() => {
  const keyword = filterKeyword.value.toLowerCase();
  if (!keyword) return allEntries.value;
  return allEntries.value.filter(entry => entry.name.toLowerCase().includes(keyword));
});

const emptyText = computed(() => {
  if (filterKeyword.value) return `没有匹配“${filterKeyword.value}”的项目`;
  return "此文件夹为空";
});

const emptyHintText = computed(() => {
  if (!filterKeyword.value) return "";
  return folderData.value.hasMore ? "当前只筛选已加载项目，清除筛选后可继续加载更多。" : "清除筛选可查看全部已加载项目。";
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

const selectedFileEntries = computed(() => selectedEntries.value.filter(entry => entry.type === "file"));
const selectedFolderCount = computed(() => selectedEntries.value.length - selectedFileEntries.value.length);

const hasLoadedFileSize = (entry: ExplorerEntry): entry is ExplorerEntry & {type: "file"; size: number} => {
  return entry.type === "file" && Number.isFinite(entry.size);
}

const selectedKnownSize = computed(() => selectedFileEntries.value.reduce((total, entry) => {
  return hasLoadedFileSize(entry) ? total + entry.size : total;
}, 0));

const selectedMissingSizeCount = computed(() => selectedFileEntries.value.filter(entry => !hasLoadedFileSize(entry)).length);

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
const selectedCountText = computed(() => {
  const count = selectedPaths.value.length;
  if (!count) return "未选择项目";
  return `已选择 ${count} 项`;
});

const totalCountText = computed(() => {
  const loadedCount = allEntries.value.length;
  const hasMore = folderData.value.hasMore ? "，还有更多" : "";
  return filterKeyword.value ? `已加载 ${loadedCount} 项，筛选 ${entries.value.length} 项${hasMore}` : `已加载 ${loadedCount} 项${hasMore}`;
});

const folderStatusText = computed(() => {
  const source = filterKeyword.value ? entries.value : allEntries.value;
  const folderCount = source.filter(entry => entry.type === "folder").length;
  const fileCount = source.length - folderCount;
  const prefix = filterKeyword.value ? "筛选结果" : "当前已加载";
  const suffix = folderData.value.hasMore && !filterKeyword.value ? "，还有更多" : "";
  return `${prefix}：${folderCount} 个文件夹，${fileCount} 个文件${suffix}`;
});

const selectedSizeText = computed(() => {
  const fileCount = selectedFileEntries.value.length;
  if (!fileCount) return "";
  const missing = selectedMissingSizeCount.value;
  if (missing === fileCount) return `${fileCount} 个文件大小未加载`;
  if (missing) return `${formatSize(selectedKnownSize.value)} 已知，${missing} 个文件未加载大小`;
  return formatSize(selectedKnownSize.value);
});

const selectedStatusText = computed(() => {
  const selectedCount = selectedEntries.value.length;
  if (!selectedCount) return "未选择项目";
  const detail = [];
  if (selectedFileEntries.value.length) detail.push(`${selectedFileEntries.value.length} 个文件`);
  if (selectedFolderCount.value) detail.push(`${selectedFolderCount.value} 个文件夹`);
  if (selectedSizeText.value) detail.push(selectedSizeText.value);
  return `已选择 ${selectedCount} 项${detail.length ? ` · ${detail.join("，")}` : ""}`;
});

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

const setRenameInputRef = (path: string, element: Element | ComponentPublicInstance | null) => {
  if (element instanceof HTMLInputElement) {
    renameInputRefs.set(path, element);
  } else {
    renameInputRefs.delete(path);
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

const selectRenameText = (input: HTMLInputElement, entry: ExplorerEntry) => {
  if (entry.type === "folder") {
    input.select();
    return;
  }
  const suffix = entry.extension ? `.${entry.extension}` : "";
  const end = suffix && entry.name.toLowerCase().endsWith(suffix.toLowerCase())
      ? Math.max(0, entry.name.length - suffix.length)
      : entry.name.length;
  input.setSelectionRange(0, end);
}

const startRename = (entry: ExplorerEntry | null) => {
  if (!entry || renameSubmitting.value) return;
  ensureEntrySelected(entry);
  contextMenu.visible = false;
  renamingPath.value = entry.path;
  renameDraft.value = entry.name;
  nextTick(() => {
    const input = renameInputRefs.get(entry.path);
    input?.focus();
    if (input) selectRenameText(input, entry);
  });
}

const cancelRename = () => {
  if (renameSubmitting.value) return;
  renamingPath.value = "";
  renameDraft.value = "";
  nextTick(() => viewportRef.value?.focus());
}

const commitRename = async () => {
  if (!renamingPath.value || renameSubmitting.value) return;
  const entry = entryByPath(renamingPath.value);
  const nextName = renameDraft.value.trim();
  if (!entry || !nextName || nextName === entry.name) {
    cancelRename();
    return;
  }
  renameSubmitting.value = true;
  try {
    emit("rename", {entry, name: nextName});
    renamingPath.value = "";
    renameDraft.value = "";
  } finally {
    renameSubmitting.value = false;
  }
}

const isRenaming = (entry: ExplorerEntry) => renamingPath.value === entry.path;

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

const copySelectedPaths = () => {
  const paths = selectedEntries.value.length ? selectedEntries.value.map(entry => entry.path) : [fileStore.currentPath || "/"];
  emit("copy-path", {paths});
}

const openEntry = async (entry: ExplorerEntry) => {
  if (isRenaming(entry)) return;
  if (entry.type === "folder") {
    if (!await fileStore.requestEditorLeave()) return;
    await loadFolder(entry.path);
    return;
  }
  if (isImageFile(entry)) {
    emit("open-image-viewer", {entry, entries: imageEntries.value});
    return;
  }
  if (canEditEntry(entry)) {
    await editEntry(entry);
  } else {
    emit("preview", entry);
  }
}

const folderRequestSignature = (path: string = fileStore.currentPath || "/") => {
  return `${path}|${fileStore.sortKey}|${fileStore.sortOrder}`;
}

const folderQuery = (offset = 0): FolderQueryParams => {
  const needsFullDetail = fileStore.sortKey !== "name";
  return {
    offset,
    limit: pageSize,
    detail: needsFullDetail ? "full" as const : "basic" as const,
    sort: fileStore.sortKey,
    order: fileStore.sortOrder
  };
}

const mergeFolderData = (current: FolderData, next: FolderData): FolderData => ({
  ...next,
  folder: [...(current.folder ?? []), ...(next.folder ?? [])],
  file: [...(current.file ?? []), ...(next.file ?? [])]
})

const loadFolder = async (path: string = fileStore.currentPath || "/") => {
  loading.value = true;
  message.value = "";
  let loaded = false;
  renamingPath.value = "";
  renameDraft.value = "";
  resetTypeahead();
  resetSelectionBox();
  clearThumbnailState();
  try {
    const data = normalizeFolderData(await getFolderData(path, folderQuery()));
    fileStore.saveFolderData(data);
    folderData.value = data;
    loadedSignature.value = folderRequestSignature(data.path || path);
    clearSelection();
    fileStore.setCurrentPath(data.path);
    fileStore.closeEditor();
    await nextTick();
    observePendingThumbnails();
    loaded = true;
  } catch (error) {
    message.value = error instanceof Error ? error.message : "加载目录失败";
  } finally {
    loading.value = false;
    if (loaded) window.requestAnimationFrame(maybeLoadMoreOnScroll);
  }
  return loaded;
}

const loadMore = async () => {
  if (loading.value || loadingMore.value || !folderData.value.hasMore) return;
  loadingMore.value = true;
  message.value = "";
  let loaded = false;
  try {
    const current = folderData.value;
    const offset = (current.offset ?? 0) + (current.limit ?? allEntries.value.length);
    const data = normalizeFolderData(await getFolderData(current.path || fileStore.currentPath || "/", folderQuery(offset)));
    const merged = normalizeFolderData(mergeFolderData(current, data));
    fileStore.saveFolderData(merged);
    folderData.value = merged;
    loadedSignature.value = folderRequestSignature(merged.path || current.path);
    await nextTick();
    observePendingThumbnails();
    loaded = true;
  } catch (error) {
    message.value = error instanceof Error ? error.message : "加载更多失败";
  } finally {
    loadingMore.value = false;
    if (loaded) window.requestAnimationFrame(maybeLoadMoreOnScroll);
  }
}

const maybeLoadMoreOnScroll = () => {
  const viewport = viewportRef.value;
  if (!viewport || props.filterText.trim() || loading.value || loadingMore.value || !folderData.value.hasMore) return;
  const distanceToBottom = viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight;
  if (distanceToBottom <= autoLoadMoreDistance) void loadMore();
}

const handleViewportScroll = () => {
  emit("scroll-change", getScrollTop());
  maybeLoadMoreOnScroll();
}

const changeSort = async (key: DirSortKey) => {
  if (loading.value) return;
  const snapshot = captureSelectionSnapshot();
  fileStore.setSort(key);
  loadedSignature.value = "";
  if (await loadFolder(fileStore.currentPath || "/")) await restoreSelectionSnapshot(snapshot);
}

const changeSortOrder = async (order: DirSortOrder) => {
  if (loading.value || fileStore.sortOrder === order) return;
  const snapshot = captureSelectionSnapshot();
  fileStore.setSort(fileStore.sortKey, order);
  loadedSignature.value = "";
  if (await loadFolder(fileStore.currentPath || "/")) await restoreSelectionSnapshot(snapshot);
}

watch(() => [fileStore.activeTabId, fileStore.currentPath] as const, async ([, path]) => {
  if (!path || fileStore.showEditor) return;
  if (loadedSignature.value === folderRequestSignature(path)) return;
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
  renameInputRefs.clear();
});

const showContextMenu = (x: number, y: number, targetPath = "", background = false) => {
  contextMenu.x = x;
  contextMenu.y = y;
  contextMenu.targetPath = targetPath;
  contextMenu.background = background;
  contextMenu.visible = true;
}

const closeContextMenuAndFocus = () => {
  closeContextMenu();
  focusViewport();
}

const openContextMenu = (event: MouseEvent, entry: ExplorerEntry) => {
  focusViewport();
  ensureEntrySelected(entry);
  showContextMenu(event.clientX, event.clientY, entry.path);
}

const openBackgroundContextMenu = (event: MouseEvent) => {
  if (event.target instanceof HTMLElement && event.target.closest(".entry-item")) return;
  focusViewport();
  showContextMenu(event.clientX, event.clientY, "", true);
}

const openKeyboardContextMenu = () => {
  const entry = entryByPath(focusedPath.value) ?? firstSelectedEntry();
  if (entry) {
    ensureEntrySelected(entry);
    const rect = itemRefs.get(entry.path)?.getBoundingClientRect();
    const x = rect ? rect.left + Math.min(36, rect.width - 8) : window.innerWidth / 2;
    const y = rect ? rect.top + Math.min(28, rect.height) : window.innerHeight / 2;
    showContextMenu(x, y, entry.path);
    return;
  }
  const viewportRect = viewportRef.value?.getBoundingClientRect();
  showContextMenu(viewportRect ? viewportRect.left + 16 : window.innerWidth / 2, viewportRect ? viewportRect.top + 16 : window.innerHeight / 2, "", true);
}

const contextEntry = () => contextMenu.background ? null : entryByPath(contextMenu.targetPath) ?? firstSelectedEntry();

const selectedOrContextEntries = () => {
  const target = contextEntry();
  if (!target) return [];
  if (selectedPaths.value.includes(target.path)) return selectedEntries.value;
  return [target];
}

const contextEntries = computed(() => selectedOrContextEntries());

const contextSelectionCount = computed(() => contextEntries.value.length);

const canEditEntry = (entry: ExplorerEntry | null) => {
  return isEditableEntry(entry, fileStore.extensions);
}

const editEntry = async (entry: ExplorerEntry) => {
  if (!canEditEntry(entry)) return;
  if (!await fileStore.requestEditorLeave()) return;
  fileStore.openEditor(entry.file ?? {
    path: entry.path,
    name: entry.name,
    size: entry.size ?? 0,
    extension: entry.extension ?? "",
    modified: entry.modified ?? ""
  });
}

const fileIcon = (entry: ExplorerEntry) => {
  return fileEntryIcon(entry, fileStore.extensions);
}

const isDimmed = (entry: ExplorerEntry) => props.dimmedPaths.includes(entry.path);

const handleKeyDown = async (event: KeyboardEvent) => {
  if (!viewportRef.value?.contains(document.activeElement)) return;
  if (renamingPath.value) {
    if (event.key === "Escape") {
      event.preventDefault();
      cancelRename();
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      await commitRename();
      return;
    }
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    if (contextMenu.visible) {
      contextMenu.visible = false;
      return;
    }
    if (selectionBox.active) {
      resetSelectionBox();
      return;
    }
    clearSelection();
    return;
  }
  if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
    event.preventDefault();
    openKeyboardContextMenu();
    return;
  }
  if (event.altKey && !event.ctrlKey && !event.metaKey && event.key === "Enter") {
    event.preventDefault();
    contextMenu.visible = false;
    const focused = focusedOrSelectedEntry();
    const entriesToShow = selectedEntries.value.length ? selectedEntries.value : focused ? [focused] : [];
    if (entriesToShow.length) emit("properties", entriesToShow);
    return;
  }
  if ((event.key === " " || event.code === "Space") && !event.altKey && !event.shiftKey && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    contextMenu.visible = false;
    toggleFocusedSelection();
    return;
  }
  if ((event.key === " " || event.code === "Space") && event.shiftKey && !event.altKey && !event.ctrlKey && !event.metaKey) {
    event.preventDefault();
    contextMenu.visible = false;
    const entry = focusedOrSelectedEntry();
    if (entry) selectRange(entry.path, false);
    return;
  }
  if ((event.key === " " || event.code === "Space") && !event.altKey && !event.shiftKey && !event.ctrlKey && !event.metaKey) {
    event.preventDefault();
    contextMenu.visible = false;
    const entry = focusedOrSelectedEntry();
    if (entry?.type === "file") emit("preview", entry);
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "c") {
    event.preventDefault();
    if (event.shiftKey) {
      contextMenu.visible = false;
      copySelectedPaths();
      return;
    }
    const entry = firstSelectedEntry();
    if (entry) emit("copy", entry);
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "x") {
    event.preventDefault();
    const entry = firstSelectedEntry();
    if (entry) emit("cut", entry);
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v") {
    event.preventDefault();
    if (props.canPaste) emit("paste");
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
    event.preventDefault();
    selectAllEntries();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "i") {
    event.preventDefault();
    contextMenu.visible = false;
    invertCurrentSelection();
    return;
  }
  if (event.key === "Enter") {
    event.preventDefault();
    contextMenu.visible = false;
    const entry = focusedOrSelectedEntry();
    if ((event.ctrlKey || event.metaKey) && entry?.type === "folder") {
      emit("open-new-tab", entry);
      return;
    }
    if ((event.ctrlKey || event.metaKey) && entry?.type === "file") {
      emit("preview", entry);
      return;
    }
    if (entry) await openEntry(entry);
    return;
  }
  if (event.key === "Delete") {
    event.preventDefault();
    contextMenu.visible = false;
    const entry = firstSelectedEntry();
    if (entry) emit("delete", entry);
    return;
  }
  if (event.key === "F2") {
    event.preventDefault();
    contextMenu.visible = false;
    const entry = selectedEntries.value.length <= 1 ? focusedOrSelectedEntry() : null;
    if (entry) startRename(entry);
    return;
  }
  if (handleTypeahead(event)) return;
  if (!["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight", "Home", "End", "PageDown", "PageUp"].includes(event.key)) return;
  event.preventDefault();
  contextMenu.visible = false;
  moveFocus(event.key, event.shiftKey, (event.ctrlKey || event.metaKey) && !event.shiftKey);
}

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

const setViewMode = (mode: ExplorerViewMode) => {
  fileStore.setViewMode(mode);
  nextTick(() => viewportRef.value?.focus());
}

const currentViewDensityIndex = () => {
  if (fileStore.viewMode === "icons") {
    const index = viewDensitySteps.findIndex(step => step.mode === "icons" && step.iconSize === fileStore.iconSize);
    return index >= 0 ? index : viewDensitySteps.findIndex(step => step.mode === "icons" && step.iconSize === "medium");
  }
  const index = viewDensitySteps.findIndex(step => step.mode === fileStore.viewMode);
  return index >= 0 ? index : 0;
}

const setViewDensityStep = async (index: number) => {
  const nextIndex = Math.min(viewDensitySteps.length - 1, Math.max(0, index));
  const step = viewDensitySteps[nextIndex];
  if (!step) return;
  if (fileStore.viewMode === step.mode && fileStore.iconSize === step.iconSize) return;
  fileStore.setViewMode(step.mode);
  fileStore.setIconSize(step.iconSize);
  await nextTick();
  viewportRef.value?.focus();
  observePendingThumbnails();
}

const wheelDeltaPixels = (event: WheelEvent) => {
  if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) return event.deltaY * 32;
  if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) return event.deltaY * (viewportRef.value?.clientHeight || 800);
  return event.deltaY;
}

const handleViewportWheel = (event: WheelEvent) => {
  if (!event.ctrlKey && !event.metaKey) {
    viewWheelDelta = 0;
    return;
  }
  event.preventDefault();
  const delta = wheelDeltaPixels(event);
  if (!delta) return;
  viewWheelDelta += delta;
  if (Math.abs(viewWheelDelta) < viewWheelStepThreshold) return;
  const direction = viewWheelDelta < 0 ? 1 : -1;
  viewWheelDelta = 0;
  void setViewDensityStep(currentViewDensityIndex() + direction);
}

const cycleIconSize = () => {
  const next = fileStore.iconSize === "small" ? "medium" : fileStore.iconSize === "medium" ? "large" : "small";
  fileStore.setIconSize(next);
  nextTick(() => viewportRef.value?.focus());
}

const primaryContextEntry = computed(() => contextEntry());

const contextCanViewImage = computed(() => Boolean(primaryContextEntry.value && isImageFile(primaryContextEntry.value)));

const contextCanEdit = computed(() => canEditEntry(primaryContextEntry.value));

const contextCanExtract = computed(() => canExtract(primaryContextEntry.value));

const primarySelected = () => firstSelectedEntry();

const openEntryFromContext = async () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) await openEntry(entry);
}

const openEntryInNewTab = (entry: ExplorerEntry) => {
  if (entry.type !== "folder") return;
  closeContextMenu();
  emit("open-new-tab", entry);
}

const openContextEntryInNewTab = () => {
  const entry = primaryContextEntry.value;
  if (entry) openEntryInNewTab(entry);
}

const previewContextEntry = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) emit("preview", entry);
}

const viewImageContextEntry = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry && isImageFile(entry)) emit("open-image-viewer", {entry, entries: imageEntries.value});
}

const editContextEntry = async () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) await editEntry(entry);
}

const downloadContextEntry = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) emit("download", entry);
}

const copyPathContextEntries = () => {
  const paths = contextMenu.background ? [fileStore.currentPath || "/"] : contextEntries.value.map(entry => entry.path);
  closeContextMenu();
  if (paths.length) emit("copy-path", {paths});
}

const copyContextEntries = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) emit("copy", entry);
}

const cutContextEntries = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) emit("cut", entry);
}

const pasteIntoCurrentFolder = () => {
  closeContextMenu();
  emit("paste");
}

const createFileFromContext = () => {
  closeContextMenu();
  emit("create-file");
}

const createFolderFromContext = () => {
  closeContextMenu();
  emit("create-folder");
}

const selectAllFromContext = () => {
  closeContextMenu();
  selectAllEntries();
}

const clearSelectionFromContext = () => {
  closeContextMenu();
  clearCurrentSelection();
}

const invertSelectionFromContext = () => {
  closeContextMenu();
  invertCurrentSelection();
}

const archiveContextEntries = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) emit("archive", entry);
}

const extractContextEntry = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) emit("extract", entry);
}

const renameContextEntry = () => {
  const entry = primaryContextEntry.value;
  if (entry) startRename(entry);
}

const deleteContextEntries = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) emit("delete", entry);
}

const showContextProperties = () => {
  const entries = contextEntries.value;
  closeContextMenu();
  if (entries.length) emit("properties", entries);
}

const handleAuxClick = (event: MouseEvent, entry: ExplorerEntry) => {
  if (event.button !== 1 || entry.type !== "folder") return;
  event.preventDefault();
  ensureEntrySelected(entry);
  openEntryInNewTab(entry);
}

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

      <div v-if="loading" class="explorer-empty">正在加载...</div>
      <div v-else-if="message" class="explorer-empty error">{{ message }}</div>
      <div v-else-if="!entries.length" class="explorer-empty">
        <span>{{ emptyText }}</span>
        <small v-if="emptyHintText">{{ emptyHintText }}</small>
        <button v-if="filterKeyword" type="button" class="empty-action" @click.stop="emit('clear-filter')">清除筛选</button>
      </div>

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

.explorer-empty {
  @apply flex h-48 flex-col items-center justify-center gap-1 text-center text-sm text-slate-500;
}

.explorer-empty small {
  @apply max-w-md px-4 text-xs leading-5 text-slate-400;
}

.empty-action {
  @apply mt-2 h-8 rounded-md border border-blue-200 bg-white px-3 text-xs font-medium text-blue-700 shadow-sm hover:border-blue-300 hover:bg-blue-50;
}

.explorer-empty.error {
  @apply text-red-600;
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
