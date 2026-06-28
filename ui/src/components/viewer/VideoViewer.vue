<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import type {ShellNoticePayload} from "../shell/types.ts";
import {downloadUrl} from "../../network/api.ts";
import {formatEntryDate, formatEntrySize} from "../../utils/file-entry.ts";
import {readBooleanStorage, readNumberStorage, writeBooleanStorage, writeNumberStorage} from "../../utils/safe-storage.ts";
import FileTypeIcon from "../FileTypeIcon.vue";
import Icon from "../Icon.vue";

const props = defineProps<{
  visible: boolean;
  entry: ExplorerEntry | null;
  entries: ExplorerEntry[];
}>();

const DEFAULT_VIDEO_ASPECT = 16 / 9;
const DEFAULT_VIDEO_RATIO = "16 / 9";
const PILLARBOX_MIN_ASPECT = 1.22;
const PILLARBOX_MIN_RATIO = "122 / 100";

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
const playing = ref(false);
const muted = ref(readBooleanStorage("explorer.videoViewer.muted", false));
const volume = ref(Math.min(1, Math.max(0, readNumberStorage("explorer.videoViewer.volume", 0.2))));
const currentTime = ref(0);
const duration = ref(0);
const bufferedTime = ref(0);
const videoWidth = ref(0);
const videoHeight = ref(0);
const shouldResumePlayback = ref(false);
const seekingProgress = ref(false);
const progressHovering = ref(false);
const progressHoverRatio = ref(0);
const progressHoverX = ref(0);
const volumeHovering = ref(false);
const volumeHoverRatio = ref(0.2);
const volumeHoverX = ref(0);
const adjustingVolume = ref(false);
let clickPlaybackTimer: number | undefined;
let volumeHoverHideTimer: number | undefined;

function removeVolumeReleaseListeners() {
  window.removeEventListener("pointerup", finishWindowVolumeAdjust, true);
  window.removeEventListener("pointercancel", finishWindowVolumeAdjust, true);
  window.removeEventListener("blur", finishWindowVolumeAdjust, true);
}

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
const playTitle = computed(() => playing.value ? "暂停视频 (Space)" : "播放视频 (Space)");
const muteTitle = computed(() => muted.value || volume.value === 0 ? "取消静音" : "静音");
const pageFullscreenIcon = computed(() => pageFullscreen.value ? "viewer.page-fullscreen-off" : "viewer.page-fullscreen");
const browserFullscreenIcon = computed(() => browserFullscreen.value ? "viewer.browser-fullscreen-off" : "viewer.browser-fullscreen");
const progressMax = computed(() => Number.isFinite(duration.value) && duration.value > 0 ? duration.value : 0);
const progressPercent = computed(() => {
  if (!progressMax.value) return "0%";
  return `${Math.min(100, Math.max(0, currentTime.value / progressMax.value * 100))}%`;
});
const bufferedPercent = computed(() => {
  if (!progressMax.value) return "0%";
  const loaded = Math.max(currentTime.value, bufferedTime.value);
  return `${Math.min(100, Math.max(0, loaded / progressMax.value * 100))}%`;
});
const volumePercent = computed(() => `${Math.round(volume.value * 100)}%`);
const audibleVolumePercent = computed(() => `${Math.round((muted.value ? 0 : volume.value) * 100)}%`);
const progressStyle = computed(() => ({
  "--video-progress": progressPercent.value,
  "--video-buffered": bufferedPercent.value,
  "--video-progress-hover-x": `${progressHoverX.value}px`
}));
const volumeStyle = computed(() => ({
  "--video-volume": muted.value ? "0%" : volumePercent.value,
  "--video-volume-hover-x": `${volumeHoverX.value}px`
}));
const cardStyle = computed(() => {
  const hasNaturalSize = videoWidth.value > 0 && videoHeight.value > 0;
  const rawAspect = hasNaturalSize
      ? videoWidth.value / videoHeight.value
      : DEFAULT_VIDEO_ASPECT;
  const isPortrait = rawAspect < 1;
  const frameAspect = isPortrait ? Math.max(rawAspect, PILLARBOX_MIN_ASPECT) : rawAspect;
  const frameRatio = hasNaturalSize && !isPortrait
      ? `${videoWidth.value} / ${videoHeight.value}`
      : isPortrait ? PILLARBOX_MIN_RATIO : DEFAULT_VIDEO_RATIO;
  return {
    "--video-aspect-ratio": frameRatio,
    "--video-aspect-value": frameAspect.toString()
  };
});

const timeText = (seconds: number) => {
  if (!Number.isFinite(seconds) || seconds <= 0) return "0:00";
  const total = Math.round(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = total % 60;
  const paddedMinutes = hours > 0 ? minutes.toString().padStart(2, "0") : minutes.toString();
  return hours > 0
      ? `${hours}:${paddedMinutes}:${rest.toString().padStart(2, "0")}`
      : `${paddedMinutes}:${rest.toString().padStart(2, "0")}`;
}

const durationText = computed(() => timeText(duration.value));
const currentTimeText = computed(() => timeText(currentTime.value));
const playbackTimeText = computed(() => `${currentTimeText.value} / ${durationText.value}`);
const progressHoverTimeText = computed(() => timeText(progressHoverRatio.value * progressMax.value));
const volumeHoverText = computed(() => `${Math.round(volumeHoverRatio.value * 100)}%`);

const subtitle = computed(() => {
  const entry = props.entry;
  if (!entry) return "";
  const position = currentIndex.value >= 0 && videoCount.value > 1 ? `${currentIndex.value + 1} / ${videoCount.value} · ` : "";
  const details = [
    formatEntrySize(entry.size, "0 B"),
    duration.value > 0 ? durationText.value : "",
    videoWidth.value && videoHeight.value ? `${videoWidth.value} × ${videoHeight.value}` : "",
    formatEntryDate(entry.modified)
  ].filter(Boolean);
  return `${position}${details.join(" · ")}`;
});

const focusViewer = async () => {
  await nextTick();
  viewerRef.value?.focus();
}

const resetRuntimeState = () => {
  if (clickPlaybackTimer !== undefined) {
    window.clearTimeout(clickPlaybackTimer);
    clickPlaybackTimer = undefined;
  }
  removeVolumeReleaseListeners();
  if (volumeHoverHideTimer !== undefined) {
    window.clearTimeout(volumeHoverHideTimer);
    volumeHoverHideTimer = undefined;
  }
  const fullscreenElement = document.fullscreenElement;
  if (fullscreenElement && viewerRef.value?.contains(fullscreenElement)) {
    void document.exitFullscreen().catch(() => undefined);
  }
  videoRef.value?.pause();
  loading.value = false;
  error.value = "";
  pageFullscreen.value = false;
  browserFullscreen.value = false;
  playing.value = false;
  currentTime.value = 0;
  duration.value = 0;
  bufferedTime.value = 0;
  videoWidth.value = 0;
  videoHeight.value = 0;
  shouldResumePlayback.value = false;
  seekingProgress.value = false;
  progressHovering.value = false;
  volumeHovering.value = false;
  adjustingVolume.value = false;
}

const prepareEntry = async () => {
  if (!props.visible || !props.entry) return;
  shouldResumePlayback.value = autoPlay.value || playing.value;
  playing.value = false;
  currentTime.value = 0;
  duration.value = 0;
  bufferedTime.value = 0;
  videoWidth.value = 0;
  videoHeight.value = 0;
  loading.value = true;
  error.value = "";
  await nextTick();
  videoRef.value?.load();
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

const syncVideoSettings = () => {
  const video = videoRef.value;
  if (!video) return;
  video.volume = volume.value;
  video.muted = muted.value;
}

const playCurrentVideo = async (source: "auto" | "manual" = "auto") => {
  try {
    syncVideoSettings();
    await videoRef.value?.play();
  } catch {
    shouldResumePlayback.value = false;
    emit("notice", {
      kind: "warning",
      title: source === "auto" ? "无法自动播放" : "无法播放",
      message: source === "auto"
          ? "浏览器阻止了本次自动播放，可手动点击视频控件继续播放。"
          : "浏览器没有成功开始播放，请检查文件是否可播放。"
    });
  }
}

const pauseCurrentVideo = () => {
  videoRef.value?.pause();
}

const togglePlayback = () => {
  shouldResumePlayback.value = false;
  if (playing.value) {
    pauseCurrentVideo();
    return;
  }
  void playCurrentVideo("manual");
}

const handleVideoClick = () => {
  if (clickPlaybackTimer !== undefined) {
    window.clearTimeout(clickPlaybackTimer);
  }
  clickPlaybackTimer = window.setTimeout(() => {
    clickPlaybackTimer = undefined;
    togglePlayback();
  }, 160);
}

const handleVideoDoubleClick = () => {
  if (clickPlaybackTimer !== undefined) {
    window.clearTimeout(clickPlaybackTimer);
    clickPlaybackTimer = undefined;
  }
  void togglePageFullscreen();
}

const toggleAutoPlay = () => {
  autoPlay.value = !autoPlay.value;
  writeBooleanStorage("explorer.videoViewer.autoPlay", autoPlay.value);
  if (autoPlay.value) void playCurrentVideo();
}

const seekBy = (seconds: number) => {
  const video = videoRef.value;
  if (!video || !progressMax.value) return;
  const nextTime = Math.min(progressMax.value, Math.max(0, video.currentTime + seconds));
  video.currentTime = nextTime;
  currentTime.value = nextTime;
  syncBufferedTime();
}

const syncBufferedTime = () => {
  const video = videoRef.value;
  if (!video || !Number.isFinite(video.duration) || video.duration <= 0) {
    bufferedTime.value = 0;
    return;
  }
  const buffered = video.buffered;
  let loaded = 0;
  for (let index = 0; index < buffered.length; index += 1) {
    const start = buffered.start(index);
    const end = buffered.end(index);
    if (video.currentTime >= start && video.currentTime <= end) {
      loaded = end;
      break;
    }
    loaded = Math.max(loaded, end);
  }
  bufferedTime.value = Math.min(video.duration, Math.max(loaded, video.currentTime));
}

const handleReady = () => {
  const video = videoRef.value;
  if (video) {
    duration.value = video.duration || 0;
    videoWidth.value = video.videoWidth || 0;
    videoHeight.value = video.videoHeight || 0;
    currentTime.value = video.currentTime || 0;
    syncVideoSettings();
    syncBufferedTime();
  }
  loading.value = false;
  error.value = "";
  if (shouldResumePlayback.value) {
    shouldResumePlayback.value = false;
    void playCurrentVideo();
  }
}

const handleError = () => {
  loading.value = false;
  error.value = "视频加载失败，请检查文件是否仍可读取或浏览器是否支持此格式。";
  shouldResumePlayback.value = false;
}

const handleFullscreenChange = () => {
  browserFullscreen.value = document.fullscreenElement === viewerRef.value;
}

const handlePlay = () => {
  playing.value = true;
}

const handlePause = () => {
  playing.value = false;
}

const handleTimeUpdate = () => {
  const video = videoRef.value;
  if (!video || seekingProgress.value) return;
  currentTime.value = video.currentTime || 0;
  syncBufferedTime();
}

const handleProgressUpdate = () => {
  syncBufferedTime();
}

const updateProgress = (event: Event) => {
  const video = videoRef.value;
  const input = event.target as HTMLInputElement;
  const value = Number(input.value);
  if (!video || !Number.isFinite(value)) return;
  video.currentTime = value;
  currentTime.value = value;
  syncBufferedTime();
}

const pointerRatio = (event: PointerEvent, fallback = 0) => {
  const target = event.currentTarget as HTMLElement | null;
  const rect = target?.getBoundingClientRect();
  if (!rect || rect.width <= 0) return {x: 0, ratio: fallback};
  const x = Math.min(rect.width, Math.max(0, event.clientX - rect.left));
  return {
    x,
    ratio: x / rect.width
  };
}

const seekProgressFromPointer = (event: PointerEvent) => {
  const video = videoRef.value;
  const {x, ratio} = pointerRatio(event, progressMax.value ? currentTime.value / progressMax.value : 0);
  progressHoverX.value = x;
  progressHoverRatio.value = ratio;
  if (!video || !progressMax.value) return;
  const nextTime = ratio * progressMax.value;
  video.currentTime = nextTime;
  currentTime.value = nextTime;
  syncBufferedTime();
}

const updateProgressHover = (event: PointerEvent) => {
  const {x, ratio} = pointerRatio(event, progressMax.value ? currentTime.value / progressMax.value : 0);
  progressHoverX.value = x;
  progressHoverRatio.value = ratio;
  if (seekingProgress.value) seekProgressFromPointer(event);
}

const showProgressHover = (event: PointerEvent) => {
  progressHovering.value = true;
  updateProgressHover(event);
}

const hideProgressHover = () => {
  if (seekingProgress.value) return;
  progressHovering.value = false;
}

const startProgressSeek = (event: PointerEvent) => {
  seekingProgress.value = true;
  progressHovering.value = true;
  (event.currentTarget as HTMLElement | null)?.setPointerCapture?.(event.pointerId);
  seekProgressFromPointer(event);
  event.preventDefault();
}

const finishProgressSeek = (event: PointerEvent | Event) => {
  if (!seekingProgress.value) return;
  seekingProgress.value = false;
  if (event instanceof PointerEvent) {
    const target = event.currentTarget as HTMLElement | null;
    if (target?.hasPointerCapture?.(event.pointerId)) {
      target.releasePointerCapture(event.pointerId);
    }
  } else {
    updateProgress(event);
  }
}

const updateVolume = (event: Event) => {
  const input = event.target as HTMLInputElement;
  const nextVolume = Math.min(1, Math.max(0, Number(input.value)));
  if (!Number.isFinite(nextVolume)) return;
  volume.value = nextVolume;
  muted.value = nextVolume === 0;
  volumeHoverRatio.value = muted.value ? 0 : nextVolume;
  syncVideoSettings();
  writeNumberStorage("explorer.videoViewer.volume", volume.value);
  writeBooleanStorage("explorer.videoViewer.muted", muted.value);
}

const updateVolumeHover = (event: PointerEvent) => {
  if (volumeHoverHideTimer !== undefined) {
    window.clearTimeout(volumeHoverHideTimer);
    volumeHoverHideTimer = undefined;
  }
  const {x, ratio} = pointerRatio(event, muted.value ? 0 : volume.value);
  volumeHoverX.value = x;
  volumeHoverRatio.value = ratio;
  volumeHovering.value = true;
}

const showVolumeHover = (event: PointerEvent) => {
  volumeHovering.value = true;
  updateVolumeHover(event);
}

const hideVolumeSoon = () => {
  adjustingVolume.value = false;
  removeVolumeReleaseListeners();
  if (volumeHoverHideTimer !== undefined) window.clearTimeout(volumeHoverHideTimer);
  volumeHoverHideTimer = window.setTimeout(() => {
    volumeHovering.value = false;
    volumeHoverHideTimer = undefined;
  }, 520);
}

function finishWindowVolumeAdjust() {
  if (!adjustingVolume.value) return;
  hideVolumeSoon();
}

const hideVolumeHover = () => {
  if (adjustingVolume.value) return;
  volumeHovering.value = false;
}

const startVolumeAdjust = (event: PointerEvent) => {
  removeVolumeReleaseListeners();
  adjustingVolume.value = true;
  volumeHovering.value = true;
  window.addEventListener("pointerup", finishWindowVolumeAdjust, true);
  window.addEventListener("pointercancel", finishWindowVolumeAdjust, true);
  window.addEventListener("blur", finishWindowVolumeAdjust, true);
  (event.currentTarget as HTMLElement | null)?.setPointerCapture?.(event.pointerId);
  updateVolumeHover(event);
}

const finishVolumeAdjust = (event?: PointerEvent | Event) => {
  if (event instanceof PointerEvent) {
    const target = event.currentTarget as HTMLElement | null;
    if (target?.hasPointerCapture?.(event.pointerId)) {
      target.releasePointerCapture(event.pointerId);
    }
  }
  finishWindowVolumeAdjust();
}

const toggleMute = () => {
  muted.value = !muted.value;
  if (!muted.value && volume.value === 0) volume.value = 0.2;
  syncVideoSettings();
  writeNumberStorage("explorer.videoViewer.volume", volume.value);
  writeBooleanStorage("explorer.videoViewer.muted", muted.value);
}

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if (!props.visible) return;
  const key = event.key.toLowerCase();
  const hasSystemModifier = event.ctrlKey || event.metaKey;
  const fromControl = event.target instanceof HTMLElement && Boolean(event.target.closest(".video-viewer-controls"));
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
  if (!event.altKey && key === "f") {
    takeOver();
    void togglePageFullscreen();
    return;
  }
  if (!event.altKey && !fromControl && event.code === "Space") {
    takeOver();
    togglePlayback();
    return;
  }
  if (!fromControl && event.altKey && event.key === "ArrowLeft") {
    takeOver();
    seekBy(-5);
    return;
  }
  if (!fromControl && event.altKey && event.key === "ArrowRight") {
    takeOver();
    seekBy(5);
    return;
  }
  if (!fromControl && !event.altKey && event.key === "ArrowLeft") {
    takeOver();
    showAdjacent(-1);
    return;
  }
  if (!fromControl && !event.altKey && event.key === "ArrowRight") {
    takeOver();
    showAdjacent(1);
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
        :class="{pageFullscreen, browserFullscreen}"
        tabindex="-1"
        @keydown.esc.prevent.stop="close">
      <div class="video-viewer-toolbar">
        <div class="video-viewer-title">
          <span class="video-viewer-title-icon">
            <file-type-icon kind="video" :name="currentEntry.name" :extension="currentEntry.extension" size="1.15rem" />
          </span>
          <span class="video-viewer-title-text">
            <strong>{{ currentEntry.name }}</strong>
            <span>{{ subtitle }}</span>
          </span>
        </div>
        <div class="video-viewer-actions">
          <div class="video-viewer-action-group">
            <button title="上一个视频 (←)" :disabled="!canShowPrevious" @click="showAdjacent(-1)">
              <icon icon="action.previous" color="currentColor" size="1.1rem" />
            </button>
            <button title="下一个视频 (→)" :disabled="!canShowNext" @click="showAdjacent(1)">
              <icon icon="action.next" color="currentColor" size="1.1rem" />
            </button>
          </div>
          <div class="video-viewer-action-group">
            <button
                class="video-auto-play"
                :title="autoPlayTitle"
                :class="{active: autoPlay}"
                :aria-pressed="autoPlay"
                @click="toggleAutoPlay">
              <icon icon="action.play" color="currentColor" />
              <span>自动播放</span>
              <span class="video-switch" aria-hidden="true"></span>
            </button>
          </div>
          <div class="video-viewer-action-group">
            <button :title="pageFullscreenTitle" :class="{active: pageFullscreen}" @click="togglePageFullscreen">
              <icon :icon="pageFullscreenIcon" color="currentColor" />
            </button>
            <button :title="browserFullscreenTitle" :class="{active: browserFullscreen}" @click="toggleBrowserFullscreen">
              <icon :icon="browserFullscreenIcon" color="currentColor" />
            </button>
          </div>
          <div class="video-viewer-action-group">
            <button title="下载" @click="downloadCurrent">
              <icon icon="action.download" color="currentColor" />
            </button>
            <button title="关闭" @click="close">
              <icon icon="action.close" color="currentColor" />
            </button>
          </div>
        </div>
      </div>
      <div class="video-viewer-stage">
        <div class="video-viewer-layout">
          <div class="video-viewer-card" :style="cardStyle">
            <div class="video-viewer-frame">
              <video
                  ref="videoRef"
                  :src="sourceUrl"
                  playsinline
                  preload="metadata"
                  @click="handleVideoClick"
                  @dblclick.stop="handleVideoDoubleClick"
                  @loadedmetadata="handleReady"
                  @durationchange="handleReady"
                  @canplay="handleReady"
                  @timeupdate="handleTimeUpdate"
                  @progress="handleProgressUpdate"
                  @loadeddata="handleProgressUpdate"
                  @play="handlePlay"
                  @pause="handlePause"
                  @ended="handlePause"
                  @error="handleError">
              </video>
            </div>
            <div class="video-viewer-controls" @click.stop>
              <div class="video-control-row">
                <button class="video-control-play" :title="playTitle" @click="togglePlayback">
                  <icon :icon="playing ? 'action.pause' : 'action.play'" color="currentColor" />
                </button>
                <span class="video-time">{{ playbackTimeText }}</span>
                <div
                    class="video-progress-row"
                    :class="{hovering: progressHovering}"
                    :style="progressStyle"
                    @pointerenter="showProgressHover"
                    @pointermove="updateProgressHover"
                    @pointerleave="hideProgressHover"
                    @pointerdown="startProgressSeek"
                    @pointerup="finishProgressSeek"
                    @pointercancel="finishProgressSeek"
                    @lostpointercapture="finishProgressSeek">
                  <input
                      type="range"
                      min="0"
                      :max="progressMax"
                      step="0.1"
                      :value="currentTime"
                      aria-label="视频播放进度"
                      @input="updateProgress"
                      @change="updateProgress">
                  <span class="video-progress-track" aria-hidden="true"></span>
                  <span class="video-progress-badge">{{ progressHoverTimeText }}</span>
                </div>
                <div class="video-volume" :style="volumeStyle">
                  <button :title="`${muteTitle}，当前音量 ${audibleVolumePercent}`" @click="toggleMute">
                    <icon :icon="muted || volume === 0 ? 'action.volume-muted' : 'action.volume'" color="currentColor" />
                  </button>
                  <div
                      class="video-volume-range"
                      :class="{hovering: volumeHovering}"
                      @pointerenter="showVolumeHover"
                      @pointermove="updateVolumeHover"
                      @pointerleave="hideVolumeHover"
                      @pointerdown="startVolumeAdjust"
                      @pointerup="finishVolumeAdjust"
                      @pointercancel="finishVolumeAdjust"
                      @lostpointercapture="finishVolumeAdjust">
                    <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.01"
                        :value="muted ? 0 : volume"
                        aria-label="视频音量"
                        @input="updateVolume"
                        @change="finishVolumeAdjust">
                    <span class="video-volume-badge">{{ volumeHoverText }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div v-if="loading" class="video-viewer-status">正在加载视频...</div>
        <div v-if="error" class="video-viewer-status error">{{ error }}</div>
      </div>
    </section>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.video-viewer {
  @apply absolute inset-0 z-40 flex flex-col overflow-hidden rounded-lg text-white outline-none backdrop-blur-sm;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 7%, rgba(2, 6, 23, 0.78));
}

.video-viewer.pageFullscreen {
  @apply fixed inset-0 z-50 rounded-none;
}

.video-viewer-toolbar {
  @apply flex min-h-14 shrink-0 items-center justify-between gap-3 border-b px-3.5 backdrop-blur;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.12));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 6%, rgba(15, 23, 42, 0.74));
}

.video-viewer-title {
  @apply flex min-w-0 items-center gap-2.5;
}

.video-viewer-title-icon {
  @apply grid h-8 w-8 shrink-0 place-items-center rounded-md border border-white/10 bg-white/10 text-pink-200 shadow-sm;
}

.video-viewer-title-text {
  @apply flex min-w-0 flex-col;
}

.video-viewer-title strong {
  @apply truncate text-sm font-semibold leading-5;
}

.video-viewer-title-text > span {
  @apply truncate text-xs leading-4 text-slate-300;
}

.video-viewer-actions {
  @apply flex shrink-0 items-center gap-1.5 text-xs text-slate-100;
}

.video-viewer-action-group {
  @apply inline-flex h-9 items-center overflow-hidden rounded-lg border p-0.5 shadow-sm;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.14));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 3%, rgba(255, 255, 255, 0.1));
}

.video-viewer-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-transparent bg-transparent px-2 text-sm font-medium text-white transition hover:bg-white/20;
}

.video-viewer-actions button.video-auto-play {
  @apply gap-1.5 px-2.5 text-xs;
}

.video-viewer-actions button:disabled {
  @apply cursor-not-allowed opacity-35 hover:bg-transparent;
}

.video-viewer-actions button:focus-visible {
  @apply outline-none;
  border-color: rgba(255, 255, 255, 0.78);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-accent, #2563eb) 45%, rgba(255, 255, 255, 0.25));
}

.video-viewer-actions button.active {
  @apply text-white;
  border-color: color-mix(in srgb, var(--app-accent-border, #bfdbfe) 72%, transparent);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 32%, rgba(255, 255, 255, 0.04));
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--app-accent-border, #bfdbfe) 22%, transparent);
}

.video-switch {
  @apply relative ml-0.5 inline-flex h-4 w-7 shrink-0 rounded-full border border-white/20 bg-black/25;
}

.video-switch::after {
  @apply absolute left-0.5 top-1/2 h-2.5 w-2.5 rounded-full bg-white/70 shadow-sm transition;
  content: "";
  transform: translateY(-50%);
}

.video-auto-play.active .video-switch {
  border-color: color-mix(in srgb, var(--app-accent-border, #bfdbfe) 55%, rgba(255, 255, 255, 0.25));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 52%, rgba(255, 255, 255, 0.08));
}

.video-auto-play.active .video-switch::after {
  @apply bg-white;
  transform: translate(0.72rem, -50%);
}

.video-viewer-stage {
  @apply relative flex min-h-0 grow items-center justify-center overflow-hidden p-5;
  container-type: size;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 4%, rgba(2, 6, 23, 0.18));
}

.video-viewer:is(.pageFullscreen, .browserFullscreen) .video-viewer-stage {
  @apply p-3;
}

.video-viewer-layout {
  @apply flex h-full w-full min-w-0 max-w-[min(100%,82rem)] items-center justify-center;
}

.video-viewer:is(.pageFullscreen, .browserFullscreen) .video-viewer-layout {
  max-width: none;
}

.video-viewer-card {
  @apply flex max-h-full min-h-0 max-w-full flex-col overflow-hidden rounded-xl border border-white/12 shadow-2xl;
  --video-frame-target-height: min(84vh, calc(100vh - 18rem));
  width: clamp(min(100%, 30rem), calc(var(--video-frame-target-height) * var(--video-aspect-value, 1.777)), 100%);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 7%, rgba(8, 13, 26, 0.84));
}

.video-viewer:is(.pageFullscreen, .browserFullscreen) .video-viewer-card {
  --video-frame-target-height: calc(100vh - 9rem);
}

@supports (height: 100cqh) {
  .video-viewer-card,
  .video-viewer:is(.pageFullscreen, .browserFullscreen) .video-viewer-card {
    --video-frame-target-height: max(16rem, calc(100cqh - 3.875rem));
  }
}

.video-viewer-frame {
  @apply relative flex min-h-0 w-full items-center justify-center overflow-hidden rounded-t-xl;
  aspect-ratio: var(--video-aspect-ratio, 16 / 9);
  max-height: var(--video-frame-target-height);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 3%, rgba(0, 0, 0, 0.9));
}

.video-viewer-frame video {
  @apply object-contain;
  @apply block h-full w-full cursor-pointer;
}

.video-viewer-controls {
  @apply shrink-0 rounded-b-xl border-t border-white/10 px-3.5 py-3 text-white backdrop-blur-xl;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 8%, rgba(15, 23, 42, 0.86));
}

.video-progress-row {
  @apply relative h-8 min-w-0 flex-1 cursor-pointer;
}

.video-progress-row input[type="range"] {
  @apply absolute inset-x-0 top-1/2 z-10 h-8 w-full cursor-pointer opacity-0;
  transform: translateY(-50%);
}

.video-progress-track {
  @apply pointer-events-none absolute left-0 right-0 top-1/2 h-2.5 overflow-hidden rounded-full border border-white/10;
  background: linear-gradient(
      to right,
      var(--app-accent, #2563eb) 0%,
      var(--app-accent, #2563eb) var(--video-progress, 0%),
      color-mix(in srgb, var(--app-accent, #2563eb) 34%, rgba(255, 255, 255, 0.34)) var(--video-progress, 0%),
      color-mix(in srgb, var(--app-accent, #2563eb) 34%, rgba(255, 255, 255, 0.34)) var(--video-buffered, 0%),
      rgba(255, 255, 255, 0.16) var(--video-buffered, 0%),
      rgba(255, 255, 255, 0.16) 100%
  );
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);
  transform: translateY(-50%);
  transition: height 140ms ease;
}

.video-progress-row:hover .video-progress-track,
.video-progress-row:focus-within .video-progress-track,
.video-progress-row.hovering .video-progress-track {
  @apply h-3;
}

.video-control-row {
  @apply grid items-center gap-3;
  grid-template-columns: auto max-content minmax(8rem, 1fr) auto;
}

.video-volume {
  @apply flex min-w-0 items-center gap-2;
}

.video-control-row button {
  @apply inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-transparent text-sm text-white transition hover:bg-white/18;
}

.video-control-row button:focus-visible {
  @apply outline-none;
  border-color: rgba(255, 255, 255, 0.72);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-accent, #2563eb) 45%, rgba(255, 255, 255, 0.25));
}

.video-control-play {
  @apply rounded-full;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 74%, rgba(255, 255, 255, 0.14));
  box-shadow: 0 6px 16px rgba(15, 23, 42, 0.24);
}

.video-control-row .video-control-play:hover {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 84%, rgba(255, 255, 255, 0.18));
}

.video-time {
  @apply min-w-[4.7rem] whitespace-nowrap text-left text-[0.78rem] font-semibold tabular-nums text-slate-100;
}

.video-progress-badge,
.video-volume-badge {
  @apply pointer-events-none absolute z-20 rounded-full border border-white/12 px-2.5 py-1 text-[0.76rem] font-semibold leading-none opacity-0 shadow-lg;
  background: color-mix(in srgb, rgba(15, 23, 42, 0.92) 86%, var(--app-accent, #2563eb) 14%);
  color: #fff;
  transition: opacity 120ms ease, transform 120ms ease;
}

.video-progress-badge {
  bottom: calc(50% + 1.05rem);
  left: clamp(1.75rem, var(--video-progress-hover-x, 0px), calc(100% - 1.75rem));
  transform: translate(-50%, 0.25rem);
}

.video-progress-row:hover .video-progress-badge,
.video-progress-row:focus-within .video-progress-badge,
.video-progress-row.hovering .video-progress-badge {
  opacity: 1;
  transform: translate(-50%, 0);
}

.video-volume-range {
  @apply relative h-8 w-24;
}

.video-volume input[type="range"] {
  @apply h-8 w-full cursor-pointer appearance-none bg-transparent;
}

.video-volume-badge {
  bottom: calc(50% + 1.05rem);
  left: clamp(1.3rem, var(--video-volume-hover-x, 0px), calc(100% - 1.3rem));
  transform: translate(-50%, 0.25rem);
}

.video-volume-range.hovering .video-volume-badge {
  opacity: 1;
  transform: translate(-50%, 0);
}

.video-volume input[type="range"]::-webkit-slider-runnable-track {
  height: 0.375rem;
  border-radius: 999px;
  background: linear-gradient(
      to right,
      color-mix(in srgb, var(--app-accent, #2563eb) 72%, white) 0%,
      color-mix(in srgb, var(--app-accent, #2563eb) 72%, white) var(--video-volume, 100%),
      rgba(255, 255, 255, 0.2) var(--video-volume, 100%),
      rgba(255, 255, 255, 0.2) 100%
  );
}

.video-volume input[type="range"]::-webkit-slider-thumb {
  @apply appearance-none rounded-full bg-white shadow-sm;
  width: 0.75rem;
  height: 0.75rem;
  margin-top: -0.1875rem;
}

.video-volume input[type="range"]::-moz-range-track {
  height: 0.375rem;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.2);
}

.video-volume input[type="range"]::-moz-range-progress {
  height: 0.375rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 72%, white);
}

.video-volume input[type="range"]::-moz-range-thumb {
  @apply rounded-full border-0 bg-white shadow-sm;
  width: 0.75rem;
  height: 0.75rem;
}

.video-viewer-status {
  @apply absolute left-1/2 top-1/2 rounded-lg border border-white/10 bg-slate-950/65 px-3 py-2 text-sm text-slate-100 shadow-xl backdrop-blur;
  transform: translate(-50%, -50%);
}

.video-viewer-status.error {
  @apply border-red-300/30 bg-red-950/70 text-red-100;
}

@media (max-width: 840px) {
  .video-viewer-toolbar {
    @apply items-start;
  }

  .video-viewer-actions {
    @apply flex-wrap justify-end;
  }

  .video-volume input[type="range"] {
    @apply w-16;
  }

  .video-volume-range {
    @apply w-16;
  }
}

@media (max-width: 560px) {
  .video-time {
    @apply min-w-[4.3rem] text-[0.72rem];
  }

  .video-control-row {
    grid-template-columns: auto auto minmax(5rem, 1fr) auto;
  }

  .video-volume-range {
    @apply hidden;
  }
}
</style>
