import {computed, ref} from "vue";

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
  const active = ref(false);
  const uploading = ref(false);
  let dragDepth = 0;

  const title = computed(() => uploading.value ? "正在上传文件..." : "释放鼠标上传文件");
  const subtitle = computed(() => uploading.value ? `目标：${currentFolder()}` : `上传到 ${currentFolder()}`);

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
