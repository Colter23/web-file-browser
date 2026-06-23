<script setup lang="ts">
defineProps<{
  status?: string;
}>();
</script>

<template>
  <div class="preview-tool-row">
    <div class="preview-tool-actions">
      <slot></slot>
    </div>
    <span v-if="$slots.status || status" class="preview-tool-status">
      <slot name="status">{{ status }}</slot>
    </span>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-tool-row {
  @apply flex h-9 shrink-0 items-center gap-2 border-b border-slate-100 bg-white px-3 text-xs text-slate-500;
}

.preview-tool-actions {
  @apply flex min-w-0 items-center gap-1;
}

:slotted(button) {
  @apply inline-flex h-6 items-center gap-1 rounded border border-transparent px-2 text-slate-600 hover:border-slate-200 hover:bg-blue-50 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:border-transparent disabled:hover:bg-transparent;
}

:slotted(button.active) {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.preview-tool-status {
  @apply ml-auto shrink-0 tabular-nums;
}
</style>
