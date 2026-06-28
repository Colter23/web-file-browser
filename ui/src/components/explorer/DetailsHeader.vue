<script setup lang="ts">
import type {DirSortKey, DirSortOrder} from "../../class.ts";
import {useI18n} from "../../i18n";
import Icon from "../Icon.vue";
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

const {t} = useI18n();

const sortButtonClass = (key: DirSortKey) => ({
  active: props.sortKey === key
});

const sortIndicatorIcon = (key: DirSortKey) => {
  if (props.sortKey !== key) return "";
  return props.sortOrder === "asc" ? "sort.asc" : "sort.desc";
}

const columnLabel = (key: DetailsColumnKey) => {
  if (key === "modified") return t("sort.modified");
  if (key === "type") return t("sort.type");
  if (key === "size") return t("sort.size");
  return t("sort.name");
}

const resizeTitle = (key: DetailsColumnKey) => t("details.resizeTitle", {label: columnLabel(key)});
</script>

<template>
  <div class="details-header" :style="gridStyle">
    <button class="sort-button name-cell" :class="sortButtonClass('name')" :disabled="loading" @click.stop="emit('change-sort', 'name')">
      <span>{{ columnLabel("name") }}</span>
      <span class="sort-indicator"><icon v-if="sortIndicatorIcon('name')" :icon="sortIndicatorIcon('name')" size="small" /></span>
      <span class="column-resizer" :title="resizeTitle('name')" @click.stop @dblclick.prevent.stop="emit('fit-column', 'name')" @pointerdown="emit('resize-column', $event, 'name')"></span>
    </button>
    <button class="sort-button" :class="sortButtonClass('modified')" :disabled="loading" @click.stop="emit('change-sort', 'modified')">
      <span>{{ columnLabel("modified") }}</span>
      <span class="sort-indicator"><icon v-if="sortIndicatorIcon('modified')" :icon="sortIndicatorIcon('modified')" size="small" /></span>
      <span class="column-resizer" :title="resizeTitle('modified')" @click.stop @dblclick.prevent.stop="emit('fit-column', 'modified')" @pointerdown="emit('resize-column', $event, 'modified')"></span>
    </button>
    <span class="header-cell">
      {{ columnLabel("type") }}
      <span class="column-resizer" :title="resizeTitle('type')" @click.stop @dblclick.prevent.stop="emit('fit-column', 'type')" @pointerdown="emit('resize-column', $event, 'type')"></span>
    </span>
    <button class="sort-button size-cell" :class="sortButtonClass('size')" :disabled="loading" @click.stop="emit('change-sort', 'size')">
      <span>{{ columnLabel("size") }}</span>
      <span class="sort-indicator"><icon v-if="sortIndicatorIcon('size')" :icon="sortIndicatorIcon('size')" size="small" /></span>
      <span class="column-resizer" :title="resizeTitle('size')" @click.stop @dblclick.prevent.stop="emit('fit-column', 'size')" @pointerdown="emit('resize-column', $event, 'size')"></span>
    </button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.details-header {
  @apply sticky top-0 z-10 grid h-9 items-center border-b px-3 text-sm;
  border-color: var(--app-border-soft);
  color: var(--app-text-subtle);
  grid-template-columns: var(--details-name-width) var(--details-modified-width) var(--details-type-width) var(--details-size-width);
  width: calc(var(--details-grid-width) + 1.5rem);
  min-width: calc(var(--details-grid-width) + 1.5rem);
  background: var(--app-panel-muted);
  box-shadow: 0 1px 0 color-mix(in srgb, var(--app-panel-solid) 72%, transparent);
}

.details-header > .header-cell {
  @apply relative flex h-full items-center truncate px-2 font-medium;
}

.sort-button {
  @apply relative flex h-full min-w-0 items-center justify-between gap-1 truncate px-2 text-left text-sm font-medium disabled:pointer-events-none;
  color: var(--app-text-subtle);
}

.sort-button:disabled {
  opacity: 0.62;
}

.sort-button:hover:not(:disabled) {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 7%, transparent);
}

.sort-button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent-soft, #eff6ff);
  box-shadow: inset 0 0 0 1px var(--app-accent, #2563eb);
}

.sort-button.active {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 10%, transparent);
  color: var(--app-accent, #2563eb);
}

.sort-button span:first-child {
  @apply min-w-0 truncate;
}

.sort-button.size-cell {
  @apply justify-end text-right;
}

.sort-indicator {
  @apply inline-flex w-4 shrink-0 justify-center text-[0.78rem];
  color: var(--app-accent, #2563eb);
}

.column-resizer {
  @apply absolute -right-1.5 top-0 z-20 h-full w-3 cursor-col-resize touch-none;
}

.column-resizer::after {
  content: "";
  @apply absolute left-1.5 top-1/2 h-4 -translate-y-1/2 border-l;
  border-color: var(--app-border-soft);
}

.column-resizer:hover::after,
.sort-button:hover .column-resizer::after,
.header-cell:hover .column-resizer::after {
  border-color: var(--app-accent, #2563eb);
}
</style>
