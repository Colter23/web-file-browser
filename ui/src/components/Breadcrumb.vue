<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import Icon from "./Icon.vue";
import {useFileStore} from "../store";
import {scrollHorizontallyWithWheel} from "../utils/wheel.ts";
import type {ExplorerEntryPathDropPayload} from "./explorer/types.ts";
import {hasInternalEntryDragData, readInternalEntryDragData} from "../utils/internal-entry-drag.ts";

const fileStore = useFileStore();
type NavigateComplete = (navigated: boolean) => void;

const emit = defineEmits<{
  (e: "navigate", path: string, complete?: NavigateComplete): void;
  (e: "drop-entries", payload: ExplorerEntryPathDropPayload): void;
}>();

const splitPath = (path: string) => path.substring(1).split("/").filter(Boolean);
const toPath = (parts: string[]) => parts.length ? `/${parts.join("/")}` : "/";
const pathList = ref(splitPath(fileStore.currentPath));
const pathText = computed(() => fileStore.currentPath || "/");
const pathBox = ref<HTMLElement | null>(null);
const input = ref<HTMLElement | null>(null);
const pathInput = ref<HTMLInputElement | null>(null);
const isInput = ref(false);
const dropTargetPath = ref("");

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
  document.addEventListener("dragend", clearDropTarget, true);
  document.addEventListener("drop", clearDropTarget, true);
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

const isCopyDrop = (event: DragEvent) => Boolean(event.ctrlKey || event.metaKey);

const canAcceptDrop = (event: DragEvent) => {
  return !isInput.value && hasInternalEntryDragData(event.dataTransfer);
}

const dragOverPath = (event: DragEvent, path: string) => {
  if (!canAcceptDrop(event)) return;
  event.preventDefault();
  event.stopPropagation();
  dropTargetPath.value = path;
  if (event.dataTransfer) event.dataTransfer.dropEffect = isCopyDrop(event) ? "copy" : "move";
}

const dragLeavePath = (event: DragEvent, path: string) => {
  if (dropTargetPath.value !== path) return;
  const related = event.relatedTarget;
  if (related instanceof Node && (event.currentTarget as HTMLElement | null)?.contains(related)) return;
  dropTargetPath.value = "";
}

const dropOnPath = (event: DragEvent, path: string, name: string) => {
  if (!canAcceptDrop(event)) return;
  const entries = readInternalEntryDragData(event.dataTransfer);
  if (!entries.length) {
    dropTargetPath.value = "";
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  dropTargetPath.value = "";
  emit("drop-entries", {
    entries,
    target: {path, name},
    action: isCopyDrop(event) ? "copy" : "move"
  });
}

const clearDropTarget = () => {
  dropTargetPath.value = "";
}

defineExpose({
  focusInput: changeInput
});

onBeforeUnmount(() => {
  cleanupInputListeners();
  pathBox.value?.removeEventListener("wheel", scrollHorizontallyWithWheel);
  document.removeEventListener("dragend", clearDropTarget, true);
  document.removeEventListener("drop", clearDropTarget, true);
});
</script>

<template>
  <div class="path-card" title="地址栏 (Ctrl+L / Alt+D)">
    <div class="flex items-center gap-1 pl-2">
      <button
          class="path-item px-1"
          :class="{dropTarget: dropTargetPath === '/'}"
          type="button"
          title="主页"
          @click="changePath(-1)"
          @dragover="dragOverPath($event, '/')"
          @dragleave="dragLeavePath($event, '/')"
          @drop="dropOnPath($event, '/', '主页')">
        <icon icon="file.home" size="large" />
      </button>
      <div v-if="pathList.length" class="separator">
        <icon icon="action.next" />
      </div>
    </div>
    <div ref="pathBox" class="path-box" :class="isInput ? 'hidden' : 'flex'">
      <div v-for="(pathItem, index) in pathList" :key="`${pathItem}-${index}`" class="flex items-center">
        <button
            class="path-item px-2"
            :class="{dropTarget: dropTargetPath === toPath(pathList.slice(0, index + 1))}"
            type="button"
            @click="changePath(index)"
            @dragover="dragOverPath($event, toPath(pathList.slice(0, index + 1)))"
            @dragleave="dragLeavePath($event, toPath(pathList.slice(0, index + 1)))"
            @drop="dropOnPath($event, toPath(pathList.slice(0, index + 1)), pathItem)">
          <span>{{ pathItem }}</span>
        </button>
        <div v-if="index !== pathList.length - 1" class="separator">
          <icon icon="action.next" />
        </div>
      </div>
    </div>
    <div
        class="h-full min-w-14 grow cursor-text"
        :class="isInput ? 'hidden' : ''"
        @click="changeInput"
        @dragover="clearDropTarget">
    </div>
    <div ref="input" class="grow" :class="isInput ? 'flex' : 'hidden'">
      <input ref="pathInput" class="h-10 w-full bg-transparent px-3 text-sm outline-none" :value="pathText">
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.path-card {
  @apply flex h-9 min-w-48 grow shrink items-center gap-1 rounded-md border shadow-[inset_0_0_0_1px_rgba(255,255,255,0.08)];
  border-color: var(--app-border);
  background: var(--app-control-solid);
}

.path-card:focus-within {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.path-box {
  @apply shrink overflow-x-scroll;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.path-item {
  @apply inline-flex h-7 cursor-pointer items-center rounded-sm border border-transparent bg-transparent text-nowrap truncate text-sm outline-none;
  color: var(--app-text-muted);
}

.path-item:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.path-item.dropTarget {
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe), 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.path-item:focus-visible {
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.separator {
  @apply inline-flex h-7 w-4 items-center justify-center;
  color: var(--app-text-subtle);
}

.path-box::-webkit-scrollbar {
  display: none;
}
</style>
