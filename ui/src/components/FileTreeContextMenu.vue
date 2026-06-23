<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {FileTreeData} from "../class.ts";
import {useMenuKeyboardNavigation} from "../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../composables/useOutsidePointerDown.ts";
import {useViewportMenuPosition} from "../composables/useViewportMenuPosition.ts";

const props = defineProps<{
  x: number;
  y: number;
  node: FileTreeData;
  expanded: boolean;
  loading: boolean;
  hasChildren: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "escape"): void;
  (e: "open"): void;
  (e: "open-new-tab"): void;
  (e: "refresh"): void;
  (e: "toggle"): void;
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
      <button :disabled="loading" @click="emit('open')">打开</button>
      <button :disabled="loading" @click="emit('open-new-tab')">在新标签页中打开</button>
      <div class="context-separator"></div>
      <button :disabled="loading" @click="emit('refresh')">刷新</button>
      <button :disabled="!canToggle" @click="emit('toggle')">{{ toggleText }}</button>
      <div class="context-separator"></div>
      <button @click="emit('copy-path')">复制路径</button>
    </div>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.tree-context-menu {
  @apply fixed z-50 w-44 rounded-md border py-1 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.tree-context-menu button {
  @apply block h-8 w-full px-3 text-left;
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
</style>
