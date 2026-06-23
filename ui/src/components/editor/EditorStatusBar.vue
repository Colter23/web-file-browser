<script setup lang="ts">
defineProps<{
  messageText: string;
  filePathText: string;
  conflict: boolean;
  cursorText: string;
  selectionText: string;
  modeText: string;
  sizeText: string;
  wrapText: string;
}>();

const emit = defineEmits<{
  (e: "reload"): void;
}>();
</script>

<template>
  <footer class="editor-statusbar">
    <div class="status-left">
      <span v-if="messageText" :class="['editor-message', {conflict}]">{{ messageText }}</span>
      <span v-else>{{ filePathText }}</span>
    </div>
    <div class="status-right">
      <button v-if="conflict" class="status-action" @click="emit('reload')">重新载入</button>
      <span>{{ cursorText }}</span>
      <span v-if="selectionText">{{ selectionText }}</span>
      <span>{{ modeText }}</span>
      <span>{{ sizeText }}</span>
      <span>{{ wrapText }}</span>
    </div>
  </footer>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-statusbar {
  @apply relative z-20 flex h-7 shrink-0 items-center justify-between gap-3 border-t px-3 text-xs backdrop-blur;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 90%, transparent);
  color: var(--app-text-subtle);
}

.status-left,
.status-right {
  @apply flex min-w-0 items-center gap-3;
}

.status-left span,
.status-right span {
  @apply truncate;
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
  @apply h-5 shrink-0 rounded border px-2 text-xs;
  border-color: var(--app-danger-border);
  background: var(--app-control-solid);
  color: var(--app-danger);
}

.status-action:hover {
  background: var(--app-danger-soft);
}
</style>
