import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useI18n} from "../i18n";
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
  const {t} = useI18n();

  const downloadEntry = async (entry: ExplorerEntry | null | undefined) => {
    if (!entry || entry.type !== "file") {
      showNotice(t("upload.selectFile"), "warning");
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
      showError(error, t("upload.downloadFailed"), t("upload.downloadFailed"));
    }
  }

  const uploadToCurrentFolder = async (files: FileList | File[]) => {
    const fileList = Array.from(files);
    if (!fileList.length) return;
    try {
      await uploadFiles(currentFolder(), fileList);
      setTaskMessage(t("upload.uploaded", {count: fileList.length}));
      await refreshCurrent();
    } catch (error) {
      showError(error, t("operation.operationFailed"));
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
