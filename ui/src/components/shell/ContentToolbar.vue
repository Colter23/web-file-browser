<script setup lang="ts">
import type {ComponentPublicInstance} from "vue";
import {computed, nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {DirEntryFilter, SearchScope} from "../../class.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import type {MessageKey} from "../../i18n";
import {useI18n} from "../../i18n";
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

const {t} = useI18n();
const breadcrumbRef = ref<BreadcrumbExpose | null>(null);
const searchOptionsMenuRef = ref<HTMLElement | null>(null);
const searchOptionsPanelRef = ref<HTMLElement | null>(null);
const searchOptionsButtonRef = ref<HTMLButtonElement | null>(null);
const searchOptionsMenuOpen = ref(false);

const searchTypeOptions: Array<{type: DirEntryFilter; labelKey: MessageKey; icon: string; titleKey: MessageKey}> = [
  {type: "all", labelKey: "search.typeAll", icon: "action.search", titleKey: "search.typeAllTitle"},
  {type: "file", labelKey: "search.typeFile", icon: "file.file", titleKey: "search.typeFileTitle"},
  {type: "folder", labelKey: "search.typeFolder", icon: "file.folder", titleKey: "search.typeFolderTitle"}
];

const searchScopeOptions: Array<{scope: SearchScope; labelKey: MessageKey; icon: string; titleKey: MessageKey}> = [
  {scope: "mount", labelKey: "search.scopeMount", icon: "file.folder", titleKey: "search.scopeMountTitle"},
  {scope: "all", labelKey: "search.scopeAll", icon: "file.home", titleKey: "search.scopeAllTitle"}
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
    <div class="nav-cluster" role="group" :aria-label="t('nav.group')">
      <button class="nav-button nav-button-arrow" :disabled="!canNavigateBack" :title="navigateBackTitle" @click="emit('navigate-back')">
        <span class="nav-icon-motion nav-icon-motion-back">
          <icon icon="nav.back" size="1.42rem" :stroke-width="2.15" />
        </span>
      </button>
      <button class="nav-button nav-button-arrow" :disabled="!canNavigateForward" :title="navigateForwardTitle" @click="emit('navigate-forward')">
        <span class="nav-icon-motion nav-icon-motion-forward">
          <icon icon="nav.forward" size="1.42rem" :stroke-width="2.15" />
        </span>
      </button>
      <button class="nav-button nav-button-arrow" :disabled="!canNavigateUp" :title="navigateUpTitle" @click="emit('navigate-up')">
        <span class="nav-icon-motion nav-icon-motion-up">
          <icon icon="nav.up" size="1.42rem" :stroke-width="2.15" />
        </span>
      </button>
      <button class="nav-button nav-button-tool" :title="t('nav.refresh')" @click="emit('refresh')">
        <icon class="icon-motion-spin" icon="nav.refresh" size="1.32rem" />
      </button>
      <button class="nav-button nav-button-tool" :title="t('nav.recent')" @click="emit('show-recent')">
        <icon icon="nav.recent" size="1.32rem" />
      </button>
    </div>
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
          :placeholder="t('search.placeholder')"
          :aria-label="t('search.placeholder')"
          :title="t('search.inputTitle')"
          @input="emit('update:search-text', ($event.target as HTMLInputElement).value)"
          @keydown.enter.prevent="emit('search-enter')"
          @keydown.escape.prevent="emit('search-escape')">
      <div ref="searchOptionsMenuRef" class="search-options-menu">
        <button
            ref="searchOptionsButtonRef"
            type="button"
            class="search-options-trigger"
            :class="{active: searchOptionsActive}"
            :title="`${t(activeSearchScope.labelKey)} · ${t(activeSearchType.titleKey)}`"
            aria-haspopup="menu"
            :aria-expanded="searchOptionsMenuOpen"
            @click.prevent="toggleSearchOptionsMenu"
            @keydown="handleSearchOptionsButtonKeyDown">
          <span class="search-options-trigger-label">{{ t(activeSearchType.labelKey) }}</span>
          <icon class="search-options-caret icon-motion-caret" :class="{'is-open': searchOptionsMenuOpen}" icon="action.down" />
        </button>
        <div
            v-if="searchOptionsMenuOpen"
            ref="searchOptionsPanelRef"
            class="search-options-panel"
            role="menu"
            :aria-label="t('search.options')"
            @keydown="handleMenuKeyDown">
          <p class="search-options-title">{{ t("search.type") }}</p>
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
            <span class="search-options-icon"><icon :icon="option.icon" /></span>
            <span>{{ t(option.labelKey) }}</span>
            <span class="search-options-check"><icon v-if="searchType === option.type" icon="action.check" size="small" /></span>
          </button>
          <div class="search-options-separator"></div>
          <p class="search-options-title">{{ t("search.scope") }}</p>
          <button
              v-for="option in searchScopeOptions"
              :key="option.scope"
              type="button"
              class="search-options-item"
              :class="{active: searchScope === option.scope}"
              role="menuitemradio"
              :aria-checked="searchScope === option.scope"
              tabindex="-1"
              :title="t(option.titleKey)"
              @click.prevent="selectSearchScope(option.scope)">
            <span class="search-options-icon"><icon :icon="option.icon" /></span>
            <span>{{ t(option.labelKey) }}</span>
            <span class="search-options-check"><icon v-if="searchScope === option.scope" icon="action.check" size="small" /></span>
          </button>
        </div>
      </div>
      <button v-if="isFiltering" type="button" class="search-clear-button" :title="t('search.clear')" @click.prevent="emit('clear-search')">
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

.nav-cluster {
  @apply flex h-9 shrink-0 items-center gap-0.5 rounded-md px-0.5;
  background: transparent;
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

.nav-icon-motion {
  @apply inline-flex items-center justify-center;
  --nav-motion-x: 0px;
  --nav-motion-y: 0px;
  transition: transform 0.16s ease;
}

.nav-icon-motion-back {
  --nav-motion-x: -1.8px;
}

.nav-icon-motion-forward {
  --nav-motion-x: 1.8px;
}

.nav-icon-motion-up {
  --nav-motion-y: -1.8px;
}

.nav-button:hover:not(:disabled) .nav-icon-motion,
.nav-button:focus-visible:not(:disabled) .nav-icon-motion {
  transform: translate3d(var(--nav-motion-x), var(--nav-motion-y), 0);
}

.nav-button:active:not(:disabled) .nav-icon-motion {
  transform: translate3d(calc(var(--nav-motion-x) * 1.45), calc(var(--nav-motion-y) * 1.45), 0) scale(0.95);
  transition-duration: 0.08s;
}

.search-box {
  @apply relative flex h-9 min-w-56 w-[18.5rem] max-w-[26vw] shrink items-center gap-1.5 rounded-md border py-0 pl-2 pr-1 shadow-[inset_0_0_0_1px_rgba(255,255,255,0.08)] transition-[width,max-width,border-color,box-shadow,background-color] duration-150 ease-out;
  border-color: var(--app-border);
  background: var(--app-control-solid);
  color: var(--app-text-subtle);
}

.search-box.active,
.search-box:focus-within {
  @apply w-[25rem] max-w-[36vw];
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
  @apply mr-0 inline-flex h-7 w-auto max-w-20 items-center gap-1 rounded-md border px-2 text-xs font-medium leading-none transition-colors;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-control-solid) 64%, transparent);
  color: var(--app-text-subtle);
}

.search-options-trigger:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
  color: var(--app-text-muted);
}

.search-options-trigger.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 10%, var(--app-control-solid));
  color: var(--app-accent, #2563eb);
}

.search-options-trigger-label {
  @apply min-w-0 truncate;
}

.search-options-caret {
  @apply shrink-0 text-[0.6rem] leading-none;
}

.search-options-panel {
  @apply absolute right-0 top-[calc(100%+0.45rem)] z-50 w-48 overflow-hidden rounded-md border p-1.5;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.search-options-title {
  @apply px-1.5 pb-1 pt-0.5 text-[0.68rem] font-medium;
  color: var(--app-text-subtle);
}

.search-options-separator {
  @apply my-1 h-px;
  background: var(--app-border-soft);
}

.search-options-item {
  @apply mr-0 grid h-auto w-full grid-cols-[1.75rem_minmax(0,1fr)_1.25rem] items-center gap-2 rounded-md border border-transparent px-1.5 py-1.5 text-left text-sm;
  color: var(--app-text-muted);
}

.search-options-item:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
}

.search-options-item.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, var(--app-panel-solid));
  color: var(--app-accent, #2563eb);
}

.search-options-icon {
  @apply grid size-7 place-items-center rounded;
  background: var(--app-control);
  color: var(--app-text-muted);
}

.search-options-item.active .search-options-icon {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 14%, transparent);
  color: var(--app-accent, #2563eb);
}

.search-options-check {
  @apply grid size-5 place-items-center rounded-full text-xs font-semibold;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, transparent);
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
  box-shadow: inset 0 0 0 1px var(--app-accent, #2563eb);
}

.search-options-item:focus-visible {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

@media (max-width: 1180px) {
  .search-box,
  .search-box.active,
  .search-box:focus-within {
    @apply min-w-48 w-[16rem] max-w-[24vw];
  }

  .search-options-trigger-label {
    @apply sr-only;
  }
}

@media (prefers-reduced-motion: reduce) {
  .nav-icon-motion {
    transition: none;
  }

  .nav-button:hover:not(:disabled) .nav-icon-motion,
  .nav-button:focus-visible:not(:disabled) .nav-icon-motion,
  .nav-button:active:not(:disabled) .nav-icon-motion {
    transform: none;
  }
}
</style>
