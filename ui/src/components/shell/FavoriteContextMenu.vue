<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {FavoriteItem} from "../../class.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import {useViewportMenuPosition} from "../../composables/useViewportMenuPosition.ts";
import Icon from "../Icon.vue";

const props = defineProps<{
  x: number;
  y: number;
  favorite: FavoriteItem;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "escape"): void;
  (e: "open"): void;
  (e: "open-new-tab"): void;
  (e: "rename"): void;
  (e: "refresh"): void;
  (e: "copy-path"): void;
  (e: "remove"): void;
}>();

const menuRef = ref<HTMLElement | null>(null);
const {menuPosition, placeMenu} = useViewportMenuPosition({menuRef});
const openDisabled = computed(() => props.loading || props.favorite.missing === true);

const {focusFirstMenuButton, handleMenuKeyDown} = useMenuKeyboardNavigation({
  menuRef,
  onEscape: () => emit("escape")
});

const refreshMenu = async () => {
  await placeMenu({x: props.x, y: props.y});
  await focusFirstMenuButton();
}

useOutsidePointerDown({
  refs: [menuRef],
  onOutsidePointerDown: () => emit("close")
});

onMounted(() => {
  void refreshMenu();
  window.addEventListener("resize", refreshMenu);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", refreshMenu);
});

watch(() => [props.x, props.y, props.favorite.id] as const, () => {
  void refreshMenu();
}, {flush: "post"});
</script>

<template>
  <Teleport to="body">
    <div
        ref="menuRef"
        class="favorite-context-menu"
        :style="{left: `${menuPosition.x}px`, top: `${menuPosition.y}px`}"
        role="menu"
        aria-label="收藏项菜单"
        @click.stop
        @contextmenu.prevent.stop
        @keydown="handleMenuKeyDown">
      <button role="menuitem" class="context-row" :disabled="openDisabled" @click="emit('open')">
        <span class="context-row-icon"><icon icon="action.open" /></span>
        <span class="context-row-label">打开</span>
      </button>
      <button role="menuitem" class="context-row" :disabled="openDisabled" @click="emit('open-new-tab')">
        <span class="context-row-icon"><icon icon="action.open-new-tab" /></span>
        <span class="context-row-label">在新标签页中打开</span>
      </button>
      <div class="context-separator"></div>
      <button role="menuitem" class="context-row" @click="emit('rename')">
        <span class="context-row-icon"><icon icon="action.rename" /></span>
        <span class="context-row-label">重命名收藏项</span>
      </button>
      <button role="menuitem" class="context-row" :disabled="loading" @click="emit('refresh')">
        <span class="context-row-icon"><icon class="icon-motion-spin" icon="action.refresh" /></span>
        <span class="context-row-label">检查收藏夹</span>
      </button>
      <button role="menuitem" class="context-row" @click="emit('copy-path')">
        <span class="context-row-icon"><icon icon="action.copy-path" /></span>
        <span class="context-row-label">复制路径</span>
      </button>
      <div class="context-separator"></div>
      <button role="menuitem" class="context-row danger" @click="emit('remove')">
        <span class="context-row-icon"><icon icon="action.trash" /></span>
        <span class="context-row-label">从收藏夹移除</span>
      </button>
    </div>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.favorite-context-menu {
  @apply fixed z-50 w-56 rounded-md border py-1 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.favorite-context-menu button {
  @apply border-0 bg-transparent;
  color: var(--app-text-muted);
}

.favorite-context-menu button:disabled {
  color: var(--app-text-disabled);
}

.favorite-context-menu button:disabled:hover {
  background: var(--app-panel-solid);
}

.favorite-context-menu button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.favorite-context-menu button:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.context-separator {
  @apply my-1 border-t;
  border-color: var(--app-border-soft);
}

.context-row {
  @apply grid h-8 w-full items-center gap-2 px-3 text-left;
  grid-template-columns: 1rem minmax(0, 1fr);
}

.context-row-icon {
  @apply inline-flex items-center justify-center text-[0.95rem];
  color: var(--app-accent, #2563eb);
}

.context-row-label {
  @apply min-w-0 truncate;
}

.context-row.danger .context-row-icon {
  color: var(--app-danger);
}

.context-row:disabled .context-row-icon {
  color: var(--app-text-disabled);
}
</style>
