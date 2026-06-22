import {computed, ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {FileClipboardAction} from "../components/operations/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
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
  const fileClipboardAction = ref<FileClipboardAction | null>(null);
  const fileClipboardEntries = ref<ExplorerEntry[]>([]);

  const clipboardPaths = computed(() => fileClipboardEntries.value.map(entry => entry.path));
  const hasClipboard = computed(() => Boolean(fileClipboardAction.value && fileClipboardEntries.value.length));
  const clipboardText = computed(() => {
    if (!hasClipboard.value) return "剪贴板为空";
    const actionText = fileClipboardAction.value === "cut" ? "剪切" : "复制";
    return `${actionText} ${fileClipboardEntries.value.length} 项`;
  });

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

  const removeEntriesFromCutClipboard = (paths: string[]) => {
    if (fileClipboardAction.value !== "cut") return;
    const removed = new Set(paths);
    fileClipboardEntries.value = fileClipboardEntries.value.filter(item => !removed.has(item.path));
    if (!fileClipboardEntries.value.length) fileClipboardAction.value = null;
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
      if (action === "move") removeEntriesFromCutClipboard(sources);
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
