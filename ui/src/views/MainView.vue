<script setup lang="ts">

import TabBar from "../components/TabBar.vue";
import FileTree from "../components/FileTree.vue";
import {ref} from "vue";
import {FileTreeData} from "../class";
import {useFileStore} from "../store";
import {getFolderData} from "../network/api";
import Split from "../components/Split.vue";
import SubCard from "../components/SubCard.vue";

const fileStore = useFileStore();


const treeData = ref<FileTreeData[]>([]);

const handleLoad = (node: FileTreeData) => {
  return new Promise<void>(async (resolve) => {
    console.log("[Load] " + node.path);
    node.children = fileStore.saveAndConvertFolderData(await getFolderData(node.path));
    resolve();
  });
}

getFolderData().then(data => {
  treeData.value = fileStore.saveAndConvertFolderData(data);
})

</script>

<template>
  <tab-bar @change="(e) => {console.log(e)}"></tab-bar>
  <split class="m-10">
    <template #left>
      <file-tree :data="treeData" :load-data="handleLoad"></file-tree>
    </template>
    <template #right>
      <sub-card class="w-full h-96"></sub-card>
    </template>
  </split>
</template>

<style scoped lang="postcss">

</style>