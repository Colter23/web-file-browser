<script setup lang="ts">
import type {ComponentPublicInstance} from "vue";
import {computed} from "vue";
import type {ExplorerIconSize, ExplorerViewMode} from "../../class.ts";
import type {FileEntryIconKind} from "../../utils/file-entry.ts";
import type {ExplorerEntry} from "./types.ts";
import FileTypeIcon from "../FileTypeIcon.vue";

const props = withDefaults(defineProps<{
  entry: ExplorerEntry;
  entryId: string;
  viewMode: ExplorerViewMode;
  iconSize: ExplorerIconSize;
  gridStyle?: Record<string, string>;
  selected: boolean;
  focused: boolean;
  image: boolean;
  dimmed: boolean;
  dragging: boolean;
  dropTarget: boolean;
  renaming: boolean;
  renameDraft: string;
  renameSubmitting: boolean;
  thumbnailVisible: boolean;
  thumbnailSrc: string;
  iconKind: FileEntryIconKind;
  typeText: string;
  modifiedText: string;
  sizeText: string;
  tileMetaText: string;
  searchHighlightText?: string;
}>(), {
  searchHighlightText: ""
});

const emit = defineEmits<{
  (e: "select", event: MouseEvent): void;
  (e: "aux-click", event: MouseEvent): void;
  (e: "open"): void;
  (e: "drag-start", event: DragEvent): void;
  (e: "drag-end"): void;
  (e: "drag-over", event: DragEvent): void;
  (e: "drag-leave", event: DragEvent): void;
  (e: "drop", event: DragEvent): void;
  (e: "context-menu", event: MouseEvent): void;
  (e: "thumbnail-error"): void;
  (e: "rename-input-ref", element: Element | ComponentPublicInstance | null): void;
  (e: "update:renameDraft", value: string): void;
  (e: "commit-rename"): void;
  (e: "cancel-rename"): void;
}>();

const updateRenameDraft = (event: Event) => {
  const target = event.target;
  if (target instanceof HTMLInputElement) emit("update:renameDraft", target.value);
}

const highlightedNameSegments = computed(() => {
  const name = props.entry.name;
  const keyword = props.searchHighlightText.trim();
  if (!keyword) return [{text: name, matched: false}];

  const lowerName = name.toLocaleLowerCase();
  const lowerKeyword = keyword.toLocaleLowerCase();
  const segments: Array<{text: string; matched: boolean}> = [];
  let cursor = 0;
  let matchIndex = lowerName.indexOf(lowerKeyword);

  while (matchIndex >= 0) {
    if (matchIndex > cursor) segments.push({text: name.slice(cursor, matchIndex), matched: false});
    const nextCursor = matchIndex + keyword.length;
    segments.push({text: name.slice(matchIndex, nextCursor), matched: true});
    cursor = nextCursor;
    matchIndex = lowerName.indexOf(lowerKeyword, cursor);
  }

  if (cursor < name.length) segments.push({text: name.slice(cursor), matched: false});
  return segments.length ? segments : [{text: name, matched: false}];
});
</script>

<template>
  <article
      :id="entryId"
      class="entry-item"
      :class="[`view-${viewMode}`, `explorer-size-${iconSize}`, {selected, focused, image, dimmed, dragging, dropTarget}]"
      :style="viewMode === 'details' ? gridStyle : undefined"
      :title="entry.name"
      role="option"
      :aria-selected="selected"
      :tabindex="focused ? 0 : -1"
      :draggable="!renaming"
      @click.stop="emit('select', $event)"
      @auxclick.stop="emit('aux-click', $event)"
      @dblclick.stop="emit('open')"
      @dragstart.stop="emit('drag-start', $event)"
      @dragend="emit('drag-end')"
      @dragover="emit('drag-over', $event)"
      @dragleave="emit('drag-leave', $event)"
      @drop="emit('drop', $event)"
      @contextmenu.prevent.stop="emit('context-menu', $event)">
    <div class="entry-name-cell">
      <div class="entry-visual">
        <img
            v-if="thumbnailVisible"
            :src="thumbnailSrc"
            :alt="entry.name"
            draggable="false"
            loading="lazy"
            decoding="async"
            @error="emit('thumbnail-error')">
        <file-type-icon v-else :kind="iconKind" />
      </div>
      <div class="entry-main">
        <input
            v-if="renaming"
            :ref="element => emit('rename-input-ref', element)"
            :value="renameDraft"
            class="entry-rename-input"
            :disabled="renameSubmitting"
            @input="updateRenameDraft"
            @click.stop
            @mousedown.stop
            @dblclick.stop
            @keydown.enter.prevent="emit('commit-rename')"
            @keydown.esc.prevent.stop="emit('cancel-rename')"
            @blur="emit('commit-rename')">
        <span v-else class="entry-name">
          <span
              v-for="(segment, index) in highlightedNameSegments"
              :key="`${index}-${segment.text}`"
              :class="{match: segment.matched}">
            {{ segment.text }}
          </span>
        </span>
        <span v-if="viewMode !== 'details'" class="entry-meta">{{ typeText }}</span>
      </div>
    </div>
    <span v-if="viewMode === 'details'" class="entry-date">{{ modifiedText }}</span>
    <span v-if="viewMode === 'details'" class="entry-type">{{ typeText }}</span>
    <span v-if="viewMode === 'details'" class="entry-size">{{ sizeText }}</span>
    <span v-if="viewMode === 'tiles'" class="entry-tile-meta">{{ tileMetaText }}</span>
  </article>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.entry-item {
  @apply relative min-w-0 cursor-default rounded-md border border-transparent text-sm outline-none;
  color: var(--app-text);
}

.entry-item:hover {
  background: var(--app-accent-hover, #ebf3ff);
}

.entry-item.selected {
  border-color: var(--app-accent-border, #7aa7f8);
  background: var(--app-accent-selected, #cfe4ff);
  color: var(--app-text);
}

.entry-item.focused {
  box-shadow: inset 0 0 0 1px var(--app-accent, #2563eb);
}

.entry-item.dimmed {
  @apply opacity-45;
}

.entry-item.dragging {
  @apply opacity-50;
}

.entry-item.dropTarget {
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent-soft, #eff6ff);
  box-shadow: inset 0 0 0 2px var(--app-accent-border, #bfdbfe);
}

.entry-item.view-details {
  @apply grid h-8 items-center px-3;
  grid-template-columns: var(--details-name-width) var(--details-modified-width) var(--details-type-width) var(--details-size-width);
  width: calc(var(--details-grid-width) + 1.5rem);
  min-width: calc(var(--details-grid-width) + 1.5rem);
}

.entry-item.view-list {
  @apply flex h-8 items-center gap-2 px-2;
}

.entry-item.view-icons {
  @apply flex h-32 flex-col items-center justify-start gap-2 p-2 text-center;
}

.entry-item.view-icons.explorer-size-small {
  @apply h-24;
}

.entry-item.view-icons.explorer-size-large {
  @apply h-40;
}

.entry-item.view-tiles {
  @apply grid min-h-20 grid-cols-[3.5rem_minmax(0,1fr)] grid-rows-[auto_auto] items-center gap-x-3 gap-y-1 p-2;
}

.entry-name-cell {
  @apply flex min-w-0 items-center gap-2;
}

.entry-item.view-details .entry-name-cell {
  @apply min-w-0 px-2;
}

.entry-item.view-icons .entry-name-cell {
  @apply flex-col justify-start gap-2 text-center;
}

.entry-item.view-tiles .entry-name-cell {
  @apply contents;
}

.entry-visual {
  @apply inline-flex shrink-0 items-center justify-center overflow-hidden;
  color: var(--app-text-muted);
}

.entry-item.view-details .entry-visual,
.entry-item.view-list .entry-visual {
  @apply h-5 w-5 text-[1.15rem];
}

.entry-item.view-icons .entry-visual {
  @apply h-16 w-20 rounded border border-transparent text-[3rem];
  background: var(--app-panel-solid);
}

.entry-item.view-icons.explorer-size-small .entry-visual {
  @apply h-11 w-14 text-[2.25rem];
}

.entry-item.view-icons.explorer-size-large .entry-visual {
  @apply h-24 w-32 text-[4.25rem];
}

.entry-item.view-tiles .entry-visual {
  @apply row-span-2 h-14 w-14 rounded border text-[2rem];
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.entry-item.view-icons.image .entry-visual,
.entry-item.view-tiles.image .entry-visual {
  @apply shadow-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.entry-visual img {
  @apply h-full w-full rounded object-cover;
}

.entry-item.view-details .entry-visual img,
.entry-item.view-list .entry-visual img {
  @apply rounded-sm;
}

.entry-main {
  @apply flex min-w-0 items-center gap-2;
}

.entry-item.view-icons .entry-main {
  @apply flex-col gap-0;
}

.entry-item.view-tiles .entry-main {
  @apply flex-col items-start gap-0 self-end;
}

.entry-name {
  @apply block w-full min-w-0 max-w-full truncate;
}

.entry-name .match {
  @apply rounded-sm px-0.5;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 18%, transparent);
  color: var(--app-accent, #2563eb);
}

.entry-item.selected .entry-name .match {
  background: color-mix(in srgb, var(--app-panel-solid) 42%, transparent);
  color: var(--app-text);
}

.entry-rename-input {
  @apply h-6 min-w-0 rounded border px-1 text-sm outline-none;
  background: var(--app-control-solid);
  color: var(--app-text);
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.entry-item.view-details .entry-rename-input,
.entry-item.view-list .entry-rename-input,
.entry-item.view-tiles .entry-rename-input {
  @apply w-full;
}

.entry-item.view-icons .entry-rename-input {
  @apply w-full text-center;
}

.entry-item.view-icons .entry-name {
  @apply line-clamp-2 whitespace-normal break-all;
}

.entry-meta,
.entry-date,
.entry-type,
.entry-size,
.entry-tile-meta {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.entry-date,
.entry-type,
.entry-size {
  @apply px-2 text-sm;
}

.entry-size {
  @apply text-right tabular-nums;
}

.entry-item.selected .entry-meta,
.entry-item.selected .entry-date,
.entry-item.selected .entry-type,
.entry-item.selected .entry-size,
.entry-item.selected .entry-tile-meta {
  color: var(--app-text-muted);
}

.entry-tile-meta {
  @apply col-start-2 self-start;
}
</style>
