<script setup lang="ts">
import type {ComponentPublicInstance} from "vue";
import {ref} from "vue";
import Breadcrumb from "../Breadcrumb.vue";
import Icon from "../Icon.vue";
import type {ExplorerEntry} from "../explorer/types.ts";

type BreadcrumbExpose = {
  focusInput: () => void;
}

type NavigateComplete = (navigated: boolean) => void;
type BreadcrumbDropPayload = {
  entries: ExplorerEntry[];
  target: {
    path: string;
    name: string;
  };
  action: "copy" | "move";
}

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
  (e: "breadcrumb-drop", payload: BreadcrumbDropPayload): void;
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
    <breadcrumb
        ref="breadcrumbRef"
        @navigate="(path, complete) => emit('breadcrumb-navigate', path, complete)"
        @drop-entries="payload => emit('breadcrumb-drop', payload)">
    </breadcrumb>
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
  @apply flex h-12 shrink-0 items-center gap-2 border-b px-3;
  border-color: var(--app-border);
  background: var(--app-panel-muted);
}

.nav-button {
  @apply inline-flex items-center justify-center rounded-md border border-transparent bg-transparent;
  color: var(--app-text-muted);
}

.nav-button:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
}

.nav-button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.nav-button {
  @apply h-9 w-9 shrink-0;
}

.nav-button:disabled {
  @apply cursor-not-allowed hover:border-transparent hover:bg-transparent;
  color: var(--app-text-disabled);
}

.search-box {
  @apply flex h-9 w-72 max-w-[30vw] shrink-0 items-center gap-2 rounded-md border px-3 shadow-[inset_0_0_0_1px_rgba(255,255,255,0.08)];
  border-color: var(--app-border);
  background: var(--app-control-solid);
  color: var(--app-text-subtle);
}

.search-box.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.55), 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.search-box:focus-within {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.search-box input {
  @apply min-w-0 grow bg-transparent text-sm outline-none;
  color: var(--app-text-muted);
}

.search-box input::placeholder {
  color: var(--app-text-subtle);
}

.search-box button {
  @apply -mr-1 inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md;
  color: var(--app-text-subtle);
}

.search-box button:hover {
  background: var(--app-control-hover);
  color: var(--app-text-muted);
}

.search-box button:focus-visible {
  @apply outline-none;
  background: var(--app-control-hover);
  box-shadow: inset 0 0 0 1px var(--app-accent, #2563eb);
}
</style>
