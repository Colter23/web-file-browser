<script setup lang="ts">
import type {ComponentPublicInstance} from "vue";
import {ref} from "vue";
import Breadcrumb from "../Breadcrumb.vue";
import Icon from "../Icon.vue";

type BreadcrumbExpose = {
  focusInput: () => void;
}

type NavigateComplete = (navigated: boolean) => void;

defineProps<{
  canNavigateBack: boolean;
  canNavigateForward: boolean;
  canNavigateUp: boolean;
  navigateBackTitle: string;
  navigateForwardTitle: string;
  navigateUpTitle: string;
  searchText: string;
  isFiltering: boolean;
  setSearchInputRef: (element: Element | ComponentPublicInstance | null) => void;
}>();

const emit = defineEmits<{
  (e: "navigate-back"): void;
  (e: "navigate-forward"): void;
  (e: "navigate-up"): void;
  (e: "refresh"): void;
  (e: "breadcrumb-navigate", path: string, complete?: NavigateComplete): void;
  (e: "update:search-text", value: string): void;
  (e: "search-enter"): void;
  (e: "search-escape"): void;
  (e: "clear-search"): void;
}>();

const breadcrumbRef = ref<BreadcrumbExpose | null>(null);

defineExpose({
  focusInput: () => breadcrumbRef.value?.focusInput()
});
</script>

<template>
  <div class="path-row">
    <button class="nav-button" :disabled="!canNavigateBack" :title="navigateBackTitle" @click="emit('navigate-back')">
      <icon icon="action.previous" size="large" />
    </button>
    <button class="nav-button" :disabled="!canNavigateForward" :title="navigateForwardTitle" @click="emit('navigate-forward')">
      <icon icon="action.next" size="large" />
    </button>
    <button class="nav-button" :disabled="!canNavigateUp" :title="navigateUpTitle" @click="emit('navigate-up')">
      <icon icon="action.up" size="large" />
    </button>
    <button class="nav-button" title="刷新 (F5 / Ctrl+R)" @click="emit('refresh')">
      <icon icon="action.refresh" size="large" />
    </button>
    <breadcrumb ref="breadcrumbRef" @navigate="(path, complete) => emit('breadcrumb-navigate', path, complete)"></breadcrumb>
    <label class="search-box" :class="{active: isFiltering}">
      <input
          :ref="setSearchInputRef"
          :value="searchText"
          type="search"
          placeholder="搜索当前文件夹"
          aria-label="搜索当前文件夹"
          title="搜索当前文件夹 (Ctrl+F / Ctrl+E)"
          @input="emit('update:search-text', ($event.target as HTMLInputElement).value)"
          @keydown.enter.prevent="emit('search-enter')"
          @keydown.escape.prevent="emit('search-escape')">
      <button v-if="isFiltering" type="button" title="清除筛选" @click.prevent="emit('clear-search')">
        <icon icon="action.close" />
      </button>
      <icon v-else icon="action.search" />
    </label>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.path-row {
  @apply flex h-12 shrink-0 items-center gap-2 border-b border-[#d8e3ee] bg-[#f8fbff] px-3;
}

.nav-button {
  @apply inline-flex items-center justify-center rounded-md border border-transparent bg-transparent text-slate-700 hover:border-slate-200 hover:bg-white;
}

.nav-button {
  @apply h-9 w-9 shrink-0;
}

.nav-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:border-transparent hover:bg-transparent;
}

.search-box {
  @apply flex h-9 w-72 max-w-[30vw] shrink-0 items-center gap-2 rounded-md border border-[#d7e1ec] bg-white px-3 text-slate-500 shadow-[inset_0_0_0_1px_rgba(255,255,255,0.55)];
}

.search-box.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.55), 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.search-box input {
  @apply min-w-0 grow bg-transparent text-sm text-slate-700 outline-none placeholder:text-slate-400;
}

.search-box button {
  @apply -mr-1 inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-slate-400 hover:bg-slate-100 hover:text-slate-700;
}
</style>
