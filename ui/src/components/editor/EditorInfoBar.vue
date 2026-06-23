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
  @apply relative z-20 flex h-9 shrink-0 items-center justify-between gap-3 border-b border-slate-200 bg-slate-50/80 px-3 text-xs text-slate-500 backdrop-blur;
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
  @apply shrink-0 rounded bg-slate-100 px-2 py-0.5 text-slate-600;
}

.status-pill.dirty {
  @apply bg-amber-100 text-amber-700;
}

.status-pill.saving {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.status-pill.conflict {
  @apply bg-red-50 text-red-600;
}
</style>
