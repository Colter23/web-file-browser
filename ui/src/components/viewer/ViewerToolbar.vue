<script setup lang="ts">
import type {FileEntryIconKind} from "../../utils/file-entry.ts";
import FileTypeIcon from "../FileTypeIcon.vue";

withDefaults(defineProps<{
  kind: FileEntryIconKind;
  name: string;
  extension?: string;
  subtitle?: string;
  iconTone?: "image" | "video" | "pdf" | "default";
}>(), {
  subtitle: "",
  iconTone: "default"
});
</script>

<template>
  <div class="viewer-toolbar">
    <div class="viewer-title">
      <span class="viewer-title-icon" :class="`tone-${iconTone}`">
        <file-type-icon :kind="kind" :name="name" :extension="extension" size="1.15rem" />
      </span>
      <span class="viewer-title-text">
        <strong>{{ name }}</strong>
        <span>{{ subtitle }}</span>
      </span>
    </div>
    <div class="viewer-actions">
      <slot />
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.viewer-toolbar {
  @apply flex min-h-14 shrink-0 items-center justify-between gap-3 border-b px-3.5 backdrop-blur;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.12));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 6%, rgba(15, 23, 42, 0.74));
}

.viewer-title {
  @apply flex min-w-0 items-center gap-2.5;
}

.viewer-title-icon {
  @apply grid h-8 w-8 shrink-0 place-items-center rounded-md border border-white/10 bg-white/10 text-blue-200 shadow-sm;
}

.viewer-title-icon.tone-image {
  @apply text-teal-200;
}

.viewer-title-icon.tone-video {
  @apply text-pink-200;
}

.viewer-title-icon.tone-pdf {
  @apply text-blue-200;
}

.viewer-title-text {
  @apply flex min-w-0 flex-col;
}

.viewer-title strong {
  @apply truncate text-sm font-semibold leading-5;
}

.viewer-title-text > span {
  @apply truncate text-xs leading-4 text-slate-300;
}

.viewer-actions {
  @apply flex shrink-0 items-center gap-1.5 text-xs text-slate-100;
}

@media (max-width: 840px) {
  .viewer-toolbar {
    @apply items-start;
  }

  .viewer-actions {
    @apply flex-wrap justify-end;
  }
}
</style>
