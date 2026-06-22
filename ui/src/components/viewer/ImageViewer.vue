<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {FileInfo} from "../../class.ts";
import {downloadUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";

type ImageViewerEntry = {
  type: "folder" | "file";
  name: string;
  path: string;
  modified?: string;
  size?: number;
  extension?: string;
  file?: FileInfo;
}

type NoticeKind = "info" | "success" | "warning" | "error";

type NoticePayload = {
  kind: NoticeKind;
  title: string;
  message: string;
}

const props = defineProps<{
  visible: boolean;
  entry: ImageViewerEntry | null;
  entries: ImageViewerEntry[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "select", entry: ImageViewerEntry): void;
  (e: "download", entry: ImageViewerEntry): void;
  (e: "notice", payload: NoticePayload): void;
}>();

const filmstripStorageKey = "explorer.imageViewer.showFilmstrip";

const readBooleanStorage = (key: string, fallback: boolean) => {
  if (typeof localStorage === "undefined") return fallback;
  try {
    const raw = localStorage.getItem(key);
    if (raw === null) return fallback;
    return raw === "true";
  } catch {
    return fallback;
  }
}

const writeBooleanStorage = (key: string, value: boolean) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(key, String(value));
  } catch {
    // 本地存储不可用时，只保留本次会话里的查看器设置。
  }
}

const viewerRef = ref<HTMLElement | null>(null);
const loading = ref(false);
const error = ref("");
const pageFullscreen = ref(false);
const browserFullscreen = ref(false);
const showFilmstrip = ref(readBooleanStorage(filmstripStorageKey, true));
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

const currentEntry = computed(() => props.visible ? props.entry : null);

const imageStyle = computed(() => ({
  maxWidth: fit.value ? "100%" : "none",
  maxHeight: fit.value ? "100%" : "none",
  transform: fit.value ? "none" : `translate3d(${offsetX.value}px, ${offsetY.value}px, 0) scale(${zoom.value / 100})`,
  transformOrigin: "center center"
}));

const zoomText = computed(() => fit.value ? "适应" : `${zoom.value}%`);
const canPan = computed(() => props.visible && !fit.value);
const actualSizeActive = computed(() => !fit.value && zoom.value === 100);

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
  return `${position}${formatBytes(entry.size)} · ${formatDate(entry.modified)}`;
});

const pageFullscreenTitle = computed(() => pageFullscreen.value ? "退出网页全屏 (F)" : "网页全屏 (F)");
const browserFullscreenTitle = computed(() => browserFullscreen.value ? "退出浏览器全屏" : "浏览器全屏");
const filmstripTitle = computed(() => showFilmstrip.value ? "隐藏缩略图 (T)" : "显示缩略图 (T)");
const stageTitle = computed(() => fit.value ? "双击按原始大小查看" : "双击适应窗口，拖拽移动图片");

const formatBytes = (bytes?: number) => {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

const formatDate = (srcDate?: string) => {
  if (!srcDate) return "-";
  const date = new Date(srcDate);
  if (Number.isNaN(date.getTime())) return srcDate;
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit"
  }).format(date);
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

const resetRuntimeState = () => {
  if (document.fullscreenElement === viewerRef.value) void document.exitFullscreen().catch(() => undefined);
  loading.value = false;
  error.value = "";
  pageFullscreen.value = false;
  resetZoom();
}

const prepareEntry = async () => {
  if (!props.visible || !props.entry) return;
  loading.value = true;
  error.value = "";
  resetZoom();
  await nextTick();
  viewerRef.value?.focus();
}

const close = () => emit("close");

const showAdjacent = (direction: -1 | 1) => {
  const next = props.entries[currentIndex.value + direction];
  if (next) emit("select", next);
}

const showImageAt = (index: number) => {
  const next = props.entries[index];
  if (!next || next.path === props.entry?.path) return;
  emit("select", next);
}

const togglePageFullscreen = async () => {
  pageFullscreen.value = !pageFullscreen.value;
  await nextTick();
  viewerRef.value?.focus();
}

const toggleBrowserFullscreen = async () => {
  const target = viewerRef.value;
  if (!target) return;
  try {
    if (document.fullscreenElement === target) {
      await document.exitFullscreen();
    } else {
      await target.requestFullscreen();
    }
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

const zoomImage = (delta: number) => {
  fit.value = false;
  zoom.value = Math.min(500, Math.max(25, zoom.value + delta));
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
  zoomImage(event.deltaY < 0 ? 25 : -25);
}

const handleLoad = () => {
  loading.value = false;
  error.value = "";
}

const handleError = () => {
  loading.value = false;
  error.value = "图片加载失败，请检查文件是否仍可读取。";
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

const handleFullscreenChange = () => {
  browserFullscreen.value = document.fullscreenElement === viewerRef.value;
}

const downloadCurrent = () => {
  if (props.entry) emit("download", props.entry);
}

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if (!props.visible) return;
  const key = event.key.toLowerCase();
  const takeOver = () => {
    event.preventDefault();
    event.stopImmediatePropagation();
  }
  if (key === "escape") {
    takeOver();
    close();
    return;
  }
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
  if (key === "+" || key === "=" || event.code === "NumpadAdd") {
    takeOver();
    zoomImage(25);
    return;
  }
  if (key === "-" || event.code === "NumpadSubtract") {
    takeOver();
    zoomImage(-25);
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
  if (key === "f") {
    takeOver();
    void togglePageFullscreen();
    return;
  }
  if (key === "t") {
    takeOver();
    toggleFilmstrip();
    return;
  }
  if (event.ctrlKey || event.metaKey || event.altKey) takeOver();
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
        @keydown.esc.prevent="close">
      <div class="image-viewer-toolbar">
        <div class="image-viewer-title">
          <strong>{{ currentEntry.name }}</strong>
          <span>{{ subtitle }}</span>
        </div>
        <div class="image-viewer-actions">
          <button title="上一张 (←)" :disabled="!canShowPrevious" @click="showAdjacent(-1)">
            <icon icon="icon-back_android" color="currentColor" />
          </button>
          <button title="下一张 (→)" :disabled="!canShowNext" @click="showAdjacent(1)">
            <icon icon="icon-back_android" color="currentColor" class="rotate-180" />
          </button>
          <button class="text-action" :class="{active: fit}" title="适应窗口 (0)" @click="resetZoom">适应</button>
          <button class="text-action" :class="{active: actualSizeActive}" title="原始大小" @click="setActualSize">1:1</button>
          <button title="缩小 (-)" @click="zoomImage(-25)">-</button>
          <span>{{ zoomText }}</span>
          <button title="放大 (+)" @click="zoomImage(25)">+</button>
          <button :title="pageFullscreenTitle" :class="{active: pageFullscreen}" @click="togglePageFullscreen">
            <icon icon="icon-renamebox" color="currentColor" />
          </button>
          <button :title="browserFullscreenTitle" :class="{active: browserFullscreen}" @click="toggleBrowserFullscreen">
            <icon icon="icon-unfold" color="currentColor" />
          </button>
          <button :title="filmstripTitle" :class="{active: showFilmstrip}" :disabled="imageCount <= 1" @click="toggleFilmstrip">
            <icon icon="icon-viewgrid" color="currentColor" />
          </button>
          <button title="下载" @click="downloadCurrent">
            <icon icon="icon-download" color="currentColor" />
          </button>
          <button title="关闭" @click="close">
            <icon icon="icon-close" color="currentColor" />
          </button>
        </div>
      </div>
      <div
          class="image-viewer-stage"
          :class="{fit, panning: canPan, dragging}"
          :title="stageTitle"
          @pointerdown="startPan"
          @pointermove="movePan"
          @pointerup="stopPan"
          @pointercancel="stopPan"
          @lostpointercapture="releasePointer"
          @wheel="handleWheel"
          @dblclick="toggleZoomMode">
        <div v-if="loading" class="image-viewer-status">正在加载图片...</div>
        <div v-if="error" class="image-viewer-status error">{{ error }}</div>
        <img
            :key="currentEntry.path"
            :src="downloadUrl(currentEntry.path)"
            :alt="currentEntry.name"
            :style="imageStyle"
            @load="handleLoad"
            @error="handleError">
      </div>
      <div v-if="canShowFilmstrip" class="image-viewer-filmstrip" aria-label="图片列表">
        <button
            v-for="item in filmstripEntries"
            :key="item.entry.path"
            class="image-viewer-thumb"
            :class="{active: item.entry.path === currentEntry.path}"
            :title="`${item.index + 1} / ${imageCount} · ${item.entry.name}`"
            @click="showImageAt(item.index)">
          <img :src="downloadUrl(item.entry.path)" :alt="item.entry.name" loading="lazy">
          <span>{{ item.index + 1 }}</span>
        </button>
      </div>
    </section>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.image-viewer {
  @apply absolute inset-0 z-40 flex flex-col overflow-hidden rounded-lg bg-slate-950/72 text-white outline-none backdrop-blur-sm;
}

.image-viewer.pageFullscreen {
  @apply fixed inset-0 z-50 rounded-none;
}

.image-viewer-toolbar {
  @apply flex min-h-14 shrink-0 items-center justify-between gap-3 border-b border-white/15 bg-slate-950/75 px-4 backdrop-blur;
}

.image-viewer-title {
  @apply flex min-w-0 flex-col;
}

.image-viewer-title strong {
  @apply truncate text-sm font-semibold;
}

.image-viewer-title span {
  @apply truncate text-xs text-slate-300;
}

.image-viewer-actions {
  @apply flex shrink-0 items-center gap-1 text-xs text-slate-100;
}

.image-viewer-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-white/30 bg-white/15 px-2 text-sm font-medium text-white shadow-sm hover:border-white/45 hover:bg-white/25;
}

.image-viewer-actions button:disabled {
  @apply cursor-not-allowed border-white/10 bg-white/5 opacity-35 hover:border-white/10 hover:bg-white/5;
}

.image-viewer-actions button.active {
  @apply border-blue-200/80 bg-blue-500/50 text-white shadow-[0_0_0_1px_rgba(191,219,254,0.22)];
}

.image-viewer-actions button.text-action {
  @apply min-w-11 px-3;
}

.image-viewer-actions span {
  @apply w-14 text-center font-medium tabular-nums text-white;
}

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

.image-viewer-filmstrip {
  @apply flex h-24 shrink-0 items-center gap-2 overflow-x-auto border-t border-white/10 bg-slate-950/45 px-4 py-2 backdrop-blur;
}

.image-viewer-thumb {
  @apply relative h-16 w-20 shrink-0 overflow-hidden rounded-md border border-white/10 bg-white/5 p-0.5 text-white opacity-75 outline-none hover:border-white/35 hover:opacity-100;
}

.image-viewer-thumb.active {
  @apply border-blue-300 bg-blue-500/20 opacity-100 shadow-[0_0_0_2px_rgba(96,165,250,0.25)];
}

.image-viewer-thumb img {
  @apply h-full w-full rounded object-cover;
}

.image-viewer-thumb span {
  @apply absolute bottom-1 right-1 rounded bg-slate-950/70 px-1 text-[10px] leading-4 text-slate-100;
}
</style>
