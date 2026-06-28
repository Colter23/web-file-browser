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
  @apply flex min-h-9 shrink-0 items-center gap-1.5 border-b px-2.5 py-1 text-xs;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 78%, var(--app-panel-muted));
  color: var(--app-text-subtle);
}

.preview-tool-actions {
  @apply flex min-w-0 shrink-0 items-center gap-1;
}

:slotted(button) {
  @apply inline-flex h-7 shrink-0 items-center gap-1.5 rounded-md border border-transparent px-2 text-xs font-medium transition disabled:cursor-not-allowed disabled:hover:border-transparent disabled:hover:bg-transparent;
  color: var(--app-text-muted);
}

:slotted(button:disabled) {
  color: var(--app-text-disabled);
}

:slotted(button:hover:not(:disabled)) {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
  color: var(--app-text);
}

:slotted(button:focus-visible) {
  outline: none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

:slotted(button.active) {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.preview-tool-status {
  @apply ml-auto min-w-0 truncate rounded-full px-2 py-0.5 tabular-nums;
  background: color-mix(in srgb, var(--app-control-solid) 72%, transparent);
  color: var(--app-text-subtle);
}
</style>
