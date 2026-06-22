import {computed, nextTick} from "vue";
import type {ExplorerIconSize, ExplorerViewMode} from "../class.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useFileStore} from "../store";

export type ExplorerViewModeSelection = {
  mode: ExplorerViewMode;
  iconSize?: ExplorerIconSize;
  label: string;
}

type ExplorerViewModeOptions = {
  focusExplorer: () => void;
  closeMenus: () => void;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
}

const viewModeMeta: Record<ExplorerViewMode, {label: string; icon: string}> = {
  details: {label: "详细信息", icon: "icon-view-list"},
  list: {label: "列表", icon: "icon-listview"},
  icons: {label: "图标", icon: "icon-viewgrid"},
  tiles: {label: "平铺", icon: "icon-file-common-filling"}
};

const viewShortcutMap: Record<string, ExplorerViewModeSelection & {iconSize: ExplorerIconSize}> = {
  Digit1: {mode: "icons", iconSize: "large", label: "大图标"},
  Digit2: {mode: "icons", iconSize: "large", label: "大图标"},
  Digit3: {mode: "icons", iconSize: "medium", label: "中图标"},
  Digit4: {mode: "icons", iconSize: "small", label: "小图标"},
  Digit5: {mode: "list", iconSize: "small", label: "列表"},
  Digit6: {mode: "details", iconSize: "small", label: "详细信息"},
  Digit7: {mode: "tiles", iconSize: "medium", label: "平铺"}
};

const iconSizeLabel: Record<ExplorerIconSize, string> = {
  small: "小图标",
  medium: "中图标",
  large: "大图标"
};

export const useExplorerViewMode = ({focusExplorer, closeMenus, showNotice}: ExplorerViewModeOptions) => {
  const fileStore = useFileStore();

  const currentViewModeMeta = computed(() => viewModeMeta[fileStore.viewMode]);
  const currentViewModeLabel = computed(() => fileStore.viewMode === "icons" ? iconSizeLabel[fileStore.iconSize] : currentViewModeMeta.value.label);
  const viewModeButtonTitle = computed(() => `当前：${currentViewModeLabel.value}，点击选择查看模式。Ctrl+Shift+1-7 可直接切换查看模式`);

  const viewShortcut = (code: string) => viewShortcutMap[code] ?? viewShortcutMap[code.replace("Numpad", "Digit")];

  const selectViewMode = (selection: ExplorerViewModeSelection) => {
    fileStore.setViewMode(selection.mode);
    if (selection.iconSize) fileStore.setIconSize(selection.iconSize);
    closeMenus();
    void nextTick(focusExplorer);
    showNotice(`已切换为${selection.label}`, "info", "查看模式", 1400);
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
