import {ref, type ComputedRef, type Ref} from "vue";
import type {ArchiveFormat} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {DeleteConfirmState, OperationPanelState} from "../components/operations/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useI18n} from "../i18n";
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
  const {t} = useI18n();
  const creatingShortcutFolder = ref(false);

  const openCreatePanel = (type: "file" | "folder") => {
    openOperationPanel({
      kind: type === "file" ? "createFile" : "createFolder",
      title: type === "file" ? t("common.createFile") : t("common.createFolder"),
      message: t("operation.location", {path: currentFolder()}),
      primaryText: t("operation.create"),
      name: type === "file" ? t("operation.newFileName") : t("operation.newFolderName"),
      format: "zip",
      entries: [],
      sourceEntry: null
    });
  }

  const createFolderFromShortcut = async () => {
    if (creatingShortcutFolder.value) return;
    creatingShortcutFolder.value = true;
    closePanels();
    const folderName = t("operation.newFolderName");
    try {
      const created = await createEntry(currentFolder(), "folder", folderName);
      setTaskMessage(t("operation.created", {name: folderName}));
      await refreshCurrent();
      const renamed = await selectPathForRename(created.path);
      if (!renamed) showNotice(t("operation.folderCreatedMissing"), "warning");
    } catch (error) {
      showError(error, t("operation.createFolderFailed"));
    } finally {
      creatingShortcutFolder.value = false;
    }
  }

  const startRenameSelected = () => {
    if (!singleSelectedEntry()) {
      showNotice(t("operation.selectOneFileOrFolder"), "warning");
      return;
    }
    startExplorerRename();
  }

  const renameSelected = async ({entry, name, complete}: RenamePayload) => {
    const nextName = name.trim();
    if (!nextName || nextName === entry.name) return;
    try {
      const renamed = await moveEntry(entry.path, joinPath(parentPath(entry.path), nextName));
      setTaskMessage(t("operation.renamed", {name: nextName}));
      await refreshCurrent();
      const selected = await selectPath(renamed.path);
      if (!selected) showNotice(t("operation.renamedMissing"), "warning");
      complete?.(true);
    } catch (error) {
      showError(error, t("operation.renameFailed"), t("operation.renameFailed"));
      complete?.(false);
    }
  }

  const deleteSelected = async (entry = selectedEntry()) => {
    const entries = selectedEntries(entry);
    if (!entries.length) {
      showNotice(t("clipboard.selectFileOrFolder"), "warning");
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
      await taskStarted(task.id, permanent ? t("operation.permanentDeleteTask") : t("operation.deleteTask"));
      removeEntriesFromCutClipboard(entries.map(item => item.path));
      resetDeleteConfirm();
      await refreshCurrent();
    } catch (error) {
      deleteConfirm.value.error = error instanceof Error ? error.message : t("operation.createDeleteTaskFailed");
    } finally {
      if (deleteConfirm.value.visible) deleteConfirm.value.submitting = false;
    }
  }

  const archiveSelected = (entry = selectedEntry()) => {
    const entries = selectedEntries(entry);
    if (!entries.length) {
      showNotice(t("clipboard.selectFileOrFolder"), "warning");
      return;
    }
    const format: ArchiveFormat = "zip";
    const defaultName = entries.length === 1 ? `${entries[0].name}${archiveFormatExtension(format)}` : `${t("operation.selectedItemsName")}${archiveFormatExtension(format)}`;
    openOperationPanel({
      kind: "archive",
      title: entries.length === 1 ? t("operation.archiveTitle", {name: entries[0].name}) : t("operation.archiveTitleMany", {count: entries.length}),
      message: t("operation.outputLocation", {path: currentFolder()}),
      primaryText: t("operation.startArchive"),
      name: defaultName,
      format,
      entries,
      sourceEntry: null
    });
  }

  const extractSelected = (entry = singleSelectedEntry()) => {
    if (!isArchiveFile(entry)) {
      showNotice(t("operation.selectArchive"), "warning");
      return;
    }
    openOperationPanel({
      kind: "extract",
      title: t("operation.extractTitle", {name: entry.name}),
      message: t("operation.outputLocation", {path: currentFolder()}),
      primaryText: t("operation.startExtract"),
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
      showNotice(t("operation.required", {label: operationPanelNameLabel.value}), "warning");
      return;
    }
    panel.submitting = true;
    try {
      if (panel.kind === "createFile" || panel.kind === "createFolder") {
        await createEntry(currentFolder(), panel.kind === "createFile" ? "file" : "folder", name);
        setTaskMessage(t("operation.created", {name}));
        resetOperationPanel();
        await refreshCurrent();
        return;
      }
      if (panel.kind === "archive") {
        const task = await createArchiveTask(panel.entries.map(item => item.path), currentFolder(), panel.format, name);
        resetOperationPanel();
        await taskStarted(task.id, t("operation.archiveTask"));
        return;
      }
      if (panel.kind === "extract" && panel.sourceEntry) {
        const task = await createExtractTask(panel.sourceEntry.path, currentFolder(), name);
        resetOperationPanel();
        await taskStarted(task.id, t("operation.extractTask"));
      }
    } catch (error) {
      operationPanel.value.submitting = false;
      showError(error, t("operation.operationFailed"));
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
