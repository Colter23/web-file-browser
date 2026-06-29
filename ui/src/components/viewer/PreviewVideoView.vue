<script setup lang="ts">
import type {ExplorerEntry} from "../explorer/types.ts";
import {useI18n} from "../../i18n";
import {fileContentUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";
import PreviewToolRow from "./PreviewToolRow.vue";

const props = defineProps<{
  entry: ExplorerEntry;
}>();

const emit = defineEmits<{
  (e: "open-video", entry: ExplorerEntry): void;
}>();

const {t} = useI18n();
const openVideoPreview = () => emit("open-video", props.entry);
</script>

<template>
  <preview-tool-row>
    <button :title="t('preview.openVideoPlayer')" @click="openVideoPreview">
      <icon icon="view.video" color="currentColor" />
      <span>{{ t("preview.openPlayer") }}</span>
    </button>
  </preview-tool-row>
  <div class="preview-body video">
    <video
        :src="fileContentUrl(entry.path)"
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
