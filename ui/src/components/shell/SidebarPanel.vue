<script setup lang="ts">
import type {FileTreeData, LoadData} from "../../class.ts";
import type {ExplorerEntry} from "../explorer/types.ts";
import FileTree from "../FileTree.vue";

defineProps<{
  treeData: FileTreeData[];
  loadData: LoadData;
  currentPath: string;
}>();

defineEmits<{
  (e: "drop-entries", payload: {entries: ExplorerEntry[]; target: FileTreeData; action: "copy" | "move"}): void;
}>();
</script>

<template>
  <aside class="sidebar">
    <file-tree
        :data="treeData"
        :load-data="loadData"
        :current-path="currentPath"
        @drop-entries="payload => $emit('drop-entries', payload)" />
  </aside>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.sidebar {
  @apply flex min-h-0 flex-col overflow-hidden px-2 py-2;
  background: var(--app-panel-muted);
}
</style>
