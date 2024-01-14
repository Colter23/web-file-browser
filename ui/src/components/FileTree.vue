<script setup lang="ts">

import SubCard from "./SubCard.vue";
import FileTreeNode from "./FileTreeNode.vue";
import {ref} from "vue";

const files = ref<{[key: string]: string}>({
  "/Download": "Download",
  "/Resources": "Resources"
})

</script>

<template>
<sub-card class="file-tree">
  <file-tree-node v-for="(file, path) in files" :content="file"></file-tree-node>
</sub-card>
</template>

<style scoped>
.file-tree {
  @apply w-64 h-96 m-10 p-1.5
}
</style>