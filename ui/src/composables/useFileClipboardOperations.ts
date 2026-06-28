import {computed, ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {FileClipboardAction} from "../components/operations/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useI18n} from "../i18n";
import {createCopyTask, createMoveTask} from "../network/api.ts";
import {isSameOrDescendantPath, parentPath} from "../utils/file-path.ts";

type CopyPathPayload = {
  paths: string[];
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

type FileClipboardOptions = {
  currentFolder: () => string;
  refreshCurrent: () => Promise<void>;
  selectedEntry: () => ExplorerEntry | null;
  selectedEntries: (fallback?: ExplorerEntry | null) => ExplorerEntry[];
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
  showError: (error: unknown, fallback: string, title?: string) => void;
  taskStarted: (id: string, label?: string) => Promise<void>;
  setTaskMessage: (message: string) => void;
}

export const useFileClipboardOperations = ({
  currentFolder,
  refreshCurrent,
  selectedEntry,
  selectedEntries,
  showNotice,
  showError,
  taskStarted,
  setTaskMessage
}: FileClipboardOptions) => {
  const {t} = useI18n();
  const fileClipboardAction = ref<FileClipboardAction | null>(null);
  const fileClipboardEntries = ref<ExplorerEntry[]>([]);

  const clipboardPaths = computed(() => fileClipboardEntries.value.map(entry => entry.path));
  const hasClipboard = computed(() => Boolean(fileClipboardAction.value && fileClipboardEntries.value.length));
  const clipboardText = computed(() => {
    if (!hasClipboard.value) return t("clipboard.empty");
    const key = fileClipboardAction.value === "cut" ? "clipboard.cutItems" : "clipboard.copyItems";
    return t(key, {count: fileClipboardEntries.value.length});
  });

  const setFileClipboard = (action: FileClipboardAction, entry = selectedEntry()) => {
    const entries = selectedEntries(entry);
    if (!entries.length) {
      showNotice(t("clipboard.selectFileOrFolder"), "warning");
      return;
    }
    fileClipboardAction.value = action;
    fileClipboardEntries.value = entries;
    const message = t(action === "cut" ? "clipboard.cutDone" : "clipboard.copyDone", {count: entries.length});
    setTaskMessage(message);
    showNotice(message, "success", t("clipboard.updated"));
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
      const message = normalizedPaths.length === 1 ? t("context.pathCopied") : t("context.pathsCopied", {count: normalizedPaths.length});
      showNotice(message, "success", t("context.pathCopiedTitle"));
    } catch {
      showNotice(t("favorite.clipboardDenied"), "error", t("favorite.copyPathFailed"));
    }
  }

  const removeEntriesFromCutClipboard = (paths: string[]) => {
    if (fileClipboardAction.value !== "cut") return;
    const removed = new Set(paths);
    fileClipboardEntries.value = fileClipboardEntries.value.filter(item => !removed.has(item.path));
    if (!fileClipboardEntries.value.length) fileClipboardAction.value = null;
  }

  const pasteSelected = async () => {
    if (!hasClipboard.value || !fileClipboardAction.value) {
      showNotice(t("clipboard.empty"), "warning");
      return;
    }
    const targetPath = currentFolder();
    const entries = fileClipboardEntries.value;
    const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(targetPath, entry.path));
    if (nestedFolder) {
      showNotice(t("clipboard.cannotPasteIntoSelf", {name: nestedFolder.name}), "warning");
      return;
    }
    const sameFolder = entries.some(entry => parentPath(entry.path) === targetPath);
    if (fileClipboardAction.value === "cut" && sameFolder) {
      showNotice(t("clipboard.cutAlreadyHere"), "warning");
      return;
    }
    try {
      const sources = entries.map(item => item.path);
      const task = fileClipboardAction.value === "cut"
          ? await createMoveTask(sources, targetPath)
          : await createCopyTask(sources, targetPath);
      await taskStarted(task.id, fileClipboardAction.value === "cut" ? t("clipboard.moveTask") : t("clipboard.copyTask"));
      if (fileClipboardAction.value === "cut") {
        fileClipboardAction.value = null;
        fileClipboardEntries.value = [];
      }
      await refreshCurrent();
    } catch (error) {
      showError(error, t("clipboard.pasteFailed"), t("clipboard.pasteFailedTitle"));
    }
  }

  const runDroppedEntriesTask = async (entries: ExplorerEntry[], targetPath: string, action: "copy" | "move") => {
    if (!entries.length) return;
    const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(targetPath, entry.path));
    if (nestedFolder) {
      showNotice(t("clipboard.cannotDropIntoSelf", {name: nestedFolder.name}), "warning");
      return;
    }
    const sameFolder = entries.some(entry => parentPath(entry.path) === targetPath);
    if (action === "move" && sameFolder) {
      setTaskMessage(t("clipboard.dropSameTarget"));
      return;
    }
    try {
      const sources = entries.map(item => item.path);
      const task = action === "copy"
          ? await createCopyTask(sources, targetPath)
          : await createMoveTask(sources, targetPath);
      await taskStarted(task.id, action === "copy" ? t("clipboard.copyTask") : t("clipboard.moveTask"));
      if (action === "move") removeEntriesFromCutClipboard(sources);
      await refreshCurrent();
    } catch (error) {
      showError(error, t("clipboard.dropFailed"), t("clipboard.dropFailedTitle"));
    }
  }

  const dropEntriesToFolder = async ({entries, target, action}: DropEntriesPayload) => {
    if (target.type !== "folder") return;
    await runDroppedEntriesTask(entries, target.path, action);
  }

  const dropEntriesToCurrentFolder = async ({entries, action}: DropToCurrentFolderPayload) => {
    await runDroppedEntriesTask(entries, currentFolder(), action);
  }

  return {
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
  };
}
