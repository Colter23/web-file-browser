<script setup lang="ts">
import {computed, onBeforeUnmount, ref, watch} from "vue";
import type {FileInfo} from "../../class.ts";
import {downloadUrl, getFile} from "../../network/api.ts";
import Icon from "../Icon.vue";

type PreviewEntry = {
  type: "folder" | "file";
  name: string;
  path: string;
  modified?: string;
  size?: number;
  extension?: string;
  file?: FileInfo;
}

type PreviewKind = "image" | "text" | "audio" | "video" | "unknown";
type NoticeKind = "info" | "success" | "warning" | "error";

type NoticePayload = {
  kind: NoticeKind;
  title: string;
  message: string;
}

const textPreviewExtensions = ["txt", "log", "md", "json", "yaml", "yml", "toml", "xml", "csv"];
const imagePreviewExtensions = ["apng", "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "webp"];
const audioPreviewExtensions = ["mp3", "wav", "ogg", "flac", "m4a", "aac"];
const videoPreviewExtensions = ["mp4", "webm", "mov", "mkv", "avi"];

const props = defineProps<{
  entry: PreviewEntry | null;
  editableExtensions: string[];
  reloadKey: number;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "edit", entry: PreviewEntry): void;
  (e: "download", entry: PreviewEntry): void;
  (e: "open-image", entry: PreviewEntry): void;
  (e: "notice", payload: NoticePayload): void;
}>();

const previewLoading = ref(false);
const previewText = ref("");
const previewError = ref("");
const previewImageFit = ref(true);
const previewImageZoom = ref(100);
const previewImageOffsetX = ref(0);
const previewImageOffsetY = ref(0);
const previewImageDragging = ref(false);
const previewTextWrap = ref(true);
const previewCopied = ref(false);
let previewLoadVersion = 0;
let previewCopyTimer: number | undefined;
let previewImagePointerId: number | null = null;
let previewImageDragStartX = 0;
let previewImageDragStartY = 0;
let previewImageDragOriginX = 0;
let previewImageDragOriginY = 0;

const normalizedExtension = computed(() => props.entry?.extension?.toLowerCase() ?? "");

const previewKind = computed<PreviewKind>(() => {
  const entry = props.entry;
  if (!entry || entry.type !== "file") return "unknown";
  const extension = normalizedExtension.value;
  if (imagePreviewExtensions.includes(extension)) return "image";
  if (audioPreviewExtensions.includes(extension)) return "audio";
  if (videoPreviewExtensions.includes(extension)) return "video";
  if (props.editableExtensions.includes(extension) || textPreviewExtensions.includes(extension)) return "text";
  return "unknown";
});

const previewTypeText = computed(() => ({
  image: "图片",
  text: "文本",
  audio: "音频",
  video: "视频",
  unknown: "文件"
}[previewKind.value]));

const previewTitleText = computed(() => props.entry?.name ?? "预览窗格");
const previewSubtitleText = computed(() => props.entry ? previewTypeText.value : "选择一个文件");

const previewTextStats = computed(() => {
  if (previewKind.value !== "text") return "";
  const lines = previewText.value ? previewText.value.split(/\r\n|\r|\n/).length : 0;
  return `${lines} 行，${previewText.value.length} 字符`;
});

const canEditPreview = computed(() => {
  const entry = props.entry;
  if (!entry || entry.type !== "file") return false;
  return props.editableExtensions.includes(normalizedExtension.value);
});

const previewImageStyle = computed(() => ({
  maxWidth: previewImageFit.value ? "100%" : "none",
  maxHeight: previewImageFit.value ? "100%" : "none",
  transform: previewImageFit.value ? "none" : `translate3d(${previewImageOffsetX.value}px, ${previewImageOffsetY.value}px, 0) scale(${previewImageZoom.value / 100})`,
  transformOrigin: "center center"
}));

const previewZoomText = computed(() => previewImageFit.value ? "适应" : `${previewImageZoom.value}%`);
const canPanPreviewImage = computed(() => previewKind.value === "image" && !previewImageFit.value);

const previewMeta = computed(() => {
  const entry = props.entry;
  if (!entry) return [];
  return [
    {label: "类型", value: previewTypeText.value},
    {label: "大小", value: formatBytes(entry.size)},
    {label: "修改", value: formatDate(entry.modified)},
    {label: "路径", value: entry.path}
  ];
});

const formatBytes = (bytes?: number) => {
  if (bytes === undefined || Number.isNaN(bytes)) return "-";
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(value >= 10 ? 1 : 2)} ${units[unitIndex]}`;
}

const formatDate = (srcDate?: string) => {
  if (!srcDate) return "-";
  const date = new Date(srcDate);
  if (Number.isNaN(date.getTime())) return srcDate;
  return date.toLocaleString("zh-CN", {hour12: false});
}

const resetPreviewImagePan = () => {
  previewImageOffsetX.value = 0;
  previewImageOffsetY.value = 0;
  previewImageDragging.value = false;
  previewImagePointerId = null;
}

const resetPreviewRuntime = () => {
  previewLoadVersion += 1;
  previewLoading.value = false;
  previewText.value = "";
  previewError.value = "";
  previewImageFit.value = true;
  previewImageZoom.value = 100;
  resetPreviewImagePan();
  previewCopied.value = false;
}

const loadPreview = async (entry: PreviewEntry) => {
  const version = ++previewLoadVersion;
  previewLoading.value = false;
  previewText.value = "";
  previewError.value = "";
  previewCopied.value = false;
  if (previewKind.value !== "text") return;
  previewLoading.value = true;
  try {
    const file = await getFile(entry.path);
    if (version !== previewLoadVersion) return;
    previewText.value = file.content;
  } catch (error) {
    if (version !== previewLoadVersion) return;
    previewError.value = error instanceof Error ? error.message : "预览失败";
  } finally {
    if (version === previewLoadVersion) previewLoading.value = false;
  }
}

const editPreview = () => {
  if (props.entry && canEditPreview.value) emit("edit", props.entry);
}

const downloadPreview = () => {
  if (props.entry) emit("download", props.entry);
}

const openImagePreview = () => {
  if (props.entry && previewKind.value === "image") emit("open-image", props.entry);
}

const copyPreviewText = async () => {
  if (previewKind.value !== "text" || !previewText.value) return;
  try {
    await navigator.clipboard.writeText(previewText.value);
    previewCopied.value = true;
    if (previewCopyTimer) window.clearTimeout(previewCopyTimer);
    previewCopyTimer = window.setTimeout(() => {
      previewCopied.value = false;
    }, 1500);
  } catch {
    emit("notice", {kind: "error", title: "复制失败", message: "复制失败，请手动选择文本复制"});
  }
}

const zoomPreviewImage = (delta: number) => {
  previewImageFit.value = false;
  previewImageZoom.value = Math.min(300, Math.max(25, previewImageZoom.value + delta));
}

const handlePreviewImageWheel = (event: WheelEvent) => {
  event.preventDefault();
  zoomPreviewImage(event.deltaY < 0 ? 25 : -25);
}

const resetPreviewImageZoom = () => {
  previewImageFit.value = true;
  previewImageZoom.value = 100;
  resetPreviewImagePan();
}

const startPreviewImagePan = (event: PointerEvent) => {
  if (!canPanPreviewImage.value || event.button !== 0) return;
  event.preventDefault();
  const stage = event.currentTarget as HTMLElement;
  previewImagePointerId = event.pointerId;
  previewImageDragging.value = true;
  previewImageDragStartX = event.clientX;
  previewImageDragStartY = event.clientY;
  previewImageDragOriginX = previewImageOffsetX.value;
  previewImageDragOriginY = previewImageOffsetY.value;
  stage.setPointerCapture?.(event.pointerId);
}

const movePreviewImagePan = (event: PointerEvent) => {
  if (!previewImageDragging.value || previewImagePointerId !== event.pointerId) return;
  event.preventDefault();
  previewImageOffsetX.value = previewImageDragOriginX + event.clientX - previewImageDragStartX;
  previewImageOffsetY.value = previewImageDragOriginY + event.clientY - previewImageDragStartY;
}

const stopPreviewImagePan = (event: PointerEvent) => {
  if (previewImagePointerId !== event.pointerId) return;
  const stage = event.currentTarget as HTMLElement;
  stage.releasePointerCapture?.(event.pointerId);
  previewImageDragging.value = false;
  previewImagePointerId = null;
}

watch(() => [props.entry?.path, props.reloadKey], () => {
  resetPreviewRuntime();
  if (props.entry) void loadPreview(props.entry);
}, {immediate: true});

onBeforeUnmount(() => {
  if (previewCopyTimer) window.clearTimeout(previewCopyTimer);
  resetPreviewRuntime();
});
</script>

<template>
  <div class="preview-header">
    <div class="preview-title-block">
      <span class="preview-title">{{ previewTitleText }}</span>
      <span class="preview-subtitle">{{ previewSubtitleText }}</span>
    </div>
    <div class="preview-actions">
      <button v-if="canEditPreview" title="编辑" @click="editPreview">
        <icon icon="icon-edit-filling" />
      </button>
      <button title="下载" :disabled="!entry" @click="downloadPreview">
        <icon icon="icon-download" />
      </button>
      <button title="关闭预览" @click="emit('close')">
        <icon icon="icon-close" />
      </button>
    </div>
  </div>
  <div v-if="entry" class="preview-meta-list">
    <div v-for="item in previewMeta" :key="item.label" :title="item.value">
      <span>{{ item.label }}</span>
      <strong>{{ item.value }}</strong>
    </div>
  </div>
  <div v-if="previewKind === 'image'" class="preview-tool-row">
    <button :class="{active: previewImageFit}" @click="resetPreviewImageZoom">适应</button>
    <button @click="zoomPreviewImage(-25)">-</button>
    <span>{{ previewZoomText }}</span>
    <button @click="zoomPreviewImage(25)">+</button>
    <button title="打开图片查看" @click="openImagePreview">
      <icon icon="icon-unfold" color="currentColor" />
      <span>打开查看</span>
    </button>
  </div>
  <div v-else-if="previewKind === 'text'" class="preview-tool-row">
    <button :class="{active: previewTextWrap}" @click="previewTextWrap = !previewTextWrap">
      {{ previewTextWrap ? "自动换行" : "不换行" }}
    </button>
    <button :disabled="!previewText" @click="copyPreviewText">{{ previewCopied ? "已复制" : "复制" }}</button>
    <span>{{ previewTextStats }}</span>
  </div>
  <div class="preview-body" :class="previewKind">
    <div v-if="!entry" class="preview-placeholder muted">
      <icon icon="icon-file-fill" size="3rem" />
      <span>选择一个文件以预览</span>
    </div>
    <div v-else-if="previewLoading" class="preview-placeholder">正在加载预览...</div>
    <div v-else-if="previewError" class="preview-placeholder error">{{ previewError }}</div>
    <div
        v-else-if="entry && previewKind === 'image'"
        class="image-stage"
        :class="{fit: previewImageFit, panning: canPanPreviewImage, dragging: previewImageDragging}"
        @pointerdown="startPreviewImagePan"
        @pointermove="movePreviewImagePan"
        @pointerup="stopPreviewImagePan"
        @pointercancel="stopPreviewImagePan"
        @lostpointercapture="previewImageDragging = false"
        @wheel="handlePreviewImageWheel"
        @dblclick="openImagePreview">
      <img :src="downloadUrl(entry.path)" :alt="entry.name" :style="previewImageStyle">
    </div>
    <pre v-else-if="previewKind === 'text'" :class="{nowrap: !previewTextWrap}">{{ previewText }}</pre>
    <audio v-else-if="entry && previewKind === 'audio'" :src="downloadUrl(entry.path)" controls></audio>
    <video v-else-if="entry && previewKind === 'video'" :src="downloadUrl(entry.path)" controls></video>
    <div v-else class="preview-placeholder">
      <icon icon="icon-file-fill" size="3rem" />
      <span>暂不支持预览此类型</span>
      <button @click="downloadPreview">下载文件</button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-header {
  @apply flex min-h-12 shrink-0 items-center justify-between gap-2 border-b border-slate-200 px-3 text-sm font-medium;
}

.preview-title-block {
  @apply flex min-w-0 flex-col;
}

.preview-title {
  @apply min-w-0 truncate;
}

.preview-subtitle {
  @apply text-xs font-normal text-slate-500;
}

.preview-actions {
  @apply flex shrink-0 items-center gap-1;
}

.preview-header button {
  @apply inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-700 hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-40;
}

.preview-meta-list {
  @apply grid shrink-0 grid-cols-2 gap-x-3 gap-y-1 border-b border-slate-100 bg-slate-50/70 px-3 py-2 text-xs;
}

.preview-meta-list div {
  @apply min-w-0;
}

.preview-meta-list span {
  @apply mr-1 text-slate-400;
}

.preview-meta-list strong {
  @apply inline-block max-w-full truncate align-bottom font-normal text-slate-700;
}

.preview-meta-list div:last-child {
  @apply col-span-2;
}

.preview-tool-row {
  @apply flex h-9 shrink-0 items-center gap-1 border-b border-slate-100 bg-white px-3 text-xs text-slate-500;
}

.preview-tool-row button {
  @apply inline-flex h-6 items-center gap-1 rounded border border-transparent px-2 text-slate-600 hover:border-slate-200 hover:bg-blue-50 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:border-transparent disabled:hover:bg-transparent;
}

.preview-tool-row button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.preview-tool-row > span {
  @apply ml-auto tabular-nums;
}

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm text-slate-700;
}

.preview-body.image,
.preview-body.audio,
.preview-body.video {
  @apply bg-slate-50;
}

.image-stage {
  @apply flex h-full min-h-0 w-full touch-none select-none items-center justify-center overflow-hidden p-3;
}

.image-stage.fit {
  @apply overflow-hidden;
}

.image-stage.panning {
  @apply cursor-grab;
}

.image-stage.dragging {
  @apply cursor-grabbing;
}

.image-stage img {
  @apply rounded object-contain shadow-sm;
  user-select: none;
  -webkit-user-drag: none;
}

.preview-body pre {
  @apply min-h-full whitespace-pre-wrap break-words bg-white p-3 font-mono text-xs leading-5 text-slate-800;
}

.preview-body pre.nowrap {
  @apply whitespace-pre break-normal;
}

.preview-body audio,
.preview-body video {
  @apply m-auto max-h-full max-w-full;
}

.preview-placeholder {
  @apply flex h-full min-h-48 flex-col items-center justify-center gap-3 text-center text-slate-500;
}

.preview-placeholder.error {
  @apply text-red-600;
}

.preview-placeholder.muted {
  @apply text-slate-400;
}

.preview-placeholder button {
  @apply rounded-md border border-slate-200 bg-white px-3 py-1.5 text-sm text-slate-700 hover:bg-blue-50;
}
</style>
