<script setup lang="ts">

import FileTreeNode from "./FileTreeNode.vue";
import {FileTreeData, LoadData} from "../class.ts";
import { PropType } from 'vue';


const props = defineProps({
  data: Object as PropType<FileTreeData[]>,
  loadData: Function as PropType<LoadData>,
});


</script>

<template>
<div class="file-tree">
  <file-tree-node v-for="file in props.data" :data="file" :load-data="loadData"></file-tree-node>
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
