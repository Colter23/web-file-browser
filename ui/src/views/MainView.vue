<script setup lang="ts">

import TabBar from "../components/TabBar.vue";
import FileTree from "../components/FileTree.vue";
import {ref} from "vue";
import {FileTreeData} from "../class";


const data = ref<FileTreeData[]>([
  {
    path: "/Res",
    name: "Res",
  }
])
const handleLoad = (node: FileTreeData) => {
  return new Promise<void>(async (resolve) => {
    console.log("LoadData")
    node.children = [
      {
        path: "/Res1",
        name: "Res1",
      },
      {
        path: "/Res2",
        name: "Res2",
      }
    ];
    resolve();
  });
}

</script>

<template>
  <tab-bar @change="(e) => {console.log(e)}"></tab-bar>
  <file-tree :data="data" :load-data="handleLoad"></file-tree>
</template>

<style scoped lang="postcss">

</style>