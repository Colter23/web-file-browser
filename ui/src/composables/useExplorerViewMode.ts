import {computed, nextTick} from "vue";
import type {ExplorerIconSize, ExplorerViewMode} from "../class.ts";
import type {MessageKey} from "../i18n";
import {useI18n} from "../i18n";
import {useFileStore} from "../store";

export type ExplorerViewModeSelection = {
  mode: ExplorerViewMode;
  iconSize?: ExplorerIconSize;
  label: string;
}

type ExplorerViewModeOptions = {
  focusExplorer: () => void;
  closeMenus: () => void;
}

const viewModeMeta: Record<ExplorerViewMode, {labelKey: MessageKey; icon: string}> = {
  details: {labelKey: "view.details", icon: "view.details"},
  list: {labelKey: "view.list", icon: "view.list"},
  icons: {labelKey: "view.icons", icon: "view.icons"},
  tiles: {labelKey: "view.tiles", icon: "view.tiles"}
};

const viewShortcutMap: Record<string, Omit<ExplorerViewModeSelection, "label"> & {iconSize: ExplorerIconSize; labelKey: MessageKey}> = {
  Digit1: {mode: "icons", iconSize: "large", labelKey: "view.largeIcons"},
  Digit2: {mode: "icons", iconSize: "large", labelKey: "view.largeIcons"},
  Digit3: {mode: "icons", iconSize: "medium", labelKey: "view.mediumIcons"},
  Digit4: {mode: "icons", iconSize: "small", labelKey: "view.smallIcons"},
  Digit5: {mode: "list", iconSize: "small", labelKey: "view.list"},
  Digit6: {mode: "details", iconSize: "small", labelKey: "view.details"},
  Digit7: {mode: "tiles", iconSize: "medium", labelKey: "view.tiles"}
};

const iconSizeLabelKey: Record<ExplorerIconSize, MessageKey> = {
  small: "view.smallIcons",
  medium: "view.mediumIcons",
  large: "view.largeIcons"
};

export const useExplorerViewMode = ({focusExplorer, closeMenus}: ExplorerViewModeOptions) => {
  const fileStore = useFileStore();
  const {t} = useI18n();

  const currentViewModeMeta = computed(() => viewModeMeta[fileStore.viewMode]);
  const currentViewModeLabel = computed(() => {
    return fileStore.viewMode === "icons" ? t(iconSizeLabelKey[fileStore.iconSize]) : t(currentViewModeMeta.value.labelKey);
  });
  const viewModeButtonTitle = computed(() => t("view.buttonTitle", {mode: currentViewModeLabel.value}));

  const viewShortcut = (code: string) => {
    const shortcut = viewShortcutMap[code] ?? viewShortcutMap[code.replace("Numpad", "Digit")];
    return shortcut ? {...shortcut, label: t(shortcut.labelKey)} : undefined;
  }

  const selectViewMode = (selection: ExplorerViewModeSelection) => {
    fileStore.setViewMode(selection.mode);
    if (selection.iconSize) fileStore.setIconSize(selection.iconSize);
    closeMenus();
    void nextTick(focusExplorer);
  }

  const applyViewShortcut = (code: string) => {
    const shortcut = viewShortcut(code);
    if (!shortcut) return false;
    selectViewMode(shortcut);
    return true;
  }

  return {
    currentViewModeMeta,
    currentViewModeLabel,
    viewModeButtonTitle,
    selectViewMode,
    applyViewShortcut
  };
}
