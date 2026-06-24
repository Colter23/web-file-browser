<script setup lang="ts">
import type {ComponentPublicInstance} from "vue";
import {computed, nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {DirEntryFilter} from "../../class.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import Breadcrumb from "../Breadcrumb.vue";
import Icon from "../Icon.vue";
import type {ExplorerEntryPathDropPayload} from "../explorer/types.ts";

type BreadcrumbExpose = {
  focusInput: () => void;
}

type NavigateComplete = (navigated: boolean) => void;

const props = defineProps<{
  canNavigateBack: boolean;
  canNavigateForward: boolean;
  canNavigateUp: boolean;
  navigateBackTitle: string;
  navigateForwardTitle: string;
  navigateUpTitle: string;
  searchText: string;
  isFiltering: boolean;
  searchType: DirEntryFilter;
  setSearchInputRef: (element: Element | ComponentPublicInstance | null) => void;
}>();

const emit = defineEmits<{
  (e: "navigate-back"): void;
  (e: "navigate-forward"): void;
  (e: "navigate-up"): void;
  (e: "refresh"): void;
  (e: "show-recent"): void;
  (e: "breadcrumb-navigate", path: string, complete?: NavigateComplete): void;
  (e: "breadcrumb-drop", payload: ExplorerEntryPathDropPayload): void;
  (e: "update:search-text", value: string): void;
  (e: "update:search-type", value: DirEntryFilter): void;
  (e: "search-enter"): void;
  (e: "search-escape"): void;
  (e: "clear-search"): void;
}>();

const breadcrumbRef = ref<BreadcrumbExpose | null>(null);
const searchTypeMenuRef = ref<HTMLElement | null>(null);
const searchTypePanelRef = ref<HTMLElement | null>(null);
const searchTypeButtonRef = ref<HTMLButtonElement | null>(null);
const searchTypeMenuOpen = ref(false);

const searchTypeOptions: Array<{type: DirEntryFilter; label: string; icon: string; title: string}> = [
  {type: "all", label: "全部", icon: "action.search", title: "搜索全部项目"},
  {type: "file", label: "文件", icon: "file.file", title: "只搜索文件"},
  {type: "folder", label: "文件夹", icon: "file.folder", title: "只搜索文件夹"}
];

const activeSearchType = computed(() => {
  return searchTypeOptions.find(option => option.type === props.searchType) ?? searchTypeOptions[0];
});

const closeSearchTypeMenu = () => {
  searchTypeMenuOpen.value = false;
}

const focusSearchTypeButton = async () => {
  await nextTick();
  searchTypeButtonRef.value?.focus({preventScroll: true});
}

const {
  focusMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef: searchTypePanelRef,
  onEscape: () => {
    closeSearchTypeMenu();
    void focusSearchTypeButton();
  }
});

const focusActiveSearchType = async () => {
  await nextTick();
  const activeIndex = searchTypeOptions.findIndex(option => option.type === props.searchType);
  focusMenuButton(activeIndex >= 0 ? activeIndex : 0);
}

const toggleSearchTypeMenu = async () => {
  searchTypeMenuOpen.value = !searchTypeMenuOpen.value;
  if (searchTypeMenuOpen.value) await focusActiveSearchType();
}

const selectSearchType = (type: DirEntryFilter) => {
  closeSearchTypeMenu();
  emit("update:search-type", type);
}

const handleSearchTypeButtonKeyDown = (event: KeyboardEvent) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (!searchTypeMenuOpen.value) searchTypeMenuOpen.value = true;
  void focusActiveSearchType();
}

useOutsidePointerDown({
  refs: [searchTypeMenuRef],
  enabled: () => searchTypeMenuOpen.value,
  onOutsidePointerDown: closeSearchTypeMenu
});

const handleDocumentKeyDown = (event: KeyboardEvent) => {
  if (event.key === "Escape") closeSearchTypeMenu();
}

onMounted(() => {
  window.addEventListener("keydown", handleDocumentKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleDocumentKeyDown);
});

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
    <button class="nav-button" title="最近文件" @click="emit('show-recent')">
      <icon icon="action.recent" size="large" />
    </button>
    <breadcrumb
        ref="breadcrumbRef"
        @navigate="(path, complete) => emit('breadcrumb-navigate', path, complete)"
        @drop-entries="payload => emit('breadcrumb-drop', payload)">
    </breadcrumb>
    <div class="search-box" :class="{active: isFiltering}" role="search">
      <icon class="search-leading-icon" icon="action.search" />
      <input
          :ref="setSearchInputRef"
          :value="searchText"
          type="search"
          placeholder="搜索当前文件夹"
          aria-label="搜索当前文件夹"
          title="输入后筛选当前文件夹，按 Enter 搜索索引 (Ctrl+F / Ctrl+E)"
          @input="emit('update:search-text', ($event.target as HTMLInputElement).value)"
          @keydown.enter.prevent="emit('search-enter')"
          @keydown.escape.prevent="emit('search-escape')">
      <div ref="searchTypeMenuRef" class="search-type-menu">
        <button
            ref="searchTypeButtonRef"
            type="button"
            class="search-type-trigger"
            :class="{active: searchTypeMenuOpen || searchType !== 'all'}"
            :title="activeSearchType.title"
            aria-haspopup="menu"
            :aria-expanded="searchTypeMenuOpen"
            @click.prevent="toggleSearchTypeMenu"
            @keydown="handleSearchTypeButtonKeyDown">
          <icon :icon="activeSearchType.icon" />
          <span>{{ activeSearchType.label }}</span>
          <icon class="search-type-caret" icon="action.down" />
        </button>
        <div
            v-if="searchTypeMenuOpen"
            ref="searchTypePanelRef"
            class="search-type-panel"
            role="menu"
            aria-label="搜索类型"
            @keydown="handleMenuKeyDown">
          <button
              v-for="option in searchTypeOptions"
              :key="option.type"
              type="button"
              class="search-type-option"
              :class="{active: searchType === option.type}"
              role="menuitemradio"
              :aria-checked="searchType === option.type"
              tabindex="-1"
              @click.prevent="selectSearchType(option.type)">
            <span class="search-type-check">{{ searchType === option.type ? "✓" : "" }}</span>
            <icon :icon="option.icon" />
            <span>{{ option.label }}</span>
          </button>
        </div>
      </div>
      <button v-if="isFiltering" type="button" class="search-clear-button" title="清除筛选" @click.prevent="emit('clear-search')">
        <icon icon="action.close" />
      </button>
    </div>
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
  @apply relative flex h-9 min-w-64 w-[24rem] max-w-[34vw] shrink items-center gap-1.5 rounded-md border px-2 shadow-[inset_0_0_0_1px_rgba(255,255,255,0.08)];
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
  @apply min-w-20 grow bg-transparent text-sm outline-none;
  color: var(--app-text-muted);
}

.search-box input::placeholder {
  color: var(--app-text-subtle);
}

.search-clear-button {
  @apply -mr-1 inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md;
  color: var(--app-text-subtle);
}

.search-leading-icon {
  @apply shrink-0;
}

.search-type-menu {
  @apply relative shrink-0;
}

.search-type-trigger {
  @apply mr-0 inline-flex h-6 w-auto max-w-20 items-center gap-1 rounded-md border border-transparent px-1.5 text-[0.68rem];
  color: var(--app-text-subtle);
}

.search-type-trigger:hover {
  background: var(--app-control-hover);
  color: var(--app-text-muted);
}

.search-type-trigger.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-control-solid);
  color: var(--app-accent, #2563eb);
}

.search-type-trigger span {
  @apply min-w-0 truncate;
}

.search-type-caret {
  @apply shrink-0 text-[0.6rem];
}

.search-type-panel {
  @apply absolute right-0 top-[calc(100%+0.45rem)] z-50 w-36 overflow-hidden rounded-md border py-1;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.search-type-option {
  @apply mr-0 grid h-auto w-full grid-cols-[1rem_1rem_minmax(0,1fr)] items-center gap-2 rounded-none px-2.5 py-1.5 text-left text-sm;
  color: var(--app-text-muted);
}

.search-type-option:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.search-type-option.active {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.search-type-check {
  @apply text-center text-xs font-semibold;
  color: var(--app-accent, #2563eb);
}

.search-clear-button:hover {
  background: var(--app-control-hover);
  color: var(--app-text-muted);
}

.search-clear-button:focus-visible,
.search-type-trigger:focus-visible,
.search-type-option:focus-visible {
  @apply outline-none;
  background: var(--app-control-hover);
  box-shadow: inset 0 0 0 1px var(--app-accent, #2563eb);
}

.search-type-option:focus-visible {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}
</style>
