<script setup lang="ts">
import type {ExplorerFolderStatus, ExplorerSelectionStatus} from "../../composables/useExplorerStatusText.ts";

defineProps<{
  folderStatus: ExplorerFolderStatus;
  selectionStatus: ExplorerSelectionStatus;
  folderStatusText: string;
  selectedStatusText: string;
}>();
</script>

<template>
  <div class="explorer-status-row">
    <div class="status-left" :title="folderStatusText">
      <span class="status-source">{{ folderStatus.sourceText }}</span>
      <span class="status-count">{{ folderStatus.countText }}</span>
      <span v-if="folderStatus.moreText" class="status-badge">{{ folderStatus.moreText }}</span>
    </div>
    <div class="status-selection" :class="{active: selectionStatus.active}" :title="selectedStatusText">
      <span class="selection-count">{{ selectionStatus.countText }}</span>
      <span v-if="selectionStatus.detailText" class="selection-detail">{{ selectionStatus.detailText }}</span>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.explorer-status-row {
  @apply flex h-8 shrink-0 items-center justify-between gap-4 border-t px-3 text-xs;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
  color: var(--app-text-subtle);
}

.status-left,
.status-selection {
  @apply flex min-w-0 items-center gap-2;
}

.status-left {
  @apply flex-1;
}

.status-selection {
  @apply shrink-0 justify-end text-right;
  color: var(--app-text-muted);
  max-width: min(34rem, 58%);
}

.status-source {
  @apply shrink-0 rounded border px-1.5 py-0.5 text-[0.68rem] font-medium leading-none;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.status-count,
.selection-count,
.selection-detail {
  @apply min-w-0 truncate;
}

.status-count {
  color: var(--app-text-muted);
}

.status-badge {
  @apply shrink-0 rounded px-1.5 py-0.5 text-[0.68rem] font-medium leading-none;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 10%, transparent);
  color: var(--app-accent, #2563eb);
}

.status-selection.active .selection-count {
  @apply rounded px-1.5 py-0.5 font-medium leading-none;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, transparent);
  color: var(--app-accent, #2563eb);
}

.selection-detail {
  color: var(--app-text-subtle);
}

@media (max-width: 720px) {
  .explorer-status-row {
    @apply gap-2 px-2;
  }

  .status-source,
  .selection-detail {
    @apply hidden;
  }

  .status-selection {
    max-width: 42%;
  }
}
</style>
