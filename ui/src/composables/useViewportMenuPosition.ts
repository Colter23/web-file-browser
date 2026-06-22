import {nextTick, ref} from "vue";
import type {Ref} from "vue";

type Point = {
  x: number;
  y: number;
}

type ViewportMenuPositionOptions = {
  menuRef: Ref<HTMLElement | null>;
  padding?: number;
}

export const useViewportMenuPosition = ({menuRef, padding = 8}: ViewportMenuPositionOptions) => {
  const menuPosition = ref<Point>({x: padding, y: padding});

  const clampToViewport = (point: Point, width: number, height: number): Point => {
    const maxX = Math.max(padding, window.innerWidth - width - padding);
    const maxY = Math.max(padding, window.innerHeight - height - padding);
    return {
      x: Math.min(Math.max(padding, point.x), maxX),
      y: Math.min(Math.max(padding, point.y), maxY)
    };
  }

  const placeMenu = async (point: Point) => {
    menuPosition.value = point;
    await nextTick();
    const rect = menuRef.value?.getBoundingClientRect();
    if (!rect) return;
    menuPosition.value = clampToViewport(point, rect.width, rect.height);
  }

  return {
    menuPosition,
    placeMenu
  };
}
