<script setup lang="ts">
import type {DirSortKey, DirSortOrder} from "../../class.ts";

type DetailsColumnKey = "name" | "modified" | "type" | "size";

const props = defineProps<{
  gridStyle: Record<string, string>;
  loading: boolean;
  sortKey: DirSortKey;
  sortOrder: DirSortOrder;
}>();

const emit = defineEmits<{
  (e: "change-sort", key: DirSortKey): void;
  (e: "resize-column", event: PointerEvent, key: DetailsColumnKey): void;
}>();

const sortButtonClass = (key: DirSortKey) => ({
  active: props.sortKey === key,
  desc: props.sortKey === key && props.sortOrder === "desc"
});

const sortIndicator = (key: DirSortKey) => {
  if (props.sortKey !== key) return "";
  return props.sortOrder === "asc" ? "↑" : "↓";
}
</script>

<template>
  <div class="details-header" :style="gridStyle">
    <button class="sort-button name-cell" :class="sortButtonClass('name')" :disabled="loading" @click.stop="emit('change-sort', 'name')">
      <span>名称</span>
      <span class="sort-indicator">{{ sortIndicator('name') }}</span>
      <span class="column-resizer" title="拖拽调整名称列宽" @click.stop @pointerdown="emit('resize-column', $event, 'name')"></span>
    </button>
    <button class="sort-button" :class="sortButtonClass('modified')" :disabled="loading" @click.stop="emit('change-sort', 'modified')">
      <span>修改日期</span>
      <span class="sort-indicator">{{ sortIndicator('modified') }}</span>
      <span class="column-resizer" title="拖拽调整修改日期列宽" @click.stop @pointerdown="emit('resize-column', $event, 'modified')"></span>
    </button>
    <span class="header-cell">
      类型
      <span class="column-resizer" title="拖拽调整类型列宽" @click.stop @pointerdown="emit('resize-column', $event, 'type')"></span>
    </span>
    <button class="sort-button size-cell" :class="sortButtonClass('size')" :disabled="loading" @click.stop="emit('change-sort', 'size')">
      <span>大小</span>
      <span class="sort-indicator">{{ sortIndicator('size') }}</span>
      <span class="column-resizer" title="拖拽调整大小列宽" @click.stop @pointerdown="emit('resize-column', $event, 'size')"></span>
    </button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.details-header {
  @apply sticky top-0 z-10 grid h-9 items-center border-b border-slate-200 bg-white px-4 text-sm text-slate-600;
  grid-template-columns: minmax(var(--details-name-width), 1fr) var(--details-modified-width) var(--details-type-width) var(--details-size-width);
  min-width: calc(var(--details-name-width) + var(--details-modified-width) + var(--details-type-width) + var(--details-size-width) + 2rem);
}

.details-header > .header-cell {
  @apply relative flex h-full items-center truncate px-2;
}

.sort-button {
  @apply relative flex h-full min-w-0 items-center justify-between gap-1 truncate px-2 text-left text-sm text-slate-600 hover:bg-blue-50 disabled:pointer-events-none;
}

.sort-button.active {
  @apply bg-blue-50 text-blue-700;
}

.sort-button span:first-child {
  @apply min-w-0 truncate;
}

.sort-button.size-cell {
  @apply text-right;
}

.sort-indicator {
  @apply inline-flex w-3 shrink-0 justify-center text-[11px] text-blue-600;
}

.column-resizer {
  @apply absolute -right-1 top-0 z-20 h-full w-2 cursor-col-resize touch-none;
}

.column-resizer::after {
  content: "";
  @apply absolute left-1 top-1/2 h-5 -translate-y-1/2 border-l border-slate-200;
}

.column-resizer:hover::after {
  @apply border-blue-500;
}
</style>
