<script setup lang="ts">
import {nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {ExplorerIconSize, ExplorerViewMode} from "../../class";
import type {ExplorerViewModeSelection} from "../../composables/useExplorerViewMode.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import Icon from "../Icon.vue";

type ViewModeOption = {
  key: string;
  mode: ExplorerViewMode;
  iconSize?: ExplorerIconSize;
  label: string;
  description: string;
  icon: string;
  shortcut: string;
}

const props = defineProps<{
  icon: string;
  label: string;
  title: string;
  viewMode: ExplorerViewMode;
  iconSize: ExplorerIconSize;
}>();

const emit = defineEmits<{
  (e: "select", selection: ExplorerViewModeSelection): void;
}>();

const viewMenuRef = ref<HTMLElement | null>(null);
const viewMenuPanelRef = ref<HTMLElement | null>(null);
const viewModeButtonRef = ref<HTMLButtonElement | null>(null);
const open = ref(false);

const viewModeOptions: ViewModeOption[] = [
  {
    key: "details",
    mode: "details",
    iconSize: "small",
    label: "详细信息",
    description: "显示日期、类型和大小",
    icon: "view.details",
    shortcut: "Ctrl+Shift+6"
  },
  {
    key: "list",
    mode: "list",
    iconSize: "small",
    label: "列表",
    description: "紧凑排列，快速扫描",
    icon: "view.list",
    shortcut: "Ctrl+Shift+5"
  },
  {
    key: "tiles",
    mode: "tiles",
    iconSize: "medium",
    label: "平铺",
    description: "图标与文件信息并列",
    icon: "view.tiles",
    shortcut: "Ctrl+Shift+7"
  },
  {
    key: "icons-large",
    mode: "icons",
    iconSize: "large",
    label: "大图标",
    description: "适合浏览图片和媒体",
    icon: "view.icons",
    shortcut: "Ctrl+Shift+1/2"
  },
  {
    key: "icons-medium",
    mode: "icons",
    iconSize: "medium",
    label: "中图标",
    description: "兼顾预览和密度",
    icon: "view.icons",
    shortcut: "Ctrl+Shift+3"
  },
  {
    key: "icons-small",
    mode: "icons",
    iconSize: "small",
    label: "小图标",
    description: "更多项目同屏展示",
    icon: "view.icons",
    shortcut: "Ctrl+Shift+4"
  }
];

const close = () => {
  open.value = false;
}

const focusButton = async () => {
  await nextTick();
  viewModeButtonRef.value?.focus({preventScroll: true});
}

const {
  focusMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef: viewMenuPanelRef,
  onEscape: () => {
    close();
    void focusButton();
  }
});

const isActive = (option: ViewModeOption) => {
  if (option.mode !== props.viewMode) return false;
  return option.mode !== "icons" || option.iconSize === props.iconSize;
}

const focusActiveOption = async () => {
  await nextTick();
  const activeIndex = viewModeOptions.findIndex(isActive);
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

const select = (option: ViewModeOption) => {
  close();
  emit("select", {
    mode: option.mode,
    iconSize: option.iconSize,
    label: option.label
  });
}

const handleButtonKeyDown = (event: KeyboardEvent) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (!open.value) openMenu();
  else void focusActiveOption();
}

useOutsidePointerDown({
  refs: [viewMenuRef],
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
  <div ref="viewMenuRef" class="view-menu">
    <button
        ref="viewModeButtonRef"
        class="view-button"
        :class="{active: open}"
        :title="title"
        aria-haspopup="menu"
        :aria-expanded="open"
        @click="toggle"
        @keydown="handleButtonKeyDown">
      <icon :icon="icon" />
      <span>{{ label }}</span>
      <icon icon="action.down" class="view-caret icon-motion-caret" :class="{'is-open': open}" />
    </button>
    <div v-if="open" ref="viewMenuPanelRef" class="view-menu-panel" role="menu" aria-label="查看模式" @keydown="handleMenuKeyDown">
      <button
          v-for="option in viewModeOptions"
          :key="option.key"
          class="view-menu-item"
          :class="{active: isActive(option)}"
          role="menuitemradio"
          :aria-checked="isActive(option)"
          tabindex="-1"
          @click="select(option)">
        <icon :icon="option.icon" />
        <span class="view-menu-copy">
          <strong>{{ option.label }}</strong>
          <small>{{ option.description }}</small>
        </span>
        <kbd>{{ option.shortcut }}</kbd>
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.view-menu {
  @apply relative shrink-0;
}

.view-button {
  @apply inline-flex h-10 shrink-0 items-center justify-center gap-2 rounded-md border px-3 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.view-button:hover {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-hover, #eff6ff);
}

.view-button.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.view-caret {
  @apply text-[0.65rem];
  color: var(--app-text-subtle);
}

.view-menu-panel {
  @apply absolute right-0 top-[calc(100%+0.35rem)] z-50 w-72 overflow-hidden rounded-md border py-1;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.view-menu-item {
  @apply grid w-full grid-cols-[1.25rem_minmax(0,1fr)_auto] items-center gap-3 px-3 py-2 text-left text-sm;
  color: var(--app-text-muted);
}

.view-menu-item:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.view-menu-item.active {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.view-menu-item:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.view-menu-copy {
  @apply flex min-w-0 flex-col;
}

.view-menu-copy strong {
  @apply truncate text-sm font-medium;
}

.view-menu-copy small {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.view-menu-item.active .view-menu-copy small {
  color: var(--app-accent, #2563eb);
}

.view-menu-item kbd {
  @apply rounded border px-1.5 py-0.5 text-[0.68rem] font-normal;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
  color: var(--app-text-subtle);
}
</style>
