<script setup lang="ts">
import type {FavoriteItem} from "../../class.ts";
import {normalizePathText} from "../../utils/file-path.ts";
import Icon from "../Icon.vue";

const props = defineProps<{
  favorites: FavoriteItem[];
  loading: boolean;
  currentPath: string;
}>();

const emit = defineEmits<{
  (e: "open", favorite: FavoriteItem): void;
  (e: "open-new-tab", favorite: FavoriteItem): void;
  (e: "remove", favorite: FavoriteItem): void;
  (e: "refresh"): void;
}>();

const isActive = (favorite: FavoriteItem) => {
  return normalizePathText(favorite.path) === normalizePathText(props.currentPath || "/");
}
</script>

<template>
  <section class="favorite-panel" aria-label="收藏夹">
    <div class="favorite-header">
      <div class="favorite-title">
        <icon icon="action.favorite" />
        <span>收藏夹</span>
      </div>
      <button class="favorite-refresh" :disabled="loading" title="刷新收藏夹" @click="emit('refresh')">
        <icon icon="action.refresh" />
      </button>
    </div>

    <div v-if="loading && !favorites.length" class="favorite-empty">正在加载...</div>
    <div v-else-if="!favorites.length" class="favorite-empty">暂无收藏</div>
    <div v-else class="favorite-list" role="list">
      <div
          v-for="favorite in favorites"
          :key="favorite.id"
          class="favorite-row"
          :class="{active: isActive(favorite), missing: favorite.missing}"
          role="listitem">
        <button
            class="favorite-open"
            :title="favorite.path"
            @click="emit('open', favorite)"
            @auxclick.middle.prevent="emit('open-new-tab', favorite)">
          <span class="favorite-icon" aria-hidden="true">
            <icon :icon="favorite.missing ? 'action.warning' : 'file.folder'" />
          </span>
          <span class="favorite-copy">
            <span class="favorite-name">{{ favorite.name }}</span>
            <small>{{ favorite.missing ? "目录缺失" : favorite.path }}</small>
          </span>
        </button>
        <span class="favorite-actions">
          <button class="favorite-action" title="在新标签页中打开" @click.stop="emit('open-new-tab', favorite)">
            <icon icon="action.open-new-tab" />
          </button>
          <button class="favorite-action" title="从收藏夹移除" @click.stop="emit('remove', favorite)">
            <icon icon="action.close" />
          </button>
        </span>
      </div>
    </div>
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.favorite-panel {
  @apply flex shrink-0 flex-col border-b px-1 pb-2;
  border-color: var(--app-border-soft);
}

.favorite-header {
  @apply flex h-7 items-center justify-between gap-2 px-1;
  color: var(--app-text-subtle);
}

.favorite-title {
  @apply flex min-w-0 items-center gap-1.5 text-[0.72rem] font-medium;
}

.favorite-title span {
  @apply truncate;
}

.favorite-refresh {
  @apply inline-flex h-6 w-6 shrink-0 items-center justify-center rounded border border-transparent;
  color: var(--app-text-subtle);
}

.favorite-refresh:hover:not(:disabled),
.favorite-refresh:focus-visible {
  background: var(--app-control-hover);
  color: var(--app-accent, #2563eb);
}

.favorite-refresh:focus-visible,
.favorite-action:focus-visible,
.favorite-open:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent-border, #bfdbfe);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.favorite-refresh:disabled {
  @apply cursor-wait;
  color: var(--app-text-disabled);
}

.favorite-list {
  @apply flex max-h-40 flex-col gap-0.5 overflow-auto pr-1;
}

.favorite-list::-webkit-scrollbar {
  width: 8px;
}

.favorite-list::-webkit-scrollbar-thumb {
  @apply rounded-full;
  background: var(--app-border);
}

.favorite-row {
  @apply grid h-8 min-w-0 grid-cols-[1.25rem_minmax(0,1fr)_auto] items-center gap-1.5 rounded-sm border border-transparent px-1.5 text-left;
  color: var(--app-text-muted);
}

.favorite-open {
  @apply col-span-2 grid h-full min-w-0 grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-1.5 rounded-sm border border-transparent bg-transparent text-left;
  color: inherit;
}

.favorite-row:hover {
  background: var(--app-accent-hover, #eaf4ff);
}

.favorite-row.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-selected, #dceeff);
  color: color-mix(in srgb, var(--app-accent, #2563eb) 62%, var(--app-text));
}

.favorite-row.missing {
  color: var(--app-text-subtle);
}

.favorite-icon {
  @apply inline-flex items-center justify-center;
  color: color-mix(in srgb, var(--app-warning) 88%, var(--app-text-muted));
}

.favorite-row.active .favorite-icon {
  color: var(--app-accent, #2563eb);
}

.favorite-copy {
  @apply flex min-w-0 flex-col gap-0.5;
}

.favorite-name {
  @apply truncate text-[13px] leading-none;
}

.favorite-copy small {
  @apply truncate text-[0.66rem] leading-none;
  color: var(--app-text-subtle);
}

.favorite-actions {
  @apply hidden shrink-0 items-center gap-0.5;
}

.favorite-row:hover .favorite-actions,
.favorite-row:focus-within .favorite-actions {
  @apply inline-flex;
}

.favorite-action {
  @apply inline-flex h-5 w-5 items-center justify-center rounded-sm border border-transparent;
  color: var(--app-text-subtle);
}

.favorite-action:hover {
  background: var(--app-control-hover);
  color: var(--app-accent, #2563eb);
}

.favorite-empty {
  @apply flex h-8 items-center px-2 text-xs;
  color: var(--app-text-disabled);
}
</style>
