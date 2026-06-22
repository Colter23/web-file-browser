import {computed, reactive, ref} from "vue";
import type {DetailsColumnKey} from "../components/explorer/types.ts";

type DetailsColumnFitOptions<Entry> = {
  entries: Entry[];
  viewport: HTMLElement | null;
  value: (entry: Entry, key: DetailsColumnKey) => string;
}

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

const labels: Record<DetailsColumnKey, string> = {
  name: "名称",
  modified: "修改日期",
  type: "类型",
  size: "大小"
};

const textExtraWidths: Record<DetailsColumnKey, number> = {
  name: 58,
  modified: 28,
  type: 28,
  size: 28
};

const headerExtraWidths: Record<DetailsColumnKey, number> = {
  name: 44,
  modified: 44,
  type: 32,
  size: 44
};

let textMeasureContext: CanvasRenderingContext2D | null = null;

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

const measureTextWidth = (text: string, font: string) => {
  if (typeof document === "undefined") return text.length * 8;
  textMeasureContext ??= document.createElement("canvas").getContext("2d");
  if (!textMeasureContext) return text.length * 8;
  textMeasureContext.font = font;
  return textMeasureContext.measureText(text).width;
}

const readFont = (viewport: HTMLElement | null, selector: string) => {
  if (typeof window === "undefined") return "14px system-ui";
  const element = viewport?.querySelector<HTMLElement>(selector) ?? viewport;
  if (!element) return "14px system-ui";
  const style = window.getComputedStyle(element);
  return `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
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

  const fitColumnToContent = <Entry>(key: DetailsColumnKey, options: DetailsColumnFitOptions<Entry>) => {
    const rowFont = readFont(options.viewport, ".entry-item");
    const headerFont = readFont(options.viewport, ".details-header");
    const headerWidth = measureTextWidth(labels[key], headerFont) + headerExtraWidths[key];
    const contentWidth = options.entries.reduce((maxWidth, entry) => {
      const textWidth = measureTextWidth(options.value(entry, key), rowFont) + textExtraWidths[key];
      return Math.max(maxWidth, textWidth);
    }, 0);

    fitColumn(key, Math.max(headerWidth, contentWidth));
  }

  return {
    gridStyle,
    startResize,
    handleResizeMove,
    finishResize,
    fitColumn,
    fitColumnToContent
  };
}
