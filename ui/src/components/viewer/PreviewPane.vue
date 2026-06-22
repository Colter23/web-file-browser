<script setup lang="ts">
import {computed, onBeforeUnmount, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {useImageZoomPan} from "../../composables/useImageZoomPan.ts";
import {downloadUrl, getFile} from "../../network/api.ts";
import Icon from "../Icon.vue";
import type {ShellNoticePayload} from "../shell/types.ts";
import {formatEntryDate, formatEntrySize, isImageEntry, isTextLikeEntry} from "../../utils/file-entry.ts";
import PreviewHeader from "./PreviewHeader.vue";
import PreviewMetaList from "./PreviewMetaList.vue";
import type {PreviewKind, PreviewMetaItem} from "./types.ts";

const audioPreviewExtensions = ["mp3", "wav", "ogg", "flac", "m4a", "aac"];
const videoPreviewExtensions = ["mp4", "webm", "mov", "mkv", "avi"];

const props = defineProps<{
  entry: ExplorerEntry | null;
  editableExtensions: string[];
  reloadKey: number;
  emptyTitle?: string;
  emptySubtitle?: string;
  emptyIcon?: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "edit", entry: ExplorerEntry): void;
  (e: "download", entry: ExplorerEntry): void;
  (e: "open-image", entry: ExplorerEntry): void;
  (e: "notice", payload: ShellNoticePayload): void;
}>();

const previewLoading = ref(false);
const previewText = ref("");
const previewError = ref("");
const previewTextWrap = ref(true);
const previewCopied = ref(false);
let previewLoadVersion = 0;
let previewCopyTimer: number | undefined;

const normalizedExtension = computed(() => props.entry?.extension?.toLowerCase() ?? "");

const previewKind = computed<PreviewKind>(() => {
  const entry = props.entry;
  if (!entry || entry.type !== "file") return "unknown";
  const extension = normalizedExtension.value;
  if (isImageEntry(entry)) return "image";
  if (audioPreviewExtensions.includes(extension)) return "audio";
  if (videoPreviewExtensions.includes(extension)) return "video";
  if (isTextLikeEntry(entry, props.editableExtensions)) return "text";
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
const emptyTitleText = computed(() => props.emptyTitle || "选择一个文件以预览");
const emptySubtitleText = computed(() => props.emptySubtitle || "");
const emptyIconName = computed(() => props.emptyIcon || "icon-file-fill");

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

const {
  fit: previewImageFit,
  dragging: previewImageDragging,
  imageStyle: previewImageStyle,
  zoomText: previewZoomText,
  canPan: canPanPreviewImage,
  resetZoom: resetPreviewImageZoom,
  releasePointer: releasePreviewImagePointer,
  zoomImage: zoomPreviewImage,
  handleWheel: handlePreviewImageWheel,
  startPan: startPreviewImagePan,
  movePan: movePreviewImagePan,
  stopPan: stopPreviewImagePan
} = useImageZoomPan({maxZoom: 300, canPan: () => previewKind.value === "image"});

const previewMeta = computed<PreviewMetaItem[]>(() => {
  const entry = props.entry;
  if (!entry) return [];
  return [
    {label: "类型", value: previewTypeText.value},
    {label: "大小", value: formatEntrySize(entry.size)},
    {label: "修改", value: formatEntryDate(entry.modified)},
    {label: "路径", value: entry.path}
  ];
});

const resetPreviewRuntime = () => {
  previewLoadVersion += 1;
  previewLoading.value = false;
  previewText.value = "";
  previewError.value = "";
  resetPreviewImageZoom();
  previewCopied.value = false;
}

const loadPreview = async (entry: ExplorerEntry) => {
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
  <preview-header
      :title="previewTitleText"
      :subtitle="previewSubtitleText"
      :can-edit="canEditPreview"
      :can-download="Boolean(entry)"
      @edit="editPreview"
      @download="downloadPreview"
      @close="emit('close')" />
  <preview-meta-list v-if="entry" :items="previewMeta" />
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
      <icon :icon="emptyIconName" size="3rem" />
      <span>{{ emptyTitleText }}</span>
      <small v-if="emptySubtitleText">{{ emptySubtitleText }}</small>
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
        @lostpointercapture="releasePreviewImagePointer"
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

.preview-placeholder small {
  @apply max-w-56 px-4 text-xs leading-5 text-slate-400;
}

.preview-placeholder button {
  @apply rounded-md border border-slate-200 bg-white px-3 py-1.5 text-sm text-slate-700 hover:bg-blue-50;
}
</style>
