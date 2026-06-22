import {computed, reactive} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type MaybePromise<T = unknown> = T | Promise<T>;

type ImageViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type CopyPathPayload = {
  paths: string[];
}

type ExplorerContextMenuOptions = {
  imageEntries: ComputedRef<ExplorerEntry[]>;
  selectedPaths: Ref<string[]>;
  selectedEntries: ComputedRef<ExplorerEntry[]>;
  focusedPath: Ref<string>;
  viewportRef: Ref<HTMLElement | null>;
  itemRefs: Map<string, HTMLElement>;
  currentPath: () => string;
  entryByPath: (path: string) => ExplorerEntry | undefined;
  firstSelectedEntry: () => ExplorerEntry | null;
  ensureEntrySelected: (entry: ExplorerEntry) => void;
  focusViewport: () => void;
  openEntry: (entry: ExplorerEntry) => MaybePromise;
  openNewTab: (entry: ExplorerEntry) => void;
  editEntry: (entry: ExplorerEntry) => MaybePromise;
  isImageFile: (entry: ExplorerEntry | null | undefined) => boolean;
  canEditEntry: (entry: ExplorerEntry | null) => boolean;
  canExtract: (entry: ExplorerEntry | null | undefined) => boolean;
  startRename: (entry: ExplorerEntry | null) => void;
  selectAllEntries: () => void;
  clearCurrentSelection: () => void;
  invertCurrentSelection: () => void;
  previewEntry: (entry: ExplorerEntry) => void;
  openImageViewer: (payload: ImageViewerPayload) => void;
  downloadEntry: (entry: ExplorerEntry) => void;
  copyPath: (payload: CopyPathPayload) => void;
  copyEntry: (entry: ExplorerEntry) => void;
  cutEntry: (entry: ExplorerEntry) => void;
  paste: () => void;
  createFile: () => void;
  createFolder: () => void;
  archiveEntry: (entry: ExplorerEntry) => void;
  extractEntry: (entry: ExplorerEntry) => void;
  deleteEntry: (entry: ExplorerEntry) => void;
  showProperties: (entries: ExplorerEntry[]) => void;
}

export const useExplorerContextMenu = ({
  imageEntries,
  selectedPaths,
  selectedEntries,
  focusedPath,
  viewportRef,
  itemRefs,
  currentPath,
  entryByPath,
  firstSelectedEntry,
  ensureEntrySelected,
  focusViewport,
  openEntry,
  openNewTab,
  editEntry,
  isImageFile,
  canEditEntry,
  canExtract,
  startRename,
  selectAllEntries,
  clearCurrentSelection,
  invertCurrentSelection,
  previewEntry,
  openImageViewer,
  downloadEntry,
  copyPath,
  copyEntry,
  cutEntry,
  paste,
  createFile,
  createFolder,
  archiveEntry,
  extractEntry,
  deleteEntry,
  showProperties
}: ExplorerContextMenuOptions) => {
  const contextMenu = reactive({visible: false, x: 0, y: 0, targetPath: "", background: false});

  const closeContextMenu = () => {
    contextMenu.visible = false;
  }

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
    const focusedEntry = entryByPath(focusedPath.value) ?? firstSelectedEntry();
    if (focusedEntry) {
      ensureEntrySelected(focusedEntry);
      const rect = itemRefs.get(focusedEntry.path)?.getBoundingClientRect();
      const x = rect ? rect.left + Math.min(36, rect.width - 8) : window.innerWidth / 2;
      const y = rect ? rect.top + Math.min(28, rect.height) : window.innerHeight / 2;
      showContextMenu(x, y, focusedEntry.path);
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
  const primaryContextEntry = computed(() => contextEntry());
  const contextCanViewImage = computed(() => Boolean(primaryContextEntry.value && isImageFile(primaryContextEntry.value)));
  const contextCanEdit = computed(() => canEditEntry(primaryContextEntry.value));
  const contextCanExtract = computed(() => canExtract(primaryContextEntry.value));

  const openEntryFromContext = async () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) await openEntry(entry);
  }

  const openContextEntryInNewTab = () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) openNewTab(entry);
  }

  const previewContextEntry = () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) previewEntry(entry);
  }

  const viewImageContextEntry = () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry && isImageFile(entry)) openImageViewer({entry, entries: imageEntries.value});
  }

  const editContextEntry = async () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) await editEntry(entry);
  }

  const downloadContextEntry = () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) downloadEntry(entry);
  }

  const copyPathContextEntries = () => {
    const paths = contextMenu.background ? [currentPath()] : contextEntries.value.map(entry => entry.path);
    closeContextMenu();
    if (paths.length) copyPath({paths});
  }

  const copyContextEntries = () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) copyEntry(entry);
  }

  const cutContextEntries = () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) cutEntry(entry);
  }

  const pasteIntoCurrentFolder = () => {
    closeContextMenu();
    paste();
  }

  const createFileFromContext = () => {
    closeContextMenu();
    createFile();
  }

  const createFolderFromContext = () => {
    closeContextMenu();
    createFolder();
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
    if (entry) archiveEntry(entry);
  }

  const extractContextEntry = () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) extractEntry(entry);
  }

  const renameContextEntry = () => {
    const entry = primaryContextEntry.value;
    if (entry) startRename(entry);
  }

  const deleteContextEntries = () => {
    const entry = primaryContextEntry.value;
    closeContextMenu();
    if (entry) deleteEntry(entry);
  }

  const showContextProperties = () => {
    const selectedEntries = contextEntries.value;
    closeContextMenu();
    if (selectedEntries.length) showProperties(selectedEntries);
  }

  return {
    contextMenu,
    closeContextMenu,
    closeContextMenuAndFocus,
    openContextMenu,
    openBackgroundContextMenu,
    openKeyboardContextMenu,
    contextEntries,
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
  };
}
