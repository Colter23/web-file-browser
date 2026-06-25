<script setup lang="ts">
import type {ComponentPublicInstance} from "vue";
import {computed, nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {DirEntryFilter, SearchScope} from "../../class.ts";
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
  searchScope: SearchScope;
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
  (e: "update:search-scope", value: SearchScope): void;
  (e: "search-enter"): void;
  (e: "search-escape"): void;
  (e: "clear-search"): void;
}>();

const breadcrumbRef = ref<BreadcrumbExpose | null>(null);
const searchOptionsMenuRef = ref<HTMLElement | null>(null);
const searchOptionsPanelRef = ref<HTMLElement | null>(null);
const searchOptionsButtonRef = ref<HTMLButtonElement | null>(null);
const searchOptionsMenuOpen = ref(false);

const searchTypeOptions: Array<{type: DirEntryFilter; label: string; icon: string; title: string}> = [
  {type: "all", label: "全部", icon: "action.search", title: "搜索全部项目"},
  {type: "file", label: "文件", icon: "file.file", title: "只搜索文件"},
  {type: "folder", label: "文件夹", icon: "file.folder", title: "只搜索文件夹"}
];

const searchScopeOptions: Array<{scope: SearchScope; label: string; icon: string; title: string}> = [
  {scope: "mount", label: "当前挂载", icon: "file.folder", title: "只搜索当前挂载点"},
  {scope: "all", label: "全部位置", icon: "file.home", title: "搜索所有挂载位置"}
];

const activeSearchType = computed(() => {
  return searchTypeOptions.find(option => option.type === props.searchType) ?? searchTypeOptions[0];
});

const activeSearchScope = computed(() => {
  return searchScopeOptions.find(option => option.scope === props.searchScope) ?? searchScopeOptions[0];
});

const searchOptionsActive = computed(() => {
  return searchOptionsMenuOpen.value || props.searchType !== "all" || props.searchScope !== "mount";
});

const closeSearchOptionsMenu = () => {
  searchOptionsMenuOpen.value = false;
}

const focusSearchOptionsButton = async () => {
  await nextTick();
  searchOptionsButtonRef.value?.focus({preventScroll: true});
}

const {
  focusMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef: searchOptionsPanelRef,
  onEscape: () => {
    closeSearchOptionsMenu();
    void focusSearchOptionsButton();
  }
});

const focusActiveSearchOption = async () => {
  await nextTick();
  const activeIndex = searchTypeOptions.findIndex(option => option.type === props.searchType);
  focusMenuButton(activeIndex >= 0 ? activeIndex : 0);
}

const toggleSearchOptionsMenu = async () => {
  searchOptionsMenuOpen.value = !searchOptionsMenuOpen.value;
  if (searchOptionsMenuOpen.value) await focusActiveSearchOption();
}

const selectSearchType = (type: DirEntryFilter) => {
  emit("update:search-type", type);
}

const selectSearchScope = (scope: SearchScope) => {
  emit("update:search-scope", scope);
}

const handleSearchOptionsButtonKeyDown = (event: KeyboardEvent) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (!searchOptionsMenuOpen.value) searchOptionsMenuOpen.value = true;
  void focusActiveSearchOption();
}

useOutsidePointerDown({
  refs: [searchOptionsMenuRef],
  enabled: () => searchOptionsMenuOpen.value,
  onOutsidePointerDown: closeSearchOptionsMenu
});

const handleDocumentKeyDown = (event: KeyboardEvent) => {
  if (event.key === "Escape") closeSearchOptionsMenu();
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
      <icon class="icon-motion-spin" icon="action.refresh" size="large" />
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
      <div ref="searchOptionsMenuRef" class="search-options-menu">
        <button
            ref="searchOptionsButtonRef"
            type="button"
            class="search-options-trigger"
            :class="{active: searchOptionsActive}"
            :title="`${activeSearchScope.label} · ${activeSearchType.title}`"
            aria-haspopup="menu"
            :aria-expanded="searchOptionsMenuOpen"
            @click.prevent="toggleSearchOptionsMenu"
            @keydown="handleSearchOptionsButtonKeyDown">
          <icon :icon="activeSearchType.icon" />
          <span>{{ activeSearchType.label }}</span>
          <icon class="search-options-caret icon-motion-caret" :class="{'is-open': searchOptionsMenuOpen}" icon="action.down" />
        </button>
        <div
            v-if="searchOptionsMenuOpen"
            ref="searchOptionsPanelRef"
            class="search-options-panel"
            role="menu"
            aria-label="搜索选项"
            @keydown="handleMenuKeyDown">
          <p class="search-options-title">类型</p>
          <button
              v-for="option in searchTypeOptions"
              :key="option.type"
              type="button"
              class="search-options-item"
              :class="{active: searchType === option.type}"
              role="menuitemradio"
              :aria-checked="searchType === option.type"
              tabindex="-1"
              @click.prevent="selectSearchType(option.type)">
            <span class="search-options-check">{{ searchType === option.type ? "✓" : "" }}</span>
            <icon :icon="option.icon" />
            <span>{{ option.label }}</span>
          </button>
          <div class="search-options-separator"></div>
          <p class="search-options-title">范围</p>
          <button
              v-for="option in searchScopeOptions"
              :key="option.scope"
              type="button"
              class="search-options-item"
              :class="{active: searchScope === option.scope}"
              role="menuitemradio"
              :aria-checked="searchScope === option.scope"
              tabindex="-1"
              :title="option.title"
              @click.prevent="selectSearchScope(option.scope)">
            <span class="search-options-check">{{ searchScope === option.scope ? "✓" : "" }}</span>
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

.search-options-menu {
  @apply relative shrink-0;
}

.search-options-trigger {
  @apply mr-0 inline-flex h-6 w-auto max-w-20 items-center gap-1 rounded-md border border-transparent px-1.5 text-[0.68rem];
  color: var(--app-text-subtle);
}

.search-options-trigger:hover {
  background: var(--app-control-hover);
  color: var(--app-text-muted);
}

.search-options-trigger.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-control-solid);
  color: var(--app-accent, #2563eb);
}

.search-options-trigger span {
  @apply min-w-0 truncate;
}

.search-options-caret {
  @apply shrink-0 text-[0.6rem];
}

.search-options-panel {
  @apply absolute right-0 top-[calc(100%+0.45rem)] z-50 w-44 overflow-hidden rounded-md border py-1;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.search-options-title {
  @apply px-3 py-1 text-[0.68rem] font-medium;
  color: var(--app-text-subtle);
}

.search-options-separator {
  @apply my-1 h-px;
  background: var(--app-border-soft);
}

.search-options-item {
  @apply mr-0 grid h-auto w-full grid-cols-[1rem_1rem_minmax(0,1fr)] items-center gap-2 rounded-none px-2.5 py-1.5 text-left text-sm;
  color: var(--app-text-muted);
}

.search-options-item:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.search-options-item.active {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.search-options-check {
  @apply text-center text-xs font-semibold;
  color: var(--app-accent, #2563eb);
}

.search-clear-button:hover {
  background: var(--app-control-hover);
  color: var(--app-text-muted);
}

.search-clear-button:focus-visible,
.search-options-trigger:focus-visible,
.search-options-item:focus-visible {
  @apply outline-none;
  background: var(--app-control-hover);
  box-shadow: inset 0 0 0 1px var(--app-accent, #2563eb);
}

.search-options-item:focus-visible {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}
</style>
