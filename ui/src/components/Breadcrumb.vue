<script setup lang="ts">
import {computed, onMounted, ref, watch} from "vue";
import Icon from "./Icon.vue";
import {useFileStore} from "../store";

const fileStore = useFileStore();

const splitPath = (path: string) => path.substring(1).split("/").filter(Boolean);
const pathList = ref(splitPath(fileStore.currentPath));
const pathText = computed(() => fileStore.currentPath || "/");
const pathBox = ref<HTMLElement | null>(null);
const input = ref<HTMLElement | null>(null);
const pathInput = ref<HTMLInputElement | null>(null);
const isInput = ref(false);

watch(() => fileStore.currentPath, (path: string) => {
  pathList.value = splitPath(path);
});

onMounted(() => {
  const pathBoxEl = pathBox.value;
  if (pathBoxEl == null) return;
  pathBoxEl.addEventListener("wheel", (event: WheelEvent) => {
    event.preventDefault();
    pathBoxEl.scrollLeft += event.deltaY;
  });
});

const stopInput = (mouseHandle: (event: MouseEvent) => void, keyHandle: (event: KeyboardEvent) => void) => {
  isInput.value = false;
  document.removeEventListener("mousedown", mouseHandle);
  document.removeEventListener("keydown", keyHandle);
}

const changeInput = (event: Event) => {
  event.stopPropagation();
  isInput.value = true;
  requestAnimationFrame(() => pathInput.value?.focus());

  const mouseHandle = (mouseEvent: MouseEvent) => {
    if (input.value != null && !input.value.contains(mouseEvent.target as Node)) {
      stopInput(mouseHandle, keyHandle);
    }
  }
  const keyHandle = (keyEvent: KeyboardEvent) => {
    if (keyEvent.code === "Enter") {
      fileStore.setCurrentPath(pathInput.value?.value ?? "/");
      stopInput(mouseHandle, keyHandle);
    } else if (keyEvent.code === "Escape") {
      stopInput(mouseHandle, keyHandle);
    }
  }

  document.addEventListener("mousedown", mouseHandle);
  document.addEventListener("keydown", keyHandle);
}

const changePath = (index: number) => {
  fileStore.showEditor = false;
  if (index === -1) fileStore.setCurrentPath("/");
  else fileStore.setCurrentPath(pathList.value.slice(0, index + 1).join("/"));
}
</script>

<template>
  <div class="path-card">
    <div class="flex items-center gap-1 pl-2">
      <div class="path-item px-1" title="主页" @click="changePath(-1)">
        <icon icon="icon-homefill" size="large" />
      </div>
      <div v-if="pathList.length" class="separator">
        <icon icon="icon-unfold" class="-rotate-90" />
      </div>
    </div>
    <div ref="pathBox" class="path-box" :class="isInput ? 'hidden' : 'flex'">
      <div v-for="(pathItem, index) in pathList" :key="`${pathItem}-${index}`" class="flex items-center">
        <div class="path-item px-2" @click="changePath(index)">
          <span>{{ pathItem }}</span>
        </div>
        <div v-if="index !== pathList.length - 1" class="separator">
          <icon icon="icon-unfold" class="-rotate-90" />
        </div>
      </div>
    </div>
    <div class="h-full min-w-14 grow cursor-text" :class="isInput ? 'hidden' : ''" @click="changeInput"></div>
    <div ref="input" class="grow" :class="isInput ? 'flex' : 'hidden'">
      <input ref="pathInput" class="h-10 w-full bg-transparent px-3 text-sm outline-none" :value="pathText">
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.path-card {
  @apply flex h-10 min-w-48 grow shrink items-center gap-1 rounded-lg border border-slate-200 bg-white;
}

.path-box {
  @apply shrink overflow-x-scroll;
}

.path-item {
  @apply inline-flex h-7 cursor-pointer items-center rounded text-nowrap truncate text-sm hover:bg-blue-50;
}

.separator {
  @apply inline-flex h-7 w-4 items-center justify-center text-slate-400;
}

.path-box::-webkit-scrollbar {
  display: none;
}
</style>
