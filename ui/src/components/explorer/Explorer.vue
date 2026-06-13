<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch} from "vue";
import type {ComponentPublicInstance} from "vue";
import {ExplorerViewMode, FileInfo, FolderData} from "../../class.ts";
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
  originX: number;
  originY: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

const emit = defineEmits<{
  (e: "rename", entry: ExplorerEntry): void;
  (e: "delete", entry: ExplorerEntry): void;
  (e: "download", entry: ExplorerEntry): void;
  (e: "archive", entry: ExplorerEntry): void;
  (e: "extract", entry: ExplorerEntry): void;
  (e: "preview", entry: ExplorerEntry): void;
  (e: "open-new-tab", entry: ExplorerEntry): void;
}>()

const props = withDefaults(defineProps<{
  filterText?: string;
}>(), {
  filterText: ""
})

const fileStore = useFileStore();
const folderData = ref<FolderData>({ path: "/", folder: [], file: [] });
const selectedPaths = ref<string[]>([]);
const focusedPath = ref("");
const anchorPath = ref("");
const loading = ref(false);
const message = ref("");
const viewportRef = ref<HTMLElement | null>(null);
const itemRefs = new Map<string, HTMLElement>();
const contextMenu = reactive({visible: false, x: 0, y: 0, targetPath: ""});
const selectionBox = reactive<SelectionBox>({
  active: false,
  originX: 0,
  originY: 0,
  x: 0,
  y: 0,
  width: 0,
  height: 0
});

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

const viewMode = computed(() => fileStore.viewMode);
const isIconLikeMode = computed(() => fileStore.viewMode === "icons" || fileStore.viewMode === "tiles");
const selectedCountText = computed(() => {
  const count = selectedPaths.value.length;
  if (!count) return "未选择项目";
  return `已选择 ${count} 项`;
});

const totalCountText = computed(() => {
  const folderCount = folderData.value.folderTotal ?? folderData.value.folder?.length ?? 0;
  const fileCount = folderData.value.fileTotal ?? folderData.value.file?.length ?? 0;
  const base = `${folderCount} 个文件夹，${fileCount} 个文件`;
  return props.filterText.trim() ? `${base}，筛选 ${entries.value.length} 项` : base;
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

const setItemRef = (path: string, element: Element | ComponentPublicInstance | null) => {
  if (element instanceof HTMLElement) {
    itemRefs.set(path, element);
  } else {
    itemRefs.delete(path);
  }
}

const selectedSet = () => new Set(selectedPaths.value);

const setSelection = (paths: string[], focusPath = paths[paths.length - 1] ?? "") => {
  selectedPaths.value = Array.from(new Set(paths));
  focusedPath.value = focusPath;
  anchorPath.value = focusPath || anchorPath.value;
}

const clearSelection = () => {
  selectedPaths.value = [];
  focusedPath.value = "";
  anchorPath.value = "";
}

const entryByPath = (path: string) => entries.value.find(entry => entry.path === path);

const firstSelectedEntry = () => {
  if (!selectedPaths.value.length) return null;
  return entryByPath(selectedPaths.value[0]) ?? null;
}

const isSelected = (path: string) => selectedPaths.value.includes(path);

const indexOfPath = (path: string) => entries.value.findIndex(entry => entry.path === path);

const selectRange = (targetPath: string, additive: boolean) => {
  const targetIndex = indexOfPath(targetPath);
  if (targetIndex < 0) return;
  const anchorIndex = anchorPath.value ? indexOfPath(anchorPath.value) : targetIndex;
  const start = Math.min(anchorIndex < 0 ? targetIndex : anchorIndex, targetIndex);
  const end = Math.max(anchorIndex < 0 ? targetIndex : anchorIndex, targetIndex);
  const range = entries.value.slice(start, end + 1).map(entry => entry.path);
  if (additive) {
    setSelection([...selectedPaths.value, ...range], targetPath);
  } else {
    setSelection(range, targetPath);
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

const ensureEntrySelected = (entry: ExplorerEntry) => {
  if (!isSelected(entry.path)) {
    setSelection([entry.path], entry.path);
  }
}

const openEntry = async (entry: ExplorerEntry) => {
  if (entry.type === "folder") {
    await loadFolder(entry.path);
    return;
  }
  if (entry.file && fileStore.extensions.includes(entry.file.extension)) {
    fileStore.showEditor = true;
    fileStore.currentFile = entry.file;
  } else {
    emit("preview", entry);
  }
}

const loadFolder = async (path: string = fileStore.currentPath || "/") => {
  loading.value = true;
  message.value = "";
  try {
    const data = normalizeFolderData(await getFolderData(path, {
      detail: true,
      sort: "name",
      order: "asc",
      includeTotal: true
    }));
    fileStore.saveAndConvertFolderData(data);
    folderData.value = data;
    clearSelection();
    fileStore.setCurrentPath(data.path);
    fileStore.showEditor = false;
  } catch (error) {
    message.value = error instanceof Error ? error.message : "加载目录失败";
  } finally {
    loading.value = false;
  }
}

watch(() => fileStore.currentPath, async (path: string) => {
  if (!path || fileStore.showEditor || path === folderData.value.path) return;
  const cached = fileStore.folderData.get(path);
  if (cached) {
    folderData.value = normalizeFolderData(cached);
    clearSelection();
  } else {
    await loadFolder(path);
  }
});

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
  itemRefs.clear();
});

const closeContextMenu = () => {
  contextMenu.visible = false;
}

const openContextMenu = (event: MouseEvent, entry: ExplorerEntry) => {
  ensureEntrySelected(entry);
  contextMenu.x = event.clientX;
  contextMenu.y = event.clientY;
  contextMenu.targetPath = entry.path;
  contextMenu.visible = true;
}

const contextEntry = () => entryByPath(contextMenu.targetPath) ?? firstSelectedEntry();

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

const isImageFile = (entry: ExplorerEntry) => {
  if (entry.type !== "file") return false;
  const extension = entry.extension?.toLowerCase() ?? "";
  return ["apng", "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "webp"].includes(extension);
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
  if (event.key === "Escape") {
    clearSelection();
    contextMenu.visible = false;
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
    event.preventDefault();
    setSelection(entries.value.map(entry => entry.path), focusedPath.value || entries.value[0]?.path || "");
    return;
  }
  if (event.key === "Enter") {
    event.preventDefault();
    const entry = entryByPath(focusedPath.value) ?? firstSelectedEntry();
    if ((event.ctrlKey || event.metaKey) && entry?.type === "folder") {
      emit("open-new-tab", entry);
      return;
    }
    if (entry) await openEntry(entry);
    return;
  }
  if (event.key === "Delete") {
    const entry = firstSelectedEntry();
    if (entry) emit("delete", entry);
    return;
  }
  if (event.key === "F2") {
    const entry = firstSelectedEntry();
    if (entry) emit("rename", entry);
    return;
  }
  if (!["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
  event.preventDefault();
  moveFocus(event.key, event.shiftKey);
}

const currentColumns = () => {
  if (!isIconLikeMode.value || !viewportRef.value) return 1;
  const first = entries.value[0] ? itemRefs.get(entries.value[0].path) : null;
  if (!first) return 1;
  const itemWidth = first.getBoundingClientRect().width + 8;
  return Math.max(1, Math.floor(viewportRef.value.clientWidth / itemWidth));
}

const moveFocus = (key: string, extend: boolean) => {
  if (!entries.value.length) return;
  const current = focusedPath.value ? indexOfPath(focusedPath.value) : -1;
  const columns = currentColumns();
  let nextIndex = current < 0 ? 0 : current;
  if (key === "ArrowDown") nextIndex = Math.min(entries.value.length - 1, current + columns);
  if (key === "ArrowUp") nextIndex = Math.max(0, current - columns);
  if (key === "ArrowRight") nextIndex = Math.min(entries.value.length - 1, current + 1);
  if (key === "ArrowLeft") nextIndex = Math.max(0, current - 1);
  if (key === "Home") nextIndex = 0;
  if (key === "End") nextIndex = entries.value.length - 1;
  const entry = entries.value[nextIndex];
  if (!entry) return;
  if (extend) {
    selectRange(entry.path, false);
  } else {
    setSelection([entry.path], entry.path);
  }
  nextTick(() => itemRefs.get(entry.path)?.scrollIntoView({block: "nearest", inline: "nearest"}));
}

const canBeginMarquee = (target: EventTarget | null) => {
  if (target === viewportRef.value) return true;
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".entry-surface")) && !Boolean(target.closest(".entry-item"));
}

const beginMarqueeSelection = (event: MouseEvent) => {
  if (event.button !== 0 || !canBeginMarquee(event.target)) return;
  const viewport = viewportRef.value;
  if (!viewport) return;
  const rect = viewport.getBoundingClientRect();
  viewport.focus();
  if (!event.ctrlKey && !event.metaKey && !event.shiftKey) {
    clearSelection();
  }
  selectionBox.active = true;
  selectionBox.originX = event.clientX - rect.left + viewport.scrollLeft;
  selectionBox.originY = event.clientY - rect.top + viewport.scrollTop;
  selectionBox.x = selectionBox.originX;
  selectionBox.y = selectionBox.originY;
  selectionBox.width = 0;
  selectionBox.height = 0;
}

const handleSelectionMove = (event: MouseEvent) => {
  if (!selectionBox.active || !viewportRef.value) return;
  const viewport = viewportRef.value;
  const rect = viewport.getBoundingClientRect();
  const currentX = event.clientX - rect.left + viewport.scrollLeft;
  const currentY = event.clientY - rect.top + viewport.scrollTop;
  selectionBox.x = Math.min(selectionBox.originX, currentX);
  selectionBox.y = Math.min(selectionBox.originY, currentY);
  selectionBox.width = Math.abs(currentX - selectionBox.originX);
  selectionBox.height = Math.abs(currentY - selectionBox.originY);
  updateMarqueeSelection();
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
  const selected = entries.value.filter(entry => {
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
  selectedPaths.value = selected;
  focusedPath.value = selected[selected.length - 1] ?? "";
}

const finishMarqueeSelection = () => {
  if (!selectionBox.active) return;
  selectionBox.active = false;
  if (focusedPath.value) anchorPath.value = focusedPath.value;
}

const activateViewport = () => {
  viewportRef.value?.focus();
}

const setViewMode = (mode: ExplorerViewMode) => {
  fileStore.setViewMode(mode);
  nextTick(() => viewportRef.value?.focus());
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
  closeContextMenu();
  if (entry) emit("rename", entry);
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
  getSelectedEntries: () => selectedEntries.value
})
</script>

<template>
  <section class="explorer-shell">
    <div class="explorer-command-row">
      <div class="explorer-summary">
        <span>{{ totalCountText }}</span>
        <span>{{ selectedCountText }}</span>
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

    <div
        ref="viewportRef"
        class="explorer-viewport"
        :class="[viewMode, itemSizeClass]"
        tabindex="0"
        @click="activateViewport"
        @mousedown="beginMarqueeSelection"
        @contextmenu.prevent>
      <div v-if="viewMode === 'details'" class="details-header">
        <span class="name-cell">名称</span>
        <span>修改日期</span>
        <span>类型</span>
        <span class="size-cell">大小</span>
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
            :class="{selected: isSelected(entry.path), focused: focusedPath === entry.path, image: isImageFile(entry)}"
            :title="entry.name"
            @click.stop="selectEntry(entry, $event)"
            @auxclick.stop="handleAuxClick($event, entry)"
            @dblclick.stop="openEntry(entry)"
            @contextmenu.prevent.stop="openContextMenu($event, entry)">
          <div class="entry-visual">
            <img v-if="isImageFile(entry)" :src="downloadUrl(entry.path)" :alt="entry.name" loading="lazy" decoding="async">
            <icon v-else :icon="fileIcon(entry)" />
          </div>
          <div class="entry-main">
            <span class="entry-name">{{ entry.name }}</span>
            <span v-if="viewMode !== 'details'" class="entry-meta">{{ entryTypeText(entry) }}</span>
          </div>
          <span v-if="viewMode === 'details'" class="entry-date">{{ formatDate(entry.modified) }}</span>
          <span v-if="viewMode === 'details'" class="entry-type">{{ entryTypeText(entry) }}</span>
          <span v-if="viewMode === 'details'" class="entry-size">{{ formatSize(entry.size) }}</span>
          <span v-if="viewMode === 'tiles'" class="entry-tile-meta">{{ formatDate(entry.modified) }} · {{ formatSize(entry.size) }}</span>
        </article>
      </div>

      <div
          v-if="selectionBox.active"
          class="selection-box"
          :style="{left: `${selectionBox.x}px`, top: `${selectionBox.y}px`, width: `${selectionBox.width}px`, height: `${selectionBox.height}px`}">
      </div>
    </div>

    <div v-if="contextMenu.visible" class="context-menu" :style="{left: `${contextMenu.x}px`, top: `${contextMenu.y}px`}">
      <button @click="openEntryFromContext">打开</button>
      <button :disabled="!primaryContextEntry || primaryContextEntry.type !== 'folder'" @click="openContextEntryInNewTab">在新标签页中打开</button>
      <button :disabled="!primaryContextEntry || primaryContextEntry.type !== 'file'" @click="previewContextEntry">预览</button>
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
  @apply flex h-9 shrink-0 items-center justify-between border-b border-slate-200 px-3 text-xs text-slate-500;
}

.explorer-summary {
  @apply flex min-w-0 items-center gap-3 truncate;
}

.view-switch {
  @apply inline-flex shrink-0 overflow-hidden rounded-md border border-slate-200 bg-slate-50;
}

.view-switch button {
  @apply inline-flex h-7 min-w-8 items-center justify-center border-r border-slate-200 px-2 text-slate-600 last:border-r-0 hover:bg-white;
}

.view-switch button.active {
  @apply bg-blue-600 text-white hover:bg-blue-600;
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

.details-header {
  @apply sticky top-0 z-10 grid h-9 grid-cols-[minmax(14rem,1fr)_11rem_9rem_7rem] items-center border-b border-slate-200 bg-white px-4 text-sm text-slate-600;
}

.details-header span {
  @apply truncate px-2;
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
  @apply h-16 w-20 rounded border border-transparent text-[3rem];
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
