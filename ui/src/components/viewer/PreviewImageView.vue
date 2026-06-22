<script setup lang="ts">
import {onBeforeUnmount, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {useImageZoomPan} from "../../composables/useImageZoomPan.ts";
import {downloadUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";

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
  <div class="preview-tool-row">
    <button :class="{active: previewImageFit}" @click="resetPreviewImageZoom">适应</button>
    <button @click="zoomPreviewImage(-25)">-</button>
    <span>{{ previewZoomText }}</span>
    <button @click="zoomPreviewImage(25)">+</button>
    <button title="打开图片查看" @click="openImagePreview">
      <icon icon="icon-unfold" color="currentColor" />
      <span>打开查看</span>
    </button>
  </div>
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

.preview-tool-row {
  @apply flex h-9 shrink-0 items-center gap-1 border-b border-slate-100 bg-white px-3 text-xs text-slate-500;
}

.preview-tool-row button {
  @apply inline-flex h-6 items-center gap-1 rounded border border-transparent px-2 text-slate-600 hover:border-slate-200 hover:bg-blue-50 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:border-transparent disabled:hover:bg-transparent;
}

.preview-tool-row button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.preview-tool-row > span {
  @apply ml-auto tabular-nums;
}

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm text-slate-700;
}

.preview-body.image {
  @apply bg-slate-50;
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
