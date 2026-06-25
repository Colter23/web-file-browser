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
  (e: "name-click", event: MouseEvent): void;
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
  if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) emit("update:renameDraft", target.value);
}

const estimateTextUnits = (text: string) => {
  return Array.from(text || " ").reduce((total, char) => {
    const code = char.codePointAt(0) ?? 0;
    if (code >= 0x2e80 && code <= 0x9fff) return total + 2;
    if (code >= 0xac00 && code <= 0xd7af) return total + 2;
    if (code >= 0xff00 && code <= 0xffef) return total + 2;
    if ("ilI|.,;:'`!".includes(char)) return total + 0.45;
    if ("mwMW@#%&".includes(char)) return total + 1.35;
    return total + 1;
  }, 0);
}

const estimateRenameWidth = (text: string) => {
  const units = estimateTextUnits(text);
  return Math.max(2.5, Math.min(96, units + 1.25));
}

const estimateRenameRows = (text: string, maxLineUnits: number) => {
  const units = estimateTextUnits(text);
  const safeLineUnits = Math.max(maxLineUnits - 0.5, 1);
  return Math.max(1, Math.min(6, Math.ceil(units / safeLineUnits)));
}

const estimateFloatingRename = (text: string, mode: ExplorerViewMode, iconSize: ExplorerIconSize) => {
  const units = estimateTextUnits(text);
  const maxWidth = mode === "tiles"
      ? iconSize === "large" ? 22 : 20
      : iconSize === "large" ? 18 : iconSize === "small" ? 12 : 14;
  const minWidth = mode === "tiles" ? 10 : 8;
  const width = Math.min(Math.max(units + 1.5, minWidth), maxWidth);
  const rows = estimateRenameRows(text, width);
  return {width, rows};
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

const renameControlStyle = computed(() => {
  if (!props.renaming) return undefined;
  const text = props.renameDraft || props.entry.name;
  if (props.viewMode === "details" || props.viewMode === "list") {
    return {"--rename-input-width": `${estimateRenameWidth(text)}ch`};
  }
  const floating = estimateFloatingRename(text, props.viewMode, props.iconSize);
  return {
    "--rename-input-width": `${floating.width}ch`
  };
});

const renameTextareaRows = computed(() => {
  if (!props.renaming || (props.viewMode !== "icons" && props.viewMode !== "tiles")) return 1;
  const text = props.renameDraft || props.entry.name;
  return estimateFloatingRename(text, props.viewMode, props.iconSize).rows;
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
      :data-renaming="renaming ? 'true' : undefined"
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
        <textarea
            v-if="renaming && (viewMode === 'icons' || viewMode === 'tiles')"
            :ref="element => emit('rename-input-ref', element)"
            :value="renameDraft"
            class="entry-rename-input entry-rename-textarea entry-rename-floating"
            :style="renameControlStyle"
            :disabled="renameSubmitting"
            :rows="renameTextareaRows"
            spellcheck="false"
            @input="updateRenameDraft"
            @click.stop
            @mousedown.stop
            @dblclick.stop
            @keydown.enter.prevent="emit('commit-rename')"
            @keydown.esc.prevent.stop="emit('cancel-rename')"
            @blur="emit('commit-rename')"></textarea>
        <input
            v-else-if="renaming"
            :ref="element => emit('rename-input-ref', element)"
            :value="renameDraft"
            class="entry-rename-input"
            :style="renameControlStyle"
            :disabled="renameSubmitting"
            autocomplete="off"
            spellcheck="false"
            @input="updateRenameDraft"
            @click.stop
            @mousedown.stop
            @dblclick.stop
            @keydown.enter.prevent="emit('commit-rename')"
            @keydown.esc.prevent.stop="emit('cancel-rename')"
            @blur="emit('commit-rename')">
        <span v-else class="entry-name" @click.stop="emit('name-click', $event)">
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
  @apply flex h-32 flex-col items-center justify-start gap-1.5 px-2 pb-3 pt-2 text-center;
}

.entry-item.view-icons[data-renaming="true"] {
  @apply z-20;
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
  @apply flex min-w-0 flex-1 items-center gap-2;
}

.entry-item.view-icons .entry-main {
  @apply w-full flex-none flex-col items-center gap-1;
}

.entry-item.view-tiles .entry-main {
  @apply flex-col items-start gap-1 self-end;
}

.entry-item.view-details .entry-main,
.entry-item.view-list .entry-main {
  @apply min-w-0 flex-1;
}

.entry-name {
  @apply block min-w-0 max-w-full truncate;
}

.entry-item.view-list .entry-name {
  @apply w-auto;
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
  @apply min-w-0 rounded-sm border px-1 text-sm leading-5 outline-none;
  background: var(--app-control-solid);
  color: var(--app-text);
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 1px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
  user-select: text;
}

.entry-item.view-details .entry-rename-input,
.entry-item.view-list .entry-rename-input {
  @apply h-6 w-full px-2.5;
}

.entry-item.view-details .entry-rename-input {
  width: min(var(--rename-input-width, 100%), 100%);
  min-width: 4ch;
  max-width: 100%;
}

@supports (field-sizing: content) {
  .entry-item.view-details .entry-rename-input {
    width: auto;
    field-sizing: content;
  }
}

.entry-item.view-tiles .entry-rename-input {
  @apply h-auto w-auto max-w-full;
}

.entry-item.view-icons .entry-rename-input,
.entry-item.view-tiles .entry-rename-input {
  @apply absolute z-30 resize-none whitespace-pre-wrap break-words;
  background: var(--app-control-solid);
  overflow: hidden;
}

.entry-item.view-icons .entry-rename-input {
  left: 50%;
  bottom: 0.35rem;
  transform: translateX(-50%);
  text-align: center;
  width: min(var(--rename-input-width, 12ch), calc(100% - 1rem));
  min-width: 8ch;
  max-width: calc(100% - 1rem);
}

.entry-item.view-tiles .entry-rename-input {
  left: calc(3.5rem + 0.75rem);
  bottom: 0.35rem;
  text-align: left;
  width: min(var(--rename-input-width, 16ch), calc(100% - 4.25rem));
  min-width: 10ch;
  max-width: calc(100% - 4.25rem);
}

.entry-item.view-icons .entry-rename-input,
.entry-item.view-tiles .entry-rename-input {
  box-sizing: border-box;
}

.entry-item.view-icons .entry-rename-input {
  padding-inline: 0.75rem;
  padding-block: 0.4rem;
}

.entry-item.view-tiles .entry-rename-input {
  padding-inline: 0.75rem;
  padding-block: 0.4rem;
}

.entry-rename-textarea {
  scrollbar-width: thin;
}

@supports (field-sizing: content) {
  .entry-item.view-icons .entry-rename-input,
  .entry-item.view-tiles .entry-rename-input {
    width: auto;
    field-sizing: content;
  }
}

.entry-item.view-icons .entry-name {
  @apply line-clamp-2 whitespace-normal break-all leading-tight;
}

.entry-item.view-icons[data-renaming="true"] .entry-meta,
.entry-item.view-tiles[data-renaming="true"] .entry-tile-meta {
  @apply opacity-0;
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

.entry-item.view-list .entry-main {
  @apply grid min-w-0 items-center gap-x-2;
  grid-template-columns: minmax(0, 1fr) max-content;
}

.entry-item.view-list .entry-meta {
  @apply shrink-0 whitespace-nowrap;
  overflow: visible;
  text-overflow: clip;
}

.entry-item.view-icons .entry-meta {
  @apply text-[11px] leading-none;
}
</style>
