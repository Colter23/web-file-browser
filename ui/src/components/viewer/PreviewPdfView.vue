<script setup lang="ts">
import {computed} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {fileContentUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";
import PreviewToolRow from "./PreviewToolRow.vue";

const props = defineProps<{
  entry: ExplorerEntry;
  reloadKey: number;
}>();

const previewUrl = computed(() => fileContentUrl(props.entry.path, {cacheKey: props.reloadKey}));

const openPdfPreview = () => {
  window.open(previewUrl.value, "_blank", "noopener,noreferrer");
}
</script>

<template>
  <preview-tool-row status="浏览器原生预览">
    <button title="在新窗口打开 PDF" @click="openPdfPreview">
      <icon icon="action.open-new-tab" color="currentColor" />
      <span>新窗口打开</span>
    </button>
  </preview-tool-row>
  <div class="preview-body pdf">
    <object class="pdf-frame" :data="previewUrl" type="application/pdf">
      <div class="preview-placeholder">
        <icon icon="file.pdf" size="3rem" />
        <span>当前浏览器无法内嵌预览 PDF</span>
        <button @click="openPdfPreview">新窗口打开</button>
      </div>
    </object>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-body {
  @apply min-h-0 grow overflow-hidden text-sm;
  color: var(--app-text-muted);
}

.preview-body.pdf {
  @apply p-2;
  background: var(--app-panel-muted);
}

.pdf-frame {
  @apply block h-full min-h-0 w-full rounded-md border shadow-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

.preview-placeholder {
  @apply flex h-full min-h-48 flex-col items-center justify-center gap-3 text-center;
  color: var(--app-text-subtle);
}

.preview-placeholder button {
  @apply rounded-md border px-3 py-1.5 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.preview-placeholder button:hover {
  background: var(--app-accent-hover, #eff6ff);
}
</style>
