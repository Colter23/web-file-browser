<script setup lang="ts">

import Icon from "../Icon.vue";

interface ExplorerItemProps {
  name: string;
  modified: string;
  size: number;
  selected: boolean;
  icon: string;
}

const props = withDefaults(defineProps<ExplorerItemProps>(), {
  name: "",
  modified: "",
  size: 0,
  selected: false,
  icon: "icon-folder"
})

const getDate = (srcDate: string) => {
  if (!srcDate) {
    return "-"
  }
  const date = new Date(srcDate)
  const year = date.getFullYear()
  const month = date.getMonth() + 1
  const day = date.getDate()
  return `${year}/${month}/${day}`
}

const formatSize = (filesize: number) => {
  if(!filesize){
    return "-";
  }
  const unitArr = ["Bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
  let index = 0;
  index=Math.floor(Math.log(filesize)/Math.log(1024));
  let size =filesize/Math.pow(1024,index);
  return size.toFixed(1)+unitArr[index];
}
</script>

<template>
  <div class="explorer-file"
       :class="{selected: props.selected}">
    <p class="flex items-center w-[50%]">
      <Icon class="flex-shrink-0" :icon="props.icon" />
      <span class="ml-1">{{props.name}}</span>
    </p>
    <p class="w-[25%]">{{ getDate(props.modified) }}</p>
    <p>{{ formatSize(props.size) }}</p>
  </div>
</template>

<style scoped lang="postcss">
.explorer-file {
  @apply flex px-2 py-1 hover:bg-[#EBF3FF] cursor-pointer
}

.explorer-file p,
.explorer-file span {
  @apply whitespace-nowrap overflow-hidden text-ellipsis mx-0.5
}

.explorer-file.selected {
  @apply bg-[#3662EC] text-white rounded-md
}
</style>
