<script setup lang="ts">
import FileTreeNode from "./FileTreeNode.vue";
import type {FileTreeData, LoadData} from "../class.ts";

defineProps<{
  data: FileTreeData[];
  loadData: LoadData;
}>();

</script>

<template>
  <div class="file-tree">
    <file-tree-node v-for="file in data" :key="file.path" :data="file" :load-data="loadData" />
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
.file-tree {
  @apply w-full h-full overflow-auto rounded-lg bg-white/40 p-1.5 select-none
}

.file-tree::-webkit-scrollbar{
  width: 8px;
  height: 8px;

}

.file-tree::-webkit-scrollbar-thumb {
  background: #ccc;
  border-radius: 5px;
}

</style>
