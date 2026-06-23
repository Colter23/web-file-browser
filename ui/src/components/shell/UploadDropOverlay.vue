<script setup lang="ts">
import Icon from "../Icon.vue";

defineProps<{
  title: string;
  subtitle: string;
}>();
</script>

<template>
  <div class="upload-drop-layer">
    <div class="upload-drop-card">
      <div class="upload-drop-icon">
        <icon icon="action.upload" />
      </div>
      <strong>{{ title }}</strong>
      <span>{{ subtitle }}</span>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.upload-drop-layer {
  @apply pointer-events-none absolute inset-0 z-20 flex items-center justify-center border-2 border-dashed p-6 backdrop-blur-[1px];
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 58%, white);
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 72%, transparent);
}

.upload-drop-card {
  @apply flex min-w-72 flex-col items-center gap-2 rounded-lg border bg-white px-8 py-6 text-center text-sm text-slate-500 shadow-2xl;
  border-color: var(--app-accent-border, #bfdbfe);
}

.upload-drop-card strong {
  @apply text-base font-semibold text-slate-900;
}

.upload-drop-icon {
  @apply flex h-12 w-12 items-center justify-center rounded-lg text-2xl text-white shadow-sm;
  background: var(--app-accent, #2563eb);
}
</style>
