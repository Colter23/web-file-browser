<script setup lang="ts">

import TabBar from "../components/TabBar.vue";
import FileTree from "../components/FileTree.vue";
import {ref} from "vue";
import {FileTreeData} from "../class";
import {useFileStore} from "../store";
import {getFolderData} from "../network/api";

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
  <file-tree :data="treeData" :load-data="handleLoad"></file-tree>
</template>

<style scoped lang="postcss">

</style>