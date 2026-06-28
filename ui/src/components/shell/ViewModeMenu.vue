<script setup lang="ts">
import {nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {ExplorerIconSize, ExplorerViewMode} from "../../class";
import type {ExplorerViewModeSelection} from "../../composables/useExplorerViewMode.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import type {MessageKey} from "../../i18n";
import {useI18n} from "../../i18n";
import Icon from "../Icon.vue";

type ViewModeOption = {
  key: string;
  mode: ExplorerViewMode;
  iconSize?: ExplorerIconSize;
  labelKey: MessageKey;
  descriptionKey: MessageKey;
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

const {t} = useI18n();
const viewMenuRef = ref<HTMLElement | null>(null);
const viewMenuPanelRef = ref<HTMLElement | null>(null);
const viewModeButtonRef = ref<HTMLButtonElement | null>(null);
const open = ref(false);

const viewModeOptions: ViewModeOption[] = [
  {
    key: "details",
    mode: "details",
    iconSize: "small",
    labelKey: "view.details",
    descriptionKey: "view.detailsDescription",
    icon: "view.details",
    shortcut: "Ctrl+Shift+6"
  },
  {
    key: "list",
    mode: "list",
    iconSize: "small",
    labelKey: "view.list",
    descriptionKey: "view.listDescription",
    icon: "view.list",
    shortcut: "Ctrl+Shift+5"
  },
  {
    key: "tiles",
    mode: "tiles",
    iconSize: "medium",
    labelKey: "view.tiles",
    descriptionKey: "view.tilesDescription",
    icon: "view.tiles",
    shortcut: "Ctrl+Shift+7"
  },
  {
    key: "icons-small",
    mode: "icons",
    iconSize: "small",
    labelKey: "view.smallIcons",
    descriptionKey: "view.smallIconsDescription",
    icon: "view.icons",
    shortcut: "Ctrl+Shift+4"
  },
  {
    key: "icons-medium",
    mode: "icons",
    iconSize: "medium",
    labelKey: "view.mediumIcons",
    descriptionKey: "view.mediumIconsDescription",
    icon: "view.icons",
    shortcut: "Ctrl+Shift+3"
  },
  {
    key: "icons-large",
    mode: "icons",
    iconSize: "large",
    labelKey: "view.largeIcons",
    descriptionKey: "view.largeIconsDescription",
    icon: "view.icons",
    shortcut: "Ctrl+Shift+1/2"
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
    label: t(option.labelKey)
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
    <div v-if="open" ref="viewMenuPanelRef" class="view-menu-panel" role="menu" :aria-label="t('view.aria')" @keydown="handleMenuKeyDown">
      <div class="view-menu-list">
        <button
            v-for="option in viewModeOptions"
            :key="option.key"
            class="view-menu-item"
            :class="{active: isActive(option)}"
            role="menuitemradio"
            :aria-checked="isActive(option)"
            tabindex="-1"
            @click="select(option)">
          <span class="view-option-icon"><icon :icon="option.icon" /></span>
          <span class="view-menu-copy">
            <strong>{{ t(option.labelKey) }}</strong>
            <small>{{ t(option.descriptionKey) }}</small>
          </span>
          <kbd>{{ option.shortcut }}</kbd>
          <span class="view-check"><icon v-if="isActive(option)" icon="action.check" size="small" /></span>
        </button>
      </div>
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

.view-button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.view-caret {
  @apply text-[0.65rem];
  color: var(--app-text-subtle);
}

.view-menu-panel {
  @apply absolute right-0 top-[calc(100%+0.35rem)] z-50 w-[20.5rem] overflow-hidden rounded-md border p-2;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.view-menu-list {
  @apply grid gap-1;
}

.view-menu-item {
  @apply grid w-full grid-cols-[2rem_minmax(0,1fr)_auto_1.25rem] items-center gap-2 rounded-md border px-2 py-1.5 text-left text-sm;
  border-color: transparent;
  color: var(--app-text-muted);
}

.view-menu-item:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
}

.view-menu-item.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, var(--app-panel-solid));
  color: var(--app-accent, #2563eb);
}

.view-menu-item:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.view-option-icon {
  @apply grid size-8 place-items-center rounded-md;
  background: var(--app-control);
  color: var(--app-text-muted);
}

.view-menu-item.active .view-option-icon {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 14%, transparent);
  color: var(--app-accent, #2563eb);
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

.view-menu-item.active kbd {
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 25%, var(--app-border-soft));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 10%, transparent);
  color: var(--app-accent, #2563eb);
}

.view-check {
  @apply grid size-5 place-items-center rounded-full text-xs font-semibold;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, transparent);
  color: var(--app-accent, #2563eb);
}
</style>
