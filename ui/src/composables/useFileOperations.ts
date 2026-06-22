import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useFileClipboardOperations} from "./useFileClipboardOperations.ts";
import {useFileMutationOperations} from "./useFileMutationOperations.ts";
import {useFileOperationPanels} from "./useFileOperationPanels.ts";
import {useFileTransferOperations} from "./useFileTransferOperations.ts";

type FileOperationsOptions = {
  currentFolder: () => string;
  refreshCurrent: () => Promise<void>;
  closeShellPanels: () => void;
  getSelectedEntry: () => ExplorerEntry | null;
  getSelectedEntries: () => ExplorerEntry[];
  startExplorerRename: () => void;
  selectPath: (path: string) => Promise<boolean>;
  selectPathForRename: (path: string) => Promise<boolean>;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
  showError: (error: unknown, fallback: string, title?: string) => void;
  taskStarted: (id: string, label?: string) => Promise<void>;
  setTaskMessage: (message: string) => void;
  focusOperationPanel: () => void;
  focusDeleteConfirm: () => void;
  focusPropertiesPanel: () => void;
}

export const useFileOperations = ({
  currentFolder,
  refreshCurrent,
  closeShellPanels,
  getSelectedEntry,
  getSelectedEntries,
  startExplorerRename,
  selectPath,
  selectPathForRename,
  showNotice,
  showError,
  taskStarted,
  setTaskMessage,
  focusOperationPanel,
  focusDeleteConfirm,
  focusPropertiesPanel
}: FileOperationsOptions) => {
  const {
    operationPanel,
    deleteConfirm,
    propertiesPanel,
    operationPanelNameLabel,
    resetOperationPanel,
    resetDeleteConfirm,
    closePropertiesPanel,
    closePanels,
    openOperationPanel,
    closeOperationPanel,
    openDeleteConfirm,
    closeDeleteConfirm,
    openPropertiesPanel
  } = useFileOperationPanels({
    closeShellPanels,
    focusOperationPanel,
    focusDeleteConfirm,
    focusPropertiesPanel
  });

  const selectedEntry = () => getSelectedEntry();

  const selectedEntries = (fallback?: ExplorerEntry | null) => {
    const selected = getSelectedEntries();
    if (selected.length) return selected;
    return fallback ? [fallback] : [];
  }

  const {
    fileClipboardAction,
    clipboardPaths,
    hasClipboard,
    clipboardText,
    removeEntriesFromCutClipboard,
    copySelected,
    cutSelected,
    copyEntryPaths,
    pasteSelected,
    dropEntriesToFolder,
    dropEntriesToCurrentFolder
  } = useFileClipboardOperations({
    currentFolder,
    refreshCurrent,
    selectedEntry,
    selectedEntries,
    showNotice,
    showError,
    taskStarted,
    setTaskMessage
  });

  const singleSelectedEntry = (entry = selectedEntry()) => {
    const selected = selectedEntries(entry);
    if (selected.length > 1) return null;
    return selected[0] ?? null;
  }

  const {
    downloadEntry,
    uploadChanged,
    uploadToCurrentFolder
  } = useFileTransferOperations({
    currentFolder,
    refreshCurrent,
    showNotice,
    showError,
    setTaskMessage
  });

  const showProperties = async (entries = selectedEntries()) => {
    if (!entries.length) {
      showNotice("请选择文件或文件夹", "warning");
      return;
    }
    await openPropertiesPanel(entries);
  }

  const downloadSelected = async (entry = singleSelectedEntry()) => {
    await downloadEntry(entry);
  }

  const {
    creatingShortcutFolder,
    openCreatePanel,
    createFolderFromShortcut,
    startRenameSelected,
    renameSelected,
    deleteSelected,
    submitDeleteConfirm,
    archiveSelected,
    extractSelected,
    submitOperationPanel
  } = useFileMutationOperations({
    currentFolder,
    refreshCurrent,
    selectedEntry,
    selectedEntries,
    singleSelectedEntry,
    startExplorerRename,
    selectPath,
    selectPathForRename,
    showNotice,
    showError,
    taskStarted,
    setTaskMessage,
    removeEntriesFromCutClipboard,
    operationPanel,
    deleteConfirm,
    operationPanelNameLabel,
    resetOperationPanel,
    resetDeleteConfirm,
    closePanels,
    openOperationPanel,
    openDeleteConfirm
  });

  return {
    fileClipboardAction,
    creatingShortcutFolder,
    operationPanel,
    deleteConfirm,
    propertiesPanel,
    clipboardPaths,
    hasClipboard,
    clipboardText,
    resetOperationPanel,
    resetDeleteConfirm,
    closePropertiesPanel,
    closeDeleteConfirm,
    showProperties,
    closeOperationPanel,
    openCreatePanel,
    createFolderFromShortcut,
    startRenameSelected,
    renameSelected,
    deleteSelected,
    submitDeleteConfirm,
    downloadSelected,
    copySelected,
    cutSelected,
    copyEntryPaths,
    pasteSelected,
    dropEntriesToFolder,
    dropEntriesToCurrentFolder,
    archiveSelected,
    extractSelected,
    submitOperationPanel,
    uploadChanged,
    uploadToCurrentFolder
  };
}
