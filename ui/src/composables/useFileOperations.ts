import {ref} from "vue";
import type {ArchiveFormat} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useFileClipboardOperations} from "./useFileClipboardOperations.ts";
import {useFileOperationPanels} from "./useFileOperationPanels.ts";
import {
  createArchiveTask,
  createDeleteTask,
  createEntry,
  createExtractTask,
  downloadFile,
  moveEntry,
  uploadFiles
} from "../network/api.ts";
import {archiveFormatExtension, archiveStem, isExtractableArchiveEntry} from "../utils/file-entry.ts";
import {joinPath, parentPath} from "../utils/file-path.ts";

type RenamePayload = {
  entry: ExplorerEntry;
  name: string;
}

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
  const creatingShortcutFolder = ref(false);
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

  const isArchiveFile = (entry: ExplorerEntry | null): entry is ExplorerEntry & { type: "file" } => {
    return isExtractableArchiveEntry(entry);
  }

  const runOperation = async (operation: () => Promise<void>) => {
    try {
      await operation();
      await refreshCurrent();
    } catch (error) {
      showError(error, "操作失败");
    }
  }

  const showProperties = async (entries = selectedEntries()) => {
    if (!entries.length) {
      showNotice("请选择文件或文件夹", "warning");
      return;
    }
    await openPropertiesPanel(entries);
  }

  const openCreatePanel = (type: "file" | "folder") => {
    openOperationPanel({
      kind: type === "file" ? "createFile" : "createFolder",
      title: type === "file" ? "新建文件" : "新建文件夹",
      message: `位置：${currentFolder()}`,
      primaryText: "创建",
      name: type === "file" ? "新建文件.txt" : "新建文件夹",
      format: "zip",
      entries: [],
      sourceEntry: null
    });
  }

  const createFolderFromShortcut = async () => {
    if (creatingShortcutFolder.value) return;
    creatingShortcutFolder.value = true;
    closePanels();
    const folderName = "新建文件夹";
    try {
      const created = await createEntry(currentFolder(), "folder", folderName);
      setTaskMessage(`已创建：${folderName}`);
      await refreshCurrent();
      const renamed = await selectPathForRename(created.path);
      if (!renamed) showNotice("新文件夹已创建，但当前页未找到它，请刷新或调整排序后重命名。", "warning");
    } catch (error) {
      showError(error, "新建文件夹失败");
    } finally {
      creatingShortcutFolder.value = false;
    }
  }

  const startRenameSelected = () => {
    if (!singleSelectedEntry()) {
      showNotice("请选择一个文件或文件夹", "warning");
      return;
    }
    startExplorerRename();
  }

  const renameSelected = async ({entry, name}: RenamePayload) => {
    const nextName = name.trim();
    if (!nextName || nextName === entry.name) return;
    try {
      const renamed = await moveEntry(entry.path, joinPath(parentPath(entry.path), nextName));
      setTaskMessage(`已重命名：${nextName}`);
      await refreshCurrent();
      const selected = await selectPath(renamed.path);
      if (!selected) showNotice("已重命名，但当前页未找到该项目，请刷新或调整排序后查看。", "warning");
    } catch (error) {
      showError(error, "重命名失败", "重命名失败");
    }
  }

  const deleteSelected = async (entry = selectedEntry()) => {
    const entries = selectedEntries(entry);
    if (!entries.length) {
      showNotice("请选择文件或文件夹", "warning");
      return;
    }
    await openDeleteConfirm(entries);
  }

  const submitDeleteConfirm = async () => {
    const entries = deleteConfirm.value.entries;
    if (!entries.length || deleteConfirm.value.submitting) return;
    deleteConfirm.value.submitting = true;
    deleteConfirm.value.error = "";
    try {
      const task = await createDeleteTask(entries.map(item => item.path));
      await taskStarted(task.id, "删除任务");
      removeEntriesFromCutClipboard(entries.map(item => item.path));
      resetDeleteConfirm();
      await refreshCurrent();
    } catch (error) {
      deleteConfirm.value.error = error instanceof Error ? error.message : "创建删除任务失败";
    } finally {
      if (deleteConfirm.value.visible) deleteConfirm.value.submitting = false;
    }
  }

  const downloadSelected = async (entry = singleSelectedEntry()) => {
    if (!entry || entry.type !== "file") {
      showNotice("请选择一个文件", "warning");
      return;
    }
    try {
      const blob = await downloadFile(entry.path);
      const url = window.URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = entry.name;
      anchor.click();
      window.URL.revokeObjectURL(url);
    } catch (error) {
      showError(error, "下载失败", "下载失败");
    }
  }

  const archiveSelected = (entry = selectedEntry()) => {
    const entries = selectedEntries(entry);
    if (!entries.length) {
      showNotice("请选择文件或文件夹", "warning");
      return;
    }
    const format: ArchiveFormat = "zip";
    const defaultName = entries.length === 1 ? `${entries[0].name}${archiveFormatExtension(format)}` : `选中项目${archiveFormatExtension(format)}`;
    openOperationPanel({
      kind: "archive",
      title: entries.length === 1 ? `压缩 ${entries[0].name}` : `压缩 ${entries.length} 项`,
      message: `输出位置：${currentFolder()}`,
      primaryText: "开始压缩",
      name: defaultName,
      format,
      entries,
      sourceEntry: null
    });
  }

  const extractSelected = (entry = singleSelectedEntry()) => {
    if (!isArchiveFile(entry)) {
      showNotice("请选择一个 zip、tar.gz 或 tgz 压缩包", "warning");
      return;
    }
    openOperationPanel({
      kind: "extract",
      title: `解压 ${entry.name}`,
      message: `输出位置：${currentFolder()}`,
      primaryText: "开始解压",
      name: archiveStem(entry.name),
      format: "zip",
      entries: [],
      sourceEntry: entry
    });
  }

  const submitOperationPanel = async () => {
    const panel = operationPanel.value;
    if (!panel.kind || panel.submitting) return;
    const name = panel.name.trim();
    if (!name) {
      showNotice(`${operationPanelNameLabel.value}不能为空`, "warning");
      return;
    }
    panel.submitting = true;
    try {
      if (panel.kind === "createFile" || panel.kind === "createFolder") {
        await createEntry(currentFolder(), panel.kind === "createFile" ? "file" : "folder", name);
        setTaskMessage(`已创建：${name}`);
        resetOperationPanel();
        await refreshCurrent();
        return;
      }
      if (panel.kind === "archive") {
        const task = await createArchiveTask(panel.entries.map(item => item.path), currentFolder(), panel.format, name);
        resetOperationPanel();
        await taskStarted(task.id, "压缩任务");
        return;
      }
      if (panel.kind === "extract" && panel.sourceEntry) {
        const task = await createExtractTask(panel.sourceEntry.path, currentFolder(), name);
        resetOperationPanel();
        await taskStarted(task.id, "解压任务");
      }
    } catch (error) {
      operationPanel.value.submitting = false;
      showError(error, "操作失败");
    }
  }

  const uploadChanged = async (event: Event) => {
    const input = event.target as HTMLInputElement;
    if (!input.files?.length) return;
    const files = Array.from(input.files);
    await uploadToCurrentFolder(files);
    input.value = "";
  }

  const uploadToCurrentFolder = async (files: FileList | File[]) => {
    const fileList = Array.from(files);
    if (!fileList.length) return;
    await runOperation(async () => {
      await uploadFiles(currentFolder(), fileList);
      setTaskMessage(`已上传 ${fileList.length} 个文件`);
    });
  }

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
