<script setup lang="ts">
defineProps<{
  folderStatusText: string;
  selectedStatusText: string;
}>();
</script>

<template>
  <div class="explorer-status-row">
    <span :title="folderStatusText">{{ folderStatusText }}</span>
    <span class="selection" :title="selectedStatusText">{{ selectedStatusText }}</span>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.explorer-status-row {
  @apply flex h-8 shrink-0 items-center justify-between gap-4 border-t px-3 text-xs;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
  color: var(--app-text-subtle);
}

.explorer-status-row span {
  @apply min-w-0 truncate;
}

.explorer-status-row .selection {
  @apply shrink-0 text-right;
  color: var(--app-text-muted);
  max-width: min(32rem, 58%);
}
</style>
