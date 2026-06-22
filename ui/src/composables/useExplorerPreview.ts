import {computed, ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useFileStore} from "../store";

type ImageViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type ExplorerPreviewOptions = {
  getSelectedEntry: () => ExplorerEntry | null;
  getImageEntries: () => ExplorerEntry[];
  shouldPersistSelection: () => boolean;
  persistSelectedPaths: (paths: string[]) => void;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
}

export const useExplorerPreview = ({
  getSelectedEntry,
  getImageEntries,
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
    if (currentSelection.value.length === 1 && currentSelection.value[0]?.type === "folder") return "icon-folder-fill";
    return currentSelection.value.length > 1 ? "icon-view-list" : "icon-file-fill";
  });

  const clearPreviewContent = () => {
    previewUpdateToken += 1;
    previewEntry.value = null;
  }

  const resetImageViewerState = () => {
    imageViewerVisible.value = false;
    imageViewerEntry.value = null;
    imageViewerEntries.value = [];
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

  const closePreviewPanel = () => {
    previewPanelVisible.value = false;
    clearPreviewContent();
    closeImageViewer();
  }

  const setImageViewerEntry = (entry: ExplorerEntry) => {
    imageViewerEntry.value = entry;
  }

  const openImageViewer = async ({entry, entries}: ImageViewerPayload) => {
    if (!await fileStore.requestEditorLeave()) return;
    fileStore.closeEditor();
    imageViewerEntries.value = entries.length ? entries : [entry];
    imageViewerVisible.value = true;
    setImageViewerEntry(entry);
  }

  const openPreviewImageViewer = async () => {
    const entry = previewEntry.value;
    if (!entry) return;
    const entries = getImageEntries();
    await openImageViewer({entry, entries: entries.some(item => item.path === entry.path) ? entries : [entry]});
  }

  const openPreviewEntryImageViewer = async (entry: ExplorerEntry) => {
    previewEntry.value = entry;
    await openPreviewImageViewer();
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
    currentSelection,
    selectedList,
    selectedCount,
    hasSelection,
    singleSelection,
    previewEmptyTitle,
    previewEmptySubtitle,
    previewEmptyIcon,
    clearPreviewContent,
    closePreviewPanel,
    closeImageViewer,
    setImageViewerEntry,
    openImageViewer,
    openPreviewEntryImageViewer,
    previewSelected,
    previewSelectedQuietly,
    showEmptyPreviewPane,
    handleSelectionChange
  };
}
