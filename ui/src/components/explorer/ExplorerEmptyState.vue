<script setup lang="ts">
defineProps<{
  loading: boolean;
  message: string;
  emptyText: string;
  emptyHintText: string;
  filterActive: boolean;
}>();

const emit = defineEmits<{
  (e: "clear-filter"): void;
}>();
</script>

<template>
  <div v-if="loading" class="explorer-empty">正在加载...</div>
  <div v-else-if="message" class="explorer-empty error">{{ message }}</div>
  <div v-else class="explorer-empty">
    <span>{{ emptyText }}</span>
    <small v-if="emptyHintText">{{ emptyHintText }}</small>
    <button v-if="filterActive" type="button" class="empty-action" @click.stop="emit('clear-filter')">清除筛选</button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.explorer-empty {
  @apply flex h-48 flex-col items-center justify-center gap-1 text-center text-sm text-slate-500;
}

.explorer-empty small {
  @apply max-w-md px-4 text-xs leading-5 text-slate-400;
}

.empty-action {
  @apply mt-2 h-8 rounded-md border border-blue-200 bg-white px-3 text-xs font-medium text-blue-700 shadow-sm hover:border-blue-300 hover:bg-blue-50;
}

.explorer-empty.error {
  @apply text-red-600;
}
</style>
