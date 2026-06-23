<script setup lang="ts">
import {nextTick, onBeforeUnmount, onMounted, watch} from "vue";
import type {ComponentPublicInstance} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {downloadUrl} from "../../network/api.ts";
import {scrollHorizontallyWithWheel} from "../../utils/wheel.ts";

const props = defineProps<{
  items: {entry: ExplorerEntry; index: number}[];
  currentPath: string;
  imageCount: number;
}>();

const emit = defineEmits<{
  (e: "select", index: number): void;
}>();

const thumbRefs = new Map<string, HTMLElement>();

const setThumbRef = (path: string, element: Element | ComponentPublicInstance | null) => {
  if (element instanceof HTMLElement) {
    thumbRefs.set(path, element);
  } else {
    thumbRefs.delete(path);
  }
}

const revealActiveThumb = async () => {
  await nextTick();
  thumbRefs.get(props.currentPath)?.scrollIntoView({block: "nearest", inline: "center"});
}

watch(() => [props.currentPath, props.items.length] as const, () => {
  void revealActiveThumb();
}, {flush: "post"});

onMounted(() => {
  void revealActiveThumb();
});

onBeforeUnmount(() => {
  thumbRefs.clear();
});
</script>

<template>
  <div class="image-viewer-filmstrip" aria-label="图片列表" @wheel="scrollHorizontallyWithWheel">
    <button
        v-for="item in items"
        :key="item.entry.path"
        :ref="element => setThumbRef(item.entry.path, element)"
        class="image-viewer-thumb"
        :class="{active: item.entry.path === currentPath}"
        :title="`${item.index + 1} / ${imageCount} · ${item.entry.name}`"
        @click="emit('select', item.index)">
      <img :src="downloadUrl(item.entry.path)" :alt="item.entry.name" loading="lazy">
      <span>{{ item.index + 1 }}</span>
    </button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.image-viewer-filmstrip {
  @apply flex h-24 shrink-0 items-center gap-2 overflow-x-auto border-t border-white/10 bg-slate-950/45 px-4 py-2 backdrop-blur;
}

.image-viewer-thumb {
  @apply relative h-16 w-20 shrink-0 overflow-hidden rounded-md border border-white/10 bg-white/5 p-0.5 text-white opacity-75 outline-none hover:border-white/35 hover:opacity-100;
}

.image-viewer-thumb.active {
  @apply opacity-100;
  border-color: var(--app-accent-border, #bfdbfe);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 22%, transparent);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.25));
}

.image-viewer-thumb img {
  @apply h-full w-full rounded object-cover;
}

.image-viewer-thumb span {
  @apply absolute bottom-1 right-1 rounded bg-slate-950/70 px-1 text-[10px] leading-4 text-slate-100;
}
</style>
