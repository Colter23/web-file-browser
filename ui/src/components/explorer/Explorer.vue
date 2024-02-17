<script setup lang="ts">
import SubCard from "../SubCard.vue";
import {ref, watch} from "vue";
import {FolderData} from "../../class.ts";
import ExplorerItem from "./ExplorerItem.vue";
import {useFileStore} from "../../store";
import {getFolderData} from "../../network/file-api.ts";


const fileStore = useFileStore();

const folderData = ref<FolderData>({ path: "", folder: [], file: [] });

watch(() => fileStore.currentPath, (path: string) => {
  const data = fileStore.folderData.get(path);
  if (data) folderData.value = data;
});

const selected = ref<string[]>([]);

const fileClickHandler = (name: string) => {
  selected.value = [name];
}

const folderDoubleClickHandler = (path: string) => {
  getFolderData(path).then(data => {
    fileStore.currentPath = path;
    fileStore.saveAndConvertFolderData(data);
  })
}

const fileDoubleClickHandler = (path: string) => {
  fileStore.showEditor = true;
}

</script>

<template>
  <sub-card class="explorer-wrapper">
    <div class="explorer-header">
      <p class="w-[50%]">名称</p>
      <p class="w-[25%]">修改时间</p>
      <p>大小</p>
    </div>
    <div class="explorer-files">
      <explorer-item v-for="file in folderData.folder"
                     :modified="file.modified"
                     :name="file.name"
                     :selected="selected.includes(file.name)"
                     @click="fileClickHandler(file.name)"
                     @dblclick="folderDoubleClickHandler(file.path)" />
      <explorer-item v-for="file in folderData.file"
                     icon="icon-file"
                     :modified="file.modified"
                     :name="file.name"
                     :selected="selected.includes(file.name)"
                     :size="file.size"
                     @click="fileClickHandler(file.name)"
                     @dblclick="fileDoubleClickHandler(file.path)"/>
    </div>
  </sub-card>
</template>
<style scoped lang="postcss">
.explorer-wrapper {
  @apply w-full h-full overflow-auto
}

.explorer-header {
  @apply border-b pb-1 text-sm flex px-4 py-2 sticky top-0 bg-white z-10
}

.explorer-files {
  @apply px-2 py-1 flex flex-col gap-1
}
</style>
