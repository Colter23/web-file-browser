<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {AppAccentColor, AppColorMode, AppIconStyle, FileIconPalette} from "../../class.ts";
import {
  accentColorOptions,
  colorModeOptions,
  fileIconPaletteOptions,
  iconStyleOptions,
  useAppearanceStore
} from "../../store/appearance.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import Icon from "../Icon.vue";

defineProps<{
  taskButtonText: string;
  taskActive: boolean;
  trashActive: boolean;
}>();

const emit = defineEmits<{
  (e: "open-settings"): void;
  (e: "toggle-tasks"): void;
  (e: "toggle-trash"): void;
  (e: "sign-out"): void;
}>();

const appearanceStore = useAppearanceStore();
const menuRef = ref<HTMLElement | null>(null);
const menuPanelRef = ref<HTMLElement | null>(null);
const menuButtonRef = ref<HTMLButtonElement | null>(null);
const open = ref(false);

const activeColorModeLabel = computed(() => {
  return colorModeOptions.find(option => option.value === appearanceStore.colorMode)?.label ?? "跟随系统";
});
const activeIconStyleLabel = computed(() => {
  return iconStyleOptions.find(option => option.value === appearanceStore.iconStyle)?.label ?? "线性";
});
const menuTitle = computed(() => `更多选项：${activeColorModeLabel.value}，${activeIconStyleLabel.value}`);

const close = () => {
  open.value = false;
}

const focusButton = async () => {
  await nextTick();
  menuButtonRef.value?.focus({preventScroll: true});
}

const {
  focusMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef: menuPanelRef,
  onEscape: () => {
    close();
    void focusButton();
  }
});

const focusFirstItem = async () => {
  await nextTick();
  focusMenuButton(0);
}

const openMenu = () => {
  open.value = true;
  void focusFirstItem();
}

const toggle = async () => {
  open.value = !open.value;
  if (open.value) await focusFirstItem();
}

const closeAndEmit = (event: "open-settings" | "toggle-tasks" | "toggle-trash" | "sign-out") => {
  close();
  if (event === "open-settings") emit("open-settings");
  if (event === "toggle-tasks") emit("toggle-tasks");
  if (event === "toggle-trash") emit("toggle-trash");
  if (event === "sign-out") emit("sign-out");
}

const selectColorMode = (mode: AppColorMode) => {
  appearanceStore.setColorMode(mode);
}

const selectIconStyle = (style: AppIconStyle) => {
  appearanceStore.setIconStyle(style);
}

const selectFileIconPalette = (palette: FileIconPalette) => {
  appearanceStore.setFileIconPalette(palette);
}

const selectAccentColor = (color: AppAccentColor) => {
  appearanceStore.setAccentColor(color);
}

const handleButtonKeyDown = (event: KeyboardEvent) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (!open.value) openMenu();
  else void focusFirstItem();
}

useOutsidePointerDown({
  refs: [menuRef],
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
  <div ref="menuRef" class="shell-more-menu">
    <button
        ref="menuButtonRef"
        class="more-button"
        :class="{active: open || taskActive}"
        :title="menuTitle"
        aria-haspopup="menu"
        :aria-expanded="open"
        @click="toggle"
        @keydown="handleButtonKeyDown">
      <icon icon="action.more" size="large" />
    </button>

    <div v-if="open" ref="menuPanelRef" class="more-menu-panel" role="menu" aria-label="更多选项" @keydown="handleMenuKeyDown">
      <button class="command-item" role="menuitem" tabindex="-1" @click="closeAndEmit('open-settings')">
        <icon icon="action.settings" />
        <span class="item-copy">
          <strong>设置</strong>
          <small>系统与外观设置</small>
        </span>
      </button>

      <button class="command-item" :class="{active: taskActive}" role="menuitem" tabindex="-1" @click="closeAndEmit('toggle-tasks')">
        <icon icon="view.details" />
        <span class="item-copy">
          <strong>{{ taskButtonText }}</strong>
          <small>后台任务与传输进度</small>
        </span>
      </button>

      <button class="command-item" :class="{active: trashActive}" role="menuitem" tabindex="-1" @click="closeAndEmit('toggle-trash')">
        <icon icon="action.trash" />
        <span class="item-copy">
          <strong>回收站</strong>
          <small>恢复或永久删除项目</small>
        </span>
      </button>

      <div class="menu-separator"></div>
      <p class="menu-group-title">颜色模式</p>
      <button
          v-for="option in colorModeOptions"
          :key="option.value"
          class="choice-item"
          :class="{active: appearanceStore.colorMode === option.value}"
          role="menuitemradio"
          :aria-checked="appearanceStore.colorMode === option.value"
          tabindex="-1"
          @click="selectColorMode(option.value)">
        <span class="choice-check">{{ appearanceStore.colorMode === option.value ? "✓" : "" }}</span>
        <span>{{ option.label }}</span>
      </button>

      <div class="menu-separator"></div>
      <p class="menu-group-title">图标</p>
      <button
          v-for="option in iconStyleOptions"
          :key="option.value"
          class="choice-item"
          :class="{active: appearanceStore.iconStyle === option.value}"
          role="menuitemradio"
          :aria-checked="appearanceStore.iconStyle === option.value"
          tabindex="-1"
          @click="selectIconStyle(option.value)">
        <span class="choice-check">{{ appearanceStore.iconStyle === option.value ? "✓" : "" }}</span>
        <span>{{ option.label }}</span>
      </button>
      <button
          v-for="option in fileIconPaletteOptions"
          :key="option.value"
          class="choice-item"
          :class="{active: appearanceStore.fileIconPalette === option.value}"
          role="menuitemradio"
          :aria-checked="appearanceStore.fileIconPalette === option.value"
          tabindex="-1"
          @click="selectFileIconPalette(option.value)">
        <span class="choice-check">{{ appearanceStore.fileIconPalette === option.value ? "✓" : "" }}</span>
        <span>文件图标：{{ option.label }}</span>
      </button>

      <div class="menu-separator"></div>
      <p class="menu-group-title">主题色</p>
      <div class="accent-grid" role="group" aria-label="主题色">
        <button
            v-for="option in accentColorOptions"
            :key="option.value"
            class="accent-button"
            :class="{active: appearanceStore.accentColor === option.value}"
            role="menuitemradio"
            :aria-checked="appearanceStore.accentColor === option.value"
            type="button"
            tabindex="-1"
            :title="option.label"
            @click="selectAccentColor(option.value)">
          <span class="accent-swatch" :style="{backgroundColor: option.color}"></span>
          <span>{{ option.label }}</span>
        </button>
      </div>

      <div class="menu-separator"></div>
      <button class="command-item danger" role="menuitem" tabindex="-1" @click="closeAndEmit('sign-out')">
        <icon icon="action.logout" />
        <span class="item-copy">
          <strong>退出</strong>
          <small>退出当前会话</small>
        </span>
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.shell-more-menu {
  @apply relative shrink-0;
}

.more-button {
  @apply inline-flex h-11 w-11 items-center justify-center rounded-xl border shadow-sm backdrop-blur;
  border-color: color-mix(in srgb, var(--app-border) 45%, transparent);
  background: var(--app-control);
  color: var(--app-text-muted);
}

.more-button:hover {
  background: var(--app-control-hover);
  color: var(--app-accent, #2563eb);
}

.more-button.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.more-button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.more-menu-panel {
  @apply absolute right-0 top-[calc(100%+0.4rem)] z-50 w-80 overflow-x-hidden overflow-y-auto rounded-md border py-1;
  max-height: min(34rem, calc(100vh - 5rem));
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.menu-group-title {
  @apply px-3 py-1 text-[0.68rem] font-medium;
  color: var(--app-text-subtle);
}

.menu-separator {
  @apply my-1 h-px;
  background: var(--app-border-soft);
}

.command-item {
  @apply grid w-full grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-3 px-3 py-2 text-left text-sm;
  color: var(--app-text-muted);
}

.choice-item {
  @apply grid w-full grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-3 px-3 py-1.5 text-left text-sm;
  color: var(--app-text-muted);
}

.command-item:hover,
.choice-item:hover,
.accent-button:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.command-item.active,
.choice-item.active,
.accent-button.active {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.command-item:focus-visible,
.choice-item:focus-visible,
.accent-button:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.command-item.danger {
  color: var(--app-danger-text);
}

.command-item.danger:hover {
  background: var(--app-danger-soft);
}

.item-copy {
  @apply flex min-w-0 flex-col;
}

.item-copy strong {
  @apply truncate text-sm font-medium;
}

.item-copy small {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.command-item.active .item-copy small {
  color: var(--app-accent, #2563eb);
}

.choice-check {
  @apply text-center text-xs font-semibold;
  color: var(--app-accent, #2563eb);
}

.accent-grid {
  @apply grid grid-cols-5 gap-1 px-2 pb-1;
}

.accent-button {
  @apply flex min-w-0 flex-col items-center gap-1 rounded-md px-1 py-1.5 text-[0.68rem];
  color: var(--app-text-subtle);
}

.accent-swatch {
  @apply h-4 w-4 rounded-full border;
  border-color: color-mix(in srgb, var(--app-border) 60%, transparent);
}
</style>
