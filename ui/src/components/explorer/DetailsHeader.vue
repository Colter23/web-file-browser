<script setup lang="ts">
import type {DirSortKey, DirSortOrder} from "../../class.ts";
import type {DetailsColumnKey} from "./types.ts";

const props = defineProps<{
  gridStyle: Record<string, string>;
  loading: boolean;
  sortKey: DirSortKey;
  sortOrder: DirSortOrder;
}>();

const emit = defineEmits<{
  (e: "change-sort", key: DirSortKey): void;
  (e: "resize-column", event: PointerEvent, key: DetailsColumnKey): void;
  (e: "fit-column", key: DetailsColumnKey): void;
}>();

const sortButtonClass = (key: DirSortKey) => ({
  active: props.sortKey === key,
  desc: props.sortKey === key && props.sortOrder === "desc"
});

const sortIndicator = (key: DirSortKey) => {
  if (props.sortKey !== key) return "";
  return props.sortOrder === "asc" ? "↑" : "↓";
}

const resizeTitle = (label: string) => `拖拽调整${label}列宽，双击自动适配`;
</script>

<template>
  <div class="details-header" :style="gridStyle">
    <button class="sort-button name-cell" :class="sortButtonClass('name')" :disabled="loading" @click.stop="emit('change-sort', 'name')">
      <span>名称</span>
      <span class="sort-indicator">{{ sortIndicator('name') }}</span>
      <span class="column-resizer" :title="resizeTitle('名称')" @click.stop @dblclick.prevent.stop="emit('fit-column', 'name')" @pointerdown="emit('resize-column', $event, 'name')"></span>
    </button>
    <button class="sort-button" :class="sortButtonClass('modified')" :disabled="loading" @click.stop="emit('change-sort', 'modified')">
      <span>修改日期</span>
      <span class="sort-indicator">{{ sortIndicator('modified') }}</span>
      <span class="column-resizer" :title="resizeTitle('修改日期')" @click.stop @dblclick.prevent.stop="emit('fit-column', 'modified')" @pointerdown="emit('resize-column', $event, 'modified')"></span>
    </button>
    <span class="header-cell">
      类型
      <span class="column-resizer" :title="resizeTitle('类型')" @click.stop @dblclick.prevent.stop="emit('fit-column', 'type')" @pointerdown="emit('resize-column', $event, 'type')"></span>
    </span>
    <button class="sort-button size-cell" :class="sortButtonClass('size')" :disabled="loading" @click.stop="emit('change-sort', 'size')">
      <span>大小</span>
      <span class="sort-indicator">{{ sortIndicator('size') }}</span>
      <span class="column-resizer" :title="resizeTitle('大小')" @click.stop @dblclick.prevent.stop="emit('fit-column', 'size')" @pointerdown="emit('resize-column', $event, 'size')"></span>
    </button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.details-header {
  @apply sticky top-0 z-10 grid h-9 items-center border-b border-slate-200 bg-white px-3 text-sm text-slate-600;
  grid-template-columns: var(--details-name-width) var(--details-modified-width) var(--details-type-width) var(--details-size-width);
  width: calc(var(--details-grid-width) + 1.5rem);
  min-width: calc(var(--details-grid-width) + 1.5rem);
}

.details-header > .header-cell {
  @apply relative flex h-full items-center truncate px-2;
}

.sort-button {
  @apply relative flex h-full min-w-0 items-center justify-between gap-1 truncate px-2 text-left text-sm text-slate-600 disabled:pointer-events-none;
}

.sort-button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.sort-button.active {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.sort-button span:first-child {
  @apply min-w-0 truncate;
}

.sort-button.size-cell {
  @apply text-right;
}

.sort-indicator {
  @apply inline-flex w-3 shrink-0 justify-center text-[11px];
  color: var(--app-accent, #2563eb);
}

.column-resizer {
  @apply absolute -right-1.5 top-0 z-20 h-full w-3 cursor-col-resize touch-none;
}

.column-resizer::after {
  content: "";
  @apply absolute left-1.5 top-1/2 h-5 -translate-y-1/2 border-l border-slate-200;
}

.column-resizer:hover::after {
  border-color: var(--app-accent, #2563eb);
}
</style>
