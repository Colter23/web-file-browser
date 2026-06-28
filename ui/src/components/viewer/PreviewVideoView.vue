<script setup lang="ts">
import type {ExplorerEntry} from "../explorer/types.ts";
import {downloadUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";
import PreviewToolRow from "./PreviewToolRow.vue";

const props = defineProps<{
  entry: ExplorerEntry;
}>();

const emit = defineEmits<{
  (e: "open-video", entry: ExplorerEntry): void;
}>();

const openVideoPreview = () => emit("open-video", props.entry);
</script>

<template>
  <preview-tool-row>
    <button title="打开视频播放器" @click="openVideoPreview">
      <icon icon="view.video" color="currentColor" />
      <span>打开播放器</span>
    </button>
  </preview-tool-row>
  <div class="preview-body video">
    <video
        :src="downloadUrl(entry.path)"
        controls
        preload="metadata"
        playsinline
        @dblclick="openVideoPreview">
    </video>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm;
  background: var(--app-panel-muted);
  color: var(--app-text-muted);
}

.preview-body.video {
  @apply flex items-center justify-center p-3;
}

.preview-body video {
  @apply max-h-full max-w-full rounded-lg border bg-black shadow-sm;
  border-color: var(--app-border-soft);
}
</style>
