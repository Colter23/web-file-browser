import {computed, ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useFileStore} from "../store";

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

type ExplorerPreviewOptions = {
  getSelectedEntry: () => ExplorerEntry | null;
  getImageEntries: () => ExplorerEntry[];
  getAudioEntries: () => ExplorerEntry[];
  getVideoEntries: () => ExplorerEntry[];
  getPdfEntries: () => ExplorerEntry[];
  shouldPersistSelection: () => boolean;
  persistSelectedPaths: (paths: string[]) => void;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
}

export const useExplorerPreview = ({
  getSelectedEntry,
  getImageEntries,
  getAudioEntries,
  getVideoEntries,
  getPdfEntries,
  shouldPersistSelection,
  persistSelectedPaths,
  showNotice
}: ExplorerPreviewOptions) => {
  const fileStore = useFileStore();
  const previewPanelVisible = ref(false);
  const previewEntry = ref<ExplorerEntry | null>(null);
  const previewReloadKey = ref(0);
  const imageViewerVisible = ref(false);
  const imageViewerEntry = ref<ExplorerEntry | null>(null);
  const imageViewerEntries = ref<ExplorerEntry[]>([]);
  const audioPlayerVisible = ref(false);
  const audioPlayerEntry = ref<ExplorerEntry | null>(null);
  const audioPlayerEntries = ref<ExplorerEntry[]>([]);
  const audioPlayerReloadKey = ref(0);
  const videoViewerVisible = ref(false);
  const videoViewerEntry = ref<ExplorerEntry | null>(null);
  const videoViewerEntries = ref<ExplorerEntry[]>([]);
  const pdfViewerVisible = ref(false);
  const pdfViewerEntry = ref<ExplorerEntry | null>(null);
  const pdfViewerEntries = ref<ExplorerEntry[]>([]);
  const currentSelection = ref<ExplorerEntry[]>([]);
  let previewUpdateToken = 0;

  const selectedList = computed(() => currentSelection.value);
  const selectedCount = computed(() => selectedList.value.length);
  const hasSelection = computed(() => selectedCount.value > 0);
  const singleSelection = computed(() => selectedCount.value === 1 ? selectedList.value[0] : null);

  const previewEmptyTitle = computed(() => {
    if (!currentSelection.value.length) return "选择一个文件以预览";
    if (currentSelection.value.length > 1) return `已选择 ${currentSelection.value.length} 项`;
    const entry = currentSelection.value[0];
    return entry?.type === "folder" ? entry.name : "正在准备预览";
  });

  const previewEmptySubtitle = computed(() => {
    if (!currentSelection.value.length) return "";
    if (currentSelection.value.length > 1) return "选择单个文件后显示预览";
    return currentSelection.value[0]?.type === "folder" ? "文件夹不能直接预览" : "";
  });

  const previewEmptyIcon = computed(() => {
    if (currentSelection.value.length === 1 && currentSelection.value[0]?.type === "folder") return "file.folder";
    return currentSelection.value.length > 1 ? "view.details" : "file.file";
  });

  const clearPreviewContent = () => {
    previewUpdateToken += 1;
    previewEntry.value = null;
  }

  const resetPreviewContext = () => {
    clearPreviewContent();
    currentSelection.value = [];
  }

  const resetImageViewerState = () => {
    imageViewerVisible.value = false;
    imageViewerEntry.value = null;
    imageViewerEntries.value = [];
  }

  const resetAudioPlayerState = () => {
    audioPlayerVisible.value = false;
    audioPlayerEntry.value = null;
    audioPlayerEntries.value = [];
  }

  const resetVideoViewerState = () => {
    videoViewerVisible.value = false;
    videoViewerEntry.value = null;
    videoViewerEntries.value = [];
  }

  const resetPdfViewerState = () => {
    pdfViewerVisible.value = false;
    pdfViewerEntry.value = null;
    pdfViewerEntries.value = [];
  }

  const setPreviewEntry = async (entry: ExplorerEntry, force = false) => {
    if (!force && previewEntry.value?.path === entry.path && previewPanelVisible.value) return;
    const token = ++previewUpdateToken;
    if (!await fileStore.requestEditorLeave()) return;
    if (token !== previewUpdateToken) return;
    fileStore.closeEditor();
    previewEntry.value = entry;
    previewReloadKey.value += 1;
    previewPanelVisible.value = true;
  }

  const closeImageViewer = () => {
    const nextPreviewEntry = previewPanelVisible.value && imageViewerEntry.value?.path !== previewEntry.value?.path
        ? imageViewerEntry.value
        : null;
    resetImageViewerState();
    if (nextPreviewEntry) void setPreviewEntry(nextPreviewEntry, true);
  }

  const closeAudioPlayer = () => {
    resetAudioPlayerState();
  }

  const closeVideoViewer = () => {
    const nextPreviewEntry = previewPanelVisible.value && videoViewerEntry.value?.path !== previewEntry.value?.path
        ? videoViewerEntry.value
        : null;
    resetVideoViewerState();
    if (nextPreviewEntry) void setPreviewEntry(nextPreviewEntry, true);
  }

  const closePdfViewer = () => {
    const nextPreviewEntry = previewPanelVisible.value && pdfViewerEntry.value?.path !== previewEntry.value?.path
        ? pdfViewerEntry.value
        : null;
    resetPdfViewerState();
    if (nextPreviewEntry) void setPreviewEntry(nextPreviewEntry, true);
  }

  const closePreviewPanel = () => {
    previewPanelVisible.value = false;
    clearPreviewContent();
    closeImageViewer();
    closeVideoViewer();
    closePdfViewer();
  }

  const setImageViewerEntry = (entry: ExplorerEntry) => {
    imageViewerEntry.value = entry;
  }

  const setAudioPlayerEntry = (entry: ExplorerEntry) => {
    audioPlayerEntry.value = entry;
  }

  const setVideoViewerEntry = (entry: ExplorerEntry) => {
    videoViewerEntry.value = entry;
  }

  const setPdfViewerEntry = (entry: ExplorerEntry) => {
    pdfViewerEntry.value = entry;
  }

  const openImageViewer = async ({entry, entries}: ImageViewerPayload) => {
    if (!await fileStore.requestEditorLeave()) return;
    resetVideoViewerState();
    resetPdfViewerState();
    fileStore.closeEditor();
    imageViewerEntries.value = entries.length ? entries : [entry];
    imageViewerVisible.value = true;
    setImageViewerEntry(entry);
  }

  const openAudioPlayer = ({entry, entries}: AudioPlayerPayload) => {
    audioPlayerEntries.value = entries.length ? entries : [entry];
    audioPlayerVisible.value = true;
    audioPlayerReloadKey.value += 1;
    setAudioPlayerEntry(entry);
  }

  const openVideoViewer = async ({entry, entries}: VideoViewerPayload) => {
    if (!await fileStore.requestEditorLeave()) return;
    resetImageViewerState();
    resetPdfViewerState();
    fileStore.closeEditor();
    videoViewerEntries.value = entries.length ? entries : [entry];
    videoViewerVisible.value = true;
    setVideoViewerEntry(entry);
  }

  const openPdfViewer = async ({entry, entries}: PdfViewerPayload) => {
    if (!await fileStore.requestEditorLeave()) return;
    resetImageViewerState();
    resetVideoViewerState();
    fileStore.closeEditor();
    pdfViewerEntries.value = entries.length ? entries : [entry];
    pdfViewerVisible.value = true;
    setPdfViewerEntry(entry);
  }

  const openPreviewImageViewer = async () => {
    const entry = previewEntry.value;
    if (!entry) return;
    const entries = getImageEntries();
    await openImageViewer({entry, entries: entries.some(item => item.path === entry.path) ? entries : [entry]});
  }

  const openPreviewAudioPlayer = () => {
    const entry = previewEntry.value;
    if (!entry) return;
    const entries = getAudioEntries();
    openAudioPlayer({entry, entries: entries.some(item => item.path === entry.path) ? entries : [entry]});
  }

  const openPreviewEntryAudioPlayer = (entry: ExplorerEntry) => {
    previewEntry.value = entry;
    openPreviewAudioPlayer();
  }

  const openPreviewEntryImageViewer = async (entry: ExplorerEntry) => {
    previewEntry.value = entry;
    await openPreviewImageViewer();
  }

  const openPreviewVideoViewer = async () => {
    const entry = previewEntry.value;
    if (!entry) return;
    const entries = getVideoEntries();
    await openVideoViewer({entry, entries: entries.some(item => item.path === entry.path) ? entries : [entry]});
  }

  const openPreviewEntryVideoViewer = async (entry: ExplorerEntry) => {
    previewEntry.value = entry;
    await openPreviewVideoViewer();
  }

  const openPreviewPdfViewer = async () => {
    const entry = previewEntry.value;
    if (!entry) return;
    const entries = getPdfEntries();
    await openPdfViewer({entry, entries: entries.some(item => item.path === entry.path) ? entries : [entry]});
  }

  const openPreviewEntryPdfViewer = async (entry: ExplorerEntry) => {
    previewEntry.value = entry;
    await openPreviewPdfViewer();
  }

  const previewSelected = async (entry = getSelectedEntry()) => {
    if (!entry || entry.type !== "file") {
      showNotice("请选择文件", "warning");
      return false;
    }
    await setPreviewEntry(entry, true);
    return true;
  }

  const previewSelectedQuietly = async () => {
    const entry = singleSelection.value;
    if (!entry || entry.type !== "file") return false;
    return previewSelected(entry);
  }

  const showEmptyPreviewPane = () => {
    clearPreviewContent();
    previewPanelVisible.value = true;
  }

  const handleSelectionChange = (entries: ExplorerEntry[]) => {
    currentSelection.value = entries;
    if (shouldPersistSelection()) persistSelectedPaths(entries.map(entry => entry.path));
    if (!previewPanelVisible.value || fileStore.showEditor) return;
    const entry = entries.length === 1 ? entries[0] : null;
    if (entry?.type === "file") {
      void setPreviewEntry(entry);
    } else {
      clearPreviewContent();
    }
  }

  return {
    previewPanelVisible,
    previewEntry,
    previewReloadKey,
    imageViewerVisible,
    imageViewerEntry,
    imageViewerEntries,
    audioPlayerVisible,
    audioPlayerEntry,
    audioPlayerEntries,
    audioPlayerReloadKey,
    videoViewerVisible,
    videoViewerEntry,
    videoViewerEntries,
    pdfViewerVisible,
    pdfViewerEntry,
    pdfViewerEntries,
    currentSelection,
    selectedList,
    selectedCount,
    hasSelection,
    singleSelection,
    previewEmptyTitle,
    previewEmptySubtitle,
    previewEmptyIcon,
    clearPreviewContent,
    resetPreviewContext,
    resetImageViewer: resetImageViewerState,
    resetAudioPlayer: resetAudioPlayerState,
    resetVideoViewer: resetVideoViewerState,
    resetPdfViewer: resetPdfViewerState,
    closePreviewPanel,
    closeImageViewer,
    closeAudioPlayer,
    closeVideoViewer,
    closePdfViewer,
    setImageViewerEntry,
    setAudioPlayerEntry,
    setVideoViewerEntry,
    setPdfViewerEntry,
    openImageViewer,
    openAudioPlayer,
    openVideoViewer,
    openPdfViewer,
    openPreviewEntryImageViewer,
    openPreviewEntryAudioPlayer,
    openPreviewEntryVideoViewer,
    openPreviewEntryPdfViewer,
    previewSelected,
    previewSelectedQuietly,
    showEmptyPreviewPane,
    handleSelectionChange
  };
}
