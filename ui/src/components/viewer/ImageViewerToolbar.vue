<script setup lang="ts">
import {computed} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import FileTypeIcon from "../FileTypeIcon.vue";
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
  rotation: number;
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
  (e: "rotate", direction: -1 | 1): void;
  (e: "toggle-page-fullscreen"): void;
  (e: "toggle-browser-fullscreen"): void;
  (e: "toggle-filmstrip"): void;
  (e: "download"): void;
  (e: "close"): void;
}>();

const pageFullscreenTitle = computed(() => props.pageFullscreen ? "退出网页全屏 (F)" : "网页全屏 (F)");
const browserFullscreenTitle = computed(() => props.browserFullscreen ? "退出浏览器全屏" : "浏览器全屏");
const filmstripTitle = computed(() => props.showFilmstrip ? "隐藏缩略图 (T)" : "显示缩略图 (T)");
const pageFullscreenIcon = computed(() => props.pageFullscreen ? "viewer.page-fullscreen-off" : "viewer.page-fullscreen");
const browserFullscreenIcon = computed(() => props.browserFullscreen ? "viewer.browser-fullscreen-off" : "viewer.browser-fullscreen");
const filmstripIcon = computed(() => props.showFilmstrip ? "viewer.filmstrip-off" : "viewer.filmstrip");
const zoomModeTitle = computed(() => props.fit ? "当前适应窗口，点击按原始大小查看 (1)" : "当前按倍率查看，点击适应窗口 (0)");
const zoomModeLabel = computed(() => props.fit ? "适应" : props.zoomText);
const rotationLabel = computed(() => props.rotation ? `${props.rotation}°` : "0°");
</script>

<template>
  <div class="image-viewer-toolbar">
    <div class="image-viewer-title">
      <span class="image-viewer-title-icon">
        <file-type-icon kind="image" :name="entry.name" :extension="entry.extension" size="1.15rem" />
      </span>
      <span class="image-viewer-title-text">
        <strong>{{ entry.name }}</strong>
        <span>{{ subtitle }}</span>
      </span>
    </div>
    <div class="image-viewer-actions">
      <div class="image-viewer-action-group">
        <button title="上一张 (←)" :disabled="!canShowPrevious" @click="emit('previous')">
          <icon icon="action.previous" color="currentColor" size="1.1rem" />
        </button>
        <button title="下一张 (→)" :disabled="!canShowNext" @click="emit('next')">
          <icon icon="action.next" color="currentColor" size="1.1rem" />
        </button>
      </div>
      <div class="image-viewer-action-group zoom-group">
        <button class="zoom-step" title="缩小 (-)" :disabled="!canZoomOut" @click="emit('zoom', -zoomStep)">
          <icon icon="viewer.zoom-out" color="currentColor" />
        </button>
        <button
            class="zoom-mode"
            :class="{fit, actual: actualSizeActive}"
            :title="zoomModeTitle"
            @click="fit ? emit('actual-size') : emit('reset-zoom')">
          <span class="zoom-mode-label">{{ zoomModeLabel }}</span>
        </button>
        <button class="zoom-step" title="放大 (+)" :disabled="!canZoomIn" @click="emit('zoom', zoomStep)">
          <icon icon="viewer.zoom-in" color="currentColor" />
        </button>
      </div>
      <div class="image-viewer-action-group rotate-group">
        <button title="向左旋转 (Shift+R)" @click="emit('rotate', -1)">
          <icon icon="viewer.rotate-left" color="currentColor" />
        </button>
        <button :title="`当前旋转 ${rotationLabel}`" class="rotation-state" disabled>
          {{ rotationLabel }}
        </button>
        <button title="向右旋转 (R)" @click="emit('rotate', 1)">
          <icon icon="viewer.rotate-right" color="currentColor" />
        </button>
      </div>
      <div class="image-viewer-action-group">
        <button :title="pageFullscreenTitle" :class="{active: pageFullscreen}" @click="emit('toggle-page-fullscreen')">
          <icon :icon="pageFullscreenIcon" color="currentColor" />
        </button>
        <button :title="browserFullscreenTitle" :class="{active: browserFullscreen}" @click="emit('toggle-browser-fullscreen')">
          <icon :icon="browserFullscreenIcon" color="currentColor" />
        </button>
        <button :title="filmstripTitle" :class="{active: showFilmstrip}" :disabled="imageCount <= 1" @click="emit('toggle-filmstrip')">
          <icon :icon="filmstripIcon" color="currentColor" />
        </button>
      </div>
      <div class="image-viewer-action-group">
        <button title="下载" @click="emit('download')">
          <icon icon="action.download" color="currentColor" />
        </button>
        <button title="关闭" @click="emit('close')">
          <icon icon="action.close" color="currentColor" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.image-viewer-toolbar {
  @apply flex min-h-14 shrink-0 items-center justify-between gap-3 border-b px-3.5 backdrop-blur;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.12));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 6%, rgba(15, 23, 42, 0.74));
}

.image-viewer-title {
  @apply flex min-w-0 items-center gap-2.5;
}

.image-viewer-title-icon {
  @apply grid h-8 w-8 shrink-0 place-items-center rounded-md border border-white/10 bg-white/10 text-teal-200 shadow-sm;
}

.image-viewer-title-text {
  @apply flex min-w-0 flex-col;
}

.image-viewer-title strong {
  @apply truncate text-sm font-semibold leading-5;
}

.image-viewer-title-text > span {
  @apply truncate text-xs leading-4 text-slate-300;
}

.image-viewer-actions {
  @apply flex shrink-0 items-center gap-1.5 text-xs text-slate-100;
}

.image-viewer-action-group {
  @apply inline-flex h-9 items-center overflow-hidden rounded-lg border p-0.5 shadow-sm;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.14));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 3%, rgba(255, 255, 255, 0.1));
}

.image-viewer-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-transparent bg-transparent px-2 text-sm font-medium text-white transition hover:bg-white/20;
}

.image-viewer-actions button:disabled {
  @apply cursor-not-allowed opacity-35 hover:bg-transparent;
}

.image-viewer-actions button:focus-visible {
  @apply outline-none;
  border-color: rgba(255, 255, 255, 0.78);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-accent, #2563eb) 45%, rgba(255, 255, 255, 0.25));
}

.image-viewer-actions button.active {
  @apply text-white;
  border-color: color-mix(in srgb, var(--app-accent-border, #bfdbfe) 72%, transparent);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 36%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--app-accent-border, #bfdbfe) 22%, transparent);
}

.image-viewer-actions button.zoom-step {
  @apply min-w-8 px-1;
}

.image-viewer-actions button.zoom-mode {
  @apply min-w-16 px-2.5 text-xs;
}

.image-viewer-actions button.zoom-mode.fit,
.image-viewer-actions button.zoom-mode.actual {
  color: #fff;
}

.image-viewer-actions button.rotation-state {
  @apply min-w-11 cursor-default px-2 text-xs tabular-nums opacity-80;
}

.image-viewer-actions button.rotation-state:disabled {
  @apply opacity-80;
}

.zoom-mode-label {
  @apply block w-12 text-center font-semibold tabular-nums;
}

@media (max-width: 840px) {
  .image-viewer-toolbar {
    @apply items-start;
  }

  .image-viewer-actions {
    @apply flex-wrap justify-end;
  }

  .zoom-group {
    @apply order-last;
  }
}
</style>
