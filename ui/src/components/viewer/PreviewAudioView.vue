<script setup lang="ts">
import type {ExplorerEntry} from "../explorer/types.ts";
import {useI18n} from "../../i18n";
import {downloadUrl} from "../../network/api.ts";
import Icon from "../Icon.vue";
import PreviewToolRow from "./PreviewToolRow.vue";

const props = defineProps<{
  entry: ExplorerEntry;
}>();

const emit = defineEmits<{
  (e: "open-audio", entry: ExplorerEntry): void;
}>();

const {t} = useI18n();
const openAudioPlayer = () => emit("open-audio", props.entry);
</script>

<template>
  <preview-tool-row>
    <button :title="t('preview.playGlobalAudio')" @click="openAudioPlayer">
      <icon icon="view.audio" color="currentColor" />
      <span>{{ t("preview.play") }}</span>
    </button>
  </preview-tool-row>
  <div class="preview-body audio">
    <div class="audio-preview-card">
      <div class="audio-preview-icon">
        <icon icon="view.audio" color="currentColor" />
      </div>
      <strong :title="entry.name">{{ entry.name }}</strong>
      <audio
          :src="downloadUrl(entry.path)"
          controls
          preload="metadata"
          @dblclick="openAudioPlayer">
      </audio>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm;
  background: var(--app-panel-muted);
  color: var(--app-text-muted);
}

.preview-body.audio {
  @apply flex items-center justify-center p-3;
}

.audio-preview-card {
  @apply flex w-full max-w-[18rem] flex-col items-center gap-3 rounded-xl border p-4 text-center shadow-sm;
  border-color: var(--app-border-soft);
  background: linear-gradient(180deg, var(--app-panel-solid), var(--app-control));
}

.audio-preview-icon {
  @apply flex h-14 w-14 items-center justify-center rounded-xl border text-2xl;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 72%, var(--app-control-solid));
  color: var(--app-accent, #2563eb);
}

.audio-preview-card strong {
  @apply w-full truncate text-sm font-semibold;
  color: var(--app-text);
}

.audio-preview-card audio {
  @apply w-full;
}
</style>
