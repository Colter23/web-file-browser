<script setup lang="ts">
import Icon from "../Icon.vue";

defineProps<{
  title: string;
  subtitle: string;
  canEdit: boolean;
  canDownload: boolean;
}>();

const emit = defineEmits<{
  (e: "edit"): void;
  (e: "download"): void;
  (e: "close"): void;
}>();
</script>

<template>
  <div class="preview-header">
    <div class="preview-title-block">
      <span class="preview-title">{{ title }}</span>
      <span class="preview-subtitle">{{ subtitle }}</span>
    </div>
    <div class="preview-actions">
      <button v-if="canEdit" title="编辑" @click="emit('edit')">
        <icon icon="action.edit" />
      </button>
      <button title="下载" :disabled="!canDownload" @click="emit('download')">
        <icon icon="action.download" />
      </button>
      <button title="关闭预览" @click="emit('close')">
        <icon icon="action.close" />
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-header {
  @apply flex min-h-12 shrink-0 items-center justify-between gap-2 border-b px-3 text-sm font-medium;
  border-color: var(--app-border-soft);
  color: var(--app-text);
}

.preview-title-block {
  @apply flex min-w-0 flex-col;
}

.preview-title {
  @apply min-w-0 truncate;
}

.preview-subtitle {
  @apply text-xs font-normal;
  color: var(--app-text-subtle);
}

.preview-actions {
  @apply flex shrink-0 items-center gap-1;
}

.preview-header button {
  @apply inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-40;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.preview-header button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.preview-header button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}
</style>
