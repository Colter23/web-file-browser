<script setup lang="ts">

import SubCard from "./SubCard.vue";
import Icon from "./Icon.vue";
import {onMounted, ref, watch} from "vue";
import {useFileStore} from "../store";

const fileStore = useFileStore();

const pathList = ref(fileStore.currentPath.substring(1, fileStore.currentPath.length).split("/"));

const pathBox = ref();
const input = ref();
const pathInput = ref();

const isInput = ref(false);


watch(() => fileStore.currentPath, (path: string) => {
  pathList.value = path.substring(1, path.length).split("/");
});

onMounted(() => {
  const pathBoxEl = pathBox.value as Element;
  pathBoxEl.addEventListener("wheel", (e) => {
    e.preventDefault();
    pathBoxEl.scrollLeft += (e.deltaY as number);
  });
});

const changeInput = (e: Event) => {
  e.stopPropagation();
  isInput.value = true;
  const mouseHandle = (e: MouseEvent) => {
    if (!input.value.contains(e.target as Node)) {
      isInput.value = false;
      document.removeEventListener("mousedown", mouseHandle);
      document.removeEventListener("keydown", keyHandle);
    }
  }
  const keyHandle = (e: KeyboardEvent) => {
    if (e.code == "Enter") {
      fileStore.setCurrentPath(pathInput.value.value);
      isInput.value = false;
      document.removeEventListener("mousedown", mouseHandle);
      document.removeEventListener("keydown", keyHandle);
    }
  }

  document.addEventListener("mousedown", mouseHandle);
  document.addEventListener("keydown", keyHandle);
}

const changePath = (index: number) => {
  if (index == -1) fileStore.setCurrentPath("/");
  else fileStore.setCurrentPath(pathList.value.slice(0, index + 1).join("/"));
}

</script>

<template>
  <sub-card class="path-card">
    <div ref="pathBox" class="path-box" :class="isInput? 'hidden':'flex'">
      <div class="flex items-center gap-1">
        <div class="path-item px-1" @click="changePath(-1)"><icon icon="icon-homefill" size="large" /></div>
        <div class="separator"><icon icon="icon-unfold" class="-rotate-90" /></div>
      </div>
      <div class="flex items-center gap-1" v-for="(pathItem, index) in pathList">
        <div class="path-item px-2" @click="changePath(index)"><span>{{ pathItem }}</span></div>
        <div class="separator" v-if="index != pathList.length - 1"><icon icon="icon-unfold" class="-rotate-90" /></div>
      </div>
    </div>
    <div class="grow h-full min-w-14 cursor-text" :class="isInput? 'hidden':''" @click="changeInput">
    </div>
    <div ref="input" class="grow" :class="isInput? 'flex':'hidden'">
      <input ref="pathInput" class="w-full h-10 px-3" :value="fileStore.currentPath">
    </div>
  </sub-card>
</template>

<style scoped lang="postcss">
.path-card {
  @apply flex grow shrink min-w-48 h-10 items-center gap-1
}
.path-box {
  @apply px-2 shrink overflow-x-scroll
}
.path-item {
  @apply inline-flex items-center h-7 hover:bg-blue-100 rounded cursor-pointer
}
.separator {
  @apply w-4 h-7 inline-flex items-center justify-center
}
.path-box::-webkit-scrollbar {
  display: none;
}
</style>