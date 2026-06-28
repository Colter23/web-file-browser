import {computed, ref, toValue} from "vue";
import type {MaybeRefOrGetter} from "vue";

type ImageZoomPanOptions = {
  minZoom?: MaybeRefOrGetter<number>;
  maxZoom?: MaybeRefOrGetter<number>;
  wheelZoomSpeed?: MaybeRefOrGetter<number>;
  canPan?: () => boolean;
}

type ZoomOrigin = {
  x: number;
  y: number;
}

export const useImageZoomPan = (options: ImageZoomPanOptions = {}) => {
  const minZoom = computed(() => toValue(options.minZoom) ?? 25);
  const maxZoom = computed(() => toValue(options.maxZoom) ?? 500);
  const wheelZoomSpeed = computed(() => toValue(options.wheelZoomSpeed) ?? 480);
  const fit = ref(true);
  const zoom = ref(100);
  const offsetX = ref(0);
  const offsetY = ref(0);
  const dragging = ref(false);
  let pointerId: number | null = null;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragOriginX = 0;
  let dragOriginY = 0;

  const imageStyle = computed(() => ({
    maxWidth: fit.value ? "100%" : "none",
    maxHeight: fit.value ? "100%" : "none",
    transform: fit.value ? "none" : `translate3d(${offsetX.value}px, ${offsetY.value}px, 0) scale(${zoom.value / 100})`,
    transformOrigin: "center center"
  }));

  const zoomText = computed(() => fit.value ? "适应" : `${Math.round(zoom.value)}%`);
  const canPan = computed(() => !fit.value && (options.canPan?.() ?? true));
  const actualSizeActive = computed(() => !fit.value && Math.abs(zoom.value - 100) < 0.5);
  const canZoomOut = computed(() => zoom.value > minZoom.value || fit.value);
  const canZoomIn = computed(() => zoom.value < maxZoom.value || fit.value);

  const clampZoom = (value: number) => Math.min(maxZoom.value, Math.max(minZoom.value, value));

  const zoomTo = (nextZoom: number, origin: ZoomOrigin = {x: 0, y: 0}, previousZoom = zoom.value) => {
    const clampedNextZoom = clampZoom(nextZoom);
    const ratio = clampedNextZoom / Math.max(1, previousZoom);
    if (fit.value) resetPan();
    fit.value = false;
    offsetX.value = origin.x - (origin.x - offsetX.value) * ratio;
    offsetY.value = origin.y - (origin.y - offsetY.value) * ratio;
    zoom.value = clampedNextZoom;
  }

  const stagePointFromWheel = (event: WheelEvent): ZoomOrigin => {
    const stage = event.currentTarget as HTMLElement | null;
    if (!stage) return {x: 0, y: 0};
    const rect = stage.getBoundingClientRect();
    return {
      x: event.clientX - rect.left - rect.width / 2,
      y: event.clientY - rect.top - rect.height / 2
    };
  }

  const fitZoomFromWheelTarget = (event: WheelEvent) => {
    const stage = event.currentTarget as HTMLElement | null;
    const image = stage?.querySelector<HTMLImageElement>("img");
    if (!image?.naturalWidth || !image?.naturalHeight) return zoom.value;
    const widthScale = image.offsetWidth > 0 ? image.offsetWidth / image.naturalWidth : 0;
    const heightScale = image.offsetHeight > 0 ? image.offsetHeight / image.naturalHeight : 0;
    const renderedZoom = Math.max(widthScale, heightScale) * 100;
    return renderedZoom > 0 ? renderedZoom : zoom.value;
  }

  const normalizedWheelDelta = (event: WheelEvent) => {
    const modeFactor = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? 240 : 1;
    return Math.max(-240, Math.min(240, event.deltaY * modeFactor));
  }

  const resetPan = () => {
    offsetX.value = 0;
    offsetY.value = 0;
    dragging.value = false;
    pointerId = null;
  }

  const releasePointer = () => {
    dragging.value = false;
    pointerId = null;
  }

  const resetZoom = () => {
    fit.value = true;
    zoom.value = 100;
    resetPan();
  }

  const zoomImage = (delta: number) => {
    zoomTo(zoom.value + delta);
  }

  const setActualSize = () => {
    fit.value = false;
    zoom.value = 100;
    resetPan();
  }

  const toggleZoomMode = () => {
    if (fit.value) {
      setActualSize();
      return;
    }
    resetZoom();
  }

  const handleWheel = (event: WheelEvent) => {
    event.preventDefault();
    event.stopPropagation();
    const previousZoom = fit.value ? fitZoomFromWheelTarget(event) : zoom.value;
    const nextZoom = previousZoom * Math.pow(2, -normalizedWheelDelta(event) / wheelZoomSpeed.value);
    zoomTo(nextZoom, stagePointFromWheel(event), previousZoom);
  }

  const startPan = (event: PointerEvent) => {
    if (!canPan.value || event.button !== 0) return;
    event.preventDefault();
    const stage = event.currentTarget as HTMLElement;
    pointerId = event.pointerId;
    dragging.value = true;
    dragStartX = event.clientX;
    dragStartY = event.clientY;
    dragOriginX = offsetX.value;
    dragOriginY = offsetY.value;
    stage.setPointerCapture?.(event.pointerId);
  }

  const movePan = (event: PointerEvent) => {
    if (!dragging.value || pointerId !== event.pointerId) return;
    event.preventDefault();
    offsetX.value = dragOriginX + event.clientX - dragStartX;
    offsetY.value = dragOriginY + event.clientY - dragStartY;
  }

  const stopPan = (event: PointerEvent) => {
    if (pointerId !== event.pointerId) return;
    const stage = event.currentTarget as HTMLElement;
    stage.releasePointerCapture?.(event.pointerId);
    dragging.value = false;
    pointerId = null;
  }

  return {
    fit,
    zoom,
    dragging,
    imageStyle,
    zoomText,
    canPan,
    actualSizeActive,
    canZoomOut,
    canZoomIn,
    resetPan,
    releasePointer,
    resetZoom,
    zoomImage,
    setActualSize,
    toggleZoomMode,
    handleWheel,
    startPan,
    movePan,
    stopPan
  };
}
