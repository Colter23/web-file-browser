<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {StyleValue} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {downloadUrl} from "../../network/api.ts";
import ViewerStatus from "./ViewerStatus.vue";

const props = defineProps<{
  entry: ExplorerEntry;
  loading: boolean;
  error: string;
  fit: boolean;
  canPan: boolean;
  dragging: boolean;
  title: string;
  imageStyle: StyleValue;
  rotatedSideways: boolean;
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

const stageRef = ref<HTMLElement | null>(null);
const stageWidth = ref(0);
const stageHeight = ref(0);
let resizeObserver: ResizeObserver | null = null;

const fitVars = computed(() => ({
  "--image-viewer-stage-fit-width": `${stageWidth.value}px`,
  "--image-viewer-stage-fit-height": `${stageHeight.value}px`
}));

const measureStage = () => {
  const stage = stageRef.value;
  if (!stage) return;
  const styles = getComputedStyle(stage);
  const horizontalPadding = Number.parseFloat(styles.paddingLeft) + Number.parseFloat(styles.paddingRight);
  const verticalPadding = Number.parseFloat(styles.paddingTop) + Number.parseFloat(styles.paddingBottom);
  stageWidth.value = Math.max(0, stage.clientWidth - horizontalPadding);
  stageHeight.value = Math.max(0, stage.clientHeight - verticalPadding);
}

watch(() => props.rotatedSideways, () => {
  void nextTick(measureStage);
});

onMounted(() => {
  measureStage();
  if (typeof ResizeObserver === "undefined") return;
  resizeObserver = new ResizeObserver(measureStage);
  if (stageRef.value) resizeObserver.observe(stageRef.value);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
});
</script>

<template>
  <div
      ref="stageRef"
      class="image-viewer-stage"
      :class="{fit, panning: canPan, dragging}"
      :style="fitVars"
      :title="title"
      @pointerdown="emit('pointer-down', $event)"
      @pointermove="emit('pointer-move', $event)"
      @pointerup="emit('pointer-up', $event)"
      @pointercancel="emit('pointer-cancel', $event)"
      @lostpointercapture="emit('lost-pointer-capture')"
      @wheel="emit('wheel', $event)"
      @dblclick="emit('toggle-zoom')">
    <viewer-status v-if="loading">正在加载图片...</viewer-status>
    <viewer-status v-if="error" tone="error">{{ error }}</viewer-status>
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
  @apply relative flex min-h-0 grow touch-none select-none items-center justify-center overflow-hidden p-5;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 4%, rgba(2, 6, 23, 0.18));
}

.image-viewer-stage.panning {
  @apply cursor-grab;
}

.image-viewer-stage.dragging {
  @apply cursor-grabbing;
}

.image-viewer-stage img {
  @apply max-h-full max-w-full select-none rounded-md border border-white/10 object-contain shadow-2xl;
  user-select: none;
  -webkit-user-drag: none;
}
</style>
