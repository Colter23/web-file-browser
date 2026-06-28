<script setup lang="ts">
import type {FileEntryMetaRow} from "../../utils/file-entry.ts";

defineProps<{
  items: FileEntryMetaRow[];
}>();
</script>

<template>
  <div class="preview-meta-list">
    <table>
      <tbody>
        <tr v-for="item in items" :key="item.label" :title="item.value">
          <th scope="row">{{ item.label }}</th>
          <td>{{ item.value }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-meta-list {
  @apply shrink-0 border-b px-2.5 py-1.5 text-[11px];
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.preview-meta-list table {
  @apply w-full table-fixed overflow-hidden rounded-md border shadow-sm;
  border-color: color-mix(in srgb, var(--app-border-soft) 58%, transparent);
  background: color-mix(in srgb, var(--app-panel-solid) 72%, transparent);
  border-collapse: separate;
  border-spacing: 0;
}

.preview-meta-list th,
.preview-meta-list td {
  @apply h-5 min-w-0 px-2 py-0.5 align-middle leading-4;
}

.preview-meta-list th {
  @apply w-[3.25rem] whitespace-nowrap text-left font-normal;
  border-right: 1px solid color-mix(in srgb, var(--app-border-soft) 46%, transparent);
  background: color-mix(in srgb, var(--app-control) 46%, transparent);
  color: var(--app-text-subtle);
}

.preview-meta-list td {
  @apply truncate text-left font-medium;
  color: var(--app-text);
}

.preview-meta-list tr + tr th,
.preview-meta-list tr + tr td {
  padding-top: 0;
}
</style>
