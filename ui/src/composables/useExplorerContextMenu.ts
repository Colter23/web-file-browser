import {computed, reactive} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {normalizePathText} from "../utils/file-path.ts";

type MaybePromise<T = unknown> = T | Promise<T>;

type ImageViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type VideoViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type AudioPlayerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type PdfViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type CopyPathPayload = {
  paths: string[];
}

type ExplorerContextMenuOptions = {
  imageEntries: ComputedRef<ExplorerEntry[]>;
  audioEntries: ComputedRef<ExplorerEntry[]>;
  videoEntries: ComputedRef<ExplorerEntry[]>;
  pdfEntries: ComputedRef<ExplorerEntry[]>;
  selectedPaths: Ref<string[]>;
  selectedEntries: ComputedRef<ExplorerEntry[]>;
  favoritePaths: ComputedRef<string[]>;
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
  isAudioFile: (entry: ExplorerEntry | null | undefined) => boolean;
  isVideoFile: (entry: ExplorerEntry | null | undefined) => boolean;
  isPdfFile: (entry: ExplorerEntry | null | undefined) => boolean;
  canEditEntry: (entry: ExplorerEntry | null) => boolean;
  canExtract: (entry: ExplorerEntry | null | undefined) => boolean;
  startRename: (entry: ExplorerEntry | null) => void;
  selectAllEntries: () => void;
  clearCurrentSelection: () => void;
  invertCurrentSelection: () => void;
  previewEntry: (entry: ExplorerEntry) => void;
  openImageViewer: (payload: ImageViewerPayload) => void;
  openAudioPlayer: (payload: AudioPlayerPayload) => void;
  openVideoViewer: (payload: VideoViewerPayload) => void;
  openPdfViewer: (payload: PdfViewerPayload) => void;
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
  addFavorite: (entry: ExplorerEntry) => MaybePromise;
  removeFavorite: (path: string) => MaybePromise;
}

export const useExplorerContextMenu = ({
  imageEntries,
  audioEntries,
  videoEntries,
  pdfEntries,
  selectedPaths,
  selectedEntries,
  favoritePaths,
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
  isAudioFile,
  isVideoFile,
  isPdfFile,
  canEditEntry,
  canExtract,
  startRename,
  selectAllEntries,
  clearCurrentSelection,
  invertCurrentSelection,
  previewEntry,
  openImageViewer,
  openAudioPlayer,
  openVideoViewer,
  openPdfViewer,
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
  showProperties,
  addFavorite,
  removeFavorite
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

  const runContextAction = (action: () => void, restoreFocus = false) => {
    closeContextMenu();
    action();
    if (restoreFocus) focusViewport();
  }

  const runAsyncContextAction = async (action: () => MaybePromise, restoreFocus = false) => {
    closeContextMenu();
    await action();
    if (restoreFocus) focusViewport();
  }

  const runEntryContextAction = (action: (entry: ExplorerEntry) => void, restoreFocus = false) => {
    const entry = primaryContextEntry.value;
    runContextAction(() => {
      if (entry) action(entry);
    }, restoreFocus);
  }

  const runAsyncEntryContextAction = async (action: (entry: ExplorerEntry) => MaybePromise, restoreFocus = false) => {
    const entry = primaryContextEntry.value;
    await runAsyncContextAction(async () => {
      if (entry) await action(entry);
    }, restoreFocus);
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
  const contextCanPlayAudio = computed(() => Boolean(primaryContextEntry.value && isAudioFile(primaryContextEntry.value)));
  const contextCanViewVideo = computed(() => Boolean(primaryContextEntry.value && isVideoFile(primaryContextEntry.value)));
  const contextCanViewPdf = computed(() => Boolean(primaryContextEntry.value && isPdfFile(primaryContextEntry.value)));
  const contextCanEdit = computed(() => canEditEntry(primaryContextEntry.value));
  const contextCanExtract = computed(() => canExtract(primaryContextEntry.value));
  const contextCanFavorite = computed(() => {
    const entry = primaryContextEntry.value;
    return Boolean(entry && entry.type === "folder" && contextSelectionCount.value === 1 && normalizePathText(entry.path) !== "/");
  });
  const contextFavorite = computed(() => {
    const entry = primaryContextEntry.value;
    if (!entry || entry.type !== "folder") return false;
    const normalized = normalizePathText(entry.path);
    return favoritePaths.value.some(path => normalizePathText(path) === normalized);
  });

  const openEntryFromContext = async () => {
    const entry = primaryContextEntry.value;
    await runAsyncEntryContextAction(openEntry, Boolean(entry && (entry.type === "folder" || !isImageFile(entry) && !isAudioFile(entry) && !isVideoFile(entry) && !isPdfFile(entry) && !canEditEntry(entry))));
  }

  const openContextEntryInNewTab = () => {
    runEntryContextAction(openNewTab);
  }

  const previewContextEntry = () => {
    runEntryContextAction(previewEntry, true);
  }

  const viewImageContextEntry = () => {
    runEntryContextAction(entry => {
      if (isImageFile(entry)) openImageViewer({entry, entries: imageEntries.value});
    });
  }

  const playAudioContextEntry = () => {
    runEntryContextAction(entry => {
      if (isAudioFile(entry)) openAudioPlayer({entry, entries: audioEntries.value});
    });
  }

  const viewVideoContextEntry = () => {
    runEntryContextAction(entry => {
      if (isVideoFile(entry)) openVideoViewer({entry, entries: videoEntries.value});
    });
  }

  const viewPdfContextEntry = () => {
    runEntryContextAction(entry => {
      if (isPdfFile(entry)) openPdfViewer({entry, entries: pdfEntries.value});
    });
  }

  const editContextEntry = async () => {
    await runAsyncEntryContextAction(editEntry);
  }

  const downloadContextEntry = () => {
    runEntryContextAction(downloadEntry, true);
  }

  const copyPathContextEntries = () => {
    const paths = contextMenu.background ? [currentPath()] : contextEntries.value.map(entry => entry.path);
    runContextAction(() => {
      if (paths.length) copyPath({paths});
    }, true);
  }

  const copyContextEntries = () => {
    runEntryContextAction(copyEntry, true);
  }

  const cutContextEntries = () => {
    runEntryContextAction(cutEntry, true);
  }

  const pasteIntoCurrentFolder = () => {
    runContextAction(paste, true);
  }

  const createFileFromContext = () => {
    runContextAction(createFile);
  }

  const createFolderFromContext = () => {
    runContextAction(createFolder);
  }

  const selectAllFromContext = () => {
    runContextAction(selectAllEntries, true);
  }

  const clearSelectionFromContext = () => {
    runContextAction(clearCurrentSelection, true);
  }

  const invertSelectionFromContext = () => {
    runContextAction(invertCurrentSelection, true);
  }

  const archiveContextEntries = () => {
    runEntryContextAction(archiveEntry);
  }

  const extractContextEntry = () => {
    runEntryContextAction(extractEntry);
  }

  const renameContextEntry = () => {
    runEntryContextAction(startRename);
  }

  const deleteContextEntries = () => {
    runEntryContextAction(deleteEntry);
  }

  const showContextProperties = () => {
    const selectedEntries = contextEntries.value;
    runContextAction(() => {
      if (selectedEntries.length) showProperties(selectedEntries);
    });
  }

  const addContextFavorite = async () => {
    await runAsyncEntryContextAction(async entry => {
      if (entry.type === "folder") await addFavorite(entry);
    }, true);
  }

  const removeContextFavorite = async () => {
    const entry = primaryContextEntry.value;
    await runAsyncContextAction(async () => {
      if (entry) await removeFavorite(entry.path);
    }, true);
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
    contextCanPlayAudio,
    contextCanViewVideo,
    contextCanViewPdf,
    contextCanEdit,
    contextCanExtract,
    contextCanFavorite,
    contextFavorite,
    openEntryFromContext,
    openContextEntryInNewTab,
    previewContextEntry,
    viewImageContextEntry,
    playAudioContextEntry,
    viewVideoContextEntry,
    viewPdfContextEntry,
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
    showContextProperties,
    addContextFavorite,
    removeContextFavorite
  };
}
