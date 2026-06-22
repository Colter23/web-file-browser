import {computed, ref} from "vue";

type ImageZoomPanOptions = {
  minZoom?: number;
  maxZoom?: number;
  zoomStep?: number;
  canPan?: () => boolean;
}

export const useImageZoomPan = (options: ImageZoomPanOptions = {}) => {
  const minZoom = options.minZoom ?? 25;
  const maxZoom = options.maxZoom ?? 500;
  const zoomStep = options.zoomStep ?? 25;
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

  const zoomText = computed(() => fit.value ? "适应" : `${zoom.value}%`);
  const canPan = computed(() => !fit.value && (options.canPan?.() ?? true));
  const actualSizeActive = computed(() => !fit.value && zoom.value === 100);
  const canZoomOut = computed(() => zoom.value > minZoom || fit.value);
  const canZoomIn = computed(() => zoom.value < maxZoom || fit.value);

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
    fit.value = false;
    zoom.value = Math.min(maxZoom, Math.max(minZoom, zoom.value + delta));
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
    zoomImage(event.deltaY < 0 ? zoomStep : -zoomStep);
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
