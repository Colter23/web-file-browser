<script setup lang="ts">
import type {FileEntryMetaRow} from "../../utils/file-entry.ts";

defineProps<{
  items: FileEntryMetaRow[];
}>();
</script>

<template>
  <div class="preview-meta-list">
    <div v-for="item in items" :key="item.label" :title="item.value">
      <span>{{ item.label }}</span>
      <strong>{{ item.value }}</strong>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-meta-list {
  @apply grid shrink-0 grid-cols-2 gap-x-3 gap-y-1 border-b px-3 py-2 text-xs;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.preview-meta-list div {
  @apply min-w-0;
}

.preview-meta-list span {
  @apply mr-1;
  color: var(--app-text-subtle);
}

.preview-meta-list strong {
  @apply inline-block max-w-full truncate align-bottom font-normal;
  color: var(--app-text-muted);
}

.preview-meta-list div:last-child {
  @apply col-span-2;
}
</style>
