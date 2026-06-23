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
  @apply flex h-9 shrink-0 items-center gap-2 border-b px-3 text-xs;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  color: var(--app-text-subtle);
}

.preview-tool-actions {
  @apply flex min-w-0 items-center gap-1;
}

:slotted(button) {
  @apply inline-flex h-6 items-center gap-1 rounded border border-transparent px-2 disabled:cursor-not-allowed disabled:hover:border-transparent disabled:hover:bg-transparent;
  color: var(--app-text-muted);
}

:slotted(button:disabled) {
  color: var(--app-text-disabled);
}

:slotted(button:hover:not(:disabled)) {
  border-color: var(--app-border-soft);
  background: var(--app-accent-hover, #eff6ff);
}

:slotted(button.active) {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.preview-tool-status {
  @apply ml-auto shrink-0 tabular-nums;
}
</style>
