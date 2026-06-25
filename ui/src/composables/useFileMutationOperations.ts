import {ref, type ComputedRef, type Ref} from "vue";
import type {ArchiveFormat} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {DeleteConfirmState, OperationPanelState} from "../components/operations/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {
  createArchiveTask,
  createDeleteTask,
  createEntry,
  createExtractTask,
  moveEntry
} from "../network/api.ts";
import {archiveFormatExtension, archiveStem, isExtractableArchiveEntry} from "../utils/file-entry.ts";
import {joinPath, parentPath} from "../utils/file-path.ts";

export type RenamePayload = {
  entry: ExplorerEntry;
  name: string;
  complete?: (success: boolean) => void;
}

type FileMutationOperationsOptions = {
  currentFolder: () => string;
  refreshCurrent: () => Promise<void>;
  selectedEntry: () => ExplorerEntry | null;
  selectedEntries: (fallback?: ExplorerEntry | null) => ExplorerEntry[];
  singleSelectedEntry: (entry?: ExplorerEntry | null) => ExplorerEntry | null;
  startExplorerRename: () => void;
  selectPath: (path: string) => Promise<boolean>;
  selectPathForRename: (path: string) => Promise<boolean>;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
  showError: (error: unknown, fallback: string, title?: string) => void;
  taskStarted: (id: string, label?: string) => Promise<void>;
  setTaskMessage: (message: string) => void;
  removeEntriesFromCutClipboard: (paths: string[]) => void;
  operationPanel: Ref<OperationPanelState>;
  deleteConfirm: Ref<DeleteConfirmState>;
  operationPanelNameLabel: ComputedRef<string>;
  resetOperationPanel: () => void;
  resetDeleteConfirm: () => void;
  closePanels: () => void;
  openOperationPanel: (next: Omit<OperationPanelState, "visible" | "submitting">) => Promise<void>;
  openDeleteConfirm: (entries: ExplorerEntry[]) => Promise<void>;
}

const isArchiveFile = (entry: ExplorerEntry | null): entry is ExplorerEntry & { type: "file" } => {
  return isExtractableArchiveEntry(entry);
}

export const useFileMutationOperations = ({
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
}: FileMutationOperationsOptions) => {
  const creatingShortcutFolder = ref(false);

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

  const renameSelected = async ({entry, name, complete}: RenamePayload) => {
    const nextName = name.trim();
    if (!nextName || nextName === entry.name) return;
    try {
      const renamed = await moveEntry(entry.path, joinPath(parentPath(entry.path), nextName));
      setTaskMessage(`已重命名：${nextName}`);
      await refreshCurrent();
      const selected = await selectPath(renamed.path);
      if (!selected) showNotice("已重命名，但当前页未找到该项目，请刷新或调整排序后查看。", "warning");
      complete?.(true);
    } catch (error) {
      showError(error, "重命名失败", "重命名失败");
      complete?.(false);
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
      const permanent = deleteConfirm.value.permanent;
      const task = await createDeleteTask(entries.map(item => item.path), permanent);
      await taskStarted(task.id, permanent ? "永久删除任务" : "删除任务");
      removeEntriesFromCutClipboard(entries.map(item => item.path));
      resetDeleteConfirm();
      await refreshCurrent();
    } catch (error) {
      deleteConfirm.value.error = error instanceof Error ? error.message : "创建删除任务失败";
    } finally {
      if (deleteConfirm.value.visible) deleteConfirm.value.submitting = false;
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

  return {
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
  };
}
