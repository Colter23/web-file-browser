<script setup lang="ts">
import {computed, onBeforeUnmount, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {getFile} from "../../network/api.ts";
import type {ShellNoticePayload} from "../shell/types.ts";
import PreviewToolRow from "./PreviewToolRow.vue";

const props = defineProps<{
  entry: ExplorerEntry;
  reloadKey: number;
}>();

const emit = defineEmits<{
  (e: "notice", payload: ShellNoticePayload): void;
}>();

const previewLoading = ref(false);
const previewText = ref("");
const previewError = ref("");
const previewTextWrap = ref(true);
const previewCopied = ref(false);
let previewLoadVersion = 0;
let previewCopyTimer: number | undefined;

const previewTextStats = computed(() => {
  const lines = previewText.value ? previewText.value.split(/\r\n|\r|\n/).length : 0;
  return `${lines} 行，${previewText.value.length} 字符`;
});

const resetCopyState = () => {
  previewCopied.value = false;
  if (previewCopyTimer) {
    window.clearTimeout(previewCopyTimer);
    previewCopyTimer = undefined;
  }
}

const resetPreviewRuntime = () => {
  previewLoadVersion += 1;
  previewLoading.value = false;
  previewText.value = "";
  previewError.value = "";
  resetCopyState();
}

const loadPreview = async () => {
  const version = ++previewLoadVersion;
  previewLoading.value = true;
  previewText.value = "";
  previewError.value = "";
  resetCopyState();
  try {
    const file = await getFile(props.entry.path);
    if (version !== previewLoadVersion) return;
    previewText.value = file.content;
  } catch (error) {
    if (version !== previewLoadVersion) return;
    previewError.value = error instanceof Error ? error.message : "预览失败";
  } finally {
    if (version === previewLoadVersion) previewLoading.value = false;
  }
}

const copyPreviewText = async () => {
  if (!previewText.value) return;
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

watch(() => [props.entry.path, props.reloadKey] as const, () => {
  resetPreviewRuntime();
  void loadPreview();
}, {immediate: true});

onBeforeUnmount(resetPreviewRuntime);
</script>

<template>
  <preview-tool-row>
    <button :class="{active: previewTextWrap}" @click="previewTextWrap = !previewTextWrap">
      {{ previewTextWrap ? "自动换行" : "不换行" }}
    </button>
    <button :disabled="!previewText" @click="copyPreviewText">{{ previewCopied ? "已复制" : "复制" }}</button>
    <template #status>{{ previewTextStats }}</template>
  </preview-tool-row>
  <div class="preview-body text">
    <div v-if="previewLoading" class="preview-placeholder">正在加载预览...</div>
    <div v-else-if="previewError" class="preview-placeholder error">{{ previewError }}</div>
    <pre v-else :class="{nowrap: !previewTextWrap}">{{ previewText }}</pre>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm;
  color: var(--app-text-muted);
}

.preview-body pre {
  @apply min-h-full whitespace-pre-wrap break-words p-3 font-mono text-xs leading-5;
  background: var(--app-panel-solid);
  color: var(--app-text);
}

.preview-body pre.nowrap {
  @apply whitespace-pre break-normal;
}

.preview-placeholder {
  @apply flex h-full min-h-48 flex-col items-center justify-center gap-3 text-center;
  color: var(--app-text-subtle);
}

.preview-placeholder.error {
  @apply text-red-600;
}
</style>
