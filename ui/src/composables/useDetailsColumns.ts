import {computed, reactive, ref} from "vue";
import type {DetailsColumnKey} from "../components/explorer/types.ts";

type DetailsColumnWidths = Record<DetailsColumnKey, number>;

const storageKey = "explorer.detailsColumnWidths";
const defaultWidths: DetailsColumnWidths = {
  name: 320,
  modified: 176,
  type: 144,
  size: 112
};
const minWidths: DetailsColumnWidths = {
  name: 160,
  modified: 132,
  type: 96,
  size: 88
};
const maxWidths: DetailsColumnWidths = {
  name: 960,
  modified: 320,
  type: 280,
  size: 220
};

const clampWidth = (key: DetailsColumnKey, width: number) => {
  const safeWidth = Number.isFinite(width) ? width : defaultWidths[key];
  return Math.round(Math.min(Math.max(safeWidth, minWidths[key]), maxWidths[key]));
}

const readWidths = (): DetailsColumnWidths => {
  if (typeof localStorage === "undefined") return {...defaultWidths};
  try {
    const raw = localStorage.getItem(storageKey);
    const parsed = raw ? JSON.parse(raw) as Partial<DetailsColumnWidths> : {};
    return {
      name: clampWidth("name", parsed.name ?? defaultWidths.name),
      modified: clampWidth("modified", parsed.modified ?? defaultWidths.modified),
      type: clampWidth("type", parsed.type ?? defaultWidths.type),
      size: clampWidth("size", parsed.size ?? defaultWidths.size)
    };
  } catch {
    return {...defaultWidths};
  }
}

const writeWidths = (widths: DetailsColumnWidths) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(storageKey, JSON.stringify(widths));
  } catch {
    // 本地存储不可用时，只保留本次会话的列宽。
  }
}

export const useDetailsColumns = () => {
  const widths = ref<DetailsColumnWidths>(readWidths());
  const resizeState = reactive<{key: DetailsColumnKey | null; startX: number; startWidth: number}>({
    key: null,
    startX: 0,
    startWidth: 0
  });

  const gridStyle = computed(() => ({
    "--details-name-width": `${widths.value.name}px`,
    "--details-modified-width": `${widths.value.modified}px`,
    "--details-type-width": `${widths.value.type}px`,
    "--details-size-width": `${widths.value.size}px`,
    "--details-grid-width": `${widths.value.name + widths.value.modified + widths.value.type + widths.value.size}px`
  }));

  const startResize = (event: PointerEvent, key: DetailsColumnKey) => {
    event.preventDefault();
    event.stopPropagation();
    if (event.currentTarget instanceof HTMLElement) event.currentTarget.setPointerCapture(event.pointerId);
    resizeState.key = key;
    resizeState.startX = event.clientX;
    resizeState.startWidth = widths.value[key];
  }

  const handleResizeMove = (event: PointerEvent) => {
    const key = resizeState.key;
    if (!key) return;
    widths.value = {
      ...widths.value,
      [key]: clampWidth(key, resizeState.startWidth + event.clientX - resizeState.startX)
    };
  }

  const finishResize = () => {
    if (!resizeState.key) return;
    resizeState.key = null;
    writeWidths(widths.value);
  }

  const fitColumn = (key: DetailsColumnKey, width: number) => {
    widths.value = {
      ...widths.value,
      [key]: clampWidth(key, width)
    };
    writeWidths(widths.value);
  }

  return {
    gridStyle,
    startResize,
    handleResizeMove,
    finishResize,
    fitColumn
  };
}
