<script setup lang="ts">
import {onBeforeUnmount, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {useImageZoomPan} from "../../composables/useImageZoomPan.ts";
import {downloadUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";
import PreviewToolRow from "./PreviewToolRow.vue";

const props = defineProps<{
  entry: ExplorerEntry;
}>();

const emit = defineEmits<{
  (e: "open-image", entry: ExplorerEntry): void;
}>();

const {
  fit: previewImageFit,
  dragging: previewImageDragging,
  imageStyle: previewImageStyle,
  zoomText: previewZoomText,
  canPan: canPanPreviewImage,
  resetZoom: resetPreviewImageZoom,
  releasePointer: releasePreviewImagePointer,
  zoomImage: zoomPreviewImage,
  handleWheel: handlePreviewImageWheel,
  startPan: startPreviewImagePan,
  movePan: movePreviewImagePan,
  stopPan: stopPreviewImagePan
} = useImageZoomPan({maxZoom: 300});

const openImagePreview = () => emit("open-image", props.entry);

watch(() => props.entry.path, () => {
  resetPreviewImageZoom();
}, {immediate: true});

onBeforeUnmount(releasePreviewImagePointer);
</script>

<template>
  <preview-tool-row>
    <button :class="{active: previewImageFit}" title="适应窗口" @click="resetPreviewImageZoom">适应</button>
    <button title="缩小" aria-label="缩小" @click="zoomPreviewImage(-25)">-</button>
    <button title="放大" aria-label="放大" @click="zoomPreviewImage(25)">+</button>
    <button title="打开图片查看" @click="openImagePreview">
      <icon icon="action.fullscreen" color="currentColor" />
      <span>打开查看</span>
    </button>
    <template #status>{{ previewZoomText }}</template>
  </preview-tool-row>
  <div class="preview-body image">
    <div
        class="image-stage"
        :class="{fit: previewImageFit, panning: canPanPreviewImage, dragging: previewImageDragging}"
        @pointerdown="startPreviewImagePan"
        @pointermove="movePreviewImagePan"
        @pointerup="stopPreviewImagePan"
        @pointercancel="stopPreviewImagePan"
        @lostpointercapture="releasePreviewImagePointer"
        @wheel="handlePreviewImageWheel"
        @dblclick="openImagePreview">
      <img :src="downloadUrl(entry.path)" :alt="entry.name" :style="previewImageStyle">
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
