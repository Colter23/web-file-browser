import {reactive} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type SelectionBox = {
  active: boolean;
  additive: boolean;
  basePaths: string[];
  originX: number;
  originY: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

type ExplorerMarqueeSelectionOptions = {
  entries: ComputedRef<ExplorerEntry[]>;
  selectedPaths: Ref<string[]>;
  focusedPath: Ref<string>;
  anchorPath: Ref<string>;
  itemRefs: Map<string, HTMLElement>;
  viewportRef: Ref<HTMLElement | null>;
  isRenaming: () => boolean;
  focusViewport: () => void;
  clearSelection: () => void;
}

export const useExplorerMarqueeSelection = ({
  entries,
  selectedPaths,
  focusedPath,
  anchorPath,
  itemRefs,
  viewportRef,
  isRenaming,
  focusViewport,
  clearSelection
}: ExplorerMarqueeSelectionOptions) => {
  const selectionBox = reactive<SelectionBox>({
    active: false,
    additive: false,
    basePaths: [],
    originX: 0,
    originY: 0,
    x: 0,
    y: 0,
    width: 0,
    height: 0
  });
  let pointerX = 0;
  let pointerY = 0;
  let tracking = false;
  let originClientX = 0;
  let originClientY = 0;
  let scrollFrame = 0;
  const startThreshold = 4;
  const scrollEdge = 48;
  const maxScrollSpeed = 24;
  const interactiveSelector = "button, a, input, textarea, select, [contenteditable='true']";

  const stopAutoScroll = () => {
    if (!scrollFrame) return;
    window.cancelAnimationFrame(scrollFrame);
    scrollFrame = 0;
  }

  const resetSelectionBox = () => {
    stopAutoScroll();
    tracking = false;
    selectionBox.active = false;
    selectionBox.additive = false;
    selectionBox.basePaths = [];
  }

  const canBeginMarquee = (target: EventTarget | null) => {
    if (target === viewportRef.value) return true;
    if (!(target instanceof HTMLElement)) return false;
    if (target.closest(interactiveSelector)) return false;
    return Boolean(target.closest(".entry-surface")) && !Boolean(target.closest(".entry-item"));
  }

  const beginMarqueeSelection = (event: MouseEvent) => {
    if (isRenaming()) return;
    if (event.button !== 0 || !canBeginMarquee(event.target)) return;
    const viewport = viewportRef.value;
    if (!viewport) return;
    const rect = viewport.getBoundingClientRect();
    event.preventDefault();
    pointerX = event.clientX;
    pointerY = event.clientY;
    originClientX = event.clientX;
    originClientY = event.clientY;
    tracking = true;
    focusViewport();
    if (!event.ctrlKey && !event.metaKey && !event.shiftKey) {
      clearSelection();
    }
    selectionBox.active = false;
    selectionBox.additive = Boolean(event.ctrlKey || event.metaKey);
    selectionBox.basePaths = selectionBox.additive ? [...selectedPaths.value] : [];
    selectionBox.originX = event.clientX - rect.left + viewport.scrollLeft;
    selectionBox.originY = event.clientY - rect.top + viewport.scrollTop;
    selectionBox.x = selectionBox.originX;
    selectionBox.y = selectionBox.originY;
    selectionBox.width = 0;
    selectionBox.height = 0;
  }

  const updateSelectionBoxFromPointer = (clientX: number, clientY: number) => {
    const viewport = viewportRef.value;
    if (!viewport) return;
    const rect = viewport.getBoundingClientRect();
    const currentX = clientX - rect.left + viewport.scrollLeft;
    const currentY = clientY - rect.top + viewport.scrollTop;
    selectionBox.x = Math.min(selectionBox.originX, currentX);
    selectionBox.y = Math.min(selectionBox.originY, currentY);
    selectionBox.width = Math.abs(currentX - selectionBox.originX);
    selectionBox.height = Math.abs(currentY - selectionBox.originY);
    updateMarqueeSelection();
  }

  const marqueeScrollSpeed = (pointer: number, start: number, end: number) => {
    if (pointer < start + scrollEdge) {
      const ratio = Math.min(1, (start + scrollEdge - pointer) / scrollEdge);
      return -Math.ceil(ratio * maxScrollSpeed);
    }
    if (pointer > end - scrollEdge) {
      const ratio = Math.min(1, (pointer - (end - scrollEdge)) / scrollEdge);
      return Math.ceil(ratio * maxScrollSpeed);
    }
    return 0;
  }

  const runAutoScroll = () => {
    scrollFrame = 0;
    const viewport = viewportRef.value;
    if (!selectionBox.active || !viewport) return;
    const rect = viewport.getBoundingClientRect();
    const dx = marqueeScrollSpeed(pointerX, rect.left, rect.right);
    const dy = marqueeScrollSpeed(pointerY, rect.top, rect.bottom);
    if (!dx && !dy) return;
    const beforeLeft = viewport.scrollLeft;
    const beforeTop = viewport.scrollTop;
    viewport.scrollBy({left: dx, top: dy});
    if (viewport.scrollLeft === beforeLeft && viewport.scrollTop === beforeTop) return;
    updateSelectionBoxFromPointer(pointerX, pointerY);
    scrollFrame = window.requestAnimationFrame(runAutoScroll);
  }

  const scheduleAutoScroll = () => {
    if (scrollFrame) return;
    scrollFrame = window.requestAnimationFrame(runAutoScroll);
  }

  const shouldStartSelectionBox = (clientX: number, clientY: number) => {
    return Math.abs(clientX - originClientX) >= startThreshold || Math.abs(clientY - originClientY) >= startThreshold;
  }

  const handleSelectionMove = (event: MouseEvent) => {
    if (!tracking) return;
    event.preventDefault();
    pointerX = event.clientX;
    pointerY = event.clientY;
    if (!selectionBox.active) {
      if (!shouldStartSelectionBox(event.clientX, event.clientY)) return;
      selectionBox.active = true;
    }
    updateSelectionBoxFromPointer(event.clientX, event.clientY);
    scheduleAutoScroll();
  }

  const updateMarqueeSelection = () => {
    const viewport = viewportRef.value;
    if (!viewport) return;
    const viewportRect = viewport.getBoundingClientRect();
    const box = {
      left: selectionBox.x,
      top: selectionBox.y,
      right: selectionBox.x + selectionBox.width,
      bottom: selectionBox.y + selectionBox.height
    };
    const marqueePaths = entries.value.filter(entry => {
      const element = itemRefs.get(entry.path);
      if (!element) return false;
      const rect = element.getBoundingClientRect();
      const item = {
        left: rect.left - viewportRect.left + viewport.scrollLeft,
        top: rect.top - viewportRect.top + viewport.scrollTop,
        right: rect.right - viewportRect.left + viewport.scrollLeft,
        bottom: rect.bottom - viewportRect.top + viewport.scrollTop
      };
      return item.left <= box.right && item.right >= box.left && item.top <= box.bottom && item.bottom >= box.top;
    }).map(entry => entry.path);
    const selected = selectionBox.additive
        ? Array.from(new Set([...selectionBox.basePaths, ...marqueePaths]))
        : marqueePaths;
    selectedPaths.value = selected;
    focusedPath.value = marqueePaths[marqueePaths.length - 1] ?? selected[selected.length - 1] ?? "";
  }

  const finishMarqueeSelection = () => {
    if (!tracking && !selectionBox.active) return;
    const finishedActiveBox = selectionBox.active;
    resetSelectionBox();
    if (finishedActiveBox && focusedPath.value) anchorPath.value = focusedPath.value;
  }

  return {
    selectionBox,
    resetSelectionBox,
    beginMarqueeSelection,
    handleSelectionMove,
    finishMarqueeSelection,
    stopAutoScroll
  };
}
