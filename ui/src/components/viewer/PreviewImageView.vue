<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {useImageZoomPan} from "../../composables/useImageZoomPan.ts";
import {useI18n} from "../../i18n";
import {downloadUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";
import PreviewToolRow from "./PreviewToolRow.vue";

const props = defineProps<{
  entry: ExplorerEntry;
}>();

const emit = defineEmits<{
  (e: "open-image", entry: ExplorerEntry): void;
}>();

const {t} = useI18n();
const previewStageRef = ref<HTMLElement | null>(null);
const imageNaturalWidth = ref(0);
const imageNaturalHeight = ref(0);
const previewStageWidth = ref(0);
const previewStageHeight = ref(0);
let previewResizeObserver: ResizeObserver | null = null;

const previewMinZoom = computed(() => {
  if (!imageNaturalWidth.value || !imageNaturalHeight.value || !previewStageWidth.value || !previewStageHeight.value) return 5;
  const fitScale = Math.min(previewStageWidth.value / imageNaturalWidth.value, previewStageHeight.value / imageNaturalHeight.value);
  return Math.max(1, Math.min(25, Math.floor(fitScale * 100)));
});

const {
  fit: previewImageFit,
  dragging: previewImageDragging,
  imageStyle: previewImageStyle,
  zoomText: previewZoomText,
  canPan: canPanPreviewImage,
  actualSizeActive: previewActualSizeActive,
  canZoomOut: canZoomOutPreviewImage,
  canZoomIn: canZoomInPreviewImage,
  resetZoom: resetPreviewImageZoom,
  releasePointer: releasePreviewImagePointer,
  zoomImage: zoomPreviewImage,
  handleWheel: handlePreviewImageWheel,
  startPan: startPreviewImagePan,
  movePan: movePreviewImagePan,
  stopPan: stopPreviewImagePan
} = useImageZoomPan({minZoom: previewMinZoom, maxZoom: 300});

const openImagePreview = () => emit("open-image", props.entry);

const measurePreviewStage = () => {
  const stage = previewStageRef.value;
  if (!stage) return;
  const styles = getComputedStyle(stage);
  const horizontalPadding = Number.parseFloat(styles.paddingLeft) + Number.parseFloat(styles.paddingRight);
  const verticalPadding = Number.parseFloat(styles.paddingTop) + Number.parseFloat(styles.paddingBottom);
  previewStageWidth.value = Math.max(0, stage.clientWidth - horizontalPadding);
  previewStageHeight.value = Math.max(0, stage.clientHeight - verticalPadding);
}

const handlePreviewImageLoad = (event: Event) => {
  const image = event.currentTarget as HTMLImageElement;
  imageNaturalWidth.value = image.naturalWidth;
  imageNaturalHeight.value = image.naturalHeight;
  void nextTick(measurePreviewStage);
}

watch(() => props.entry.path, () => {
  imageNaturalWidth.value = 0;
  imageNaturalHeight.value = 0;
  resetPreviewImageZoom();
}, {immediate: true});

onMounted(() => {
  measurePreviewStage();
  if (typeof ResizeObserver === "undefined") return;
  previewResizeObserver = new ResizeObserver(measurePreviewStage);
  if (previewStageRef.value) previewResizeObserver.observe(previewStageRef.value);
});

onBeforeUnmount(() => {
  previewResizeObserver?.disconnect();
  releasePreviewImagePointer();
});
</script>

<template>
  <preview-tool-row class="image-preview-tool-row">
    <div class="preview-image-tool-group">
      <button :title="t('preview.zoomOut')" :aria-label="t('preview.zoomOut')" :disabled="!canZoomOutPreviewImage" @click="zoomPreviewImage(-25)">
        <icon icon="viewer.zoom-out" color="currentColor" />
      </button>
      <button
          class="zoom-mode-button"
          :class="{active: previewImageFit || previewActualSizeActive}"
          :title="previewImageFit ? t('preview.currentFit') : t('preview.currentZoom')"
          @click="resetPreviewImageZoom">
        <span>{{ previewImageFit ? t("preview.fit") : previewZoomText }}</span>
      </button>
      <button :title="t('preview.zoomIn')" :aria-label="t('preview.zoomIn')" :disabled="!canZoomInPreviewImage" @click="zoomPreviewImage(25)">
        <icon icon="viewer.zoom-in" color="currentColor" />
      </button>
    </div>
    <button class="open-viewer-button" :title="t('preview.openImage')" @click="openImagePreview">
      <icon icon="view.image" color="currentColor" />
      <span>{{ t("preview.view") }}</span>
    </button>
  </preview-tool-row>
  <div class="preview-body image">
    <div
        ref="previewStageRef"
        class="image-stage"
        :class="{fit: previewImageFit, panning: canPanPreviewImage, dragging: previewImageDragging}"
        @pointerdown="startPreviewImagePan"
        @pointermove="movePreviewImagePan"
        @pointerup="stopPreviewImagePan"
        @pointercancel="stopPreviewImagePan"
        @lostpointercapture="releasePreviewImagePointer"
        @wheel="handlePreviewImageWheel"
        @dblclick="openImagePreview">
      <img :src="downloadUrl(entry.path)" :alt="entry.name" :style="previewImageStyle" @load="handlePreviewImageLoad">
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm;
  background: var(--app-panel-muted);
  color: var(--app-text-muted);
}

.preview-body.image {
  @apply p-2;
}

.image-preview-tool-row {
  @apply gap-2 px-2;
}

.image-preview-tool-row :deep(.preview-tool-actions) {
  @apply min-w-0 grow gap-1.5;
}

.preview-image-tool-group {
  @apply inline-flex h-8 shrink-0 items-center overflow-hidden rounded-lg border p-0.5 shadow-sm;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-control-solid) 76%, transparent);
}

.preview-image-tool-group button,
.open-viewer-button {
  @apply inline-flex h-7 min-w-7 shrink-0 items-center justify-center rounded-md border border-transparent px-1.5 text-xs font-medium transition;
  color: var(--app-text-muted);
}

.preview-image-tool-group button:hover:not(:disabled),
.open-viewer-button:hover:not(:disabled) {
  background: var(--app-control-hover);
  color: var(--app-text);
}

.preview-image-tool-group button:disabled {
  @apply cursor-not-allowed opacity-45;
}

.preview-image-tool-group button.active,
.open-viewer-button.active {
  border-color: color-mix(in srgb, var(--app-accent-border, #bfdbfe) 76%, transparent);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.preview-image-tool-group button:focus-visible,
.open-viewer-button:focus-visible {
  outline: none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.preview-image-tool-group .zoom-mode-button {
  @apply min-w-14 px-2;
}

.preview-image-tool-group .zoom-mode-button span {
  @apply w-10 text-center tabular-nums;
}

.open-viewer-button {
  @apply ml-auto gap-1.5 border;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
}

.open-viewer-button span {
  @apply whitespace-nowrap;
}

.image-stage {
  @apply flex h-full min-h-0 w-full touch-none select-none items-center justify-center overflow-hidden rounded-lg border p-3;
  border-color: var(--app-border-soft);
  background:
      linear-gradient(45deg, color-mix(in srgb, var(--app-control-solid) 72%, transparent) 25%, transparent 25%),
      linear-gradient(-45deg, color-mix(in srgb, var(--app-control-solid) 72%, transparent) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, color-mix(in srgb, var(--app-control-solid) 72%, transparent) 75%),
      linear-gradient(-45deg, transparent 75%, color-mix(in srgb, var(--app-control-solid) 72%, transparent) 75%);
  background-color: var(--app-panel-solid);
  background-position: 0 0, 0 0.5rem, 0.5rem -0.5rem, -0.5rem 0;
  background-size: 1rem 1rem;
}

.image-stage.fit {
  @apply overflow-hidden;
}

.image-stage.panning {
  @apply cursor-grab;
}

.image-stage.dragging {
  @apply cursor-grabbing;
}

.image-stage img {
  @apply rounded object-contain shadow-sm;
  max-width: 100%;
  max-height: 100%;
  user-select: none;
  -webkit-user-drag: none;
}
</style>
