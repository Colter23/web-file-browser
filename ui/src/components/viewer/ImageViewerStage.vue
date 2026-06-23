<script setup lang="ts">
import type {StyleValue} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {downloadUrl} from "../../network/api.ts";

defineProps<{
  entry: ExplorerEntry;
  loading: boolean;
  error: string;
  fit: boolean;
  canPan: boolean;
  dragging: boolean;
  title: string;
  imageStyle: StyleValue;
}>();

const emit = defineEmits<{
  (e: "pointer-down", event: PointerEvent): void;
  (e: "pointer-move", event: PointerEvent): void;
  (e: "pointer-up", event: PointerEvent): void;
  (e: "pointer-cancel", event: PointerEvent): void;
  (e: "lost-pointer-capture"): void;
  (e: "wheel", event: WheelEvent): void;
  (e: "toggle-zoom"): void;
  (e: "load"): void;
  (e: "error"): void;
}>();
</script>

<template>
  <div
      class="image-viewer-stage"
      :class="{fit, panning: canPan, dragging}"
      :title="title"
      @pointerdown="emit('pointer-down', $event)"
      @pointermove="emit('pointer-move', $event)"
      @pointerup="emit('pointer-up', $event)"
      @pointercancel="emit('pointer-cancel', $event)"
      @lostpointercapture="emit('lost-pointer-capture')"
      @wheel="emit('wheel', $event)"
      @dblclick="emit('toggle-zoom')">
    <div v-if="loading" class="image-viewer-status">正在加载图片...</div>
    <div v-if="error" class="image-viewer-status error">{{ error }}</div>
    <img
        :key="entry.path"
        :src="downloadUrl(entry.path)"
        :alt="entry.name"
        :style="imageStyle"
        @load="emit('load')"
        @error="emit('error')">
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.image-viewer-stage {
  @apply relative flex min-h-0 grow touch-none select-none items-center justify-center overflow-hidden bg-transparent p-5;
}

.image-viewer-status {
  @apply absolute rounded-md border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 shadow-xl backdrop-blur;
}

.image-viewer-status.error {
  @apply border-red-300/30 bg-red-950/70 text-red-100;
}

.image-viewer-stage.panning {
  @apply cursor-grab;
}

.image-viewer-stage.dragging {
  @apply cursor-grabbing;
}

.image-viewer-stage img {
  @apply max-h-full max-w-full select-none rounded object-contain shadow-2xl;
  user-select: none;
  -webkit-user-drag: none;
}
</style>
