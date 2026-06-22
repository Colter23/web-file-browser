import {computed, ref} from "vue";

const storageKey = "explorer.previewPaneWidth";
const defaultWidth = 352;
const minWidth = 280;
const maxWidth = 720;
const viewportReserve = 520;

const maxForViewport = () => {
  if (typeof window === "undefined") return maxWidth;
  return Math.max(minWidth, Math.min(maxWidth, window.innerWidth - viewportReserve));
}

const clampWidth = (width: number) => {
  const safeWidth = Number.isFinite(width) ? width : defaultWidth;
  return Math.round(Math.min(Math.max(safeWidth, minWidth), maxForViewport()));
}

const readWidth = () => {
  if (typeof localStorage === "undefined") return clampWidth(defaultWidth);
  try {
    const raw = localStorage.getItem(storageKey);
    return clampWidth(raw ? Number(raw) : defaultWidth);
  } catch {
    return clampWidth(defaultWidth);
  }
}

const writeWidth = (width: number) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(storageKey, String(clampWidth(width)));
  } catch {
    // 本地存储不可用时，只保留本次会话里的宽度。
  }
}

export const usePreviewPaneResize = () => {
  const width = ref(readWidth());
  const resizing = ref(false);
  let startX = 0;
  let startWidth = 0;

  const areaStyle = computed(() => ({
    "--preview-pane-width": `${width.value}px`
  }));

  const setWidth = (nextWidth: number, persist = true) => {
    width.value = clampWidth(nextWidth);
    if (persist) writeWidth(width.value);
  }

  const resetWidth = () => {
    setWidth(defaultWidth);
  }

  const finishResize = () => {
    if (resizing.value) writeWidth(width.value);
    resizing.value = false;
  }

  const handleResizeMove = (event: PointerEvent) => {
    if (!resizing.value) return;
    event.preventDefault();
    setWidth(startWidth + startX - event.clientX, false);
  }

  const startResize = (event: PointerEvent) => {
    if (event.button !== 0) return;
    event.preventDefault();
    startX = event.clientX;
    startWidth = width.value;
    resizing.value = true;
  }

  const handleWindowResize = () => {
    setWidth(width.value);
  }

  const adjustWidth = (delta: number) => {
    setWidth(width.value + delta);
  }

  const handleResizeKeyDown = (event: KeyboardEvent) => {
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      adjustWidth(event.shiftKey ? 64 : 24);
      return;
    }
    if (event.key === "ArrowRight") {
      event.preventDefault();
      adjustWidth(event.shiftKey ? -64 : -24);
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      setWidth(minWidth);
      return;
    }
    if (event.key === "End") {
      event.preventDefault();
      setWidth(maxWidth);
    }
  }

  return {
    width,
    resizing,
    minWidth,
    maxWidth,
    areaStyle,
    startResize,
    handleResizeMove,
    finishResize,
    resetWidth,
    handleWindowResize,
    handleResizeKeyDown
  };
}
