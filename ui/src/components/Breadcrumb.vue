<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import Icon from "./Icon.vue";
import {useFileStore} from "../store";
import {scrollHorizontallyWithWheel} from "../utils/wheel.ts";

const fileStore = useFileStore();
type NavigateComplete = (navigated: boolean) => void;
const emit = defineEmits<{
  (e: "navigate", path: string, complete?: NavigateComplete): void;
}>();

const splitPath = (path: string) => path.substring(1).split("/").filter(Boolean);
const toPath = (parts: string[]) => parts.length ? `/${parts.join("/")}` : "/";
const pathList = ref(splitPath(fileStore.currentPath));
const pathText = computed(() => fileStore.currentPath || "/");
const pathBox = ref<HTMLElement | null>(null);
const input = ref<HTMLElement | null>(null);
const pathInput = ref<HTMLInputElement | null>(null);
const isInput = ref(false);

const scrollPathBoxToEnd = async () => {
  await nextTick();
  const pathBoxEl = pathBox.value;
  if (!pathBoxEl) return;
  pathBoxEl.scrollLeft = pathBoxEl.scrollWidth;
}

watch(() => fileStore.currentPath, (path: string) => {
  pathList.value = splitPath(path);
  if (pathInput.value) pathInput.value.value = path || "/";
  if (!isInput.value) void scrollPathBoxToEnd();
});

onMounted(() => {
  const pathBoxEl = pathBox.value;
  if (pathBoxEl == null) return;
  pathBoxEl.addEventListener("wheel", scrollHorizontallyWithWheel, {passive: false});
  void scrollPathBoxToEnd();
});

let stopInputListeners: (() => void) | null = null;

const cleanupInputListeners = () => {
  stopInputListeners?.();
  stopInputListeners = null;
}

const stopInput = () => {
  isInput.value = false;
  cleanupInputListeners();
  void scrollPathBoxToEnd();
}

const focusInput = async () => {
  isInput.value = true;
  await nextTick();
  pathInput.value?.focus();
  pathInput.value?.select();
}

const changeInput = (event?: Event) => {
  event?.stopPropagation();
  cleanupInputListeners();

  const pointerHandle = (pointerEvent: PointerEvent) => {
    if (input.value != null && !input.value.contains(pointerEvent.target as Node)) {
      stopInput();
    }
  }
  const keyHandle = (keyEvent: KeyboardEvent) => {
    if (keyEvent.code === "Enter") {
      keyEvent.preventDefault();
      keyEvent.stopPropagation();
      emit("navigate", pathInput.value?.value ?? "/", (navigated) => {
        if (navigated) stopInput();
      });
    } else if (keyEvent.code === "Escape") {
      keyEvent.preventDefault();
      keyEvent.stopPropagation();
      stopInput();
    }
  }

  document.addEventListener("pointerdown", pointerHandle);
  document.addEventListener("keydown", keyHandle);
  stopInputListeners = () => {
    document.removeEventListener("pointerdown", pointerHandle);
    document.removeEventListener("keydown", keyHandle);
  };
  void focusInput();
}

const changePath = (index: number) => {
  emit("navigate", index === -1 ? "/" : toPath(pathList.value.slice(0, index + 1)));
}

defineExpose({
  focusInput: changeInput
});

onBeforeUnmount(() => {
  cleanupInputListeners();
  pathBox.value?.removeEventListener("wheel", scrollHorizontallyWithWheel);
});
</script>

<template>
  <div class="path-card" title="地址栏 (Ctrl+L / Alt+D)">
    <div class="flex items-center gap-1 pl-2">
      <div class="path-item px-1" title="主页" @click="changePath(-1)">
        <icon icon="file.home" size="large" />
      </div>
      <div v-if="pathList.length" class="separator">
        <icon icon="action.next" />
      </div>
    </div>
    <div ref="pathBox" class="path-box" :class="isInput ? 'hidden' : 'flex'">
      <div v-for="(pathItem, index) in pathList" :key="`${pathItem}-${index}`" class="flex items-center">
        <div class="path-item px-2" @click="changePath(index)">
          <span>{{ pathItem }}</span>
        </div>
        <div v-if="index !== pathList.length - 1" class="separator">
          <icon icon="action.next" />
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
  @apply flex h-9 min-w-48 grow shrink items-center gap-1 rounded-md border border-[#d7e1ec] bg-white shadow-[inset_0_0_0_1px_rgba(255,255,255,0.55)];
}

.path-box {
  @apply shrink overflow-x-scroll;
}

.path-item {
  @apply inline-flex h-7 cursor-pointer items-center rounded-sm text-nowrap truncate text-sm;
}

.path-item:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.separator {
  @apply inline-flex h-7 w-4 items-center justify-center text-slate-400;
}

.path-box::-webkit-scrollbar {
  display: none;
}
</style>
