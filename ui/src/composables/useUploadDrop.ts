import {computed, ref} from "vue";
import {useI18n} from "../i18n";

type UploadDropOptions = {
  canAccept: () => boolean;
  currentFolder: () => string;
  upload: (files: File[]) => Promise<void>;
}

const hasDraggedFiles = (event: DragEvent) => {
  const dataTransfer = event.dataTransfer;
  if (!dataTransfer || !Array.from(dataTransfer.types ?? []).includes("Files")) return false;
  return Array.from(dataTransfer.items ?? []).some(item => item.kind === "file");
}

export const useUploadDrop = ({canAccept, currentFolder, upload}: UploadDropOptions) => {
  const {t} = useI18n();
  const active = ref(false);
  const uploading = ref(false);
  let dragDepth = 0;

  const title = computed(() => uploading.value ? t("upload.uploading") : t("upload.dropToUpload"));
  const subtitle = computed(() => uploading.value ? t("upload.target", {path: currentFolder()}) : t("upload.to", {path: currentFolder()}));

  const reset = () => {
    dragDepth = 0;
    active.value = false;
  }

  const canHandle = (event: DragEvent) => canAccept() && hasDraggedFiles(event);

  const handleDragEnter = (event: DragEvent) => {
    if (!canHandle(event)) return;
    event.preventDefault();
    event.stopPropagation();
    dragDepth += 1;
    active.value = true;
  }

  const handleDragOver = (event: DragEvent) => {
    if (!canHandle(event)) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "copy";
    active.value = true;
  }

  const handleDragLeave = (event: DragEvent) => {
    if (!canAccept() || !active.value) return;
    event.preventDefault();
    event.stopPropagation();
    dragDepth = Math.max(0, dragDepth - 1);
    if (!dragDepth) active.value = false;
  }

  const handleDrop = async (event: DragEvent) => {
    if (!canHandle(event)) return;
    event.preventDefault();
    event.stopPropagation();
    const files = Array.from(event.dataTransfer?.files ?? []);
    reset();
    if (!files.length) return;
    uploading.value = true;
    try {
      await upload(files);
    } finally {
      uploading.value = false;
    }
  }

  return {
    active,
    uploading,
    title,
    subtitle,
    reset,
    handleDragEnter,
    handleDragOver,
    handleDragLeave,
    handleDrop
  };
}
