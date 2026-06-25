import {computed, onBeforeUnmount, onMounted, ref} from "vue";
import type {Ref, StyleValue} from "vue";

type Point = {
  x: number;
  y: number;
}

type DraggablePanelOptions = {
  panelRef: Ref<HTMLElement | null>;
  padding?: number;
}

const interactiveSelector = "button, input, textarea, select, a, [contenteditable='true'], [data-drag-block]";

const isInteractiveTarget = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(interactiveSelector));
}

export const useDraggablePanel = ({panelRef, padding = 12}: DraggablePanelOptions) => {
  const position = ref<Point | null>(null);
  const dragging = ref(false);
  let offsetX = 0;
  let offsetY = 0;

  const clampToViewport = (point: Point): Point => {
    const rect = panelRef.value?.getBoundingClientRect();
    const width = rect?.width ?? 0;
    const height = rect?.height ?? 0;
    const maxX = Math.max(padding, window.innerWidth - width - padding);
    const maxY = Math.max(padding, window.innerHeight - height - padding);
    return {
      x: Math.min(Math.max(point.x, padding), maxX),
      y: Math.min(Math.max(point.y, padding), maxY)
    };
  }

  const panelStyle = computed<StyleValue>(() => {
    if (!position.value) return {};
    return {
      left: `${position.value.x}px`,
      top: `${position.value.y}px`,
      transform: "none"
    };
  });

  const handlePointerMove = (event: PointerEvent) => {
    if (!dragging.value) return;
    event.preventDefault();
    position.value = clampToViewport({
      x: event.clientX - offsetX,
      y: event.clientY - offsetY
    });
  }

  const finishDrag = () => {
    if (!dragging.value) return;
    dragging.value = false;
    window.removeEventListener("pointermove", handlePointerMove);
    window.removeEventListener("pointerup", finishDrag);
    window.removeEventListener("pointercancel", finishDrag);
  }

  const startDrag = (event: PointerEvent) => {
    if (event.button !== 0 || isInteractiveTarget(event.target)) return;
    const panel = panelRef.value;
    if (!panel) return;
    const rect = panel.getBoundingClientRect();
    offsetX = event.clientX - rect.left;
    offsetY = event.clientY - rect.top;
    position.value = clampToViewport({x: rect.left, y: rect.top});
    dragging.value = true;
    event.preventDefault();
    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", finishDrag);
    window.addEventListener("pointercancel", finishDrag);
  }

  const resetPosition = () => {
    position.value = null;
  }

  const handleWindowResize = () => {
    if (!position.value) return;
    position.value = clampToViewport(position.value);
  }

  onMounted(() => {
    window.addEventListener("resize", handleWindowResize);
  });

  onBeforeUnmount(() => {
    finishDrag();
    window.removeEventListener("resize", handleWindowResize);
  });

  return {
    dragging,
    panelStyle,
    resetPosition,
    startDrag
  };
}
