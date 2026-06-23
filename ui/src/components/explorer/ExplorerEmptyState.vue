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
  @apply flex h-48 flex-col items-center justify-center gap-1 text-center text-sm;
  color: var(--app-text-subtle);
}

.explorer-empty small {
  @apply max-w-md px-4 text-xs leading-5;
  color: var(--app-text-subtle);
}

.empty-action {
  @apply mt-2 h-8 rounded-md border px-3 text-xs font-medium shadow-sm;
  background: var(--app-control-solid);
  border-color: var(--app-accent-border, #bfdbfe);
  color: var(--app-accent, #2563eb);
}

.empty-action:hover {
  background: var(--app-accent-soft, #eff6ff);
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 34%, white);
}

.explorer-empty.error {
  @apply text-red-600;
}
</style>
