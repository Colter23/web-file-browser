<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {AppAccentColor, AppColorMode, AppIconStyle, FileIconPalette, FileIconStyle} from "../../class.ts";
import {
  accentColorOptions,
  colorModeOptions,
  fileIconStyleOptions,
  fileIconPaletteOptions,
  iconStyleOptions,
  useAppearanceStore
} from "../../store/appearance.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import {useI18n} from "../../i18n";
import type {LocaleCode} from "../../i18n";
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
const {locale, localeOptions, setLocale, t} = useI18n();
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
  const option = colorModeOptions.find(item => item.value === appearanceStore.colorMode) ?? colorModeOptions[0];
  return t(option.labelKey);
});
const activeIconStyleLabel = computed(() => {
  const option = iconStyleOptions.find(item => item.value === appearanceStore.iconStyle) ?? iconStyleOptions[0];
  return t(option.labelKey);
});
const activeFileIconStyleLabel = computed(() => {
  const option = fileIconStyleOptions.find(item => item.value === appearanceStore.fileIconStyle) ?? fileIconStyleOptions[0];
  return t(option.labelKey);
});
const activeAccentColorLabel = computed(() => {
  const option = accentColorOptions.find(item => item.value === appearanceStore.accentColor) ?? accentColorOptions[0];
  return t(option.labelKey);
});
const menuTitle = computed(() => t("moreMenu.title", {colorMode: activeColorModeLabel.value, iconStyle: activeIconStyleLabel.value}));
const menuButtonLabel = computed(() => open.value ? t("moreMenu.closeMain") : menuTitle.value);
const appearanceSummary = computed(() => {
  return t("appearance.menuSummary", {
    colorMode: activeColorModeLabel.value,
    accentColor: activeAccentColorLabel.value,
    iconStyle: activeIconStyleLabel.value,
    fileIconStyle: activeFileIconStyleLabel.value
  });
});

const close = () => {
  open.value = false;
}

const resetMenuView = () => {
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

const selectFileIconStyle = (style: FileIconStyle) => {
  appearanceStore.setFileIconStyle(style);
}

const selectFileIconPalette = (palette: FileIconPalette) => {
  appearanceStore.setFileIconPalette(palette);
}

const selectAccentColor = (color: AppAccentColor) => {
  appearanceStore.setAccentColor(color);
}

const selectLocale = (value: LocaleCode) => {
  setLocale(value);
}

const handleButtonKeyDown = (event: KeyboardEvent) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (!open.value) openMenu();
  else void focusFirstItem();
}

const fileIconPreviewStyle = (style: FileIconStyle): AppIconStyle => {
  return style === "inherit" ? appearanceStore.iconStyle : style;
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

    <Transition name="more-menu" @after-leave="resetMenuView">
      <div
          v-if="open"
          ref="menuPanelRef"
          class="more-menu-panel"
          :class="{'appearance-view': activeView === 'appearance'}"
          role="menu"
          :aria-label="t('moreMenu.aria')"
          @keydown="handleMenuKeyDown">
        <Transition :name="activeView === 'appearance' ? 'menu-view-forward' : 'menu-view-back'" mode="out-in">
          <div v-if="activeView === 'main'" key="main" class="menu-view">
            <button class="command-item" role="menuitem" tabindex="-1" @click="closeAndEmit('open-settings')">
              <icon icon="action.settings" />
              <span class="item-copy">
                <strong>{{ t("common.settings") }}</strong>
                <small>{{ t("moreMenu.settingsHint") }}</small>
              </span>
            </button>

            <button class="command-item submenu-launch" role="menuitem" tabindex="-1" @click="showAppearance">
              <icon icon="action.appearance" />
              <span class="item-copy">
                <strong>{{ t("appearance.menu") }}</strong>
                <small>{{ appearanceSummary }}</small>
              </span>
              <icon class="submenu-caret" icon="action.next" />
            </button>

            <div class="language-menu-item" role="group" :aria-label="t('app.language')">
              <icon icon="action.language" />
              <span class="item-copy">
                <strong>{{ t("app.language") }}</strong>
              </span>
              <div class="locale-switch" role="radiogroup" :aria-label="t('app.language')">
                <button
                    v-for="option in localeOptions"
                    :key="option.value"
                    class="locale-option"
                    :class="{active: locale === option.value}"
                    role="radio"
                    type="button"
                    tabindex="-1"
                    :aria-checked="locale === option.value"
                    @click="selectLocale(option.value)">
                  {{ t(option.labelKey) }}
                </button>
              </div>
            </div>

            <div class="menu-separator"></div>

            <button class="command-item" :class="{active: taskActive}" role="menuitem" tabindex="-1" @click="closeAndEmit('toggle-tasks')">
              <icon icon="view.details" />
              <span class="item-copy">
                <strong>{{ taskButtonText }}</strong>
                <small>{{ t("moreMenu.tasksHint") }}</small>
              </span>
            </button>

            <button class="command-item" :class="{active: trashActive}" role="menuitem" tabindex="-1" @click="closeAndEmit('toggle-trash')">
              <icon icon="action.trash" />
              <span class="item-copy">
                <strong>{{ t("common.trash") }}</strong>
                <small>{{ t("moreMenu.trashHint") }}</small>
              </span>
            </button>

            <div class="menu-separator"></div>

            <button class="command-item danger" role="menuitem" tabindex="-1" @click="closeAndEmit('sign-out')">
              <icon icon="action.logout" />
              <span class="item-copy">
                <strong>{{ t("common.signOut") }}</strong>
                <small>{{ t("moreMenu.signOutHint") }}</small>
              </span>
            </button>
          </div>

          <div v-else key="appearance" class="menu-view appearance-menu-view">
            <div class="submenu-header">
              <button class="submenu-back" type="button" role="menuitem" tabindex="-1" :aria-label="t('moreMenu.back')" @click="showMain">
                <icon icon="action.previous" />
              </button>
              <span class="item-copy">
                <strong>{{ t("appearance.menu") }}</strong>
              </span>
            </div>

            <section class="preference-section" :aria-label="t('appearance.colorMode')">
              <div class="preference-heading">
                <span>{{ t("appearance.colorMode") }}</span>
              </div>
              <div class="segmented-group" role="radiogroup" :aria-label="t('appearance.colorMode')">
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
                  <span>{{ t(option.labelKey) }}</span>
                </button>
              </div>
            </section>

            <section class="preference-section" :aria-label="t('appearance.iconStyle')">
              <div class="preference-heading">
                <span>{{ t("appearance.iconStyle") }}</span>
              </div>
              <div class="segmented-group" role="radiogroup" :aria-label="t('appearance.iconStyle')">
                <button
                    v-for="option in iconStyleOptions"
                    :key="option.value"
                    class="segmented-option icon-style-option"
                    :class="{active: appearanceStore.iconStyle === option.value}"
                    role="radio"
                    type="button"
                    tabindex="-1"
                    :aria-checked="appearanceStore.iconStyle === option.value"
                    @click="selectIconStyle(option.value)">
                  <span class="style-preview-frame">
                    <icon class="style-preview-icon" icon="file.folder" size="1.15rem" :icon-style="option.value" />
                  </span>
                  <span>{{ t(option.labelKey) }}</span>
                </button>
              </div>
            </section>

            <section class="preference-section" :aria-label="t('appearance.fileIconStyle')">
              <div class="preference-heading">
                <span>{{ t("appearance.fileIconStyle") }}</span>
              </div>
              <div class="segmented-group" role="radiogroup" :aria-label="t('appearance.fileIconStyle')">
                <button
                    v-for="option in fileIconStyleOptions"
                    :key="option.value"
                    class="segmented-option file-icon-style-option"
                    :class="{active: appearanceStore.fileIconStyle === option.value}"
                    role="radio"
                    type="button"
                    tabindex="-1"
                    :aria-checked="appearanceStore.fileIconStyle === option.value"
                    @click="selectFileIconStyle(option.value)">
                  <span class="style-preview-frame">
                    <icon class="style-preview-icon" icon="file.folder" size="1.15rem" :icon-style="fileIconPreviewStyle(option.value)" />
                  </span>
                  <span>{{ t(option.labelKey) }}</span>
                </button>
              </div>
            </section>

            <section class="preference-section" :aria-label="t('appearance.fileIconPalette')">
              <div class="preference-heading">
                <span>{{ t("appearance.fileIconPalette") }}</span>
              </div>
              <div class="segmented-group" role="radiogroup" :aria-label="t('appearance.fileIconPalette')">
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
                  <span>{{ t(option.labelKey) }}</span>
                </button>
              </div>
            </section>

            <section class="preference-section" :aria-label="t('appearance.accentColor')">
              <div class="preference-heading">
                <span>{{ t("appearance.accentColor") }}</span>
              </div>
              <div class="accent-grid" role="radiogroup" :aria-label="t('appearance.accentColor')">
                <button
                    v-for="option in accentColorOptions"
                    :key="option.value"
                    class="accent-button"
                    :class="{active: appearanceStore.accentColor === option.value}"
                    role="radio"
                    :aria-checked="appearanceStore.accentColor === option.value"
                    type="button"
                    tabindex="-1"
                    :title="t(option.labelKey)"
                    @click="selectAccentColor(option.value)">
                  <span class="accent-swatch" :style="{backgroundColor: option.color}"></span>
                  <span>{{ t(option.labelKey) }}</span>
                </button>
              </div>
            </section>
          </div>
        </Transition>
      </div>
    </Transition>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.shell-more-menu {
  @apply relative shrink-0;
}

.more-button {
  @apply inline-flex h-12 w-12 items-center justify-center rounded-xl border shadow-sm backdrop-blur;
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
  transform-origin: top right;
}

.menu-view {
  @apply flex flex-col gap-0.5;
}

.more-menu-enter-active {
  transition: opacity 0.12s ease, transform 0.14s cubic-bezier(0.16, 1, 0.3, 1);
}

.more-menu-leave-active {
  transition: opacity 0.09s ease, transform 0.1s ease;
}

.more-menu-enter-from,
.more-menu-leave-to {
  opacity: 0;
  transform: translateY(-0.25rem) scale(0.985);
}

.menu-view-forward-enter-active,
.menu-view-forward-leave-active,
.menu-view-back-enter-active,
.menu-view-back-leave-active {
  transition: opacity 0.08s ease, transform 0.1s ease;
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

.language-menu-item {
  @apply grid w-full grid-cols-[1.25rem_minmax(0,1fr)_auto] items-center gap-3 rounded-md px-3 py-2 text-sm;
  color: var(--app-text-muted);
}

.language-menu-item:focus-within {
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 58%, transparent);
}

.submenu-launch {
  @apply grid-cols-[1.25rem_minmax(0,1fr)_1rem];
}

.submenu-caret {
  color: var(--app-text-subtle);
}

.command-item:hover,
.segmented-option:hover,
.accent-button:hover,
.locale-option:hover,
.submenu-back:hover {
  background: var(--app-accent-hover, #eff6ff);
}

.command-item.active,
.segmented-option.active,
.accent-button.active,
.locale-option.active {
  background: var(--app-accent-selected, #dbeafe);
  color: var(--app-accent-strong, #1d4ed8);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.command-item:focus-visible,
.segmented-option:focus-visible,
.accent-button:focus-visible,
.locale-option:focus-visible,
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

.locale-switch {
  @apply grid grid-cols-2 gap-0.5 rounded-md border p-0.5;
  min-width: 8.75rem;
  border-color: color-mix(in srgb, var(--app-border) 54%, transparent);
  background: var(--app-control-solid);
}

.locale-option {
  @apply inline-flex min-h-[1.8rem] items-center justify-center rounded px-2 text-xs font-medium;
  color: var(--app-text-muted);
  transition: background 0.12s ease, color 0.12s ease, box-shadow 0.12s ease;
}

.locale-option.active {
  @apply font-semibold;
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast, #fff);
  box-shadow: 0 1px 2px color-mix(in srgb, var(--app-accent, #2563eb) 24%, transparent);
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
  @apply grid w-full gap-0.5 rounded-md border p-0.5;
  grid-template-columns: repeat(auto-fit, minmax(5.75rem, 1fr));
  border-color: color-mix(in srgb, var(--app-border) 54%, transparent);
  background: var(--app-control-solid);
}

.segmented-option {
  @apply inline-flex min-h-[2.15rem] min-w-0 items-center justify-center gap-1.5 rounded px-2 py-2 text-center text-xs font-medium;
  color: var(--app-text-muted);
  transition: background 0.12s ease, color 0.12s ease, box-shadow 0.12s ease;
}

.mode-option,
.palette-option {
  @apply w-full;
}

.option-icon {
  color: var(--app-text-subtle);
}

.palette-option.palette-category .option-icon {
  color: #2563eb;
}

.palette-option.palette-accent .option-icon {
  color: var(--app-accent, #2563eb);
}

.segmented-option.active {
  @apply font-semibold;
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast, #fff);
  box-shadow: 0 1px 2px color-mix(in srgb, var(--app-accent, #2563eb) 24%, transparent);
}

.segmented-option.active .option-icon,
.segmented-option.active .style-preview-frame {
  color: currentColor;
}

.style-preview-frame {
  @apply grid h-5 w-5 place-items-center;
  color: #d97706;
}

.style-preview-icon {
  @apply block;
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
  .more-menu-enter-active,
  .more-menu-leave-active,
  .menu-view-forward-enter-active,
  .menu-view-forward-leave-active,
  .menu-view-back-enter-active,
  .menu-view-back-leave-active {
    transition: none;
  }
}
</style>
