import {computed, ref} from "vue";
import {readNumberStorage, writeNumberStorage} from "../utils/safe-storage.ts";

const storageKey = "explorer.sidebarWidth";
const defaultWidth = 272;
const minWidth = 220;
const maxWidth = 460;
const viewportReserve = 680;

const maxForViewport = () => {
  if (typeof window === "undefined") return maxWidth;
  return Math.max(minWidth, Math.min(maxWidth, window.innerWidth - viewportReserve));
}

const clampWidth = (width: number) => {
  const safeWidth = Number.isFinite(width) ? width : defaultWidth;
  return Math.round(Math.min(Math.max(safeWidth, minWidth), maxForViewport()));
}

const readWidth = () => {
  return clampWidth(readNumberStorage(storageKey, defaultWidth));
}

const writeWidth = (width: number) => {
  writeNumberStorage(storageKey, clampWidth(width));
}

export const useSidebarResize = () => {
  const width = ref(readWidth());
  const resizing = ref(false);
  let startX = 0;
  let startWidth = 0;

  const workspaceStyle = computed(() => ({
    "--sidebar-width": `${width.value}px`
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
    setWidth(startWidth + event.clientX - startX, false);
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
      adjustWidth(event.shiftKey ? -64 : -24);
      return;
    }
    if (event.key === "ArrowRight") {
      event.preventDefault();
      adjustWidth(event.shiftKey ? 64 : 24);
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
    workspaceStyle,
    startResize,
    handleResizeMove,
    finishResize,
    resetWidth,
    handleWindowResize,
    handleResizeKeyDown
  };
}
