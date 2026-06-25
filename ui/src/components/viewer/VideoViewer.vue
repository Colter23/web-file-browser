<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import type {ShellNoticePayload} from "../shell/types.ts";
import {downloadUrl} from "../../network/api.ts";
import {formatEntryDate, formatEntrySize} from "../../utils/file-entry.ts";
import {readBooleanStorage, writeBooleanStorage} from "../../utils/safe-storage.ts";
import Icon from "../Icon.vue";

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

const viewerRef = ref<HTMLElement | null>(null);
const videoRef = ref<HTMLVideoElement | null>(null);
const loading = ref(false);
const error = ref("");
const pageFullscreen = ref(false);
const browserFullscreen = ref(false);
const autoPlay = ref(readBooleanStorage("explorer.videoViewer.autoPlay", false));

const currentEntry = computed(() => props.visible ? props.entry : null);
const sourceUrl = computed(() => currentEntry.value ? downloadUrl(currentEntry.value.path) : "");

const currentIndex = computed(() => {
  const entry = props.entry;
  if (!entry) return -1;
  return props.entries.findIndex(item => item.path === entry.path);
});

const videoCount = computed(() => props.entries.length);
const canShowPrevious = computed(() => currentIndex.value > 0);
const canShowNext = computed(() => currentIndex.value >= 0 && currentIndex.value < props.entries.length - 1);
const pageFullscreenTitle = computed(() => pageFullscreen.value ? "退出网页全屏 (F)" : "网页全屏 (F)");
const browserFullscreenTitle = computed(() => browserFullscreen.value ? "退出浏览器全屏" : "浏览器全屏");
const autoPlayTitle = computed(() => autoPlay.value ? "关闭自动播放" : "开启自动播放");

const subtitle = computed(() => {
  const entry = props.entry;
  if (!entry) return "";
  const position = currentIndex.value >= 0 && videoCount.value > 1 ? `${currentIndex.value + 1} / ${videoCount.value} · ` : "";
  return `${position}${formatEntrySize(entry.size, "0 B")} · ${formatEntryDate(entry.modified)}`;
});

const focusViewer = async () => {
  await nextTick();
  viewerRef.value?.focus();
}

const resetRuntimeState = () => {
  const fullscreenElement = document.fullscreenElement;
  if (fullscreenElement && viewerRef.value?.contains(fullscreenElement)) {
    void document.exitFullscreen().catch(() => undefined);
  }
  loading.value = false;
  error.value = "";
  pageFullscreen.value = false;
  browserFullscreen.value = false;
}

const prepareEntry = async () => {
  if (!props.visible || !props.entry) return;
  loading.value = true;
  error.value = "";
  await nextTick();
  videoRef.value?.load();
  if (autoPlay.value) void playCurrentVideo();
  await focusViewer();
}

const close = () => emit("close");

const showAdjacent = (direction: -1 | 1) => {
  const next = props.entries[currentIndex.value + direction];
  if (next) emit("select", next);
}

const togglePageFullscreen = async () => {
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
      await target.requestFullscreen();
    }
    await focusViewer();
  } catch {
    emit("notice", {
      kind: "warning",
      title: "无法全屏",
      message: "当前浏览器未允许进入全屏，仍可在页面内播放视频。"
    });
  }
}

const downloadCurrent = () => {
  if (props.entry) emit("download", props.entry);
}

const playCurrentVideo = async () => {
  try {
    await videoRef.value?.play();
  } catch {
    emit("notice", {
      kind: "warning",
      title: "无法自动播放",
      message: "浏览器阻止了本次自动播放，可手动点击视频控件继续播放。"
    });
  }
}

const toggleAutoPlay = () => {
  autoPlay.value = !autoPlay.value;
  writeBooleanStorage("explorer.videoViewer.autoPlay", autoPlay.value);
  if (autoPlay.value) void playCurrentVideo();
}

const handleReady = () => {
  loading.value = false;
  error.value = "";
}

const handleError = () => {
  loading.value = false;
  error.value = "视频加载失败，请检查文件是否仍可读取或浏览器是否支持此格式。";
}

const handleFullscreenChange = () => {
  browserFullscreen.value = document.fullscreenElement === viewerRef.value;
}

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if (!props.visible) return;
  const key = event.key.toLowerCase();
  if (key === "escape") {
    event.preventDefault();
    event.stopImmediatePropagation();
    close();
    return;
  }
  if (key === "f" && !event.ctrlKey && !event.metaKey && !event.altKey) {
    event.preventDefault();
    event.stopImmediatePropagation();
    void togglePageFullscreen();
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
        class="video-viewer"
        :class="{pageFullscreen}"
        tabindex="-1"
        @keydown.esc.prevent.stop="close">
      <div class="video-viewer-toolbar">
        <div class="video-viewer-title">
          <strong>{{ currentEntry.name }}</strong>
          <span>{{ subtitle }}</span>
        </div>
        <div class="video-viewer-actions">
          <button title="上一个视频" :disabled="!canShowPrevious" @click="showAdjacent(-1)">
            <icon icon="action.previous" color="currentColor" />
          </button>
          <button title="下一个视频" :disabled="!canShowNext" @click="showAdjacent(1)">
            <icon icon="action.next" color="currentColor" />
          </button>
          <button class="text-action" :title="autoPlayTitle" :class="{active: autoPlay}" @click="toggleAutoPlay">
            <icon icon="view.video" color="currentColor" />
            <span>自动播放</span>
          </button>
          <button :title="pageFullscreenTitle" :class="{active: pageFullscreen}" @click="togglePageFullscreen">
            <icon icon="action.fullscreen" color="currentColor" />
          </button>
          <button :title="browserFullscreenTitle" :class="{active: browserFullscreen}" @click="toggleBrowserFullscreen">
            <icon icon="action.exit-fullscreen" color="currentColor" />
          </button>
          <button title="下载" @click="downloadCurrent">
            <icon icon="action.download" color="currentColor" />
          </button>
          <button title="关闭" @click="close">
            <icon icon="action.close" color="currentColor" />
          </button>
        </div>
      </div>
      <div class="video-viewer-stage">
        <video
            ref="videoRef"
            :src="sourceUrl"
            controls
            :autoplay="autoPlay"
            playsinline
            preload="metadata"
            @loadedmetadata="handleReady"
            @canplay="handleReady"
            @error="handleError">
        </video>
        <div v-if="loading" class="video-viewer-status">正在加载视频...</div>
        <div v-if="error" class="video-viewer-status error">{{ error }}</div>
      </div>
    </section>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.video-viewer {
  @apply absolute inset-0 z-40 flex flex-col overflow-hidden rounded-lg bg-slate-950/78 text-white outline-none backdrop-blur-sm;
}

.video-viewer.pageFullscreen {
  @apply fixed inset-0 z-50 rounded-none;
}

.video-viewer-toolbar {
  @apply flex min-h-14 shrink-0 items-center justify-between gap-3 border-b border-white/15 bg-slate-950/75 px-4 backdrop-blur;
}

.video-viewer-title {
  @apply flex min-w-0 flex-col;
}

.video-viewer-title strong {
  @apply truncate text-sm font-semibold;
}

.video-viewer-title span {
  @apply truncate text-xs text-slate-300;
}

.video-viewer-actions {
  @apply flex shrink-0 items-center gap-1 text-xs text-slate-100;
}

.video-viewer-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-white/30 bg-white/15 px-2 text-sm font-medium text-white shadow-sm hover:border-white/45 hover:bg-white/25;
}

.video-viewer-actions button.text-action {
  @apply gap-1.5 px-3 text-xs;
}

.video-viewer-actions button:disabled {
  @apply cursor-not-allowed border-white/10 bg-white/5 opacity-35 hover:border-white/10 hover:bg-white/5;
}

.video-viewer-actions button:focus-visible {
  @apply outline-none;
  border-color: rgba(255, 255, 255, 0.78);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-accent, #2563eb) 45%, rgba(255, 255, 255, 0.25));
}

.video-viewer-actions button.active {
  @apply text-white;
  border-color: color-mix(in srgb, var(--app-accent-border, #bfdbfe) 80%, transparent);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 52%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--app-accent-border, #bfdbfe) 22%, transparent);
}

.video-viewer-stage {
  @apply relative flex min-h-0 grow items-center justify-center overflow-hidden bg-black/35 p-4;
}

.video-viewer-stage video {
  @apply max-h-full max-w-full rounded-md bg-black shadow-2xl;
}

.video-viewer-status {
  @apply absolute left-1/2 top-1/2 rounded-md border border-white/15 bg-black/45 px-3 py-2 text-sm text-slate-100;
  transform: translate(-50%, -50%);
}

.video-viewer-status.error {
  @apply text-red-100;
}
</style>
