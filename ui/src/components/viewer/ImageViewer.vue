<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {useImageZoomPan} from "../../composables/useImageZoomPan.ts";
import type {ShellNoticePayload} from "../shell/types.ts";
import {formatEntryDate, formatEntrySize} from "../../utils/file-entry.ts";
import {readBooleanStorage, writeBooleanStorage} from "../../utils/safe-storage.ts";
import ImageViewerFilmstrip from "./ImageViewerFilmstrip.vue";
import ImageViewerStage from "./ImageViewerStage.vue";
import ImageViewerToolbar from "./ImageViewerToolbar.vue";

const props = defineProps<{
  visible: boolean;
  entry: ExplorerEntry | null;
  entries: ExplorerEntry[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "select", entry: ExplorerEntry): void;
  (e: "download", entry: ExplorerEntry): void;
  (e: "notice", payload: ShellNoticePayload): void;
}>();

const filmstripStorageKey = "explorer.imageViewer.showFilmstrip";
const minZoom = 25;
const maxZoom = 500;
const zoomStep = 25;

const viewerRef = ref<HTMLElement | null>(null);
const loading = ref(false);
const error = ref("");
const pageFullscreen = ref(false);
const browserFullscreen = ref(false);
const showFilmstrip = ref(readBooleanStorage(filmstripStorageKey, false));
const rotation = ref(0);

const currentEntry = computed(() => props.visible ? props.entry : null);

const {
  fit,
  dragging,
  imageStyle,
  zoomText,
  canPan,
  actualSizeActive,
  canZoomOut,
  canZoomIn,
  resetZoom,
  releasePointer,
  zoomImage,
  setActualSize,
  toggleZoomMode,
  handleWheel,
  startPan,
  movePan,
  stopPan
} = useImageZoomPan({minZoom, maxZoom, canPan: () => props.visible});

const normalizedRotation = computed(() => ((rotation.value % 360) + 360) % 360);
const rotatedSideways = computed(() => normalizedRotation.value === 90 || normalizedRotation.value === 270);
const transformedImageStyle = computed(() => {
  const style = {...imageStyle.value};
  if (fit.value && rotatedSideways.value) {
    style.maxWidth = "var(--image-viewer-stage-fit-height, 100%)";
    style.maxHeight = "var(--image-viewer-stage-fit-width, 100%)";
  }
  style.transform = [
    style.transform === "none" ? "" : style.transform,
    normalizedRotation.value ? `rotate(${normalizedRotation.value}deg)` : ""
  ].filter(Boolean).join(" ") || "none";
  return style;
});

const currentIndex = computed(() => {
  const entry = props.entry;
  if (!entry) return -1;
  return props.entries.findIndex(item => item.path === entry.path);
});

const imageCount = computed(() => props.entries.length);
const canShowPrevious = computed(() => currentIndex.value > 0);
const canShowNext = computed(() => currentIndex.value >= 0 && currentIndex.value < props.entries.length - 1);
const canShowFilmstrip = computed(() => imageCount.value > 1 && showFilmstrip.value);

const filmstripEntries = computed(() => {
  const entries = props.entries;
  if (entries.length <= 12) return entries.map((entry, index) => ({entry, index}));
  const visibleCount = 11;
  const half = Math.floor(visibleCount / 2);
  let start = Math.max(0, Math.max(0, currentIndex.value) - half);
  let end = Math.min(entries.length, start + visibleCount);
  start = Math.max(0, end - visibleCount);
  return entries.slice(start, end).map((entry, offset) => ({entry, index: start + offset}));
});

const subtitle = computed(() => {
  const entry = props.entry;
  if (!entry) return "";
  const position = currentIndex.value >= 0 && imageCount.value > 1 ? `${currentIndex.value + 1} / ${imageCount.value} · ` : "";
  return `${position}${formatEntrySize(entry.size, "0 B")} · ${formatEntryDate(entry.modified)}`;
});

const stageTitle = computed(() => fit.value ? "双击按原始大小查看" : "双击适应窗口，拖拽移动图片");

const resetRuntimeState = () => {
  if (document.fullscreenElement === viewerRef.value) void document.exitFullscreen().catch(() => undefined);
  loading.value = false;
  error.value = "";
  pageFullscreen.value = false;
  browserFullscreen.value = false;
  rotation.value = 0;
  resetZoom();
}

const prepareEntry = async () => {
  if (!props.visible || !props.entry) return;
  loading.value = true;
  error.value = "";
  rotation.value = 0;
  resetZoom();
  await nextTick();
  viewerRef.value?.focus();
}

const close = () => emit("close");

const focusViewer = async () => {
  await nextTick();
  viewerRef.value?.focus();
}

const showAdjacent = (direction: -1 | 1) => {
  const next = props.entries[currentIndex.value + direction];
  if (next) emit("select", next);
}

const showEdgeImage = (edge: "first" | "last") => {
  if (!props.entries.length) return;
  showImageAt(edge === "first" ? 0 : props.entries.length - 1);
}

const showImageAt = (index: number) => {
  const next = props.entries[index];
  if (!next || next.path === props.entry?.path) return;
  emit("select", next);
}

const togglePageFullscreen = async () => {
  releasePointer();
  pageFullscreen.value = !pageFullscreen.value;
  await focusViewer();
}

const toggleBrowserFullscreen = async () => {
  const target = viewerRef.value;
  if (!target) return;
  try {
    if (document.fullscreenElement === target) {
      await document.exitFullscreen();
    } else {
      releasePointer();
      await target.requestFullscreen();
    }
    await focusViewer();
  } catch {
    emit("notice", {
      kind: "warning",
      title: "无法全屏",
      message: "当前浏览器未允许进入全屏，仍可在页面内查看大图。"
    });
  }
}

const toggleFilmstrip = () => {
  if (imageCount.value <= 1) return;
  showFilmstrip.value = !showFilmstrip.value;
  writeBooleanStorage(filmstripStorageKey, showFilmstrip.value);
}

const rotateImage = (direction: -1 | 1) => {
  rotation.value = (normalizedRotation.value + direction * 90 + 360) % 360;
}

const handleLoad = () => {
  loading.value = false;
  error.value = "";
}

const handleError = () => {
  loading.value = false;
  error.value = "图片加载失败，请检查文件是否仍可读取。";
}

const handleFullscreenChange = () => {
  browserFullscreen.value = document.fullscreenElement === viewerRef.value;
}

const downloadCurrent = () => {
  if (props.entry) emit("download", props.entry);
}

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if (!props.visible) return;
  const key = event.key.toLowerCase();
  const hasSystemModifier = event.ctrlKey || event.metaKey || event.altKey;
  const takeOver = () => {
    event.preventDefault();
    event.stopImmediatePropagation();
  }
  if (!hasSystemModifier && key === "escape") {
    takeOver();
    close();
    return;
  }
  if (hasSystemModifier) return;
  if (event.key === "ArrowLeft") {
    takeOver();
    showAdjacent(-1);
    return;
  }
  if (event.key === "ArrowRight") {
    takeOver();
    showAdjacent(1);
    return;
  }
  if (event.key === "Home") {
    takeOver();
    showEdgeImage("first");
    return;
  }
  if (event.key === "End") {
    takeOver();
    showEdgeImage("last");
    return;
  }
  if (key === "+" || key === "=" || event.code === "NumpadAdd") {
    takeOver();
    zoomImage(zoomStep);
    return;
  }
  if (key === "-" || event.code === "NumpadSubtract") {
    takeOver();
    zoomImage(-zoomStep);
    return;
  }
  if (key === "0") {
    takeOver();
    resetZoom();
    return;
  }
  if (key === "1" || event.code === "Numpad1") {
    takeOver();
    setActualSize();
    return;
  }
  if (key === "r") {
    takeOver();
    rotateImage(event.shiftKey ? -1 : 1);
    return;
  }
  if (key === "f") {
    takeOver();
    void togglePageFullscreen();
    return;
  }
  if (key === "t") {
    takeOver();
    toggleFilmstrip();
  }
}

watch(() => props.visible, visible => {
  if (visible) {
    void prepareEntry();
    return;
  }
  resetRuntimeState();
});

watch(() => props.entry?.path, () => {
  void prepareEntry();
});

onMounted(() => {
  window.addEventListener("keydown", handleWindowKeyDown, true);
  document.addEventListener("fullscreenchange", handleFullscreenChange);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleWindowKeyDown, true);
  document.removeEventListener("fullscreenchange", handleFullscreenChange);
  resetRuntimeState();
});
</script>

<template>
  <Teleport to="body" :disabled="!pageFullscreen">
    <section
        v-if="currentEntry"
        ref="viewerRef"
        class="image-viewer"
        :class="{pageFullscreen}"
        tabindex="-1"
        @keydown.esc.prevent.stop="close">
      <image-viewer-toolbar
          :entry="currentEntry"
          :subtitle="subtitle"
          :can-show-previous="canShowPrevious"
          :can-show-next="canShowNext"
          :fit="fit"
          :actual-size-active="actualSizeActive"
          :zoom-text="zoomText"
          :can-zoom-out="canZoomOut"
          :can-zoom-in="canZoomIn"
          :zoom-step="zoomStep"
          :rotation="normalizedRotation"
          :page-fullscreen="pageFullscreen"
          :browser-fullscreen="browserFullscreen"
          :show-filmstrip="showFilmstrip"
          :image-count="imageCount"
          @previous="showAdjacent(-1)"
          @next="showAdjacent(1)"
          @reset-zoom="resetZoom"
          @actual-size="setActualSize"
          @zoom="zoomImage"
          @rotate="rotateImage"
          @toggle-page-fullscreen="togglePageFullscreen"
          @toggle-browser-fullscreen="toggleBrowserFullscreen"
          @toggle-filmstrip="toggleFilmstrip"
          @download="downloadCurrent"
          @close="close" />
      <image-viewer-stage
          :entry="currentEntry"
          :loading="loading"
          :error="error"
          :fit="fit"
          :can-pan="canPan"
          :dragging="dragging"
          :title="stageTitle"
          :image-style="transformedImageStyle"
          :rotated-sideways="rotatedSideways"
          @pointer-down="startPan"
          @pointer-move="movePan"
          @pointer-up="stopPan"
          @pointer-cancel="stopPan"
          @lost-pointer-capture="releasePointer"
          @wheel="handleWheel"
          @toggle-zoom="toggleZoomMode"
          @load="handleLoad"
          @error="handleError" />
      <image-viewer-filmstrip
          v-if="canShowFilmstrip"
          :items="filmstripEntries"
          :current-path="currentEntry.path"
          :image-count="imageCount"
          @select="showImageAt" />
    </section>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.image-viewer {
  @apply absolute inset-0 z-40 flex flex-col overflow-hidden rounded-lg text-white outline-none backdrop-blur-sm;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 7%, rgba(2, 6, 23, 0.78));
}

.image-viewer.pageFullscreen {
  @apply fixed inset-0 z-50 rounded-none;
}

</style>
