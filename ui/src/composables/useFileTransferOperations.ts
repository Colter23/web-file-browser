import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useI18n} from "../i18n";
import {downloadFile, uploadFiles} from "../network/api.ts";
import {apiErrorMessage} from "../utils/api-error-message.ts";

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
      const response = await uploadFiles(currentFolder(), fileList);
      const uploadMessage = uploadResultMessage(response.success, response.failed, response.errors);
      setTaskMessage(uploadMessage);
      if (response.failed > 0) showNotice(uploadMessage, "warning", t("upload.partialFailedTitle"), 6000);
      await refreshCurrent();
    } catch (error) {
      showError(error, t("operation.operationFailed"));
    }
  }

  const uploadResultMessage = (
      success: number,
      failed: number,
      errors: Awaited<ReturnType<typeof uploadFiles>>["errors"] = []
  ) => {
    if (failed <= 0) return t("upload.uploaded", {count: success});
    const summary = t("upload.uploadedPartial", {success, failed});
    const firstError = errors[0];
    if (!firstError) return summary;
    return t("upload.uploadedPartialWithReason", {
      summary,
      fileName: firstError.fileName,
      reason: apiErrorMessage(firstError, firstError.message)
    });
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
