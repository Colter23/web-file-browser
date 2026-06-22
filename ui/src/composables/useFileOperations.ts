import {computed, nextTick, ref} from "vue";
import type {ArchiveFormat} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import type {FileClipboardAction, OperationPanelState, DeleteConfirmState, PropertiesPanelState} from "../components/operations/types.ts";
import {
  createArchiveTask,
  createCopyTask,
  createDeleteTask,
  createEntry,
  createExtractTask,
  createMoveTask,
  downloadFile,
  moveEntry,
  uploadFiles
} from "../network/api.ts";
import {archiveFormatExtension, archiveStem, isExtractableArchiveEntry} from "../utils/file-entry.ts";
import {isSameOrDescendantPath, joinPath, parentPath} from "../utils/file-path.ts";

type CopyPathPayload = {
  paths: string[];
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
  focusDeleteConfirm: () => void;
  focusPropertiesPanel: () => void;
}

const emptyOperationPanel = (): OperationPanelState => ({
  visible: false,
  kind: null,
  title: "",
  message: "",
  primaryText: "确定",
  name: "",
  format: "zip",
  entries: [],
  sourceEntry: null,
  submitting: false
});

const emptyDeleteConfirm = (): DeleteConfirmState => ({
  visible: false,
  entries: [],
  submitting: false,
  error: ""
});

const emptyPropertiesPanel = (): PropertiesPanelState => ({
  visible: false,
  entries: []
});

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
  focusDeleteConfirm,
  focusPropertiesPanel
}: FileOperationsOptions) => {
  const fileClipboardAction = ref<FileClipboardAction | null>(null);
  const fileClipboardEntries = ref<ExplorerEntry[]>([]);
  const creatingShortcutFolder = ref(false);
  const operationPanel = ref<OperationPanelState>(emptyOperationPanel());
  const deleteConfirm = ref<DeleteConfirmState>(emptyDeleteConfirm());
  const propertiesPanel = ref<PropertiesPanelState>(emptyPropertiesPanel());

  const clipboardPaths = computed(() => fileClipboardEntries.value.map(entry => entry.path));
  const hasClipboard = computed(() => Boolean(fileClipboardAction.value && fileClipboardEntries.value.length));
  const clipboardText = computed(() => {
    if (!hasClipboard.value) return "剪贴板为空";
    const actionText = fileClipboardAction.value === "cut" ? "剪切" : "复制";
    return `${actionText} ${fileClipboardEntries.value.length} 项`;
  });

  const operationPanelNameLabel = computed(() => {
    switch (operationPanel.value.kind) {
      case "createFile":
        return "文件名";
      case "createFolder":
        return "文件夹名";
      case "archive":
        return "压缩包名称";
      case "extract":
        return "解压到文件夹";
      default:
        return "名称";
    }
  });

  const selectedEntry = () => getSelectedEntry();

  const selectedEntries = (fallback?: ExplorerEntry | null) => {
    const selected = getSelectedEntries();
    if (selected.length) return selected;
    return fallback ? [fallback] : [];
  }

  const singleSelectedEntry = (entry = selectedEntry()) => {
    const selected = selectedEntries(entry);
    if (selected.length > 1) return null;
    return selected[0] ?? null;
  }

  const isArchiveFile = (entry: ExplorerEntry | null): entry is ExplorerEntry & { type: "file" } => {
    return isExtractableArchiveEntry(entry);
  }

  const resetOperationPanel = () => {
    operationPanel.value = emptyOperationPanel();
  }

  const resetDeleteConfirm = () => {
    deleteConfirm.value = emptyDeleteConfirm();
  }

  const closePropertiesPanel = () => {
    propertiesPanel.value = emptyPropertiesPanel();
  }

  const closePanels = () => {
    closeShellPanels();
    resetOperationPanel();
    resetDeleteConfirm();
    closePropertiesPanel();
  }

  const runOperation = async (operation: () => Promise<void>) => {
    try {
      await operation();
      await refreshCurrent();
    } catch (error) {
      showError(error, "操作失败");
    }
  }

  const closeDeleteConfirm = () => {
    if (deleteConfirm.value.submitting) return;
    resetDeleteConfirm();
  }

  const showProperties = async (entries = selectedEntries()) => {
    if (!entries.length) {
      showNotice("请选择文件或文件夹", "warning");
      return;
    }
    closeShellPanels();
    resetOperationPanel();
    resetDeleteConfirm();
    propertiesPanel.value = {
      visible: true,
      entries
    };
    await nextTick();
    focusPropertiesPanel();
  }

  const openOperationPanel = (next: Omit<OperationPanelState, "visible" | "submitting">) => {
    closePanels();
    operationPanel.value = {
      ...next,
      visible: true,
      submitting: false
    };
  }

  const closeOperationPanel = () => {
    if (operationPanel.value.submitting) return;
    resetOperationPanel();
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
    closePanels();
    deleteConfirm.value = {
      visible: true,
      entries,
      submitting: false,
      error: ""
    };
    await nextTick();
    focusDeleteConfirm();
  }

  const submitDeleteConfirm = async () => {
    const entries = deleteConfirm.value.entries;
    if (!entries.length || deleteConfirm.value.submitting) return;
    deleteConfirm.value.submitting = true;
    deleteConfirm.value.error = "";
    try {
      const task = await createDeleteTask(entries.map(item => item.path));
      await taskStarted(task.id, "删除任务");
      if (fileClipboardAction.value === "cut") {
        const deleted = new Set(entries.map(item => item.path));
        fileClipboardEntries.value = fileClipboardEntries.value.filter(item => !deleted.has(item.path));
        if (!fileClipboardEntries.value.length) fileClipboardAction.value = null;
      }
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

  const setFileClipboard = (action: FileClipboardAction, entry = selectedEntry()) => {
    const entries = selectedEntries(entry);
    if (!entries.length) {
      showNotice("请选择文件或文件夹", "warning");
      return;
    }
    fileClipboardAction.value = action;
    fileClipboardEntries.value = entries;
    const message = `${action === "cut" ? "已剪切" : "已复制"} ${entries.length} 项`;
    setTaskMessage(message);
    showNotice(message, "success", "剪贴板已更新");
  }

  const copySelected = (entry?: ExplorerEntry) => {
    setFileClipboard("copy", entry ?? selectedEntry());
  }

  const cutSelected = (entry?: ExplorerEntry) => {
    setFileClipboard("cut", entry ?? selectedEntry());
  }

  const copyEntryPaths = async ({paths}: CopyPathPayload) => {
    const normalizedPaths = paths.map(path => path.trim()).filter(Boolean);
    if (!normalizedPaths.length) return;
    try {
      await navigator.clipboard.writeText(normalizedPaths.join("\n"));
      const message = normalizedPaths.length === 1 ? "已复制路径" : `已复制 ${normalizedPaths.length} 个路径`;
      showNotice(message, "success", "路径已复制");
    } catch {
      showNotice("浏览器未允许写入剪贴板，请手动复制路径。", "error", "复制路径失败");
    }
  }

  const pasteSelected = async () => {
    if (!hasClipboard.value || !fileClipboardAction.value) {
      showNotice("剪贴板为空", "warning");
      return;
    }
    const targetPath = currentFolder();
    const entries = fileClipboardEntries.value;
    const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(targetPath, entry.path));
    if (nestedFolder) {
      showNotice(`不能将 ${nestedFolder.name} 粘贴到它自身或子文件夹中`, "warning");
      return;
    }
    const sameFolder = entries.some(entry => parentPath(entry.path) === targetPath);
    if (fileClipboardAction.value === "cut" && sameFolder) {
      showNotice("剪切项已经在当前文件夹中", "warning");
      return;
    }
    try {
      const sources = entries.map(item => item.path);
      const task = fileClipboardAction.value === "cut"
          ? await createMoveTask(sources, targetPath)
          : await createCopyTask(sources, targetPath);
      await taskStarted(task.id, fileClipboardAction.value === "cut" ? "移动任务" : "复制任务");
      if (fileClipboardAction.value === "cut") {
        fileClipboardAction.value = null;
        fileClipboardEntries.value = [];
      }
      await refreshCurrent();
    } catch (error) {
      showError(error, "创建粘贴任务失败", "粘贴失败");
    }
  }

  const removeMovedEntriesFromCutClipboard = (sources: string[]) => {
    if (fileClipboardAction.value !== "cut") return;
    const moved = new Set(sources);
    fileClipboardEntries.value = fileClipboardEntries.value.filter(item => !moved.has(item.path));
    if (!fileClipboardEntries.value.length) fileClipboardAction.value = null;
  }

  const runDroppedEntriesTask = async (entries: ExplorerEntry[], targetPath: string, action: "copy" | "move") => {
    if (!entries.length) return;
    const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(targetPath, entry.path));
    if (nestedFolder) {
      showNotice(`不能将 ${nestedFolder.name} 放入它自身或子文件夹中`, "warning");
      return;
    }
    const sameFolder = entries.some(entry => parentPath(entry.path) === targetPath);
    if (action === "move" && sameFolder) {
      setTaskMessage("拖拽目标已经是当前位置");
      return;
    }
    try {
      const sources = entries.map(item => item.path);
      const task = action === "copy"
          ? await createCopyTask(sources, targetPath)
          : await createMoveTask(sources, targetPath);
      await taskStarted(task.id, action === "copy" ? "复制任务" : "移动任务");
      if (action === "move") removeMovedEntriesFromCutClipboard(sources);
      await refreshCurrent();
    } catch (error) {
      showError(error, "创建拖拽任务失败", "拖拽失败");
    }
  }

  const dropEntriesToFolder = async ({entries, target, action}: DropEntriesPayload) => {
    if (target.type !== "folder") return;
    await runDroppedEntriesTask(entries, target.path, action);
  }

  const dropEntriesToCurrentFolder = async ({entries, action}: DropToCurrentFolderPayload) => {
    await runDroppedEntriesTask(entries, currentFolder(), action);
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
