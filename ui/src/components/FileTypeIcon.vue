<script setup lang="ts">
import {computed} from "vue";
import {useAppearanceStore} from "../store/appearance.ts";
import type {FileEntryIconKind} from "../utils/file-entry.ts";
import AppIcon from "./AppIcon.vue";

const props = withDefaults(defineProps<{
  kind: FileEntryIconKind;
  open?: boolean;
  size?: string | "large" | "small" | "normal";
}>(), {
  open: false,
  size: "normal"
});

const appearanceStore = useAppearanceStore();
const normalizedKind = computed<FileEntryIconKind>(() => props.open && props.kind === "folder" ? "folder-open" : props.kind);
const iconName = computed(() => `file.${normalizedKind.value}`);
const paletteClass = computed(() => `palette-${appearanceStore.fileIconPalette}`);
</script>

<template>
  <app-icon
      class="file-type-icon"
      :class="[`kind-${normalizedKind}`, paletteClass]"
      :icon="iconName"
      :size="size" />
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.file-type-icon {
  color: var(--file-icon-color, #64748b);
}

.kind-folder,
.kind-folder-open {
  --file-icon-color: #d49220;
}

.kind-image {
  --file-icon-color: #0f9f8e;
}

.kind-text,
.kind-document,
.kind-pdf {
  --file-icon-color: #3b82f6;
}

.kind-code,
.kind-markup,
.kind-config {
  --file-icon-color: #6366f1;
}

.kind-archive,
.kind-package {
  --file-icon-color: #a855f7;
}

.kind-audio,
.kind-video {
  --file-icon-color: #ec4899;
}

.kind-spreadsheet {
  --file-icon-color: #16a34a;
}

.kind-presentation {
  --file-icon-color: #f97316;
}

.kind-executable,
.kind-shortcut {
  --file-icon-color: #475569;
}

.kind-database,
.kind-font {
  --file-icon-color: #0891b2;
}

.palette-accent {
  --file-icon-color: var(--app-accent, #2563eb);
}

.palette-accent.kind-folder,
.palette-accent.kind-folder-open {
  --file-icon-color: #d49220;
}
</style>
