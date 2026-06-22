<script setup lang="ts">
import {computed} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import Icon from "../Icon.vue";

const props = defineProps<{
  entry: ExplorerEntry;
  subtitle: string;
  canShowPrevious: boolean;
  canShowNext: boolean;
  fit: boolean;
  actualSizeActive: boolean;
  zoomText: string;
  canZoomOut: boolean;
  canZoomIn: boolean;
  zoomStep: number;
  pageFullscreen: boolean;
  browserFullscreen: boolean;
  showFilmstrip: boolean;
  imageCount: number;
}>();

const emit = defineEmits<{
  (e: "previous"): void;
  (e: "next"): void;
  (e: "reset-zoom"): void;
  (e: "actual-size"): void;
  (e: "zoom", delta: number): void;
  (e: "toggle-page-fullscreen"): void;
  (e: "toggle-browser-fullscreen"): void;
  (e: "toggle-filmstrip"): void;
  (e: "download"): void;
  (e: "close"): void;
}>();

const pageFullscreenTitle = computed(() => props.pageFullscreen ? "退出网页全屏 (F)" : "网页全屏 (F)");
const browserFullscreenTitle = computed(() => props.browserFullscreen ? "退出浏览器全屏" : "浏览器全屏");
const filmstripTitle = computed(() => props.showFilmstrip ? "隐藏缩略图 (T)" : "显示缩略图 (T)");
</script>

<template>
  <div class="image-viewer-toolbar">
    <div class="image-viewer-title">
      <strong>{{ entry.name }}</strong>
      <span>{{ subtitle }}</span>
    </div>
    <div class="image-viewer-actions">
      <button title="上一张 (←)" :disabled="!canShowPrevious" @click="emit('previous')">
        <icon icon="icon-back_android" color="currentColor" />
      </button>
      <button title="下一张 (→)" :disabled="!canShowNext" @click="emit('next')">
        <icon icon="icon-back_android" color="currentColor" class="rotate-180" />
      </button>
      <button class="text-action" :class="{active: fit}" title="适应窗口 (0)" @click="emit('reset-zoom')">适应</button>
      <button class="text-action" :class="{active: actualSizeActive}" title="原始大小 (1)" @click="emit('actual-size')">1:1</button>
      <button title="缩小 (-)" :disabled="!canZoomOut" @click="emit('zoom', -zoomStep)">-</button>
      <span>{{ zoomText }}</span>
      <button title="放大 (+)" :disabled="!canZoomIn" @click="emit('zoom', zoomStep)">+</button>
      <button :title="pageFullscreenTitle" :class="{active: pageFullscreen}" @click="emit('toggle-page-fullscreen')">
        <icon icon="icon-renamebox" color="currentColor" />
      </button>
      <button :title="browserFullscreenTitle" :class="{active: browserFullscreen}" @click="emit('toggle-browser-fullscreen')">
        <icon icon="icon-unfold" color="currentColor" />
      </button>
      <button :title="filmstripTitle" :class="{active: showFilmstrip}" :disabled="imageCount <= 1" @click="emit('toggle-filmstrip')">
        <icon icon="icon-viewgrid" color="currentColor" />
      </button>
      <button title="下载" @click="emit('download')">
        <icon icon="icon-download" color="currentColor" />
      </button>
      <button title="关闭" @click="emit('close')">
        <icon icon="icon-close" color="currentColor" />
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

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
</style>
