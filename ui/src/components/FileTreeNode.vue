<script setup lang="ts">
import Icon from "./Icon.vue";
import {PropType, ref} from "vue";
import {FileTreeData, LoadData} from "../class.ts";

const props = defineProps({
  deep: {
    type: Number,
    default: 0
  },
  data: Object as PropType<FileTreeData>,
  loadData: Function as PropType<LoadData>,
});

const fold = ref(false)

function clickHandler(file: FileTreeData) {
  if (file.isFile) return
  if (props.data?.children == undefined || props.data?.children.length == 0) {
    props.loadData?.call("node", file).then(call => {
      console.log(props.data)
      fold.value = true
    })
  }else {
    fold.value = !fold.value
  }
}

</script>

<template>
  <div class="tree-node" @click="clickHandler(<FileTreeData>data)">
    <div class="node-indent" v-for="_ in deep"></div>
    <div v-if="!data?.isFile" class="node-icon node-fold-icon" :class="fold? 'unfold-icon':'fold-icon'">
      <icon icon="icon-unfold" size="normal"></icon>
    </div>
    <div v-else class="node-indent"></div>
    <div class="node-icon">
      <icon :icon="data?.isFile?'icon-file':'icon-folder-fill'" size="normal"></icon>
    </div>
    <div class="node-content">{{ data.name }}</div>
  </div>
  <div class="flex flex-col overflow-hidden transition-all" :class="fold? 'h-max': 'h-0'">
    <file-tree-node v-for="file in data?.children" :deep="deep + 1" :data="file" :load-data="loadData"></file-tree-node>
  </div>
</template>

<style scoped lang="postcss">
.tree-node {
  @apply flex w-full p-1 rounded-md hover:bg-blue-100
}
.node-indent {
  @apply w-6 grow-0 shrink-0
}
.node-icon {
  @apply w-7 inline-flex grow-0 shrink-0 items-center justify-center cursor-pointer
}
.node-fold-icon {
  @apply w-6 transition-transform
}
.fold-icon {
  @apply -rotate-90
}
.unfold-icon {
  @apply rotate-0
}
.node-content {
  @apply inline-flex grow items-center cursor-pointer
}
</style>