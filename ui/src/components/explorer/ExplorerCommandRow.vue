<script setup lang="ts">
import type {DirSortKey, DirSortOrder, ExplorerViewMode} from "../../class.ts";
import Icon from "../Icon.vue";

type SortOption = {
  key: DirSortKey;
  label: string;
}

const props = defineProps<{
  totalCountText: string;
  selectedCountText: string;
  sortText: string;
  sortOptions: SortOption[];
  sortKey: DirSortKey;
  sortOrder: DirSortOrder;
  nextSortOrder: DirSortOrder;
  viewMode: ExplorerViewMode;
  iconSizeText: string;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "change-sort", key: DirSortKey): void;
  (e: "change-sort-order", order: DirSortOrder): void;
  (e: "set-view-mode", mode: ExplorerViewMode): void;
  (e: "cycle-icon-size"): void;
}>();

const sortIndicator = (key: DirSortKey) => {
  if (props.sortKey !== key) return "";
  return props.sortOrder === "asc" ? "↑" : "↓";
}
</script>

<template>
  <div class="explorer-command-row">
    <div class="explorer-summary">
      <span>{{ totalCountText }}</span>
      <span>{{ selectedCountText }}</span>
      <span>排序：{{ sortText }}</span>
    </div>
    <div class="explorer-controls">
      <div class="sort-switch" aria-label="排序方式">
        <button
            v-for="option in sortOptions"
            :key="option.key"
            :class="{active: sortKey === option.key}"
            :disabled="loading"
            :title="`按${option.label}排序`"
            @click="emit('change-sort', option.key)">
          <span>{{ option.label }}</span>
          <span class="sort-chip-indicator">{{ sortIndicator(option.key) }}</span>
        </button>
        <button class="order-toggle" :disabled="loading" :title="`切换为${nextSortOrder === 'asc' ? '升序' : '降序'}`" @click="emit('change-sort-order', nextSortOrder)">
          {{ sortOrder === "asc" ? "升序" : "降序" }}
        </button>
      </div>
      <div class="view-switch" aria-label="查看模式">
        <button :class="{active: viewMode === 'details'}" title="详细信息 (Ctrl+Shift+6)" @click="emit('set-view-mode', 'details')">
          <icon icon="icon-view-list" />
        </button>
        <button :class="{active: viewMode === 'list'}" title="列表 (Ctrl+Shift+5)" @click="emit('set-view-mode', 'list')">
          <icon icon="icon-listview" />
        </button>
        <button :class="{active: viewMode === 'icons'}" title="图标 (Ctrl+Shift+1-4)" @click="emit('set-view-mode', 'icons')">
          <icon icon="icon-viewgrid" />
        </button>
        <button :class="{active: viewMode === 'tiles'}" title="平铺 (Ctrl+Shift+7)" @click="emit('set-view-mode', 'tiles')">
          <icon icon="icon-file-common-filling" />
        </button>
        <button title="图标大小 (Ctrl+鼠标滚轮)" @click="emit('cycle-icon-size')">
          <span class="size-mark">{{ iconSizeText }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.explorer-command-row {
  @apply flex min-h-9 shrink-0 items-center justify-between gap-2 border-b border-slate-200 px-3 py-1 text-xs text-slate-500;
}

.explorer-summary {
  @apply flex min-w-0 items-center gap-3 truncate;
}

.explorer-controls {
  @apply flex shrink-0 items-center gap-2;
}

.sort-switch,
.view-switch {
  @apply inline-flex shrink-0 overflow-hidden rounded-md border border-slate-200 bg-slate-50;
}

.sort-switch button,
.view-switch button {
  @apply inline-flex h-7 min-w-8 items-center justify-center gap-1 border-r border-slate-200 px-2 text-slate-600 last:border-r-0 hover:bg-white disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:bg-slate-50;
}

.sort-switch button.active,
.view-switch button.active {
  @apply bg-blue-600 text-white hover:bg-blue-600;
}

.sort-switch .order-toggle {
  @apply min-w-12 font-medium;
}

.sort-chip-indicator {
  @apply inline-flex w-2 justify-center text-[10px];
}

.size-mark {
  @apply text-[11px] leading-none;
}
</style>
