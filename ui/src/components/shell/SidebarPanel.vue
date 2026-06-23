<script setup lang="ts">
import type {FileTreeData, LoadData} from "../../class.ts";
import FileTree from "../FileTree.vue";

defineProps<{
  treeData: FileTreeData[];
  loadData: LoadData;
  currentPath: string;
}>();
</script>

<template>
  <aside class="sidebar">
    <file-tree :data="treeData" :load-data="loadData" :current-path="currentPath" />
  </aside>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.sidebar {
  @apply flex min-h-0 flex-col overflow-hidden border-r border-slate-200 bg-white/55 p-2;
}
</style>
