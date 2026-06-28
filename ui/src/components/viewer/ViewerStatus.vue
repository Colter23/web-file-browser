<script setup lang="ts">
withDefaults(defineProps<{
  tone?: "default" | "error";
}>(), {
  tone: "default"
});
</script>

<template>
  <div class="viewer-status" :class="{error: tone === 'error'}">
    <slot />
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.viewer-status {
  @apply absolute left-1/2 top-1/2 rounded-lg border border-white/10 bg-slate-950/65 px-3 py-2 text-sm text-slate-100 shadow-xl backdrop-blur;
  transform: translate(-50%, -50%);
}

.viewer-status.error {
  @apply border-red-300/30 bg-red-950/70 text-red-100;
}
</style>
