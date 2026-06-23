<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {DirSortKey, DirSortOrder} from "../../class";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import Icon from "../Icon.vue";

type SortOption = {
  key: DirSortKey;
  label: string;
  description: string;
}

type OrderOption = {
  key: DirSortOrder;
  label: string;
  description: string;
}

const props = defineProps<{
  sortKey: DirSortKey;
  sortOrder: DirSortOrder;
}>();

const emit = defineEmits<{
  (e: "set-sort-key", key: DirSortKey): void;
  (e: "set-sort-order", order: DirSortOrder): void;
}>();

const sortMenuRef = ref<HTMLElement | null>(null);
const sortMenuPanelRef = ref<HTMLElement | null>(null);
const sortButtonRef = ref<HTMLButtonElement | null>(null);
const open = ref(false);

const sortOptions: SortOption[] = [
  {key: "name", label: "名称", description: "按文件名排序"},
  {key: "modified", label: "修改日期", description: "按最近修改时间排序"},
  {key: "size", label: "大小", description: "按文件大小排序"}
];

const orderOptions: OrderOption[] = [
  {key: "asc", label: "升序", description: "从小到大或从早到晚"},
  {key: "desc", label: "降序", description: "从大到小或从晚到早"}
];

const activeSortLabel = computed(() => sortOptions.find(option => option.key === props.sortKey)?.label ?? "名称");
const activeOrderLabel = computed(() => props.sortOrder === "asc" ? "升序" : "降序");
const buttonTitle = computed(() => `排序：${activeSortLabel.value} ${activeOrderLabel.value}`);

const close = () => {
  open.value = false;
}

const focusButton = async () => {
  await nextTick();
  sortButtonRef.value?.focus({preventScroll: true});
}

const {
  focusMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef: sortMenuPanelRef,
  onEscape: () => {
    close();
    void focusButton();
  }
});

const focusActiveOption = async () => {
  await nextTick();
  const activeIndex = sortOptions.findIndex(option => option.key === props.sortKey);
  focusMenuButton(activeIndex >= 0 ? activeIndex : 0);
}

const openMenu = () => {
  open.value = true;
  void focusActiveOption();
}

const toggle = async () => {
  open.value = !open.value;
  if (open.value) await focusActiveOption();
}

const selectSortKey = (key: DirSortKey) => {
  close();
  emit("set-sort-key", key);
}

const selectSortOrder = (order: DirSortOrder) => {
  close();
  emit("set-sort-order", order);
}

const handleButtonKeyDown = (event: KeyboardEvent) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (!open.value) openMenu();
  else void focusActiveOption();
}

useOutsidePointerDown({
  refs: [sortMenuRef],
  enabled: () => open.value,
  onOutsidePointerDown: close
});

const handleDocumentKeyDown = (event: KeyboardEvent) => {
  if (event.key === "Escape") close();
}

onMounted(() => {
  window.addEventListener("keydown", handleDocumentKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleDocumentKeyDown);
});
</script>

<template>
  <div ref="sortMenuRef" class="sort-menu">
    <button
        ref="sortButtonRef"
        class="sort-button"
        :class="{active: open}"
        :title="buttonTitle"
        aria-haspopup="menu"
        :aria-expanded="open"
        @click="toggle"
        @keydown="handleButtonKeyDown">
      <icon icon="icon-view-list" />
      <span>{{ activeSortLabel }} {{ activeOrderLabel }}</span>
      <icon icon="icon-unfold" class="sort-caret" />
    </button>
    <div v-if="open" ref="sortMenuPanelRef" class="sort-menu-panel" role="menu" aria-label="排序方式" @keydown="handleMenuKeyDown">
      <p class="menu-group-title">排序方式</p>
      <button
          v-for="option in sortOptions"
          :key="option.key"
          class="sort-menu-item"
          :class="{active: sortKey === option.key}"
          role="menuitemradio"
          :aria-checked="sortKey === option.key"
          tabindex="-1"
          @click="selectSortKey(option.key)">
        <span class="sort-check">{{ sortKey === option.key ? "✓" : "" }}</span>
        <span class="sort-menu-copy">
          <strong>{{ option.label }}</strong>
          <small>{{ option.description }}</small>
        </span>
      </button>
      <div class="menu-separator"></div>
      <p class="menu-group-title">顺序</p>
      <button
          v-for="option in orderOptions"
          :key="option.key"
          class="sort-menu-item"
          :class="{active: sortOrder === option.key}"
          role="menuitemradio"
          :aria-checked="sortOrder === option.key"
          tabindex="-1"
          @click="selectSortOrder(option.key)">
        <span class="sort-check">{{ sortOrder === option.key ? "✓" : "" }}</span>
        <span class="sort-menu-copy">
          <strong>{{ option.label }}</strong>
          <small>{{ option.description }}</small>
        </span>
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.sort-menu {
  @apply relative shrink-0;
}

.sort-button {
  @apply inline-flex h-10 shrink-0 items-center justify-center gap-2 rounded-lg border border-slate-200 bg-white px-3 text-sm text-slate-700 hover:bg-blue-50;
}

.sort-button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.sort-caret {
  @apply text-[0.65rem] text-slate-500;
}

.sort-menu-panel {
  @apply absolute right-0 top-[calc(100%+0.35rem)] z-30 w-72 overflow-hidden rounded-lg border border-slate-200 bg-white py-1 shadow-xl;
}

.menu-group-title {
  @apply px-3 py-1 text-[0.68rem] font-medium text-slate-400;
}

.menu-separator {
  @apply my-1 h-px bg-slate-100;
}

.sort-menu-item {
  @apply grid w-full grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-3 px-3 py-2 text-left text-sm text-slate-700 hover:bg-blue-50;
}

.sort-menu-item.active {
  @apply bg-blue-50 text-blue-700;
}

.sort-menu-item:focus-visible {
  @apply bg-blue-50 text-blue-700 outline-none ring-1 ring-inset ring-blue-300;
}

.sort-check {
  @apply text-center text-xs font-semibold text-blue-600;
}

.sort-menu-copy {
  @apply flex min-w-0 flex-col;
}

.sort-menu-copy strong {
  @apply truncate text-sm font-medium;
}

.sort-menu-copy small {
  @apply truncate text-xs text-slate-500;
}

.sort-menu-item.active .sort-menu-copy small {
  @apply text-blue-600;
}
</style>
