<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {StyleValue} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import type {ShellNoticePayload} from "../shell/types.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import {downloadUrl} from "../../network/api.ts";
import {formatEntrySize} from "../../utils/file-entry.ts";
import {
  readBooleanStorage,
  readJsonStorage,
  readNumberStorage,
  readStorageItem,
  writeBooleanStorage,
  writeJsonStorage,
  writeNumberStorage,
  writeStorageItem
} from "../../utils/safe-storage.ts";
import Icon from "../Icon.vue";
import AudioVolumeControl from "./AudioVolumeControl.vue";

const playModes = ["sequence", "repeat-one", "repeat-all", "shuffle"] as const;
type AudioPlayMode = typeof playModes[number];
const playerSizes = ["large", "small", "mini"] as const;
type AudioPlayerSize = typeof playerSizes[number];
type AudioPlayerPoint = {
  x: number;
  y: number;
}

const playModeStorageKey = "explorer.audioPlayer.playMode";
const playerSizeStorageKey = "explorer.audioPlayer.size";
const playerPositionStorageKey = "explorer.audioPlayer.position";
const defaultPlayerBottomOffset = 90;
const legacyPlayerBottomOffset = 16;
const titleLoopGap = 32;
const playModeMeta: Record<AudioPlayMode, {label: string; icon: string; title: string}> = {
  "sequence": {
    label: "播完暂停",
    icon: "playback.sequence",
    title: "按列表播放，播完后暂停"
  },
  "repeat-one": {
    label: "单曲循环",
    icon: "playback.repeat-one",
    title: "循环播放当前音频"
  },
  "repeat-all": {
    label: "列表循环",
    icon: "playback.repeat-all",
    title: "列表播完后从头继续"
  },
  "shuffle": {
    label: "随机播放",
    icon: "playback.shuffle",
    title: "随机播放列表中的音频"
  }
};
const playerSizeMeta: Record<AudioPlayerSize, {label: string; icon: string}> = {
  large: {
    label: "大面板",
    icon: "action.player-small"
  },
  small: {
    label: "小面板",
    icon: "action.player-mini"
  },
  mini: {
    label: "mini 面板",
    icon: "action.player-large"
  }
};

const readPlayMode = (): AudioPlayMode => {
  const value = readStorageItem(playModeStorageKey);
  return playModes.includes(value as AudioPlayMode) ? value as AudioPlayMode : "sequence";
}

const readPlayerSize = (): AudioPlayerSize => {
  const value = readStorageItem(playerSizeStorageKey);
  return playerSizes.includes(value as AudioPlayerSize) ? value as AudioPlayerSize : "large";
}

const isPlayerPoint = (value: unknown): value is AudioPlayerPoint => {
  if (!value || typeof value !== "object") return false;
  const point = value as AudioPlayerPoint;
  return Number.isFinite(point.x) && Number.isFinite(point.y);
}

const readPlayerPosition = () => {
  const value = readJsonStorage<unknown>(playerPositionStorageKey, null);
  return isPlayerPoint(value) ? value : null;
}

const props = defineProps<{
  visible: boolean;
  entry: ExplorerEntry | null;
  entries: ExplorerEntry[];
  reloadKey: number;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "select", entry: ExplorerEntry): void;
  (e: "download", entry: ExplorerEntry): void;
  (e: "notice", payload: ShellNoticePayload): void;
}>();

const audioRef = ref<HTMLAudioElement | null>(null);
const playerRef = ref<HTMLElement | null>(null);
const playlistRef = ref<HTMLElement | null>(null);
const titleFrameRef = ref<HTMLElement | null>(null);
const titleTextRef = ref<HTMLElement | null>(null);
const loading = ref(false);
const error = ref("");
const isPlaying = ref(false);
const playlistVisible = ref(false);
const titleOverflow = ref(false);
const titleScrollDistance = ref(0);
const playerSize = ref<AudioPlayerSize>(readPlayerSize());
const playerPosition = ref<AudioPlayerPoint | null>(readPlayerPosition());
const playerRect = ref({width: 0, height: 0});
const viewportSize = ref({width: window.innerWidth, height: window.innerHeight});
const draggingPlayer = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const bufferedTime = ref(0);
const progressHovering = ref(false);
const progressHoverRatio = ref(0);
const progressHoverX = ref(0);
const seekingProgress = ref(false);
const volume = ref(Math.min(1, Math.max(0, readNumberStorage("explorer.audioPlayer.volume", 0.2))));
const muted = ref(readBooleanStorage("explorer.audioPlayer.muted", false));
const playMode = ref<AudioPlayMode>(readPlayMode());
let dragOffsetX = 0;
let dragOffsetY = 0;
let positionFrame: number | undefined;
let titleResizeObserver: ResizeObserver | undefined;

const currentEntry = computed(() => props.visible ? props.entry : null);
const sourceUrl = computed(() => currentEntry.value ? downloadUrl(currentEntry.value.path) : "");
const playlistEntries = computed(() => props.entries.length ? props.entries : (currentEntry.value ? [currentEntry.value] : []));

const currentIndex = computed(() => {
  const entry = props.entry;
  if (!entry) return -1;
  return playlistEntries.value.findIndex(item => item.path === entry.path);
});

const canShowPrevious = computed(() => {
  const count = playlistEntries.value.length;
  if (playMode.value === "repeat-all") return count > 1;
  return currentIndex.value > 0;
});
const canShowNext = computed(() => {
  const count = playlistEntries.value.length;
  if (playMode.value === "repeat-all" || playMode.value === "shuffle") return count > 1;
  return currentIndex.value >= 0 && currentIndex.value < count - 1;
});
const playTitle = computed(() => isPlaying.value ? "暂停" : "播放");
const muteTitle = computed(() => muted.value || volume.value === 0 ? "取消静音" : "静音");
const playlistTitle = computed(() => playlistVisible.value ? "关闭播放列表" : "打开播放列表");
const playlistCountText = computed(() => `${playlistEntries.value.length} 首`);
const playModeLabel = computed(() => playModeMeta[playMode.value].label);
const playModeIcon = computed(() => playModeMeta[playMode.value].icon);
const playModeTitle = computed(() => `播放模式：${playModeMeta[playMode.value].title}`);
const nextPlayerSize = computed(() => playerSizes[(playerSizes.indexOf(playerSize.value) + 1) % playerSizes.length]);
const playerSizeIcon = computed(() => playerSizeMeta[playerSize.value].icon);
const playerSizeTitle = computed(() => `当前${playerSizeMeta[playerSize.value].label}，切换为${playerSizeMeta[nextPlayerSize.value].label}`);
const miniToolsSide = computed(() => {
  const position = playerPosition.value;
  if (playerSize.value !== "mini" || !position) return "right";
  const padding = 12;
  const playerWidth = playerRect.value.width || 56;
  const toolsWidth = 10.5 * 16;
  return position.x + playerWidth + toolsWidth > viewportSize.value.width - padding ? "left" : "right";
});
const playerClass = computed(() => [
  "audio-player",
  `size-${playerSize.value}`,
  {
    dragging: draggingPlayer.value,
    playing: isPlaying.value,
    titleOverflow: titleOverflow.value,
    "mini-tools-left": miniToolsSide.value === "left"
  }
]);

const timeText = (seconds: number) => {
  if (!Number.isFinite(seconds) || seconds <= 0) return "0:00";
  const totalSeconds = Math.floor(seconds);
  const minutes = Math.floor(totalSeconds / 60);
  const restSeconds = totalSeconds % 60;
  return `${minutes}:${String(restSeconds).padStart(2, "0")}`;
}

const currentTimeText = computed(() => timeText(currentTime.value));
const durationText = computed(() => timeText(duration.value));
const subtitle = computed(() => {
  const entry = currentEntry.value;
  if (!entry) return "";
  const countText = playlistEntries.value.length > 1 && currentIndex.value >= 0
      ? `${currentIndex.value + 1} / ${playlistEntries.value.length} 首`
      : "1 首";
  return `${countText} · ${formatEntrySize(entry.size, "0 B")} · ${currentTimeText.value} / ${durationText.value}`;
});
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
const progressHoverPercent = computed(() => `${Math.min(100, Math.max(0, progressHoverRatio.value * 100))}%`);
const progressHoverTimeText = computed(() => timeText(progressHoverRatio.value * progressMax.value));
const volumePercent = computed(() => `${Math.round(volume.value * 100)}%`);
const audibleVolumePercent = computed(() => `${Math.round((muted.value ? 0 : volume.value) * 100)}%`);
const volumeTitle = computed(() => `音量 ${audibleVolumePercent.value}`);

const progressStyle = computed(() => ({
  "--audio-progress": progressPercent.value,
  "--audio-buffered": bufferedPercent.value,
  "--audio-progress-hover": progressHoverPercent.value,
  "--audio-progress-hover-x": `${progressHoverX.value}px`
}));

const volumeStyle = computed(() => ({
  "--audio-progress": muted.value ? "0%" : volumePercent.value,
  "--audio-volume": muted.value ? "0%" : volumePercent.value
}));

const titleScrollStyle = computed<StyleValue>(() => {
  const distance = Math.max(0, titleScrollDistance.value);
  return {
    "--audio-title-gap": `${titleLoopGap}px`,
    "--audio-title-scroll": `-${distance}px`,
    "--audio-title-duration": `${Math.min(20, Math.max(8, distance / 16 + 5))}s`
  };
});

const playerStyle = computed<StyleValue>(() => {
  const position = playerPosition.value;
  if (!position) return {
    visibility: "hidden"
  };
  return {
    left: `${position.x}px`,
    top: `${position.y}px`,
    visibility: "visible"
  };
});

const playlistStyle = computed<StyleValue>(() => {
  const position = playerPosition.value;
  if (!position) return {};
  const padding = 12;
  const width = Math.min(34 * 16, Math.max(18 * 16, viewportSize.value.width - padding * 2));
  const estimatedHeight = Math.min(21 * 16, 3.25 * 16 + playlistEntries.value.length * 36);
  const playerWidth = playerRect.value.width || 0;
  const playerHeight = playerRect.value.height || 0;
  const minCenter = padding + width / 2;
  const maxCenter = Math.max(minCenter, viewportSize.value.width - padding - width / 2);
  const centerX = Math.min(Math.max(position.x + playerWidth / 2, minCenter), maxCenter);
  const aboveTop = position.y - estimatedHeight - 8;
  const belowTop = position.y + playerHeight + 8;
  const top = aboveTop >= padding
      ? aboveTop
      : Math.min(belowTop, Math.max(padding, viewportSize.value.height - estimatedHeight - padding));
  return {
    left: `${centerX}px`,
    top: `${Math.max(padding, top)}px`
  };
});

const updateViewportSize = () => {
  viewportSize.value = {
    width: window.innerWidth,
    height: window.innerHeight
  };
}

const measurePlayer = () => {
  const rect = playerRef.value?.getBoundingClientRect();
  if (!rect) return null;
  playerRect.value = {
    width: rect.width,
    height: rect.height
  };
  return rect;
}

const measureTitleOverflow = () => {
  const frame = titleFrameRef.value;
  const text = titleTextRef.value;
  if (!frame || !text) {
    titleOverflow.value = false;
    titleScrollDistance.value = 0;
    return;
  }
  const textWidth = Math.max(text.scrollWidth, text.offsetWidth);
  titleOverflow.value = textWidth > frame.clientWidth + 2;
  titleScrollDistance.value = titleOverflow.value ? textWidth + titleLoopGap : 0;
}

const syncTitleResizeObserver = () => {
  if (!titleResizeObserver) return;
  titleResizeObserver.disconnect();
  if (titleFrameRef.value) titleResizeObserver.observe(titleFrameRef.value);
  if (titleTextRef.value) titleResizeObserver.observe(titleTextRef.value);
}

const scheduleTitleMeasure = async () => {
  await nextTick();
  syncTitleResizeObserver();
  measureTitleOverflow();
  await nextTick();
  syncTitleResizeObserver();
  measureTitleOverflow();
}

const clampPlayerPosition = (point: AudioPlayerPoint): AudioPlayerPoint => {
  const rect = measurePlayer();
  const padding = 12;
  const width = rect?.width ?? playerRect.value.width;
  const height = rect?.height ?? playerRect.value.height;
  const maxX = Math.max(padding, viewportSize.value.width - width - padding);
  const maxY = Math.max(padding, viewportSize.value.height - height - padding);
  return {
    x: Math.min(Math.max(point.x, padding), maxX),
    y: Math.min(Math.max(point.y, padding), maxY)
  };
}

const writePlayerPosition = () => {
  if (playerPosition.value) writeJsonStorage(playerPositionStorageKey, playerPosition.value);
}

const setPlayerPosition = (point: AudioPlayerPoint, persist = false) => {
  playerPosition.value = clampPlayerPosition(point);
  if (persist) writePlayerPosition();
}

const placeDefaultPlayerPosition = (persist = false) => {
  const rect = measurePlayer();
  if (!rect) return;
  setPlayerPosition({
    x: (viewportSize.value.width - rect.width) / 2,
    y: viewportSize.value.height - rect.height - defaultPlayerBottomOffset
  }, persist);
}

const isLegacyDefaultPosition = (position: AudioPlayerPoint) => {
  const rect = playerRect.value;
  if (!rect.width || !rect.height) return false;
  const defaultX = (viewportSize.value.width - rect.width) / 2;
  const defaultY = viewportSize.value.height - rect.height - legacyPlayerBottomOffset;
  return Math.abs(position.x - defaultX) < 4 && Math.abs(position.y - defaultY) < 4;
}

const schedulePlayerPosition = async (forceDefault = false) => {
  await nextTick();
  if (positionFrame) window.cancelAnimationFrame(positionFrame);
  positionFrame = window.requestAnimationFrame(() => {
    positionFrame = undefined;
    updateViewportSize();
    measurePlayer();
    measureTitleOverflow();
    if (forceDefault || !playerPosition.value) {
      placeDefaultPlayerPosition(forceDefault);
      return;
    }
    if (isLegacyDefaultPosition(playerPosition.value)) {
      placeDefaultPlayerPosition(true);
      return;
    }
    setPlayerPosition(playerPosition.value, true);
  });
}

const syncAudioSettings = () => {
  const audio = audioRef.value;
  if (!audio) return;
  audio.volume = volume.value;
  audio.muted = muted.value;
}

const playAudio = async () => {
  const audio = audioRef.value;
  if (!audio) return;
  syncAudioSettings();
  try {
    await audio.play();
    isPlaying.value = true;
  } catch {
    isPlaying.value = false;
    emit("notice", {
      kind: "warning",
      title: "无法自动播放",
      message: "浏览器阻止了本次自动播放，可手动点击播放按钮继续。"
    });
  }
}

const pauseAudio = () => {
  audioRef.value?.pause();
  isPlaying.value = false;
}

const prepareEntry = async () => {
  if (!props.visible || !props.entry) return;
  loading.value = true;
  error.value = "";
  currentTime.value = 0;
  duration.value = 0;
  bufferedTime.value = 0;
  await nextTick();
  syncAudioSettings();
  audioRef.value?.load();
  await playAudio();
}

const resetPlayer = () => {
  pauseAudio();
  playlistVisible.value = false;
  loading.value = false;
  error.value = "";
  currentTime.value = 0;
  duration.value = 0;
  bufferedTime.value = 0;
}

const togglePlay = () => {
  if (isPlaying.value) {
    pauseAudio();
    return;
  }
  void playAudio();
}

const close = () => {
  resetPlayer();
  emit("close");
}

const replayCurrent = async () => {
  const audio = audioRef.value;
  if (!audio) return;
  audio.currentTime = 0;
  currentTime.value = 0;
  await playAudio();
}

const linearNextEntry = () => {
  return currentIndex.value >= 0 ? playlistEntries.value[currentIndex.value + 1] : undefined;
}

const pickRandomEntry = () => {
  const entries = playlistEntries.value;
  if (!entries.length) return undefined;
  if (entries.length === 1) return entries[0];
  const candidates = entries.filter(item => item.path !== props.entry?.path);
  return candidates[Math.floor(Math.random() * candidates.length)] ?? entries[0];
}

const showAdjacent = (direction: -1 | 1) => {
  const entries = playlistEntries.value;
  if (!entries.length || currentIndex.value < 0) return;
  const nextIndex = playMode.value === "repeat-all"
      ? (currentIndex.value + direction + entries.length) % entries.length
      : currentIndex.value + direction;
  const next = entries[nextIndex];
  if (next) emit("select", next);
}

const showNextTrack = () => {
  if (playMode.value !== "shuffle") {
    showAdjacent(1);
    return;
  }
  const next = pickRandomEntry();
  if (!next) return;
  if (next.path === props.entry?.path) {
    void replayCurrent();
    return;
  }
  emit("select", next);
}

const showPreviousTrack = () => {
  showAdjacent(-1);
}

const selectTrack = (entry: ExplorerEntry) => {
  if (entry.path === props.entry?.path) {
    playlistVisible.value = false;
    void playAudio();
    return;
  }
  emit("select", entry);
}

const downloadCurrent = () => {
  if (props.entry) emit("download", props.entry);
}

const updateProgress = (event: Event) => {
  const audio = audioRef.value;
  const input = event.target as HTMLInputElement;
  const value = Number(input.value);
  if (!audio || !Number.isFinite(value)) return;
  audio.currentTime = value;
  currentTime.value = value;
  syncBufferedTime();
}

const progressPointerState = (event: PointerEvent) => {
  const target = event.currentTarget as HTMLElement | null;
  const rect = target?.getBoundingClientRect();
  if (!rect || rect.width <= 0) {
    return {x: 0, ratio: 0};
  }
  const x = Math.min(rect.width, Math.max(0, event.clientX - rect.left));
  return {
    x,
    ratio: progressMax.value ? x / rect.width : 0
  };
}

const seekProgressFromPointer = (event: PointerEvent) => {
  const audio = audioRef.value;
  const {x, ratio} = progressPointerState(event);
  progressHoverX.value = x;
  progressHoverRatio.value = ratio;
  if (!audio || !progressMax.value) return;
  const nextTime = ratio * progressMax.value;
  audio.currentTime = nextTime;
  currentTime.value = nextTime;
  syncBufferedTime();
}

const syncBufferedTime = () => {
  const audio = audioRef.value;
  if (!audio || !Number.isFinite(audio.duration) || audio.duration <= 0) {
    bufferedTime.value = 0;
    return;
  }
  const buffered = audio.buffered;
  let loaded = 0;
  for (let index = 0; index < buffered.length; index += 1) {
    const start = buffered.start(index);
    const end = buffered.end(index);
    if (audio.currentTime >= start && audio.currentTime <= end) {
      loaded = end;
      break;
    }
    loaded = Math.max(loaded, end);
  }
  bufferedTime.value = Math.min(audio.duration, Math.max(loaded, audio.currentTime));
}

const updateProgressHover = (event: PointerEvent) => {
  const {x, ratio} = progressPointerState(event);
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

const finishProgressSeek = (event: PointerEvent) => {
  if (!seekingProgress.value) return;
  seekingProgress.value = false;
  const target = event.currentTarget as HTMLElement | null;
  if (target?.hasPointerCapture?.(event.pointerId)) {
    target.releasePointerCapture(event.pointerId);
  }
}

const updateVolume = (event: Event) => {
  const input = event.target as HTMLInputElement;
  const nextVolume = Math.min(1, Math.max(0, Number(input.value)));
  if (!Number.isFinite(nextVolume)) return;
  volume.value = nextVolume;
  muted.value = nextVolume === 0;
  syncAudioSettings();
  writeNumberStorage("explorer.audioPlayer.volume", volume.value);
  writeBooleanStorage("explorer.audioPlayer.muted", muted.value);
}

const toggleMute = () => {
  muted.value = !muted.value;
  syncAudioSettings();
  writeBooleanStorage("explorer.audioPlayer.muted", muted.value);
}

const togglePlaylist = () => {
  playlistVisible.value = !playlistVisible.value;
}

const cyclePlayerSize = () => {
  playerSize.value = nextPlayerSize.value;
  if (playerSize.value === "mini") playlistVisible.value = false;
  writeStorageItem(playerSizeStorageKey, playerSize.value);
  void schedulePlayerPosition();
}

const togglePlayMode = () => {
  const currentModeIndex = playModes.indexOf(playMode.value);
  playMode.value = playModes[(currentModeIndex + 1) % playModes.length];
  writeStorageItem(playModeStorageKey, playMode.value);
}

const handleLoadedMetadata = () => {
  duration.value = audioRef.value?.duration ?? 0;
  syncBufferedTime();
  loading.value = false;
}

const handleCanPlay = () => {
  syncBufferedTime();
  loading.value = false;
}

const handleTimeUpdate = () => {
  currentTime.value = audioRef.value?.currentTime ?? 0;
  syncBufferedTime();
}

const handlePlay = () => {
  isPlaying.value = true;
}

const handlePause = () => {
  isPlaying.value = false;
}

const handleEnded = () => {
  if (playMode.value === "repeat-one") {
    void replayCurrent();
    return;
  }
  if (playMode.value === "shuffle") {
    const next = pickRandomEntry();
    if (!next) {
      isPlaying.value = false;
      return;
    }
    if (next.path === props.entry?.path) {
      void replayCurrent();
      return;
    }
    emit("select", next);
    return;
  }
  const next = linearNextEntry();
  if (next) {
    emit("select", next);
    return;
  }
  if (playMode.value === "repeat-all") {
    const firstEntry = playlistEntries.value[0];
    if (!firstEntry) {
      isPlaying.value = false;
      return;
    }
    if (firstEntry.path === props.entry?.path) {
      void replayCurrent();
      return;
    }
    emit("select", firstEntry);
    return;
  }
  isPlaying.value = false;
}

const handleError = () => {
  loading.value = false;
  isPlaying.value = false;
  error.value = "音频加载失败，请检查文件是否仍可读取或浏览器是否支持此格式。";
}

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if (!playlistVisible.value || event.key !== "Escape") return;
  event.preventDefault();
  event.stopImmediatePropagation();
  playlistVisible.value = false;
}

const handlePlayerPointerMove = (event: PointerEvent) => {
  if (!draggingPlayer.value) return;
  event.preventDefault();
  setPlayerPosition({
    x: event.clientX - dragOffsetX,
    y: event.clientY - dragOffsetY
  });
}

const finishPlayerDrag = () => {
  if (!draggingPlayer.value) return;
  draggingPlayer.value = false;
  window.removeEventListener("pointermove", handlePlayerPointerMove);
  window.removeEventListener("pointerup", finishPlayerDrag);
  window.removeEventListener("pointercancel", finishPlayerDrag);
  writePlayerPosition();
}

const startPlayerDrag = (event: PointerEvent) => {
  if (event.button !== 0) return;
  const player = playerRef.value;
  if (!player) return;
  const rect = player.getBoundingClientRect();
  dragOffsetX = event.clientX - rect.left;
  dragOffsetY = event.clientY - rect.top;
  playerPosition.value = clampPlayerPosition({x: rect.left, y: rect.top});
  draggingPlayer.value = true;
  event.preventDefault();
  window.addEventListener("pointermove", handlePlayerPointerMove);
  window.addEventListener("pointerup", finishPlayerDrag);
  window.addEventListener("pointercancel", finishPlayerDrag);
}

const movePlayerBy = (deltaX: number, deltaY: number) => {
  const position = playerPosition.value;
  if (!position) return;
  setPlayerPosition({
    x: position.x + deltaX,
    y: position.y + deltaY
  }, true);
}

const handlePlayerHandleKeyDown = (event: KeyboardEvent) => {
  if (!["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) return;
  event.preventDefault();
  const step = event.shiftKey ? 32 : 8;
  if (event.key === "ArrowLeft") movePlayerBy(-step, 0);
  if (event.key === "ArrowRight") movePlayerBy(step, 0);
  if (event.key === "ArrowUp") movePlayerBy(0, -step);
  if (event.key === "ArrowDown") movePlayerBy(0, step);
}

const resetPlayerPosition = () => {
  placeDefaultPlayerPosition(true);
}

const handleWindowResize = () => {
  updateViewportSize();
  measureTitleOverflow();
  if (playerPosition.value) setPlayerPosition(playerPosition.value, true);
}

useOutsidePointerDown({
  refs: [playerRef, playlistRef],
  enabled: () => playlistVisible.value,
  onOutsidePointerDown: () => playlistVisible.value = false
});

watch(() => [props.visible, props.entry?.path, props.reloadKey] as const, ([visible, path]) => {
  if (visible && path) {
    void schedulePlayerPosition();
    void prepareEntry();
    return;
  }
  resetPlayer();
}, {flush: "post"});

watch([volume, muted], syncAudioSettings);

watch(() => [currentEntry.value?.name, playerSize.value] as const, () => {
  void scheduleTitleMeasure();
});

onMounted(() => {
  window.addEventListener("keydown", handleWindowKeyDown, true);
  window.addEventListener("resize", handleWindowResize);
  titleResizeObserver = new ResizeObserver(() => {
    measureTitleOverflow();
  });
  syncTitleResizeObserver();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleWindowKeyDown, true);
  window.removeEventListener("resize", handleWindowResize);
  titleResizeObserver?.disconnect();
  if (positionFrame) window.cancelAnimationFrame(positionFrame);
  finishPlayerDrag();
});

onBeforeUnmount(resetPlayer);
</script>

<template>
  <Teleport to="body">
    <Transition name="audio-player">
      <section v-if="currentEntry" ref="playerRef" :class="playerClass" :style="playerStyle" aria-label="音乐播放器">
        <audio
            ref="audioRef"
            :src="sourceUrl"
            preload="metadata"
            @loadedmetadata="handleLoadedMetadata"
            @durationchange="handleLoadedMetadata"
            @canplay="handleCanPlay"
            @progress="syncBufferedTime"
            @timeupdate="handleTimeUpdate"
            @play="handlePlay"
            @pause="handlePause"
            @ended="handleEnded"
            @error="handleError">
        </audio>

        <div class="audio-player-body">
          <div class="audio-player-track">
            <div
                class="audio-player-art"
                :class="{playing: isPlaying}"
                tabindex="0"
                title="拖动移动播放器，双击回到底部"
                aria-label="拖动移动音乐播放器"
                @pointerdown="startPlayerDrag"
                @dblclick="resetPlayerPosition"
                @keydown="handlePlayerHandleKeyDown">
              <icon icon="view.audio" color="currentColor" />
              <button
                  class="audio-player-art-play"
                  :title="playTitle"
                  @pointerdown.stop
                  @dblclick.stop
                  @click.stop="togglePlay">
                <icon :icon="isPlaying ? 'action.pause' : 'action.play'" color="currentColor" />
              </button>
            </div>

            <div class="audio-player-main">
              <div class="audio-player-title-wrap">
                <div class="audio-player-title-row">
                  <strong ref="titleFrameRef" :style="titleScrollStyle" :title="currentEntry.name">
                    <span ref="titleTextRef" class="audio-player-title-text">{{ currentEntry.name }}</span>
                    <span v-show="titleOverflow" class="audio-player-title-text audio-player-title-copy" aria-hidden="true">
                      {{ currentEntry.name }}
                    </span>
                  </strong>
                </div>
                <div class="audio-player-compact-controls" aria-label="播放控制">
                  <button class="mode-button" :title="playModeTitle" @click="togglePlayMode">
                    <icon :icon="playModeIcon" color="currentColor" />
                  </button>
                  <button class="track-button" title="上一首" :disabled="!canShowPrevious" @click="showPreviousTrack">
                    <icon icon="action.previous" color="currentColor" />
                  </button>
                  <button class="play-button" :title="playTitle" @click="togglePlay">
                    <icon :icon="isPlaying ? 'action.pause' : 'action.play'" color="currentColor" />
                  </button>
                  <button class="track-button" title="下一首" :disabled="!canShowNext" @click="showNextTrack">
                    <icon icon="action.next" color="currentColor" />
                  </button>
                  <button
                      class="icon-action playlist-action"
                      :class="{active: playlistVisible}"
                      :title="playlistTitle"
                      :aria-expanded="playlistVisible"
                      aria-controls="audio-player-playlist"
                      @click="togglePlaylist">
                    <icon icon="view.playlist" color="currentColor" />
                  </button>
                </div>
              </div>
              <div class="audio-player-subtitle">{{ subtitle }}</div>
              <div v-if="loading || error" class="audio-player-status" :class="{error: Boolean(error)}">
                {{ error || "正在加载音频..." }}
              </div>
            </div>
          </div>

          <div class="audio-player-controls">
            <button class="mode-button" :title="playModeTitle" @click="togglePlayMode">
              <icon :icon="playModeIcon" color="currentColor" />
              <span>{{ playModeLabel }}</span>
            </button>
            <button class="track-button" title="上一首" :disabled="!canShowPrevious" @click="showPreviousTrack">
              <icon icon="action.previous" color="currentColor" />
            </button>
            <button class="play-button" :title="playTitle" @click="togglePlay">
              <icon :icon="isPlaying ? 'action.pause' : 'action.play'" color="currentColor" />
            </button>
            <button class="track-button" title="下一首" :disabled="!canShowNext" @click="showNextTrack">
              <icon icon="action.next" color="currentColor" />
            </button>
            <button
                class="icon-action playlist-action"
                :class="{active: playlistVisible}"
                :title="playlistTitle"
                :aria-expanded="playlistVisible"
                aria-controls="audio-player-playlist"
                @click="togglePlaylist">
              <icon icon="view.playlist" color="currentColor" />
            </button>
          </div>

          <div class="audio-player-tools">
            <audio-volume-control
                :muted="muted"
                :volume="volume"
                :volume-style="volumeStyle"
                :volume-title="volumeTitle"
                :mute-title="muteTitle"
                :audible-volume-percent="audibleVolumePercent"
                :variant="playerSize === 'large' ? 'inline' : 'floating'"
                @toggle-mute="toggleMute"
                @update-volume="updateVolume" />
            <button class="icon-action download-action" title="下载" @click="downloadCurrent">
              <icon icon="action.download" color="currentColor" />
            </button>
            <button class="icon-action size-action" :title="playerSizeTitle" @click="cyclePlayerSize">
              <icon :icon="playerSizeIcon" color="currentColor" />
            </button>
            <button class="icon-action close-action" title="关闭播放器" @click="close">
              <icon icon="action.close" color="currentColor" />
            </button>
          </div>

          <div class="audio-player-mini-tools" aria-label="mini 播放器工具">
            <button
                class="icon-action playlist-action"
                :class="{active: playlistVisible}"
                :title="playlistTitle"
                :aria-expanded="playlistVisible"
                aria-controls="audio-player-playlist"
                @click="togglePlaylist">
              <icon icon="view.playlist" color="currentColor" />
            </button>
            <audio-volume-control
                :muted="muted"
                :volume="volume"
                :volume-style="volumeStyle"
                :volume-title="volumeTitle"
                :mute-title="muteTitle"
                :audible-volume-percent="audibleVolumePercent"
                variant="floating"
                @toggle-mute="toggleMute"
                @update-volume="updateVolume" />
            <button class="icon-action size-action" :title="playerSizeTitle" @click="cyclePlayerSize">
              <icon :icon="playerSizeIcon" color="currentColor" />
            </button>
            <button class="icon-action close-action" title="关闭播放器" @click="close">
              <icon icon="action.close" color="currentColor" />
            </button>
          </div>
        </div>

        <div
            class="audio-player-progress"
            :class="{hovering: progressHovering}"
            :style="progressStyle"
            @pointerenter="showProgressHover"
            @pointermove="updateProgressHover"
            @pointerleave="hideProgressHover"
            @pointerdown="startProgressSeek"
            @pointerup="finishProgressSeek"
            @pointercancel="finishProgressSeek"
            @lostpointercapture="finishProgressSeek">
          <span class="audio-player-progress-track" aria-hidden="true"></span>
          <input
              type="range"
              min="0"
              :max="progressMax"
              step="0.1"
              :value="currentTime"
              aria-label="播放进度"
              @input="updateProgress">
          <span class="audio-player-time-badge">{{ progressHoverTimeText }}</span>
        </div>
      </section>
    </Transition>
    <Transition name="audio-playlist">
      <section
          v-if="currentEntry && playlistVisible"
          id="audio-player-playlist"
          ref="playlistRef"
          class="audio-playlist"
          :style="playlistStyle"
          aria-label="播放列表">
        <div class="audio-playlist-header">
          <div>
            <strong>播放列表</strong>
            <span>{{ playlistCountText }}</span>
          </div>
          <button title="关闭播放列表" @click="playlistVisible = false">
            <icon icon="action.close" color="currentColor" />
          </button>
        </div>
        <div class="audio-playlist-list">
          <button
              v-for="(item, index) in playlistEntries"
              :key="item.path"
              class="audio-playlist-item"
              :class="{active: item.path === currentEntry.path}"
              :title="item.name"
              @click="selectTrack(item)">
            <span class="audio-playlist-index">
              <icon v-if="item.path === currentEntry.path && isPlaying" icon="view.audio" color="currentColor" />
              <template v-else>{{ index + 1 }}</template>
            </span>
            <span class="audio-playlist-name">{{ item.name }}</span>
            <span class="audio-playlist-meta">{{ formatEntrySize(item.size, "0 B") }}</span>
          </button>
        </div>
      </section>
    </Transition>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.audio-player {
  @apply fixed z-[60] flex flex-col overflow-visible rounded-xl border px-0 py-0 shadow-2xl backdrop-blur-xl;
  min-height: 4.75rem;
  width: min(62rem, calc(100vw - 2rem));
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 92%, transparent);
  color: var(--app-text);
  transition: width 160ms ease, min-height 160ms ease, border-radius 160ms ease, padding 160ms ease, opacity 160ms ease, transform 160ms ease;
}

.audio-player.size-small {
  @apply gap-0 rounded-full px-0;
  min-height: 3.5rem;
  width: min(24rem, calc(100vw - 1.5rem));
}

.audio-player.size-mini {
  @apply gap-0 rounded-full border-0 bg-transparent px-0 py-0 shadow-none backdrop-blur-none;
  min-height: 3.5rem;
  width: 3.5rem;
}

.audio-player.dragging {
  @apply select-none;
  transition: none;
  cursor: grabbing;
}

.audio-player-enter-active,
.audio-player-leave-active {
  transition: opacity 160ms ease, transform 160ms ease;
}

.audio-player-enter-from,
.audio-player-leave-to {
  opacity: 0;
  transform: translateY(0.75rem) scale(0.98);
}

.audio-player-progress {
  @apply absolute left-0 right-0 bottom-0 h-5 cursor-pointer rounded-b-xl text-[0.68rem] tabular-nums;
  color: var(--app-text-subtle);
}

.audio-player-progress-track {
  @apply absolute bottom-0 left-1.5 right-1.5 cursor-pointer overflow-hidden rounded-full;
  height: 0.1875rem;
  background: linear-gradient(
      to right,
      var(--app-accent, #2563eb) 0%,
      var(--app-accent, #2563eb) var(--audio-progress, 0%),
      color-mix(in srgb, var(--app-accent, #2563eb) 28%, var(--app-control-solid)) var(--audio-progress, 0%),
      color-mix(in srgb, var(--app-accent, #2563eb) 28%, var(--app-control-solid)) var(--audio-buffered, 0%),
      color-mix(in srgb, var(--app-text-subtle) 6%, var(--app-border-soft)) var(--audio-buffered, 0%),
      color-mix(in srgb, var(--app-text-subtle) 6%, var(--app-border-soft)) 100%
  );
  box-shadow: inset 0 1px 0 color-mix(in srgb, var(--app-panel-solid) 38%, transparent);
  transition: height 140ms ease, border-radius 140ms ease, background 140ms ease, opacity 140ms ease;
}

.audio-player-progress:hover,
.audio-player-progress:focus-within,
.audio-player-progress.hovering {
  @apply h-7;
}

.audio-player-progress:hover .audio-player-progress-track,
.audio-player-progress:focus-within .audio-player-progress-track,
.audio-player-progress.hovering .audio-player-progress-track {
  @apply left-0 right-0 rounded-full;
  height: 0.5625rem;
  background: linear-gradient(
      to right,
      var(--app-accent, #2563eb) 0%,
      var(--app-accent, #2563eb) var(--audio-progress, 0%),
      color-mix(in srgb, var(--app-accent, #2563eb) 36%, var(--app-control-solid)) var(--audio-progress, 0%),
      color-mix(in srgb, var(--app-accent, #2563eb) 36%, var(--app-control-solid)) var(--audio-buffered, 0%),
      color-mix(in srgb, var(--app-text-subtle) 10%, var(--app-border-soft)) var(--audio-buffered, 0%),
      color-mix(in srgb, var(--app-text-subtle) 10%, var(--app-border-soft)) 100%
  );
}

.audio-player-progress::after {
  content: "";
  @apply pointer-events-none absolute bottom-[-0.15625rem] z-10 h-2 w-2 rounded-full opacity-0;
  left: clamp(0rem, calc(var(--audio-progress-hover-x, 0px) - 0.25rem), calc(100% - 0.5rem));
  background: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-panel-solid), 0 2px 8px rgba(15, 23, 42, 0.22);
  transition: opacity 120ms ease, bottom 140ms ease, height 140ms ease, width 140ms ease, left 40ms linear;
}

.audio-player-progress:hover::after,
.audio-player-progress:focus-within::after,
.audio-player-progress.hovering::after {
  @apply bottom-0 h-3 w-3 opacity-100;
  left: clamp(0rem, calc(var(--audio-progress-hover-x, 0px) - 0.375rem), calc(100% - 0.75rem));
}

.audio-player .audio-player-progress input[type="range"] {
  @apply absolute inset-x-0 bottom-0 h-px w-full px-0 opacity-0;
  display: block;
  pointer-events: none;
}

.audio-player-time-badge {
  @apply pointer-events-none absolute bottom-4 z-20 rounded-full px-2 py-0.5 text-[0.68rem] font-semibold opacity-0;
  left: clamp(1.625rem, var(--audio-progress-hover-x, 0px), calc(100% - 1.625rem));
  background: color-mix(in srgb, var(--app-control-solid) 86%, transparent);
  color: var(--app-text);
  box-shadow: 0 1px 6px rgba(15, 23, 42, 0.12);
  transition: opacity 120ms ease, transform 120ms ease;
  transform: translate(-50%, 0.25rem);
}

.audio-player-progress:hover .audio-player-time-badge,
.audio-player-progress:focus-within .audio-player-time-badge,
.audio-player-progress.hovering .audio-player-time-badge {
  opacity: 1;
  transform: translate(-50%, 0);
}

.audio-player-body {
  @apply relative grid w-full min-w-0 content-center items-center gap-3 px-4 py-2;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  min-height: inherit;
}

.audio-player.size-small .audio-player-body {
  @apply gap-2 px-2 py-1;
  grid-template-columns: minmax(0, 1fr) auto;
}

.audio-player.size-mini .audio-player-body {
  @apply block h-14 w-14 p-0;
  min-height: 3.5rem;
}

.audio-player-track {
  @apply flex min-w-0 items-center gap-3 pr-32;
  grid-column: 1;
}

.audio-player-art {
  @apply flex h-12 w-12 shrink-0 cursor-move touch-none select-none items-center justify-center rounded-lg text-xl outline-none;
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 72%, var(--app-control-solid));
  color: var(--app-accent, #2563eb);
}

.audio-player-art:hover,
.audio-player-art:focus-visible {
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe), 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.audio-player.dragging .audio-player-art {
  cursor: grabbing;
}

.audio-player-art.playing :deep(.icon) {
  animation: audioPulse 1.8s ease-in-out infinite;
}

.audio-player.size-small .audio-player-art {
  @apply h-10 w-10 rounded-full text-lg;
}

.audio-player.size-mini .audio-player-art {
  @apply relative h-14 w-14 rounded-full text-xl shadow-lg;
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 78%, var(--app-panel-solid));
  box-shadow: 0 12px 30px rgba(15, 23, 42, 0.2), inset 0 0 0 1px var(--app-border-soft);
  z-index: 2;
}

.audio-player.size-small .audio-player-track,
.audio-player.size-mini .audio-player-track {
  @apply pr-0;
}

.audio-player.size-mini .audio-player-track {
  @apply block;
}

.audio-player-main {
  @apply min-w-0 grow;
}

.audio-player-title-wrap {
  @apply relative min-w-0;
}

.audio-player-title-row {
  @apply flex min-w-0 items-baseline gap-2;
}

.audio-player-title-row strong {
  @apply relative block min-w-0 overflow-hidden whitespace-nowrap text-base font-semibold;
  max-width: 100%;
}

.audio-player.size-small .audio-player-title-row strong,
.audio-player.size-mini .audio-player-title-row strong {
  @apply text-sm;
}

.audio-player-title-text {
  @apply inline-block align-bottom;
  transform: translateX(0);
}

.audio-player.titleOverflow .audio-player-title-text {
  max-width: none;
  min-width: max-content;
  animation: audioTitleMarquee var(--audio-title-duration, 9s) linear infinite;
}

.audio-player-title-copy {
  padding-left: var(--audio-title-gap, 2rem);
}

.audio-player-compact-controls {
  @apply pointer-events-none absolute inset-y-[-0.4375rem] left-[-0.25rem] right-[-0.25rem] hidden items-center justify-center gap-1 rounded-full px-1 opacity-0 backdrop-blur-md;
  background: color-mix(in srgb, var(--app-panel-solid) 82%, transparent);
  box-shadow: inset 0 0 0 1px var(--app-border-soft);
  transition: opacity 140ms ease, transform 140ms ease;
  transform: scale(0.98);
}

.audio-player.size-small .audio-player-compact-controls {
  @apply flex;
}

.audio-player.size-small .audio-player-title-wrap:hover .audio-player-compact-controls,
.audio-player.size-small .audio-player-title-wrap:focus-within .audio-player-compact-controls {
  @apply pointer-events-auto opacity-100;
  transform: scale(1);
}

.audio-player-subtitle {
  @apply mt-0.5 block truncate text-xs;
  color: var(--app-text-subtle);
}

.audio-player.size-small .audio-player-subtitle,
.audio-player.size-mini .audio-player-subtitle {
  @apply hidden;
}

.audio-player.size-small .audio-player-progress,
.audio-player.size-mini .audio-player-progress,
.audio-player.size-mini .audio-player-status,
.audio-player.size-mini .audio-player-main {
  @apply hidden;
}

.audio-player-status {
  @apply mt-1 truncate text-xs;
  color: var(--app-text-subtle);
}

.audio-player-status.error {
  color: var(--app-danger);
}

.audio-player-controls,
.audio-player-tools {
  @apply flex shrink-0 items-center gap-1;
}

.audio-player-controls {
  @apply absolute left-1/2 top-1/2 z-10 justify-center;
  transform: translate(-50%, -50%);
}

.audio-player.size-small .audio-player-controls,
.audio-player.size-mini .audio-player-controls {
  @apply hidden;
}

.audio-player-tools {
  @apply min-w-0 justify-self-end pl-32;
  grid-column: 2;
}

.audio-player.size-small .audio-player-tools,
.audio-player.size-mini .audio-player-tools {
  @apply pl-0;
  grid-column: auto;
}

.audio-player.size-mini .audio-player-tools {
  @apply hidden;
}

.audio-player-mini-tools {
  @apply pointer-events-none absolute left-7 top-1/2 hidden w-[10.5rem] items-center justify-end gap-1 rounded-full border py-1 pr-2 pl-8 opacity-0 shadow-2xl backdrop-blur-xl;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 94%, transparent);
  transform: translate(-0.75rem, -50%) scale(0.98);
  transform-origin: left center;
  transition: opacity 140ms ease, transform 140ms ease;
}

.audio-player.size-mini .audio-player-mini-tools {
  @apply flex;
}

.audio-player.size-mini.mini-tools-left .audio-player-mini-tools {
  @apply right-7 left-auto justify-start pr-8 pl-2;
  transform: translate(0.75rem, -50%) scale(0.98);
  transform-origin: right center;
}

.audio-player.size-mini:hover .audio-player-mini-tools {
  @apply pointer-events-auto opacity-100;
  transform: translate(0, -50%) scale(1);
}

.audio-player button {
  @apply inline-flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md border border-transparent text-sm;
  color: var(--app-text-muted);
}

.audio-player.size-mini button {
  @apply h-7 w-7 rounded-full;
}

.audio-player-art-play {
  @apply pointer-events-auto absolute left-1/2 top-1/2 hidden h-10 w-10 items-center justify-center rounded-full border opacity-0;
  border-color: color-mix(in srgb, white 42%, transparent);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 82%, transparent);
  color: white;
  box-shadow: 0 8px 20px rgba(15, 23, 42, 0.22);
  transition: opacity 140ms ease, transform 140ms ease;
  transform: translate(-50%, -50%) scale(0.9);
}

.audio-player.size-mini .audio-player-art-play {
  @apply flex;
}

.audio-player.size-mini .audio-player-art-play:hover,
.audio-player.size-mini .audio-player-art-play:focus-visible {
  @apply opacity-100;
  transform: translate(-50%, -50%) scale(1);
}

.audio-player.size-mini .audio-player-art-play:hover,
.audio-player.size-mini .audio-player-art-play:focus-visible {
  border-color: color-mix(in srgb, white 58%, transparent);
  background: var(--app-accent, #2563eb);
  color: white;
}

.audio-player button:hover:not(:disabled) {
  border-color: var(--app-border-soft);
  background: var(--app-accent-hover, #eff6ff);
  color: var(--app-text);
}

.audio-player button.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.audio-player button:disabled {
  @apply cursor-not-allowed;
  color: var(--app-text-disabled);
}

.audio-player button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.audio-player .play-button {
  @apply h-11 w-11 rounded-full text-base;
  background: var(--app-accent, #2563eb);
  color: white;
}

.audio-player.size-small .play-button {
  @apply h-8 w-8;
}

.audio-player.size-mini .play-button {
  @apply h-8 w-8;
}

.audio-player .play-button:hover:not(:disabled),
.audio-player.size-small .audio-player-compact-controls .play-button:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 72%, white);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 86%, black);
  color: white;
}

.audio-player .mode-button {
  @apply w-8 gap-1.5 px-0 text-xs;
}

.audio-player.size-small .audio-player-compact-controls button {
  @apply h-7 w-7 rounded-full;
}

.audio-player.size-small .audio-player-compact-controls .play-button {
  @apply h-8 w-8;
}

.audio-player .mode-button span {
  @apply hidden whitespace-nowrap;
}

.audio-player.size-mini .mode-button,
.audio-player.size-mini .track-button,
.audio-player.size-mini .download-action {
  @apply hidden;
}

.audio-player.size-small .download-action {
  @apply hidden;
}

.audio-playlist {
  @apply fixed z-[59] w-[min(34rem,calc(100vw-2rem))] overflow-hidden rounded-xl border shadow-2xl backdrop-blur-xl;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 96%, transparent);
  color: var(--app-text);
  transform: translate(-50%, 0);
  transform-origin: bottom center;
}

.audio-playlist-enter-active,
.audio-playlist-leave-active {
  transition: opacity 140ms ease, transform 140ms ease;
}

.audio-playlist-enter-from,
.audio-playlist-leave-to {
  opacity: 0;
  transform: translate(-50%, 1.25rem) scale(0.98);
}

.audio-playlist-header {
  @apply flex h-11 items-center justify-between border-b px-3;
  border-color: var(--app-border-soft);
}

.audio-playlist-header div {
  @apply flex min-w-0 items-baseline gap-2;
}

.audio-playlist-header strong {
  @apply text-sm font-semibold;
}

.audio-playlist-header span {
  @apply text-xs;
  color: var(--app-text-subtle);
}

.audio-playlist-header button {
  @apply inline-flex h-7 w-7 items-center justify-center rounded-md border border-transparent text-sm;
  color: var(--app-text-muted);
}

.audio-playlist-header button:hover {
  border-color: var(--app-border-soft);
  background: var(--app-accent-hover, #eff6ff);
  color: var(--app-text);
}

.audio-playlist-header button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.audio-playlist-list {
  @apply max-h-72 overflow-y-auto py-1;
}

.audio-playlist-item {
  @apply grid h-9 w-full items-center gap-2 border-0 bg-transparent px-3 text-left text-sm;
  grid-template-columns: 2rem minmax(0, 1fr) auto;
  color: var(--app-text-muted);
}

.audio-playlist-item:hover {
  background: var(--app-accent-hover, #eff6ff);
  color: var(--app-text);
}

.audio-playlist-item:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.audio-playlist-item.active {
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 78%, transparent);
  color: var(--app-accent, #2563eb);
}

.audio-playlist-index {
  @apply flex min-w-0 items-center justify-center text-xs tabular-nums;
  color: var(--app-text-subtle);
}

.audio-playlist-item.active .audio-playlist-index {
  color: var(--app-accent, #2563eb);
}

.audio-playlist-name {
  @apply min-w-0 truncate;
}

.audio-playlist-meta {
  @apply text-xs tabular-nums;
  color: var(--app-text-subtle);
}

@keyframes audioPulse {
  0%,
  100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.08);
  }
}

@keyframes audioTitleMarquee {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(var(--audio-title-scroll, 0));
  }
}

@media (max-width: 760px) {
  .audio-player {
    @apply gap-2;
  }

  .audio-player .mode-button span {
    @apply hidden;
  }

  .audio-playlist {
    width: min(30rem, calc(100vw - 1rem));
  }
}

@media (max-width: 560px) {
  .audio-player {
    @apply min-h-[4rem] px-2;
  }

  .audio-player.size-mini {
    min-height: 3.25rem;
  }

  .audio-player .download-action,
  .audio-player.size-large .playlist-action {
    @apply hidden;
  }
}
</style>
