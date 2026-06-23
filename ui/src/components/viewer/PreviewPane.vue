<script setup lang="ts">
import {computed} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {downloadUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";
import type {ShellNoticePayload} from "../shell/types.ts";
import {entryMetaRows, entryPreviewKind, entryPreviewTypeText, isEditableEntry} from "../../utils/file-entry.ts";
import PreviewHeader from "./PreviewHeader.vue";
import PreviewImageView from "./PreviewImageView.vue";
import PreviewMetaList from "./PreviewMetaList.vue";
import PreviewTextView from "./PreviewTextView.vue";
import type {PreviewKind} from "./types.ts";

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

const previewKind = computed<PreviewKind>(() => entryPreviewKind(props.entry, props.editableExtensions));
const previewTypeText = computed(() => entryPreviewTypeText(previewKind.value));

const previewTitleText = computed(() => props.entry?.name ?? "预览窗格");
const previewSubtitleText = computed(() => props.entry ? previewTypeText.value : "选择一个文件");
const emptyTitleText = computed(() => props.emptyTitle || "选择一个文件以预览");
const emptySubtitleText = computed(() => props.emptySubtitle || "");
const emptyIconName = computed(() => props.emptyIcon || "file.file");

const canEditPreview = computed(() => {
  return isEditableEntry(props.entry, props.editableExtensions);
});

const previewMeta = computed(() => {
  const entry = props.entry;
  if (!entry) return [];
  return entryMetaRows(entry, {typeText: previewTypeText.value, includePath: true});
});

const editPreview = () => {
  if (props.entry && canEditPreview.value) emit("edit", props.entry);
}

const downloadPreview = () => {
  if (props.entry) emit("download", props.entry);
}
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
  <preview-image-view v-if="entry && previewKind === 'image'" :entry="entry" @open-image="emit('open-image', $event)" />
  <preview-text-view v-else-if="entry && previewKind === 'text'" :entry="entry" :reload-key="reloadKey" @notice="emit('notice', $event)" />
  <div v-else class="preview-body" :class="previewKind">
    <div v-if="!entry" class="preview-placeholder muted">
      <icon :icon="emptyIconName" size="3rem" />
      <span>{{ emptyTitleText }}</span>
      <small v-if="emptySubtitleText">{{ emptySubtitleText }}</small>
    </div>
    <audio v-else-if="entry && previewKind === 'audio'" :src="downloadUrl(entry.path)" controls></audio>
    <video v-else-if="entry && previewKind === 'video'" :src="downloadUrl(entry.path)" controls></video>
    <div v-else class="preview-placeholder">
      <icon icon="file.file" size="3rem" />
      <span>暂不支持预览此类型</span>
      <button @click="downloadPreview">下载文件</button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm text-slate-700;
}

.preview-body.audio,
.preview-body.video {
  @apply bg-slate-50;
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
  @apply rounded-md border border-slate-200 bg-white px-3 py-1.5 text-sm text-slate-700;
}

.preview-placeholder button:hover {
  background: var(--app-accent-hover, #eff6ff);
}
</style>
