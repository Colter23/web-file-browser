import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {downloadFile, uploadFiles} from "../network/api.ts";

type FileTransferOperationsOptions = {
  currentFolder: () => string;
  refreshCurrent: () => Promise<void>;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
  showError: (error: unknown, fallback: string, title?: string) => void;
  setTaskMessage: (message: string) => void;
}

export const useFileTransferOperations = ({
  currentFolder,
  refreshCurrent,
  showNotice,
  showError,
  setTaskMessage
}: FileTransferOperationsOptions) => {
  const downloadEntry = async (entry: ExplorerEntry | null | undefined) => {
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

  const uploadToCurrentFolder = async (files: FileList | File[]) => {
    const fileList = Array.from(files);
    if (!fileList.length) return;
    try {
      await uploadFiles(currentFolder(), fileList);
      setTaskMessage(`已上传 ${fileList.length} 个文件`);
      await refreshCurrent();
    } catch (error) {
      showError(error, "操作失败");
    }
  }

  const uploadChanged = async (event: Event) => {
    if (!(event.target instanceof HTMLInputElement)) return;
    const input = event.target;
    const selectedFiles = input.files;
    if (!selectedFiles?.length) return;
    const files = Array.from(selectedFiles);
    await uploadToCurrentFolder(files);
    input.value = "";
  }

  return {
    downloadEntry,
    uploadChanged,
    uploadToCurrentFolder
  };
}
