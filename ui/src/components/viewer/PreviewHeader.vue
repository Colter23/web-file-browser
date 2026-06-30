<script setup lang="ts">
import Icon from "../Icon.vue";
import {useI18n} from "../../i18n";

defineProps<{
  title: string;
  subtitle: string;
  canDownload: boolean;
}>();

const emit = defineEmits<{
  (e: "download"): void;
  (e: "close"): void;
}>();

const {t} = useI18n();
</script>

<template>
  <div class="preview-header">
    <div class="preview-heading">
      <div class="preview-icon">
        <slot name="icon">
          <icon icon="view.preview-pane" />
        </slot>
      </div>
      <div class="preview-title-block">
        <span class="preview-title">{{ title }}</span>
        <span class="preview-subtitle">{{ subtitle }}</span>
      </div>
    </div>
    <div class="preview-actions">
      <button v-if="canDownload" :title="t('preview.download')" :aria-label="t('preview.download')" @click="emit('download')">
        <icon icon="action.download" />
      </button>
      <button :title="t('preview.close')" :aria-label="t('preview.close')" @click="emit('close')">
        <icon icon="action.close" />
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-header {
  @apply flex min-h-[3.25rem] shrink-0 items-center justify-between gap-2 border-b px-3 py-2 text-sm font-medium;
  border-color: var(--app-border-soft);
  background: linear-gradient(180deg, var(--app-panel-solid), var(--app-panel-muted));
  color: var(--app-text);
}

.preview-heading {
  @apply flex min-w-0 flex-1 items-center gap-2.5;
}

.preview-icon {
  @apply flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border text-xl;
  border-color: var(--app-border-soft);
  background: var(--app-control);
  color: var(--app-accent, #2563eb);
}

.preview-title-block {
  @apply flex min-w-0 flex-col;
}

.preview-title {
  @apply min-w-0 truncate text-[13px] leading-5;
}

.preview-subtitle {
  @apply truncate text-xs font-normal leading-4;
  color: var(--app-text-subtle);
}

.preview-actions {
  @apply flex shrink-0 items-center gap-1;
}

.preview-header button {
  @apply inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-transparent transition disabled:cursor-not-allowed disabled:opacity-40;
  background: transparent;
  color: var(--app-text-muted);
}

.preview-header button:hover:not(:disabled) {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
  color: var(--app-text);
  transform: translateY(-1px);
}

.preview-header button:active:not(:disabled) {
  transform: translateY(0);
}

.preview-header button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}
</style>
