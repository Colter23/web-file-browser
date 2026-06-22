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
  @apply flex h-8 shrink-0 items-center justify-between gap-4 border-t border-slate-200 bg-slate-50 px-3 text-xs text-slate-500;
}

.explorer-status-row span {
  @apply min-w-0 truncate;
}

.explorer-status-row .selection {
  @apply shrink-0 text-right text-slate-600;
  max-width: min(32rem, 58%);
}
</style>
