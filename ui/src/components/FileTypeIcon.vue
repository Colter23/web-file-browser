<script setup lang="ts">
import {computed} from "vue";
import type {AppIconStyle} from "../class.ts";
import {useAppearanceStore} from "../store/appearance.ts";
import type {FileEntryIconKind} from "../utils/file-entry.ts";
import AppIcon from "./AppIcon.vue";

const props = withDefaults(defineProps<{
  kind: FileEntryIconKind;
  name?: string;
  extension?: string;
  open?: boolean;
  size?: string | "large" | "small" | "normal";
}>(), {
  open: false,
  size: "normal"
});

const appearanceStore = useAppearanceStore();
const normalizedKind = computed<FileEntryIconKind>(() => props.open && props.kind === "folder" ? "folder-open" : props.kind);
const fileIconStyle = computed<AppIconStyle | undefined>(() => {
  return appearanceStore.fileIconStyle === "inherit" ? undefined : appearanceStore.fileIconStyle;
});
const normalizedFolderKey = computed(() => {
  const name = props.name?.trim().toLowerCase() ?? "";
  return name
      .replace(/^\.+/, "")
      .replace(/[.\s_]+/g, "-");
});
const normalizedExtension = computed(() => {
  const extension = props.extension?.trim().replace(/^\./, "").toLowerCase();
  if (extension) return extension;
  const name = props.name?.trim().toLowerCase() ?? "";
  const index = name.lastIndexOf(".");
  return index > 0 && index < name.length - 1 ? name.slice(index + 1) : "";
});
const normalizedNameKey = computed(() => {
  const name = props.name?.trim().toLowerCase() ?? "";
  if (!name) return "";
  if (name === "dockerfile") return "dockerfile";
  if (name === "nginx.conf") return "nginx";
  if (name.startsWith(".env")) return "env";
  return "";
});
const iconName = computed(() => {
  const baseName = `file.${normalizedKind.value}`;
  if (!fileIconStyle.value) return baseName;
  const specificKey = normalizedKind.value === "folder" || normalizedKind.value === "folder-open"
      ? normalizedFolderKey.value
      : normalizedNameKey.value || normalizedExtension.value;
  return specificKey ? `${baseName}.${specificKey}` : baseName;
});
const paletteClass = computed(() => `palette-${appearanceStore.fileIconPalette}`);
</script>

<template>
  <app-icon
      class="file-type-icon"
      :class="[`kind-${normalizedKind}`, paletteClass]"
      :icon="iconName"
      :size="size"
      :icon-style="fileIconStyle" />
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

.kind-home {
  --file-icon-color: var(--app-accent, #2563eb);
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
</style>
