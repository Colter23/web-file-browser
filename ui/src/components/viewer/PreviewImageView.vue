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
    <button :class="{active: previewImageFit}" @click="resetPreviewImageZoom">适应</button>
    <button @click="zoomPreviewImage(-25)">-</button>
    <button @click="zoomPreviewImage(25)">+</button>
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
  color: var(--app-text-muted);
}

.preview-body.image {
  background: var(--app-panel-muted);
}

.image-stage {
  @apply flex h-full min-h-0 w-full touch-none select-none items-center justify-center overflow-hidden p-3;
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
  user-select: none;
  -webkit-user-drag: none;
}
</style>
