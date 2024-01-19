<script setup lang="ts">

import TabBar from "../components/TabBar.vue";
import FileTree from "../components/FileTree.vue";
import {ref} from "vue";
import {FileTreeData} from "../class";
import {useFileStore} from "../store";
import {getFolderData} from "../network/api";
import Split from "../components/Split.vue";
import SubCard from "../components/SubCard.vue";
import Icon from "../components/Icon.vue";
import Card from "../components/Card.vue";

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
  <div class="h-screen flex flex-col p-3 gap-y-3">
    <div class="flex gap-x-3">
      <tab-bar class="grow" @change="(e) => {console.log(e)}"></tab-bar>
      <card class="w-12 h-12 icon">
        <icon icon="icon-setting" size="2"></icon>
      </card>
    </div>

    <card class="flex flex-col shrink grow overflow-auto p-2 gap-y-2">
      <div class="flex gap-x-2">
        <sub-card class="w-60 h-11"></sub-card>
        <sub-card class="w-11 h-11 icon rotate-90">
          <icon icon="icon-unfold" size="large"/>
        </sub-card>
        <sub-card class="w-11 h-11 icon">
          <icon icon="icon-refresh" size="large"/>
        </sub-card>
        <sub-card class="grow h-11"></sub-card>
        <sub-card class="w-28 h-11"></sub-card>
      </div>
      <split class="shrink grow overflow-auto">
        <template #left>
          <file-tree :data="treeData" :load-data="handleLoad"></file-tree>
        </template>
        <template #right>
          <sub-card class="w-full h-full"></sub-card>
        </template>
      </split>
    </card>
  </div>


</template>

<style scoped lang="postcss">
.icon {
  @apply inline-flex items-center justify-center
}
</style>