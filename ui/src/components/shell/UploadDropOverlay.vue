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
        <icon icon="icon-upload" />
      </div>
      <strong>{{ title }}</strong>
      <span>{{ subtitle }}</span>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.upload-drop-layer {
  @apply pointer-events-none absolute inset-0 z-20 flex items-center justify-center border-2 border-dashed border-blue-400 bg-blue-50/55 p-6 backdrop-blur-[1px];
}

.upload-drop-card {
  @apply flex min-w-72 flex-col items-center gap-2 rounded-lg border border-blue-200 bg-white px-8 py-6 text-center text-sm text-slate-500 shadow-2xl;
}

.upload-drop-card strong {
  @apply text-base font-semibold text-slate-900;
}

.upload-drop-icon {
  @apply flex h-12 w-12 items-center justify-center rounded-lg bg-blue-600 text-2xl text-white shadow-sm;
}
</style>
