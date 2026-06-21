<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch} from "vue";
import type {ComponentPublicInstance} from "vue";
import type {DirSortKey, DirSortOrder, ExplorerIconSize, ExplorerViewMode, FileInfo, FolderData, FolderQueryParams} from "../../class.ts";
import {useFileStore} from "../../store";
import {downloadUrl, getFolderData} from "../../network/file-api.ts";
import Icon from "../Icon.vue";

type ExplorerEntryType = "folder" | "file";

type ExplorerEntry = {
  type: ExplorerEntryType;
  name: string;
  path: string;
  modified: string;
  size?: number;
  extension?: string;
  file?: FileInfo;
}

type SelectionBox = {
  active: boolean;
  additive: boolean;
  basePaths: string[];
  originX: number;
  originY: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

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
  (e: "selection-change", entries: ExplorerEntry[]): void;
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
const selectedPaths = ref<string[]>([]);
const focusedPath = ref("");
const anchorPath = ref("");
const loading = ref(false);
const loadingMore = ref(false);
const message = ref("");
const loadedSignature = ref("");
const viewportRef = ref<HTMLElement | null>(null);
const itemRefs = new Map<string, HTMLElement>();
const renameInputRefs = new Map<string, HTMLInputElement>();
const visibleThumbnailPaths = ref<Set<string>>(new Set());
const failedThumbnailPaths = ref<Set<string>>(new Set());
const pageSize = 200;
const contextMenu = reactive({visible: false, x: 0, y: 0, targetPath: "", background: false});
const renamingPath = ref("");
const renameDraft = ref("");
const renameSubmitting = ref(false);
const draggingEntries = ref<ExplorerEntry[]>([]);
const dragState = reactive({active: false, overPath: "", overCurrentFolder: false, copy: false});
const selectionBox = reactive<SelectionBox>({
  active: false,
  additive: false,
  basePaths: [],
  originX: 0,
  originY: 0,
  x: 0,
  y: 0,
  width: 0,
  height: 0
});
let thumbnailObserver: IntersectionObserver | null = null;
let marqueePointerX = 0;
let marqueePointerY = 0;
let marqueeScrollFrame = 0;
const marqueeScrollEdge = 48;
const marqueeMaxScrollSpeed = 24;
const contextMenuViewportPadding = 8;
const contextMenuEstimatedWidth = 176;
const contextMenuEstimatedHeights = {
  entry: 360,
  background: 184
};
const viewWheelStepThreshold = 80;
const autoLoadMoreDistance = 360;
const typeaheadQuery = ref("");
const typeaheadResetMs = 900;
let typeaheadResetTimer = 0;
let viewWheelDelta = 0;

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

const entries = computed<ExplorerEntry[]>(() => {
  const keyword = props.filterText.trim().toLowerCase();
  if (!keyword) return allEntries.value;
  return allEntries.value.filter(entry => entry.name.toLowerCase().includes(keyword));
});

const selectedEntries = computed(() => {
  const selected = new Set(selectedPaths.value);
  return entries.value.filter(entry => selected.has(entry.path));
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
const selectedCountText = computed(() => {
  const count = selectedPaths.value.length;
  if (!count) return "未选择项目";
  return `已选择 ${count} 项`;
});

const totalCountText = computed(() => {
  const loadedCount = allEntries.value.length;
  const hasMore = folderData.value.hasMore ? "，还有更多" : "";
  return props.filterText.trim() ? `已加载 ${loadedCount} 项，筛选 ${entries.value.length} 项${hasMore}` : `已加载 ${loadedCount} 项${hasMore}`;
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

const isImageFile = (entry: ExplorerEntry) => {
  if (entry.type !== "file") return false;
  const extension = entry.extension?.toLowerCase() ?? "";
  return ["apng", "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "webp"].includes(extension);
}

const shouldLoadThumbnail = (entry: ExplorerEntry) => {
  return isIconLikeMode.value && isImageFile(entry) && visibleThumbnailPaths.value.has(entry.path) && !failedThumbnailPaths.value.has(entry.path);
}

const thumbnailUrl = (entry: ExplorerEntry) => downloadUrl(entry.path);

const handleThumbnailError = (entry: ExplorerEntry) => {
  addFailedThumbnailPath(entry.path);
  unobserveThumbnail(entry.path);
}

const addVisibleThumbnailPath = (path: string) => {
  if (visibleThumbnailPaths.value.has(path)) return;
  visibleThumbnailPaths.value = new Set([...visibleThumbnailPaths.value, path]);
}

const addFailedThumbnailPath = (path: string) => {
  if (failedThumbnailPaths.value.has(path)) return;
  failedThumbnailPaths.value = new Set([...failedThumbnailPaths.value, path]);
}

const clearThumbnailState = () => {
  visibleThumbnailPaths.value = new Set();
  failedThumbnailPaths.value = new Set();
  thumbnailObserver?.disconnect();
  thumbnailObserver = null;
}

const createThumbnailObserver = () => {
  if (thumbnailObserver || typeof IntersectionObserver === "undefined") return thumbnailObserver;
  thumbnailObserver = new IntersectionObserver(records => {
    records.forEach(record => {
      if (!record.isIntersecting) return;
      const path = (record.target as HTMLElement).dataset.thumbnailPath;
      if (!path) return;
      addVisibleThumbnailPath(path);
      thumbnailObserver?.unobserve(record.target);
    });
  }, {
    root: viewportRef.value,
    rootMargin: "240px",
    threshold: 0.01
  });
  return thumbnailObserver;
}

const unobserveThumbnail = (path: string) => {
  const element = itemRefs.get(path);
  if (!element) return;
  thumbnailObserver?.unobserve(element);
  delete element.dataset.thumbnailPath;
}

const observeThumbnail = (entry: ExplorerEntry, element: HTMLElement) => {
  if (!isIconLikeMode.value || !isImageFile(entry) || visibleThumbnailPaths.value.has(entry.path) || failedThumbnailPaths.value.has(entry.path)) return;
  if (typeof IntersectionObserver === "undefined") {
    addVisibleThumbnailPath(entry.path);
    return;
  }
  element.dataset.thumbnailPath = entry.path;
  createThumbnailObserver()?.observe(element);
}

const observePendingThumbnails = () => {
  if (!isIconLikeMode.value) return;
  entries.value.forEach(entry => {
    const element = itemRefs.get(entry.path);
    if (element) observeThumbnail(entry, element);
  });
}

const setItemRef = (path: string, element: Element | ComponentPublicInstance | null) => {
  const current = itemRefs.get(path);
  if (current && current !== element) unobserveThumbnail(path);
  if (element instanceof HTMLElement) {
    itemRefs.set(path, element);
    const entry = entryByPath(path);
    if (entry) observeThumbnail(entry, element);
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

const selectedSet = () => new Set(selectedPaths.value);

const setSelection = (paths: string[], focusPath = paths[paths.length - 1] ?? "", keepAnchor = false) => {
  selectedPaths.value = Array.from(new Set(paths));
  focusedPath.value = focusPath;
  if (!keepAnchor) anchorPath.value = focusPath || anchorPath.value;
}

const clearSelection = () => {
  selectedPaths.value = [];
  focusedPath.value = "";
  anchorPath.value = "";
}

const stopMarqueeAutoScroll = () => {
  if (!marqueeScrollFrame) return;
  window.cancelAnimationFrame(marqueeScrollFrame);
  marqueeScrollFrame = 0;
}

const resetSelectionBox = () => {
  stopMarqueeAutoScroll();
  selectionBox.active = false;
  selectionBox.additive = false;
  selectionBox.basePaths = [];
}

const resetTypeahead = () => {
  if (typeaheadResetTimer) {
    window.clearTimeout(typeaheadResetTimer);
    typeaheadResetTimer = 0;
  }
  typeaheadQuery.value = "";
}

const scheduleTypeaheadReset = () => {
  if (typeaheadResetTimer) window.clearTimeout(typeaheadResetTimer);
  typeaheadResetTimer = window.setTimeout(() => {
    typeaheadQuery.value = "";
    typeaheadResetTimer = 0;
  }, typeaheadResetMs);
}

const entryByPath = (path: string) => entries.value.find(entry => entry.path === path);

const firstSelectedEntry = () => {
  if (!selectedPaths.value.length) return null;
  return entryByPath(selectedPaths.value[0]) ?? null;
}

const focusedOrSelectedEntry = () => entryByPath(focusedPath.value) ?? firstSelectedEntry();

const isSelected = (path: string) => selectedPaths.value.includes(path);

const indexOfPath = (path: string) => entries.value.findIndex(entry => entry.path === path);

const selectRange = (targetPath: string, additive: boolean) => {
  const targetIndex = indexOfPath(targetPath);
  if (targetIndex < 0) return;
  const anchorCandidate = anchorPath.value || focusedPath.value || targetPath;
  const anchorIndex = indexOfPath(anchorCandidate);
  const start = Math.min(anchorIndex < 0 ? targetIndex : anchorIndex, targetIndex);
  const end = Math.max(anchorIndex < 0 ? targetIndex : anchorIndex, targetIndex);
  const range = entries.value.slice(start, end + 1).map(entry => entry.path);
  if (additive) {
    setSelection([...selectedPaths.value, ...range], targetPath, true);
  } else {
    setSelection(range, targetPath, true);
  }
}

const selectEntry = (entry: ExplorerEntry, event?: MouseEvent) => {
  const ctrl = Boolean(event?.ctrlKey || event?.metaKey);
  const shift = Boolean(event?.shiftKey);
  if (shift) {
    selectRange(entry.path, ctrl);
    return;
  }
  if (ctrl) {
    const selected = selectedSet();
    if (selected.has(entry.path)) {
      selected.delete(entry.path);
      setSelection(Array.from(selected), entry.path);
    } else {
      setSelection([...selectedPaths.value, entry.path], entry.path);
    }
    return;
  }
  setSelection([entry.path], entry.path);
}

const toggleFocusedSelection = () => {
  const entry = focusedOrSelectedEntry();
  if (!entry) return false;
  const selected = selectedSet();
  if (selected.has(entry.path)) {
    selected.delete(entry.path);
  } else {
    selected.add(entry.path);
  }
  setSelection(Array.from(selected), entry.path);
  return true;
}

const ensureEntrySelected = (entry: ExplorerEntry) => {
  if (!isSelected(entry.path)) {
    setSelection([entry.path], entry.path);
  }
}

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

const selectAllEntries = () => {
  if (!entries.value.length) return false;
  setSelection(entries.value.map(entry => entry.path), focusedPath.value || entries.value[0]?.path || "");
  return true;
}

const openEntry = async (entry: ExplorerEntry) => {
  if (isRenaming(entry)) return;
  if (entry.type === "folder") {
    await loadFolder(entry.path);
    return;
  }
  if (isImageFile(entry)) {
    emit("open-image-viewer", {entry, entries: entries.value.filter(isImageFile)});
    return;
  }
  if (entry.file && fileStore.extensions.includes(entry.file.extension)) {
    fileStore.showEditor = true;
    fileStore.currentFile = entry.file;
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
    fileStore.showEditor = false;
    await nextTick();
    observePendingThumbnails();
    loaded = true;
  } catch (error) {
    message.value = error instanceof Error ? error.message : "加载目录失败";
  } finally {
    loading.value = false;
    if (loaded) window.requestAnimationFrame(maybeLoadMoreOnScroll);
  }
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

const changeSort = async (key: DirSortKey) => {
  if (loading.value) return;
  fileStore.setSort(key);
  loadedSignature.value = "";
  await loadFolder(fileStore.currentPath || "/");
}

const changeSortOrder = async (order: DirSortOrder) => {
  if (loading.value || fileStore.sortOrder === order) return;
  fileStore.setSort(fileStore.sortKey, order);
  loadedSignature.value = "";
  await loadFolder(fileStore.currentPath || "/");
}

const sortButtonClass = (key: DirSortKey) => ({
  active: sortKey.value === key,
  desc: sortKey.value === key && sortOrder.value === "desc"
});

const sortIndicator = (key: DirSortKey) => {
  if (sortKey.value !== key) return "";
  return sortOrder.value === "asc" ? "↑" : "↓";
}

watch(() => [fileStore.activeTabId, fileStore.currentPath] as const, async ([, path]) => {
  if (!path || fileStore.showEditor) return;
  if (loadedSignature.value === folderRequestSignature(path)) return;
  await loadFolder(path);
});

watch(isIconLikeMode, async iconLike => {
  resetTypeahead();
  if (!iconLike) {
    thumbnailObserver?.disconnect();
    thumbnailObserver = null;
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
});

onBeforeUnmount(() => {
  window.removeEventListener("click", closeContextMenu);
  window.removeEventListener("keydown", handleKeyDown);
  window.removeEventListener("mousemove", handleSelectionMove);
  window.removeEventListener("mouseup", finishMarqueeSelection);
  stopMarqueeAutoScroll();
  resetTypeahead();
  thumbnailObserver?.disconnect();
  itemRefs.clear();
  renameInputRefs.clear();
});

const closeContextMenu = () => {
  contextMenu.visible = false;
}

const clampContextMenuPosition = (x: number, y: number, background: boolean) => {
  const maxX = Math.max(contextMenuViewportPadding, window.innerWidth - contextMenuEstimatedWidth - contextMenuViewportPadding);
  const maxY = Math.max(
      contextMenuViewportPadding,
      window.innerHeight - (background ? contextMenuEstimatedHeights.background : contextMenuEstimatedHeights.entry) - contextMenuViewportPadding
  );
  return {
    x: Math.min(Math.max(contextMenuViewportPadding, x), maxX),
    y: Math.min(Math.max(contextMenuViewportPadding, y), maxY)
  };
}

const showContextMenu = (x: number, y: number, targetPath = "", background = false) => {
  const position = clampContextMenuPosition(x, y, background);
  contextMenu.x = position.x;
  contextMenu.y = position.y;
  contextMenu.targetPath = targetPath;
  contextMenu.background = background;
  contextMenu.visible = true;
}

const openContextMenu = (event: MouseEvent, entry: ExplorerEntry) => {
  ensureEntrySelected(entry);
  showContextMenu(event.clientX, event.clientY, entry.path);
}

const openBackgroundContextMenu = (event: MouseEvent) => {
  if (event.target instanceof HTMLElement && event.target.closest(".entry-item")) return;
  viewportRef.value?.focus();
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

const isContextMultiSelect = computed(() => contextSelectionCount.value > 1);

const contextLabel = (single: string, multiple: string) => {
  return isContextMultiSelect.value ? `${multiple}（${contextSelectionCount.value} 项）` : single;
}

const canExtract = (entry: ExplorerEntry | null) => {
  if (!entry || entry.type !== "file") return false;
  const name = entry.name.toLowerCase();
  return name.endsWith(".zip") || name.endsWith(".tar.gz") || name.endsWith(".tgz");
}

const isTextLike = (entry: ExplorerEntry) => {
  if (entry.type !== "file") return false;
  const extension = entry.extension?.toLowerCase() ?? "";
  return fileStore.extensions.includes(extension) || ["txt", "log", "md", "json", "yaml", "yml", "toml", "xml", "csv"].includes(extension);
}

const fileIcon = (entry: ExplorerEntry) => {
  if (entry.type === "folder") return "icon-folder-fill";
  const extension = entry.extension?.toLowerCase() ?? "";
  if (["zip", "rar", "7z", "tar", "gz", "tgz"].includes(extension) || entry.name.toLowerCase().endsWith(".tar.gz")) {
    return "icon-file-zip-fill";
  }
  if (isImageFile(entry)) return "icon-file-image-fill";
  if (isTextLike(entry)) return "icon-file-common-filling";
  return "icon-file-fill";
}

const isDimmed = (entry: ExplorerEntry) => props.dimmedPaths.includes(entry.path);

const isDragged = (entry: ExplorerEntry) => draggingEntries.value.some(item => item.path === entry.path);

const isDropTarget = (entry: ExplorerEntry) => dragState.active && dragState.overPath === entry.path;

const canDropOnEntry = (entry: ExplorerEntry) => {
  if (entry.type !== "folder") return false;
  if (!draggingEntries.value.length) return false;
  return !draggingEntries.value.some(item => item.path === entry.path || entry.path.startsWith(`${item.path}/`));
}

const dragHintText = computed(() => {
  if (!dragState.active || !draggingEntries.value.length) return "";
  const actionText = dragState.copy ? "复制" : "移动";
  return `${actionText} ${draggingEntries.value.length} 项`;
});

const selectedEntriesForDrag = (entry: ExplorerEntry) => {
  if (selectedPaths.value.includes(entry.path)) return selectedEntries.value;
  return [entry];
}

const beginEntryDrag = (event: DragEvent, entry: ExplorerEntry) => {
  if (isRenaming(entry)) return;
  const entriesToDrag = selectedEntriesForDrag(entry);
  if (!entriesToDrag.length) return;
  if (!isSelected(entry.path)) setSelection([entry.path], entry.path);
  draggingEntries.value = entriesToDrag;
  dragState.active = true;
  dragState.overPath = "";
  dragState.copy = Boolean(event.ctrlKey || event.metaKey);
  contextMenu.visible = false;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "copyMove";
    event.dataTransfer.dropEffect = dragState.copy ? "copy" : "move";
    event.dataTransfer.setData("text/plain", entriesToDrag.map(item => item.path).join("\n"));
  }
}

const resetEntryDrag = () => {
  draggingEntries.value = [];
  dragState.active = false;
  dragState.overPath = "";
  dragState.overCurrentFolder = false;
  dragState.copy = false;
}

const dragOverEntry = (event: DragEvent, entry: ExplorerEntry) => {
  if (!dragState.active || !canDropOnEntry(entry)) return;
  event.preventDefault();
  event.stopPropagation();
  dragState.overPath = entry.path;
  dragState.overCurrentFolder = false;
  dragState.copy = Boolean(event.ctrlKey || event.metaKey);
  if (event.dataTransfer) event.dataTransfer.dropEffect = dragState.copy ? "copy" : "move";
}

const dragLeaveEntry = (event: DragEvent, entry: ExplorerEntry) => {
  if (!dragState.active || dragState.overPath !== entry.path) return;
  const related = event.relatedTarget;
  const element = itemRefs.get(entry.path);
  if (related instanceof Node && element?.contains(related)) return;
  dragState.overPath = "";
}

const dropOnEntry = (event: DragEvent, entry: ExplorerEntry) => {
  if (!dragState.active || !canDropOnEntry(entry)) return;
  event.preventDefault();
  event.stopPropagation();
  const entriesToDrop = draggingEntries.value;
  const action = event.ctrlKey || event.metaKey ? "copy" : "move";
  resetEntryDrag();
  emit("drop-entries", {entries: entriesToDrop, target: entry, action});
}

const isInternalEntryDrag = (event: DragEvent) => {
  const types = Array.from(event.dataTransfer?.types ?? []);
  return dragState.active && types.includes("text/plain");
}

const isEntryDragSurface = (target: EventTarget | null) => target instanceof HTMLElement && Boolean(target.closest(".entry-item"));

const dragOverCurrentFolder = (event: DragEvent) => {
  if (!isInternalEntryDrag(event)) return;
  if (isEntryDragSurface(event.target)) {
    dragState.overCurrentFolder = false;
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  dragState.overPath = "";
  dragState.overCurrentFolder = true;
  dragState.copy = Boolean(event.ctrlKey || event.metaKey);
  if (event.dataTransfer) event.dataTransfer.dropEffect = dragState.copy ? "copy" : "move";
}

const dragLeaveCurrentFolder = (event: DragEvent) => {
  if (!dragState.overCurrentFolder) return;
  const related = event.relatedTarget;
  if (related instanceof Node && viewportRef.value?.contains(related)) return;
  dragState.overCurrentFolder = false;
}

const dropOnCurrentFolder = (event: DragEvent) => {
  if (!isInternalEntryDrag(event) || !dragState.overCurrentFolder) return;
  if (isEntryDragSurface(event.target)) return;
  event.preventDefault();
  event.stopPropagation();
  const entriesToDrop = draggingEntries.value;
  const action = event.ctrlKey || event.metaKey ? "copy" : "move";
  resetEntryDrag();
  emit("drop-to-current-folder", {entries: entriesToDrop, action});
}

const formatDate = (srcDate: string) => {
  if (!srcDate) return "-";
  const date = new Date(srcDate);
  if (Number.isNaN(date.getTime())) return srcDate;
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit"
  }).format(date);
}

const formatSize = (size?: number) => {
  if (!size) return "-";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = size;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

const entryTypeText = (entry: ExplorerEntry) => {
  if (entry.type === "folder") return "文件夹";
  return entry.extension ? `${entry.extension.toUpperCase()} 文件` : "文件";
}

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
  if ((event.key === " " || event.code === "Space") && !event.altKey && !event.shiftKey && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    contextMenu.visible = false;
    toggleFocusedSelection();
    return;
  }
  if ((event.key === " " || event.code === "Space") && !event.altKey && !event.ctrlKey && !event.metaKey) {
    event.preventDefault();
    contextMenu.visible = false;
    const entry = focusedOrSelectedEntry();
    if (entry?.type === "file") emit("preview", entry);
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "c") {
    event.preventDefault();
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

const moveFocus = (key: string, extend: boolean, preserveSelection = false) => {
  if (!entries.value.length) return;
  const current = focusedPath.value ? indexOfPath(focusedPath.value) : -1;
  const columns = currentColumns();
  const pageStep = currentPageStep(columns);
  let nextIndex = current < 0 ? 0 : current;
  if (current >= 0) {
    if (key === "ArrowDown") nextIndex = Math.min(entries.value.length - 1, current + columns);
    if (key === "ArrowUp") nextIndex = Math.max(0, current - columns);
    if (key === "ArrowRight") nextIndex = Math.min(entries.value.length - 1, current + 1);
    if (key === "ArrowLeft") nextIndex = Math.max(0, current - 1);
    if (key === "PageDown") nextIndex = Math.min(entries.value.length - 1, current + pageStep);
    if (key === "PageUp") nextIndex = Math.max(0, current - pageStep);
  }
  if (key === "Home") nextIndex = 0;
  if (key === "End") nextIndex = entries.value.length - 1;
  const entry = entries.value[nextIndex];
  if (!entry) return;
  if (preserveSelection) {
    focusedPath.value = entry.path;
  } else if (extend) {
    selectRange(entry.path, false);
  } else {
    setSelection([entry.path], entry.path);
  }
  nextTick(() => itemRefs.get(entry.path)?.scrollIntoView({block: "nearest", inline: "nearest"}));
}

const focusEntryByTypeahead = (entry: ExplorerEntry) => {
  setSelection([entry.path], entry.path);
  nextTick(() => itemRefs.get(entry.path)?.scrollIntoView({block: "nearest", inline: "nearest"}));
}

const selectPath = async (path: string, additive = false) => {
  const entry = entryByPath(path);
  if (!entry) return false;
  setSelection(additive ? [...selectedPaths.value, entry.path] : [entry.path], entry.path);
  await nextTick();
  itemRefs.get(entry.path)?.scrollIntoView({block: "nearest", inline: "nearest"});
  return true;
}

const selectPaths = async (paths: string[]) => {
  const existingPaths = paths.filter(path => Boolean(entryByPath(path)));
  if (!existingPaths.length) return false;
  setSelection(existingPaths, existingPaths[existingPaths.length - 1]);
  await nextTick();
  itemRefs.get(existingPaths[existingPaths.length - 1])?.scrollIntoView({block: "nearest", inline: "nearest"});
  return true;
}

const selectPathForRename = async (path: string) => {
  const entry = entryByPath(path);
  if (!entry) return false;
  await selectPath(path);
  startRename(entry);
  return true;
}

const findTypeaheadEntry = (query: string, startIndex: number) => {
  if (!query || !entries.value.length) return null;
  const normalizedQuery = query.toLocaleLowerCase("zh-CN");
  const total = entries.value.length;
  for (let offset = 0; offset < total; offset += 1) {
    const index = (startIndex + offset + total) % total;
    const entry = entries.value[index];
    if (entry.name.toLocaleLowerCase("zh-CN").startsWith(normalizedQuery)) return entry;
  }
  return null;
}

const handleTypeahead = (event: KeyboardEvent) => {
  if (event.isComposing || event.ctrlKey || event.metaKey || event.altKey || event.key.length !== 1 || event.key === " ") return false;
  event.preventDefault();
  contextMenu.visible = false;
  const key = event.key.toLocaleLowerCase("zh-CN");
  const previous = typeaheadQuery.value;
  const repeatingSingleKey = Boolean(previous) && Array.from(previous).every(char => char === key);
  const query = repeatingSingleKey ? key : `${previous}${key}`;
  const currentIndex = focusedPath.value ? indexOfPath(focusedPath.value) : -1;
  const startIndex = previous && !repeatingSingleKey ? Math.max(0, currentIndex) : currentIndex + 1;
  let matched = findTypeaheadEntry(query, startIndex);
  let matchedQuery = query;
  if (!matched && query !== key) {
    matched = findTypeaheadEntry(key, currentIndex + 1);
    matchedQuery = key;
  }
  if (matched) {
    focusEntryByTypeahead(matched);
    typeaheadQuery.value = matchedQuery;
  } else {
    typeaheadQuery.value = key;
  }
  scheduleTypeaheadReset();
  return true;
}

const canBeginMarquee = (target: EventTarget | null) => {
  if (target === viewportRef.value) return true;
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".entry-surface")) && !Boolean(target.closest(".entry-item"));
}

const beginMarqueeSelection = (event: MouseEvent) => {
  if (renamingPath.value) return;
  if (event.button !== 0 || !canBeginMarquee(event.target)) return;
  const viewport = viewportRef.value;
  if (!viewport) return;
  const rect = viewport.getBoundingClientRect();
  marqueePointerX = event.clientX;
  marqueePointerY = event.clientY;
  viewport.focus();
  if (!event.ctrlKey && !event.metaKey && !event.shiftKey) {
    clearSelection();
  }
  selectionBox.active = true;
  selectionBox.additive = Boolean(event.ctrlKey || event.metaKey);
  selectionBox.basePaths = selectionBox.additive ? [...selectedPaths.value] : [];
  selectionBox.originX = event.clientX - rect.left + viewport.scrollLeft;
  selectionBox.originY = event.clientY - rect.top + viewport.scrollTop;
  selectionBox.x = selectionBox.originX;
  selectionBox.y = selectionBox.originY;
  selectionBox.width = 0;
  selectionBox.height = 0;
}

const updateSelectionBoxFromPointer = (clientX: number, clientY: number) => {
  if (!viewportRef.value) return;
  const viewport = viewportRef.value;
  const rect = viewport.getBoundingClientRect();
  const currentX = clientX - rect.left + viewport.scrollLeft;
  const currentY = clientY - rect.top + viewport.scrollTop;
  selectionBox.x = Math.min(selectionBox.originX, currentX);
  selectionBox.y = Math.min(selectionBox.originY, currentY);
  selectionBox.width = Math.abs(currentX - selectionBox.originX);
  selectionBox.height = Math.abs(currentY - selectionBox.originY);
  updateMarqueeSelection();
}

const marqueeScrollSpeed = (pointer: number, start: number, end: number) => {
  if (pointer < start + marqueeScrollEdge) {
    const ratio = Math.min(1, (start + marqueeScrollEdge - pointer) / marqueeScrollEdge);
    return -Math.ceil(ratio * marqueeMaxScrollSpeed);
  }
  if (pointer > end - marqueeScrollEdge) {
    const ratio = Math.min(1, (pointer - (end - marqueeScrollEdge)) / marqueeScrollEdge);
    return Math.ceil(ratio * marqueeMaxScrollSpeed);
  }
  return 0;
}

const runMarqueeAutoScroll = () => {
  marqueeScrollFrame = 0;
  const viewport = viewportRef.value;
  if (!selectionBox.active || !viewport) return;
  const rect = viewport.getBoundingClientRect();
  const dx = marqueeScrollSpeed(marqueePointerX, rect.left, rect.right);
  const dy = marqueeScrollSpeed(marqueePointerY, rect.top, rect.bottom);
  if (!dx && !dy) return;
  const beforeLeft = viewport.scrollLeft;
  const beforeTop = viewport.scrollTop;
  viewport.scrollBy({left: dx, top: dy});
  if (viewport.scrollLeft === beforeLeft && viewport.scrollTop === beforeTop) return;
  updateSelectionBoxFromPointer(marqueePointerX, marqueePointerY);
  marqueeScrollFrame = window.requestAnimationFrame(runMarqueeAutoScroll);
}

const scheduleMarqueeAutoScroll = () => {
  if (marqueeScrollFrame) return;
  marqueeScrollFrame = window.requestAnimationFrame(runMarqueeAutoScroll);
}

const handleSelectionMove = (event: MouseEvent) => {
  if (!selectionBox.active) return;
  marqueePointerX = event.clientX;
  marqueePointerY = event.clientY;
  updateSelectionBoxFromPointer(event.clientX, event.clientY);
  scheduleMarqueeAutoScroll();
}

const updateMarqueeSelection = () => {
  if (!viewportRef.value) return;
  const viewport = viewportRef.value;
  const viewportRect = viewport.getBoundingClientRect();
  const box = {
    left: selectionBox.x,
    top: selectionBox.y,
    right: selectionBox.x + selectionBox.width,
    bottom: selectionBox.y + selectionBox.height
  };
  const marqueePaths = entries.value.filter(entry => {
    const element = itemRefs.get(entry.path);
    if (!element) return false;
    const rect = element.getBoundingClientRect();
    const item = {
      left: rect.left - viewportRect.left + viewport.scrollLeft,
      top: rect.top - viewportRect.top + viewport.scrollTop,
      right: rect.right - viewportRect.left + viewport.scrollLeft,
      bottom: rect.bottom - viewportRect.top + viewport.scrollTop
    };
    return item.left <= box.right && item.right >= box.left && item.top <= box.bottom && item.bottom >= box.top;
  }).map(entry => entry.path);
  const selected = selectionBox.additive
      ? Array.from(new Set([...selectionBox.basePaths, ...marqueePaths]))
      : marqueePaths;
  selectedPaths.value = selected;
  focusedPath.value = marqueePaths[marqueePaths.length - 1] ?? selected[selected.length - 1] ?? "";
}

const finishMarqueeSelection = () => {
  if (!selectionBox.active) return;
  resetSelectionBox();
  if (focusedPath.value) anchorPath.value = focusedPath.value;
}

const activateViewport = () => {
  viewportRef.value?.focus();
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

const downloadContextEntry = () => {
  const entry = primaryContextEntry.value;
  closeContextMenu();
  if (entry) emit("download", entry);
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
  startRename: () => startRename(firstSelectedEntry()),
  selectPath,
  selectPaths,
  selectPathForRename,
  selectAllEntries
})
</script>

<template>
  <section class="explorer-shell">
    <div class="explorer-command-row">
      <div class="explorer-summary">
        <span>{{ totalCountText }}</span>
        <span>{{ selectedCountText }}</span>
        <span>排序：{{ sortText }}</span>
      </div>
      <div class="explorer-controls">
        <div class="sort-switch" aria-label="排序方式">
          <button
              v-for="option in sortOptions"
              :key="option.key"
              :class="{active: sortKey === option.key}"
              :disabled="loading"
              :title="`按${option.label}排序`"
              @click="changeSort(option.key)">
            <span>{{ option.label }}</span>
            <span class="sort-chip-indicator">{{ sortIndicator(option.key) }}</span>
          </button>
          <button class="order-toggle" :disabled="loading" :title="`切换为${nextSortOrder === 'asc' ? '升序' : '降序'}`" @click="changeSortOrder(nextSortOrder)">
            {{ sortOrder === "asc" ? "升序" : "降序" }}
          </button>
        </div>
        <div class="view-switch" aria-label="查看模式">
          <button :class="{active: viewMode === 'details'}" title="详细信息" @click="setViewMode('details')">
            <icon icon="icon-view-list" />
          </button>
          <button :class="{active: viewMode === 'list'}" title="列表" @click="setViewMode('list')">
            <icon icon="icon-listview" />
          </button>
          <button :class="{active: viewMode === 'icons'}" title="图标" @click="setViewMode('icons')">
            <icon icon="icon-viewgrid" />
          </button>
          <button :class="{active: viewMode === 'tiles'}" title="平铺" @click="setViewMode('tiles')">
            <icon icon="icon-file-common-filling" />
          </button>
          <button title="图标大小" @click="cycleIconSize">
            <span class="size-mark">{{ iconSizeText }}</span>
          </button>
        </div>
      </div>
    </div>

    <div
        ref="viewportRef"
        class="explorer-viewport"
        :class="[viewMode, itemSizeClass, {dropCurrent: dragState.overCurrentFolder}]"
        tabindex="0"
        @click="activateViewport"
        @mousedown="beginMarqueeSelection"
        @scroll="maybeLoadMoreOnScroll"
        @wheel="handleViewportWheel"
        @dragover="dragOverCurrentFolder"
        @dragleave="dragLeaveCurrentFolder"
        @drop="dropOnCurrentFolder"
        @contextmenu.prevent="openBackgroundContextMenu">
      <div v-if="viewMode === 'details'" class="details-header">
        <button class="sort-button name-cell" :class="sortButtonClass('name')" :disabled="loading" @click.stop="changeSort('name')">
          <span>名称</span>
          <span class="sort-indicator">{{ sortIndicator('name') }}</span>
        </button>
        <button class="sort-button" :class="sortButtonClass('modified')" :disabled="loading" @click.stop="changeSort('modified')">
          <span>修改日期</span>
          <span class="sort-indicator">{{ sortIndicator('modified') }}</span>
        </button>
        <span class="header-cell">类型</span>
        <button class="sort-button size-cell" :class="sortButtonClass('size')" :disabled="loading" @click.stop="changeSort('size')">
          <span>大小</span>
          <span class="sort-indicator">{{ sortIndicator('size') }}</span>
        </button>
      </div>

      <div v-if="loading" class="explorer-empty">正在加载...</div>
      <div v-else-if="message" class="explorer-empty error">{{ message }}</div>
      <div v-else-if="!entries.length" class="explorer-empty">此文件夹为空</div>

      <div v-else class="entry-surface" @mousedown="beginMarqueeSelection">
        <article
            v-for="entry in entries"
            :key="entry.path"
            :ref="element => setItemRef(entry.path, element)"
            class="entry-item"
            :class="{selected: isSelected(entry.path), focused: focusedPath === entry.path, image: isImageFile(entry), dimmed: isDimmed(entry), dragging: isDragged(entry), dropTarget: isDropTarget(entry)}"
            :title="entry.name"
            draggable="true"
            @click.stop="selectEntry(entry, $event)"
            @auxclick.stop="handleAuxClick($event, entry)"
            @dblclick.stop="openEntry(entry)"
            @dragstart.stop="beginEntryDrag($event, entry)"
            @dragend="resetEntryDrag"
            @dragover="dragOverEntry($event, entry)"
            @dragleave="dragLeaveEntry($event, entry)"
            @drop="dropOnEntry($event, entry)"
            @contextmenu.prevent.stop="openContextMenu($event, entry)">
          <div class="entry-visual">
            <img
                v-if="shouldLoadThumbnail(entry)"
                :src="thumbnailUrl(entry)"
                :alt="entry.name"
                loading="lazy"
                decoding="async"
                @error="handleThumbnailError(entry)">
            <icon v-else :icon="fileIcon(entry)" />
          </div>
          <div class="entry-main">
            <input
                v-if="isRenaming(entry)"
                :ref="element => setRenameInputRef(entry.path, element)"
                v-model="renameDraft"
                class="entry-rename-input"
                :disabled="renameSubmitting"
                @click.stop
                @mousedown.stop
                @dblclick.stop
                @keydown.enter.prevent="commitRename"
                @keydown.esc.prevent="cancelRename"
                @blur="commitRename">
            <span v-else class="entry-name">{{ entry.name }}</span>
            <span v-if="viewMode !== 'details'" class="entry-meta">{{ entryTypeText(entry) }}</span>
          </div>
          <span v-if="viewMode === 'details'" class="entry-date">{{ formatDate(entry.modified) }}</span>
          <span v-if="viewMode === 'details'" class="entry-type">{{ entryTypeText(entry) }}</span>
          <span v-if="viewMode === 'details'" class="entry-size">{{ formatSize(entry.size) }}</span>
          <span v-if="viewMode === 'tiles'" class="entry-tile-meta">{{ formatDate(entry.modified) }} · {{ formatSize(entry.size) }}</span>
        </article>

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

    <div v-if="contextMenu.visible && contextMenu.background" class="context-menu" :style="{left: `${contextMenu.x}px`, top: `${contextMenu.y}px`}">
      <button @click="createFileFromContext">新建文件</button>
      <button @click="createFolderFromContext">新建文件夹</button>
      <div class="context-separator"></div>
      <button :disabled="!props.canPaste" @click="pasteIntoCurrentFolder">粘贴</button>
      <button :disabled="!entries.length" @click="selectAllFromContext">全选</button>
    </div>

    <div v-else-if="contextMenu.visible" class="context-menu" :style="{left: `${contextMenu.x}px`, top: `${contextMenu.y}px`}">
      <button @click="openEntryFromContext">打开</button>
      <button :disabled="!primaryContextEntry || primaryContextEntry.type !== 'folder'" @click="openContextEntryInNewTab">在新标签页中打开</button>
      <button :disabled="!primaryContextEntry || primaryContextEntry.type !== 'file'" @click="previewContextEntry">预览</button>
      <div class="context-separator"></div>
      <button :disabled="!contextSelectionCount" @click="cutContextEntries">{{ contextLabel("剪切", "剪切选中项") }}</button>
      <button :disabled="!contextSelectionCount" @click="copyContextEntries">{{ contextLabel("复制", "复制选中项") }}</button>
      <button :disabled="!props.canPaste" @click="pasteIntoCurrentFolder">粘贴</button>
      <div class="context-separator"></div>
      <button :disabled="!primaryContextEntry || primaryContextEntry.type !== 'file'" @click="downloadContextEntry">下载</button>
      <button :disabled="!contextSelectionCount" @click="archiveContextEntries">{{ contextLabel("压缩", "压缩选中项") }}</button>
      <button :disabled="!canExtract(primaryContextEntry)" @click="extractContextEntry">解压</button>
      <div class="context-separator"></div>
      <button :disabled="!primaryContextEntry || isContextMultiSelect" @click="renameContextEntry">重命名</button>
      <button class="danger" :disabled="!primaryContextEntry" @click="deleteContextEntries">{{ contextLabel("删除", "删除选中项") }}</button>
    </div>
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.explorer-shell {
  @apply relative flex h-full min-h-0 flex-col overflow-hidden bg-white;
}

.explorer-command-row {
  @apply flex min-h-9 shrink-0 items-center justify-between gap-2 border-b border-slate-200 px-3 py-1 text-xs text-slate-500;
}

.explorer-summary {
  @apply flex min-w-0 items-center gap-3 truncate;
}

.explorer-controls {
  @apply flex shrink-0 items-center gap-2;
}

.sort-switch,
.view-switch {
  @apply inline-flex shrink-0 overflow-hidden rounded-md border border-slate-200 bg-slate-50;
}

.sort-switch button,
.view-switch button {
  @apply inline-flex h-7 min-w-8 items-center justify-center gap-1 border-r border-slate-200 px-2 text-slate-600 last:border-r-0 hover:bg-white disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:bg-slate-50;
}

.sort-switch button.active,
.view-switch button.active {
  @apply bg-blue-600 text-white hover:bg-blue-600;
}

.sort-switch .order-toggle {
  @apply min-w-12 font-medium;
}

.sort-chip-indicator {
  @apply inline-flex w-2 justify-center text-[10px];
}

.size-mark {
  @apply text-[11px] leading-none;
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

.details-header {
  @apply sticky top-0 z-10 grid h-9 grid-cols-[minmax(14rem,1fr)_11rem_9rem_7rem] items-center border-b border-slate-200 bg-white px-4 text-sm text-slate-600;
}

.details-header > .header-cell {
  @apply truncate px-2;
}

.sort-button {
  @apply flex h-full min-w-0 items-center justify-between gap-1 truncate px-2 text-left text-sm text-slate-600 hover:bg-blue-50 disabled:pointer-events-none;
}

.sort-button.active {
  @apply bg-blue-50 text-blue-700;
}

.sort-button span:first-child {
  @apply min-w-0 truncate;
}

.sort-button.size-cell {
  @apply text-right;
}

.sort-indicator {
  @apply inline-flex w-3 shrink-0 justify-center text-[11px] text-blue-600;
}

.entry-surface {
  @apply min-h-full p-2;
}

.details .entry-surface {
  @apply flex flex-col gap-0 p-1;
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

.entry-item {
  @apply relative min-w-0 cursor-default rounded-md border border-transparent text-sm text-slate-800 outline-none;
}

.entry-item:hover {
  @apply bg-[#ebf3ff];
}

.entry-item.selected {
  @apply border-[#7aa7f8] bg-[#cfe4ff] text-slate-950;
}

.entry-item.focused {
  @apply ring-1 ring-inset ring-blue-600;
}

.entry-item.dimmed {
  @apply opacity-45;
}

.entry-item.dragging {
  @apply opacity-50;
}

.entry-item.dropTarget {
  @apply border-blue-500 bg-blue-50 ring-2 ring-inset ring-blue-400;
}

.details .entry-item {
  @apply grid h-8 grid-cols-[minmax(14rem,1fr)_11rem_9rem_7rem] items-center px-3;
}

.list .entry-item {
  @apply flex h-8 items-center gap-2 px-2;
}

.icons .entry-item {
  @apply flex h-32 flex-col items-center justify-start gap-2 p-2 text-center;
}

.icons.small .entry-item {
  @apply h-24;
}

.icons.large .entry-item {
  @apply h-40;
}

.tiles .entry-item {
  @apply grid min-h-20 grid-cols-[3.5rem_minmax(0,1fr)] grid-rows-[auto_auto] items-center gap-x-3 gap-y-1 p-2;
}

.entry-visual {
  @apply inline-flex shrink-0 items-center justify-center overflow-hidden text-slate-700;
}

.details .entry-visual,
.list .entry-visual {
  @apply h-5 w-5 text-[1.15rem];
}

.icons .entry-visual {
  @apply h-16 w-20 rounded border border-transparent bg-white text-[3rem];
}

.icons.small .entry-visual {
  @apply h-11 w-14 text-[2.25rem];
}

.icons.large .entry-visual {
  @apply h-24 w-32 text-[4.25rem];
}

.tiles .entry-visual {
  @apply row-span-2 h-14 w-14 rounded border border-slate-200 bg-slate-50 text-[2rem];
}

.icons .entry-item.image .entry-visual,
.tiles .entry-item.image .entry-visual {
  @apply border-slate-200 bg-slate-50 shadow-sm;
}

.entry-visual img {
  @apply h-full w-full rounded object-cover;
}

.details .entry-visual img,
.list .entry-visual img {
  @apply rounded-sm;
}

.entry-main {
  @apply flex min-w-0 items-center gap-2;
}

.details .entry-main {
  @apply px-2;
}

.icons .entry-main {
  @apply flex-col gap-0;
}

.tiles .entry-main {
  @apply flex-col items-start gap-0 self-end;
}

.entry-name {
  @apply min-w-0 truncate;
}

.entry-rename-input {
  @apply h-6 min-w-0 rounded border border-blue-500 bg-white px-1 text-sm text-slate-900 outline-none ring-2 ring-blue-200;
}

.details .entry-rename-input,
.list .entry-rename-input,
.tiles .entry-rename-input {
  @apply w-full;
}

.icons .entry-rename-input {
  @apply w-full text-center;
}

.icons .entry-name {
  @apply line-clamp-2 whitespace-normal break-all;
}

.entry-meta,
.entry-date,
.entry-type,
.entry-size,
.entry-tile-meta {
  @apply truncate text-xs text-slate-500;
}

.entry-date,
.entry-type,
.entry-size {
  @apply px-2 text-sm;
}

.entry-size {
  @apply text-right tabular-nums;
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

.entry-item.selected .entry-meta,
.entry-item.selected .entry-date,
.entry-item.selected .entry-type,
.entry-item.selected .entry-size,
.entry-item.selected .entry-tile-meta {
  @apply text-slate-700;
}

.entry-tile-meta {
  @apply col-start-2 self-start;
}

.explorer-empty {
  @apply flex h-48 items-center justify-center text-sm text-slate-500;
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

.context-menu {
  @apply fixed z-50 w-44 rounded-md border border-slate-200 bg-white py-1 text-sm shadow-xl;
}

.context-menu button {
  @apply block h-8 w-full px-3 text-left text-slate-700 hover:bg-blue-50 disabled:text-slate-300 disabled:hover:bg-white;
}

.context-separator {
  @apply my-1 border-t border-slate-100;
}

.context-menu .danger {
  @apply text-red-600 hover:bg-red-50;
}
</style>
