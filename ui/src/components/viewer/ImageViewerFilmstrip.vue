<script setup lang="ts">
import {nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
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
const reduceMotion = ref(false);
let reduceMotionMedia: MediaQueryList | null = null;

const setThumbRef = (path: string, element: Element | ComponentPublicInstance | null) => {
  if (element instanceof HTMLElement) {
    thumbRefs.set(path, element);
  } else {
    thumbRefs.delete(path);
  }
}

const handleReduceMotionChange = (event: MediaQueryListEvent) => {
  reduceMotion.value = event.matches;
}

const revealActiveThumb = async () => {
  await nextTick();
  thumbRefs.get(props.currentPath)?.scrollIntoView({
    block: "nearest",
    inline: "center",
    behavior: reduceMotion.value ? "auto" : "smooth"
  });
}

watch(() => [props.currentPath, props.items.length] as const, () => {
  void revealActiveThumb();
}, {flush: "post"});

onMounted(() => {
  reduceMotionMedia = window.matchMedia("(prefers-reduced-motion: reduce)");
  reduceMotion.value = reduceMotionMedia.matches;
  reduceMotionMedia.addEventListener("change", handleReduceMotionChange);
  void revealActiveThumb();
});

onBeforeUnmount(() => {
  reduceMotionMedia?.removeEventListener("change", handleReduceMotionChange);
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
      <span class="image-viewer-thumb-index">{{ item.index + 1 }}</span>
    </button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.image-viewer-filmstrip {
  @apply flex h-20 shrink-0 items-center gap-2 overflow-x-auto border-t px-3 py-2 backdrop-blur;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.1));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 4%, rgba(15, 23, 42, 0.48));
}

.image-viewer-thumb {
  @apply relative h-14 w-[4.5rem] shrink-0 overflow-hidden rounded-md border border-white/10 bg-white/5 p-0.5 text-white outline-none transition hover:border-white/35 hover:bg-white/10 hover:opacity-100;
  opacity: 0.72;
}

.image-viewer-thumb.active {
  @apply opacity-100;
  border-color: var(--app-accent-border, #bfdbfe);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 22%, transparent);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.25));
}

.image-viewer-thumb:focus-visible {
  @apply outline-none opacity-100;
  border-color: rgba(255, 255, 255, 0.72);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-accent, #2563eb) 45%, rgba(255, 255, 255, 0.25));
}

.image-viewer-thumb img {
  @apply h-full w-full rounded object-cover;
}

.image-viewer-thumb-index {
  @apply absolute bottom-1 right-1 min-w-4 rounded bg-slate-950/72 px-1 text-center text-[10px] font-semibold leading-4 text-slate-100;
}
</style>
