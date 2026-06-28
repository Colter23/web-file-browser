<script setup lang="ts">
import {computed} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import Icon from "../Icon.vue";
import ViewerActionGroup from "./ViewerActionGroup.vue";
import ViewerToolbar from "./ViewerToolbar.vue";

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
  <viewer-toolbar kind="image" :name="entry.name" :extension="entry.extension" :subtitle="subtitle" icon-tone="image">
    <viewer-action-group>
      <button title="上一张 (←)" :disabled="!canShowPrevious" @click="emit('previous')">
        <icon icon="action.previous" color="currentColor" size="1.1rem" />
      </button>
      <button title="下一张 (→)" :disabled="!canShowNext" @click="emit('next')">
        <icon icon="action.next" color="currentColor" size="1.1rem" />
      </button>
    </viewer-action-group>
    <viewer-action-group class="zoom-group">
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
    </viewer-action-group>
    <viewer-action-group>
      <button title="向左旋转 (Shift+R)" @click="emit('rotate', -1)">
        <icon icon="viewer.rotate-left" color="currentColor" />
      </button>
      <button :title="`当前旋转 ${rotationLabel}`" class="rotation-state" disabled>
        {{ rotationLabel }}
      </button>
      <button title="向右旋转 (R)" @click="emit('rotate', 1)">
        <icon icon="viewer.rotate-right" color="currentColor" />
      </button>
    </viewer-action-group>
    <viewer-action-group>
      <button :title="pageFullscreenTitle" :class="{active: pageFullscreen}" @click="emit('toggle-page-fullscreen')">
        <icon :icon="pageFullscreenIcon" color="currentColor" />
      </button>
      <button :title="browserFullscreenTitle" :class="{active: browserFullscreen}" @click="emit('toggle-browser-fullscreen')">
        <icon :icon="browserFullscreenIcon" color="currentColor" />
      </button>
      <button :title="filmstripTitle" :class="{active: showFilmstrip}" :disabled="imageCount <= 1" @click="emit('toggle-filmstrip')">
        <icon :icon="filmstripIcon" color="currentColor" />
      </button>
    </viewer-action-group>
    <viewer-action-group>
      <button title="下载" @click="emit('download')">
        <icon icon="action.download" color="currentColor" />
      </button>
      <button title="关闭" @click="emit('close')">
        <icon icon="action.close" color="currentColor" />
      </button>
    </viewer-action-group>
  </viewer-toolbar>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.zoom-mode-label {
  @apply block w-12 text-center font-semibold tabular-nums;
}

@media (max-width: 840px) {
  .zoom-group {
    @apply order-last;
  }
}
</style>
