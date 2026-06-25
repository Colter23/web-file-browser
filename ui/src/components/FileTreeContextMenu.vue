<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {FileTreeData} from "../class.ts";
import {useMenuKeyboardNavigation} from "../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../composables/useOutsidePointerDown.ts";
import {useViewportMenuPosition} from "../composables/useViewportMenuPosition.ts";
import Icon from "./Icon.vue";

const props = defineProps<{
  x: number;
  y: number;
  node: FileTreeData;
  expanded: boolean;
  loading: boolean;
  hasChildren: boolean;
  favorite: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "escape"): void;
  (e: "open"): void;
  (e: "open-new-tab"): void;
  (e: "refresh"): void;
  (e: "toggle"): void;
  (e: "add-favorite"): void;
  (e: "remove-favorite"): void;
  (e: "copy-path"): void;
}>();

const menuRef = ref<HTMLElement | null>(null);
const {menuPosition, placeMenu} = useViewportMenuPosition({menuRef});
const canToggle = computed(() => !props.loading && (props.expanded || props.hasChildren || props.node.children === undefined));
const toggleText = computed(() => props.expanded ? "折叠" : "展开");

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

watch(() => [props.x, props.y, props.node.path] as const, () => {
  void refreshMenu();
}, {flush: "post"});
</script>

<template>
  <Teleport to="body">
      <div
        ref="menuRef"
        class="tree-context-menu"
        :style="{left: `${menuPosition.x}px`, top: `${menuPosition.y}px`}"
        @click.stop
        @contextmenu.prevent.stop
        @keydown="handleMenuKeyDown">
      <button class="context-row" :disabled="loading" @click="emit('open')">
        <span class="context-row-icon"><icon icon="action.open" /></span>
        <span class="context-row-label">打开</span>
      </button>
      <button class="context-row" :disabled="loading" @click="emit('open-new-tab')">
        <span class="context-row-icon"><icon icon="action.open-new-tab" /></span>
        <span class="context-row-label">在新标签页中打开</span>
      </button>
      <div class="context-separator"></div>
      <button class="context-row" :disabled="loading" @click="emit('refresh')">
        <span class="context-row-icon"><icon class="icon-motion-spin" icon="action.refresh" /></span>
        <span class="context-row-label">刷新</span>
      </button>
      <button class="context-row" :disabled="!canToggle" @click="emit('toggle')">
        <span class="context-row-icon"><icon icon="action.down" class="icon-motion-caret" :class="{'is-open': expanded}" /></span>
        <span class="context-row-label">{{ toggleText }}</span>
      </button>
      <button v-if="favorite" class="context-row" :disabled="node.path === '/'" @click="emit('remove-favorite')">
        <span class="context-row-icon favorite"><icon icon="action.favorite-filled" /></span>
        <span class="context-row-label">从收藏夹移除</span>
      </button>
      <button v-else class="context-row" :disabled="node.path === '/'" @click="emit('add-favorite')">
        <span class="context-row-icon favorite"><icon icon="action.favorite" /></span>
        <span class="context-row-label">添加到收藏夹</span>
      </button>
      <div class="context-separator"></div>
      <button class="context-row" @click="emit('copy-path')">
        <span class="context-row-icon"><icon icon="action.copy-path" /></span>
        <span class="context-row-label">复制路径</span>
      </button>
    </div>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.tree-context-menu {
  @apply fixed z-50 w-56 rounded-md border py-1 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.tree-context-menu button {
  @apply border-0 bg-transparent;
  color: var(--app-text-muted);
}

.tree-context-menu button:disabled {
  color: var(--app-text-disabled);
}

.tree-context-menu button:disabled:hover {
  background: var(--app-panel-solid);
}

.tree-context-menu button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.tree-context-menu button:focus-visible {
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

.context-row-icon.favorite {
  color: color-mix(in srgb, var(--app-warning) 88%, var(--app-accent, #2563eb));
}

.context-row-label {
  @apply min-w-0 truncate;
}

.context-row:disabled .context-row-icon {
  color: var(--app-text-disabled);
}
</style>
