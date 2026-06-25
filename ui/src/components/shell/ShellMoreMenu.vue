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
type MenuView = "main" | "appearance";
const activeView = ref<MenuView>("main");
const colorModeIcons: Record<AppColorMode, string> = {
  system: "appearance.system",
  light: "appearance.light",
  dark: "appearance.dark"
};
const fileIconPaletteIcons: Record<FileIconPalette, string> = {
  category: "file.folder",
  accent: "file.folder"
};

const activeColorModeLabel = computed(() => {
  return colorModeOptions.find(option => option.value === appearanceStore.colorMode)?.label ?? "跟随系统";
});
const activeIconStyleLabel = computed(() => {
  return iconStyleOptions.find(option => option.value === appearanceStore.iconStyle)?.label ?? "线性";
});
const activeAccentColorLabel = computed(() => {
  return accentColorOptions.find(option => option.value === appearanceStore.accentColor)?.label ?? "蓝色";
});
const menuTitle = computed(() => `更多选项：${activeColorModeLabel.value}，${activeIconStyleLabel.value}`);
const menuButtonLabel = computed(() => open.value ? "关闭主菜单" : menuTitle.value);
const appearanceSummary = computed(() => {
  return `${activeColorModeLabel.value} · ${activeAccentColorLabel.value} · ${activeIconStyleLabel.value}`;
});

const close = () => {
  open.value = false;
  activeView.value = "main";
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
  activeView.value = "main";
  open.value = true;
  void focusFirstItem();
}

const toggle = async () => {
  if (open.value) {
    close();
    return;
  }
  activeView.value = "main";
  open.value = true;
  await focusFirstItem();
}

const closeAndEmit = (event: "open-settings" | "toggle-tasks" | "toggle-trash" | "sign-out") => {
  close();
  if (event === "open-settings") emit("open-settings");
  if (event === "toggle-tasks") emit("toggle-tasks");
  if (event === "toggle-trash") emit("toggle-trash");
  if (event === "sign-out") emit("sign-out");
}

const showAppearance = () => {
  activeView.value = "appearance";
  void focusFirstItem();
}

const showMain = () => {
  activeView.value = "main";
  void focusFirstItem();
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
        :title="menuButtonLabel"
        :aria-label="menuButtonLabel"
        aria-haspopup="menu"
        :aria-expanded="open"
        @click="toggle"
        @keydown="handleButtonKeyDown">
      <span class="menu-icon-stack" :class="{open}">
        <icon class="menu-icon menu-icon-menu" icon="action.main-menu" size="large" />
        <icon class="menu-icon menu-icon-close" icon="action.close" size="large" />
      </span>
    </button>

    <div
        v-if="open"
        ref="menuPanelRef"
        class="more-menu-panel"
        :class="{'appearance-view': activeView === 'appearance'}"
        role="menu"
        aria-label="更多选项"
        @keydown="handleMenuKeyDown">
      <Transition :name="activeView === 'appearance' ? 'menu-view-forward' : 'menu-view-back'" mode="out-in">
        <div v-if="activeView === 'main'" key="main" class="menu-view">
          <button class="command-item" role="menuitem" tabindex="-1" @click="closeAndEmit('open-settings')">
            <icon icon="action.settings" />
            <span class="item-copy">
              <strong>设置</strong>
              <small>系统、挂载与运行状态</small>
            </span>
          </button>

          <button class="command-item submenu-launch" role="menuitem" tabindex="-1" @click="showAppearance">
            <icon icon="action.appearance" />
            <span class="item-copy">
              <strong>外观和主题</strong>
              <small>{{ appearanceSummary }}</small>
            </span>
            <icon class="submenu-caret" icon="action.next" />
          </button>

          <div class="menu-separator"></div>

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

          <button class="command-item danger" role="menuitem" tabindex="-1" @click="closeAndEmit('sign-out')">
            <icon icon="action.logout" />
            <span class="item-copy">
              <strong>退出</strong>
              <small>退出当前会话</small>
            </span>
          </button>
        </div>

        <div v-else key="appearance" class="menu-view appearance-menu-view">
          <div class="submenu-header">
            <button class="submenu-back" type="button" role="menuitem" tabindex="-1" aria-label="返回主菜单" @click="showMain">
              <icon icon="action.previous" />
            </button>
            <span class="item-copy">
              <strong>外观和主题</strong>
            </span>
          </div>

          <section class="preference-section" aria-label="颜色模式">
            <div class="preference-heading">
              <span>颜色模式</span>
            </div>
            <div class="segmented-group three-columns" role="radiogroup" aria-label="颜色模式">
              <button
                  v-for="option in colorModeOptions"
                  :key="option.value"
                  class="segmented-option mode-option"
                  :class="{active: appearanceStore.colorMode === option.value}"
                  role="radio"
                  type="button"
                  tabindex="-1"
                  :aria-checked="appearanceStore.colorMode === option.value"
                  @click="selectColorMode(option.value)">
                <icon class="option-icon" :icon="colorModeIcons[option.value]" />
                <span>{{ option.label }}</span>
              </button>
            </div>
          </section>

          <section class="preference-section" aria-label="图标样式">
            <div class="preference-heading">
              <span>图标样式</span>
            </div>
            <div class="option-wrap" role="radiogroup" aria-label="图标样式">
              <button
                  v-for="option in iconStyleOptions"
                  :key="option.value"
                  class="option-chip icon-style-option"
                  :class="{active: appearanceStore.iconStyle === option.value}"
                  role="radio"
                  type="button"
                  tabindex="-1"
                  :aria-checked="appearanceStore.iconStyle === option.value"
                  @click="selectIconStyle(option.value)">
                <span class="style-preview-frame">
                  <icon class="style-preview-icon" icon="file.folder" size="large" :icon-style="option.value" />
                </span>
                <span>{{ option.label }}</span>
              </button>
            </div>
          </section>

          <section class="preference-section" aria-label="图标着色">
            <div class="preference-heading">
              <span>图标着色</span>
            </div>
            <div class="segmented-group two-columns" role="radiogroup" aria-label="图标着色">
              <button
                  v-for="option in fileIconPaletteOptions"
                  :key="option.value"
                  class="segmented-option palette-option"
                  :class="[`palette-${option.value}`, {active: appearanceStore.fileIconPalette === option.value}]"
                  role="radio"
                  type="button"
                  tabindex="-1"
                  :aria-checked="appearanceStore.fileIconPalette === option.value"
                  @click="selectFileIconPalette(option.value)">
                <icon class="option-icon" :icon="fileIconPaletteIcons[option.value]" />
                <span>{{ option.label }}</span>
              </button>
            </div>
          </section>

          <section class="preference-section" aria-label="主题色">
            <div class="preference-heading">
              <span>主题色</span>
            </div>
            <div class="accent-grid" role="radiogroup" aria-label="主题色">
              <button
                  v-for="option in accentColorOptions"
                  :key="option.value"
                  class="accent-button"
                  :class="{active: appearanceStore.accentColor === option.value}"
                  role="radio"
                  :aria-checked="appearanceStore.accentColor === option.value"
                  type="button"
                  tabindex="-1"
                  :title="option.label"
                  @click="selectAccentColor(option.value)">
                <span class="accent-swatch" :style="{backgroundColor: option.color}"></span>
                <span>{{ option.label }}</span>
              </button>
            </div>
          </section>
        </div>
      </Transition>
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
  transition: background 0.16s ease, border-color 0.16s ease, color 0.16s ease, box-shadow 0.16s ease;
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

.menu-icon-stack {
  @apply relative grid h-6 w-6 place-items-center;
}

.menu-icon {
  @apply col-start-1 row-start-1;
  transition: opacity 0.16s ease, transform 0.18s ease;
}

.menu-icon-menu {
  opacity: 1;
  transform: rotate(0deg) scale(1);
}

.menu-icon-close {
  opacity: 0;
  transform: rotate(-45deg) scale(0.72);
}

.menu-icon-stack.open .menu-icon-menu {
  opacity: 0;
  transform: rotate(45deg) scale(0.72);
}

.menu-icon-stack.open .menu-icon-close {
  opacity: 1;
  transform: rotate(0deg) scale(1);
}

.more-menu-panel {
  @apply absolute right-0 top-[calc(100%+0.4rem)] z-50 w-[20rem] overflow-hidden rounded-md border p-1;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.menu-view {
  @apply flex flex-col gap-0.5;
}

.menu-view-forward-enter-active,
.menu-view-forward-leave-active,
.menu-view-back-enter-active,
.menu-view-back-leave-active {
  transition: opacity 0.16s ease, transform 0.18s ease;
}

.menu-view-forward-enter-from,
.menu-view-back-leave-to {
  opacity: 0;
  transform: translateX(0.75rem);
}

.menu-view-forward-leave-to,
.menu-view-back-enter-from {
  opacity: 0;
  transform: translateX(-0.75rem);
}

.menu-separator {
  @apply my-1 h-px;
  background: var(--app-border-soft);
}

.command-item {
  @apply grid w-full grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-3 rounded-md px-3 py-2 text-left text-sm;
  color: var(--app-text-muted);
}

.submenu-launch {
  @apply grid-cols-[1.25rem_minmax(0,1fr)_1rem];
}

.submenu-caret {
  color: var(--app-text-subtle);
}

.command-item:hover,
.segmented-option:hover,
.option-chip:hover,
.accent-button:hover,
.submenu-back:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.command-item.active,
.segmented-option.active,
.option-chip.active,
.accent-button.active {
  background: var(--app-accent-selected, #dbeafe);
  color: var(--app-accent-strong, #1d4ed8);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.command-item:focus-visible,
.segmented-option:focus-visible,
.option-chip:focus-visible,
.accent-button:focus-visible,
.submenu-back:focus-visible {
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

.submenu-header {
  @apply grid grid-cols-[2rem_minmax(0,1fr)] items-center gap-2 rounded-md px-2 py-1.5;
  color: var(--app-text-muted);
}

.submenu-back {
  @apply inline-flex h-8 w-8 items-center justify-center rounded-md;
  color: var(--app-text-muted);
}

.preference-section {
  @apply rounded-md px-2 py-1.5;
}

.preference-section + .preference-section {
  border-top: 1px solid var(--app-border-soft);
}

.preference-heading {
  @apply mb-1.5 flex items-center gap-2 px-1 text-xs font-medium;
  color: var(--app-text-muted);
}

.segmented-group {
  @apply grid gap-1 rounded-lg border p-0.5;
  border-color: color-mix(in srgb, var(--app-border) 54%, transparent);
  background: var(--app-control);
}

.segmented-group.three-columns {
  @apply grid-cols-3;
}

.segmented-group.two-columns {
  @apply grid-cols-2;
}

.segmented-option {
  @apply min-w-0 rounded-md px-2 py-1.5 text-center text-xs font-medium;
  color: var(--app-text-muted);
}

.mode-option,
.palette-option {
  @apply flex items-center justify-center gap-1.5;
}

.option-icon {
  color: var(--app-text-subtle);
}

.mode-option.active .option-icon,
.palette-option.active .option-icon {
  color: var(--app-accent-strong, #1d4ed8);
}

.palette-option.palette-category .option-icon {
  color: #d97706;
}

.palette-option.palette-accent .option-icon {
  color: var(--app-accent, #2563eb);
}

.segmented-option.active,
.option-chip.active {
  @apply font-semibold;
}

.option-wrap {
  @apply grid grid-cols-[repeat(auto-fit,minmax(5.75rem,1fr))] gap-1;
}

.option-chip {
  @apply flex min-w-0 items-center justify-center gap-2 rounded-lg border px-2 py-1.5 text-center text-xs font-medium;
  border-color: color-mix(in srgb, var(--app-border) 54%, transparent);
  background: var(--app-control);
  color: var(--app-text-muted);
}

.style-preview-frame {
  @apply grid h-6 w-6 place-items-center rounded-md;
  background: color-mix(in srgb, var(--app-accent) 9%, transparent);
  color: #d97706;
}

.style-preview-icon {
  filter: drop-shadow(0 1px 0 color-mix(in srgb, white 50%, transparent));
}

.accent-grid {
  @apply grid grid-cols-5 gap-1;
}

.accent-button {
  @apply flex min-w-0 flex-col items-center gap-1 rounded-md px-1 py-1.5 text-[0.68rem];
  color: var(--app-text-subtle);
}

.accent-swatch {
  @apply h-4 w-4 rounded-full border;
  border-color: color-mix(in srgb, var(--app-border) 60%, transparent);
}

.accent-button.active .accent-swatch {
  border-color: var(--app-accent-contrast, #fff);
  box-shadow: 0 0 0 2px var(--app-accent, #2563eb);
}

@media (prefers-reduced-motion: reduce) {
  .menu-view-forward-enter-active,
  .menu-view-forward-leave-active,
  .menu-view-back-enter-active,
  .menu-view-back-leave-active {
    transition: none;
  }
}
</style>
