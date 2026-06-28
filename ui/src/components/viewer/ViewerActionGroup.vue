<template>
  <div class="viewer-action-group">
    <slot />
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.viewer-action-group {
  @apply inline-flex h-9 items-center overflow-hidden rounded-lg border p-0.5 shadow-sm;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.14));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 3%, rgba(255, 255, 255, 0.1));
}

.viewer-action-group :deep(button) {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-transparent bg-transparent px-2 text-sm font-medium text-white transition hover:bg-white/20;
}

.viewer-action-group :deep(button:disabled) {
  @apply cursor-not-allowed opacity-35 hover:bg-transparent;
}

.viewer-action-group :deep(button:focus-visible) {
  @apply outline-none;
  border-color: rgba(255, 255, 255, 0.78);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-accent, #2563eb) 45%, rgba(255, 255, 255, 0.25));
}

.viewer-action-group :deep(button.active) {
  @apply text-white;
  border-color: color-mix(in srgb, var(--app-accent-border, #bfdbfe) 72%, transparent);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 36%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--app-accent-border, #bfdbfe) 22%, transparent);
}

.viewer-action-group :deep(button.zoom-step) {
  @apply min-w-8 px-1;
}

.viewer-action-group :deep(button.zoom-mode) {
  @apply min-w-16 px-2.5 text-xs;
}

.viewer-action-group :deep(button.zoom-mode.fit),
.viewer-action-group :deep(button.zoom-mode.actual) {
  color: #fff;
}

.viewer-action-group :deep(button.rotation-state) {
  @apply min-w-11 cursor-default px-2 text-xs tabular-nums opacity-80;
}

.viewer-action-group :deep(button.rotation-state:disabled) {
  @apply opacity-80;
}
</style>
