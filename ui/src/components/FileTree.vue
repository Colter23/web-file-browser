<script setup lang="ts">

import SubCard from "./SubCard.vue";
import FileTreeNode from "./FileTreeNode.vue";
import {FileTreeData, LoadData} from "../class.ts";
import { PropType } from 'vue';


const props = defineProps({
  data: Object as PropType<FileTreeData[]>,
  loadData: Function as PropType<LoadData>,
});


</script>

<template>
<sub-card class="file-tree">
  <file-tree-node v-for="file in props.data" :data="file" :load-data="loadData"></file-tree-node>
</sub-card>
</template>

<style scoped lang="postcss">
.file-tree {
  @apply w-64 h-96 m-10 p-1.5 overflow-auto
}
</style>