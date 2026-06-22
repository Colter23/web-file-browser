<script setup lang="ts">
import type {PreviewMetaItem} from "./types.ts";

defineProps<{
  items: PreviewMetaItem[];
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
  @apply grid shrink-0 grid-cols-2 gap-x-3 gap-y-1 border-b border-slate-100 bg-slate-50/70 px-3 py-2 text-xs;
}

.preview-meta-list div {
  @apply min-w-0;
}

.preview-meta-list span {
  @apply mr-1 text-slate-400;
}

.preview-meta-list strong {
  @apply inline-block max-w-full truncate align-bottom font-normal text-slate-700;
}

.preview-meta-list div:last-child {
  @apply col-span-2;
}
</style>
