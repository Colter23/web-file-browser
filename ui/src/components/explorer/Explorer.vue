<script setup lang="ts">
import SubCard from "../SubCard.vue";
import {ref} from "vue";
import {FolderData} from "../../class.ts";
import ExplorerItem from "./ExplorerItem.vue";

interface ExplorerProps {
  data: FolderData
}

const props = withDefaults(defineProps<ExplorerProps>(), {
  data: () => {
    return {
      path: "",
      folder: [],
      file: []
    }
  }
})

const selected = ref<string[]>([])

const fileClickHandler = (name: string) => {
  selected.value = [name]
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
      <explorer-item v-for="file in props.data.folder"
                     :modified="file.modified"
                     :name="file.name"
                     :selected="selected.includes(file.name)"
                     @click="fileClickHandler(file.name)" />
      <explorer-item v-for="file in props.data.file"
                     icon="icon-file"
                     :modified="file.modified"
                     :name="file.name"
                     :selected="selected.includes(file.name)"
                     :size="file.size"
                     @click="fileClickHandler(file.name)" />
    </div>
  </sub-card>
</template>
<style scoped lang="postcss">
.explorer-header {
  @apply border-b pb-1 text-sm flex px-4 py-2
}

.explorer-files {
  @apply px-2 py-1 flex flex-col gap-1
}
</style>
