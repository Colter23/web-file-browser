<script setup lang="ts">
import Icon from "./Icon.vue";
import {computed, ref} from "vue";
import type {FileTreeData, LoadData} from "../class.ts";
import {useFileStore} from "../store";

const fileStore = useFileStore();

const props = withDefaults(defineProps<{
  deep?: number;
  data: FileTreeData;
  loadData: LoadData;
}>(), {
  deep: 0
});

const fold = ref(false)
const loading = ref(false);
const active = computed(() => props.data.path === fileStore.currentPath);

const hasLoadedChildren = computed(() => props.data.children !== undefined);

async function clickHandler(file: FileTreeData) {
  if (file.isFile || loading.value) return
  if (hasLoadedChildren.value) {
    fold.value = !fold.value
    return;
  }
  loading.value = true;
  try {
    if (await props.loadData(file)) fold.value = true;
  } finally {
    loading.value = false;
  }
}

</script>

<template>
  <div class="tree-node" :class="{loading}" @click="clickHandler(data)">
    <div class="node-indent" v-for="_ in deep"></div>
    <div v-if="!data.isFile" class="node-icon node-fold-icon" :class="fold? 'unfold-icon':'fold-icon'">
      <icon icon="icon-unfold" size="normal"></icon>
    </div>
    <div v-else class="node-indent"></div>
    <div class="node-icon">
      <icon :icon="data.isFile?'icon-file':'icon-folder-fill'" size="normal"></icon>
    </div>
    <div class="node-content" :class="{active}">{{ data.name }}</div>
  </div>
  <div class="flex flex-col overflow-hidden transition-all" :class="fold? 'h-max': 'h-0'">
    <file-tree-node v-for="file in data.children" :key="file.path" :deep="deep + 1" :data="file" :load-data="loadData" />
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
.tree-node {
  @apply flex w-full items-center break-words rounded-md py-0.5 hover:bg-blue-50
}
.tree-node.loading {
  @apply cursor-wait opacity-70;
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
  @apply inline-flex h-7 min-w-0 grow cursor-pointer items-center truncate rounded-md px-1 text-sm
}
.node-content.active {
  @apply bg-blue-600 text-white
}
</style>
