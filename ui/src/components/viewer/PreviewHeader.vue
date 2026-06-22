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
        <icon icon="icon-edit-filling" />
      </button>
      <button title="下载" :disabled="!canDownload" @click="emit('download')">
        <icon icon="icon-download" />
      </button>
      <button title="关闭预览" @click="emit('close')">
        <icon icon="icon-close" />
      </button>
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
</style>
