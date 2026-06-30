<script setup lang="ts">
import {useI18n} from "../../i18n";

defineProps<{
  messageText: string;
  filePathText: string;
  conflict: boolean;
  dirty: boolean;
  saving: boolean;
  editMode: boolean;
  autoSave: boolean;
  fontSize: number;
  tabSize: number;
  cursorText: string;
  selectionText: string;
  modeText: string;
  sizeText: string;
  wrapText: string;
}>();

const emit = defineEmits<{
  (e: "reload"): void;
}>();

const {t} = useI18n();
</script>

<template>
  <footer class="editor-statusbar">
    <div class="status-left">
      <span v-if="messageText" :class="['editor-message', {conflict}]">{{ messageText }}</span>
      <span v-else class="status-path" :title="filePathText">{{ filePathText }}</span>
    </div>
    <div class="status-right">
      <button v-if="conflict" class="status-action" @click="emit('reload')">{{ t("editor.statusReload") }}</button>
      <span v-if="saving" class="status-pill active">{{ t("editor.saving") }}</span>
      <span v-else-if="dirty" class="status-pill dirty">{{ t("editor.unsaved") }}</span>
      <span v-else-if="autoSave" class="status-pill">{{ t("editor.autoSave") }}</span>
      <span class="status-pill mode" :class="{editing: editMode}">{{ editMode ? t("editor.edit") : t("editor.view") }}</span>
      <span class="status-item cursor">{{ cursorText }}</span>
      <span v-if="selectionText" class="status-item selection">{{ selectionText }}</span>
      <span class="status-item">{{ modeText }}</span>
      <span class="status-item">{{ sizeText }}</span>
      <span class="status-item optional">{{ t("editor.fontStatus", {size: fontSize}) }}</span>
      <span class="status-item optional">{{ t("editor.tabStatus", {size: tabSize}) }}</span>
      <span class="status-item optional">{{ wrapText }}</span>
    </div>
  </footer>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-statusbar {
  @apply relative z-20 flex h-8 shrink-0 items-center justify-between gap-3 border-t px-3 text-xs backdrop-blur;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 90%, transparent);
  color: var(--app-text-subtle);
}

.status-left,
.status-right {
  @apply flex min-w-0 items-center;
}

.status-left {
  @apply grow;
}

.status-right {
  @apply shrink-0 justify-end gap-1.5;
}

.status-path,
.status-item,
.status-pill,
.editor-message {
  @apply truncate;
}

.status-path {
  @apply min-w-0;
}

.editor-message {
  @apply truncate rounded px-2 py-0.5;
  color: var(--app-danger);
}

.editor-message.conflict {
  background: var(--app-danger-soft);
  color: var(--app-danger);
}

.status-action {
  @apply h-5 shrink-0 rounded border px-2 text-xs font-medium;
  border-color: var(--app-danger-border);
  background: var(--app-control-solid);
  color: var(--app-danger);
}

.status-action:hover {
  background: var(--app-danger-soft);
}

.status-item,
.status-pill {
  @apply inline-flex h-5 shrink-0 items-center rounded px-1.5 leading-none;
}

.status-item {
  background: transparent;
  color: var(--app-text-subtle);
}

.status-pill {
  border: 1px solid var(--app-border-soft);
  background: color-mix(in srgb, var(--app-control-solid) 78%, transparent);
  color: var(--app-text-muted);
}

.status-pill.active {
  border-color: var(--app-accent-border);
  background: var(--app-accent-soft);
  color: var(--app-accent);
}

.status-pill.dirty {
  border-color: var(--app-warning-border);
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.status-pill.mode.editing {
  border-color: var(--app-accent-border);
  color: var(--app-accent);
}

.selection {
  color: var(--app-accent);
}

@media (max-width: 900px) {
  .optional {
    @apply hidden;
  }
}

@media (max-width: 720px) {
  .status-item:not(.cursor),
  .status-pill:not(.dirty):not(.active) {
    @apply hidden;
  }
}
</style>
