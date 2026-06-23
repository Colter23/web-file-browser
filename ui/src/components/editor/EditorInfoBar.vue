<script setup lang="ts">
defineProps<{
  dirtyText: string;
  metaText: string;
  modifiedText: string;
  dirty: boolean;
  saving: boolean;
  conflict: boolean;
}>();
</script>

<template>
  <div class="editor-infobar" @click.stop>
    <div class="editor-info-left">
      <span :class="['status-pill', {dirty, saving, conflict}]">{{ dirtyText }}</span>
      <span>{{ metaText }}</span>
    </div>
    <div class="editor-info-right">
      <span>修改时间：{{ modifiedText }}</span>
      <span>UTF-8</span>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-infobar {
  @apply relative z-20 flex h-9 shrink-0 items-center justify-between gap-3 border-b px-3 text-xs backdrop-blur;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-muted) 82%, transparent);
  color: var(--app-text-subtle);
}

.editor-info-left,
.editor-info-right {
  @apply flex min-w-0 items-center gap-3;
}

.editor-info-left span,
.editor-info-right span {
  @apply truncate;
}

.status-pill {
  @apply shrink-0 rounded px-2 py-0.5;
  background: var(--app-control);
  color: var(--app-text-muted);
}

.status-pill.dirty {
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.status-pill.saving {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.status-pill.conflict {
  background: var(--app-danger-soft);
  color: var(--app-danger);
}
</style>
