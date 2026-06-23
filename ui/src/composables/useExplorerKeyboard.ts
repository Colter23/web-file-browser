import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ExplorerFocusMoveMode} from "./useExplorerSelection.ts";

type MaybePromise<T = unknown> = T | Promise<T>;

type ExplorerKeyboardOptions = {
  isViewportActive: () => boolean;
  isRenaming: () => boolean;
  isContextMenuVisible: () => boolean;
  isSelectionBoxActive: () => boolean;
  canPaste: () => boolean;
  selectedEntries: () => ExplorerEntry[];
  focusedOrSelectedEntry: () => ExplorerEntry | null;
  firstSelectedEntry: () => ExplorerEntry | null;
  cancelRename: () => void;
  commitRename: () => MaybePromise;
  closeContextMenu: () => void;
  resetSelectionBox: () => void;
  clearSelectionKeepingFocus: () => void;
  openKeyboardContextMenu: () => void;
  showProperties: (entries: ExplorerEntry[]) => void;
  toggleFocusedSelection: () => void;
  selectRange: (targetPath: string, additive: boolean) => void;
  previewEntry: (entry: ExplorerEntry) => void;
  copySelectedPaths: (fallbackEntry?: ExplorerEntry | null) => void;
  copyEntry: (entry: ExplorerEntry) => void;
  cutEntry: (entry: ExplorerEntry) => void;
  paste: () => void;
  selectAllEntries: () => void;
  invertCurrentSelection: () => void;
  openEntry: (entry: ExplorerEntry) => MaybePromise;
  openEntryInNewTab: (entry: ExplorerEntry) => void;
  deleteEntry: (entry: ExplorerEntry) => void;
  startRename: (entry: ExplorerEntry | null) => void;
  handleTypeahead: (event: KeyboardEvent) => boolean;
  moveFocus: (key: string, mode?: ExplorerFocusMoveMode) => void;
  applyViewShortcut?: (code: string) => boolean;
}

const navigationKeys = new Set(["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight", "Home", "End", "PageDown", "PageUp"]);

const isSpaceKey = (event: KeyboardEvent) => event.key === " " || event.code === "Space";

export const useExplorerKeyboard = ({
  isViewportActive,
  isRenaming,
  isContextMenuVisible,
  isSelectionBoxActive,
  canPaste,
  selectedEntries,
  focusedOrSelectedEntry,
  firstSelectedEntry,
  cancelRename,
  commitRename,
  closeContextMenu,
  resetSelectionBox,
  clearSelectionKeepingFocus,
  openKeyboardContextMenu,
  showProperties,
  toggleFocusedSelection,
  selectRange,
  previewEntry,
  copySelectedPaths,
  copyEntry,
  cutEntry,
  paste,
  selectAllEntries,
  invertCurrentSelection,
  openEntry,
  openEntryInNewTab,
  deleteEntry,
  startRename,
  handleTypeahead,
  moveFocus,
  applyViewShortcut
}: ExplorerKeyboardOptions) => {
  const selectedOrFocusedEntry = () => firstSelectedEntry() ?? focusedOrSelectedEntry();

  const handleRenameKey = async (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      cancelRename();
      return true;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      await commitRename();
      return true;
    }
    return false;
  }

  const handleEscape = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return false;
    event.preventDefault();
    if (isContextMenuVisible()) {
      closeContextMenu();
      return true;
    }
    if (isSelectionBoxActive()) {
      resetSelectionBox();
      return true;
    }
    clearSelectionKeepingFocus();
    return true;
  }

  const handleContextMenuKey = (event: KeyboardEvent) => {
    if (event.key !== "ContextMenu" && !(event.shiftKey && event.key === "F10")) return false;
    event.preventDefault();
    openKeyboardContextMenu();
    return true;
  }

  const handlePropertiesShortcut = (event: KeyboardEvent) => {
    if (!event.altKey || event.ctrlKey || event.metaKey || event.key !== "Enter") return false;
    event.preventDefault();
    closeContextMenu();
    const focused = focusedOrSelectedEntry();
    const entriesToShow = selectedEntries().length ? selectedEntries() : focused ? [focused] : [];
    if (entriesToShow.length) showProperties(entriesToShow);
    return true;
  }

  const handleSpaceShortcut = (event: KeyboardEvent) => {
    if (!isSpaceKey(event)) return false;
    if (!event.altKey && !event.shiftKey && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      closeContextMenu();
      toggleFocusedSelection();
      return true;
    }
    if (event.shiftKey && !event.altKey && !event.ctrlKey && !event.metaKey) {
      event.preventDefault();
      closeContextMenu();
      const entry = focusedOrSelectedEntry();
      if (entry) selectRange(entry.path, false);
      return true;
    }
    if (!event.altKey && !event.shiftKey && !event.ctrlKey && !event.metaKey) {
      event.preventDefault();
      closeContextMenu();
      const entry = focusedOrSelectedEntry();
      if (entry?.type === "file") previewEntry(entry);
      return true;
    }
    return false;
  }

  const handleClipboardShortcut = (event: KeyboardEvent) => {
    if (!event.ctrlKey && !event.metaKey) return false;
    const key = event.key.toLowerCase();
    if (key === "c") {
      event.preventDefault();
      if (event.shiftKey) {
        closeContextMenu();
        copySelectedPaths(focusedOrSelectedEntry());
        return true;
      }
      const entry = selectedOrFocusedEntry();
      if (entry) copyEntry(entry);
      return true;
    }
    if (key === "x") {
      event.preventDefault();
      const entry = selectedOrFocusedEntry();
      if (entry) cutEntry(entry);
      return true;
    }
    if (key === "v") {
      event.preventDefault();
      if (canPaste()) paste();
      return true;
    }
    return false;
  }

  const handleSelectionShortcut = (event: KeyboardEvent) => {
    if (!event.ctrlKey && !event.metaKey) return false;
    const key = event.key.toLowerCase();
    if (key === "a") {
      event.preventDefault();
      selectAllEntries();
      return true;
    }
    if (key === "i") {
      event.preventDefault();
      closeContextMenu();
      invertCurrentSelection();
      return true;
    }
    return false;
  }

  const handleViewShortcut = (event: KeyboardEvent) => {
    if (!applyViewShortcut || (!event.ctrlKey && !event.metaKey) || !event.shiftKey || event.altKey) return false;
    if (!applyViewShortcut(event.code)) return false;
    event.preventDefault();
    event.stopPropagation();
    return true;
  }

  const handleOpenShortcut = async (event: KeyboardEvent) => {
    if (event.key !== "Enter") return false;
    event.preventDefault();
    closeContextMenu();
    const entry = focusedOrSelectedEntry();
    if ((event.ctrlKey || event.metaKey) && entry?.type === "folder") {
      openEntryInNewTab(entry);
      return true;
    }
    if ((event.ctrlKey || event.metaKey) && entry?.type === "file") {
      previewEntry(entry);
      return true;
    }
    if (entry) await openEntry(entry);
    return true;
  }

  const handleActionShortcut = (event: KeyboardEvent) => {
    if (event.key === "Delete") {
      event.preventDefault();
      closeContextMenu();
      const entry = selectedOrFocusedEntry();
      if (entry) deleteEntry(entry);
      return true;
    }
    if (event.key === "F2") {
      event.preventDefault();
      closeContextMenu();
      const entry = selectedEntries().length <= 1 ? focusedOrSelectedEntry() : null;
      if (entry) startRename(entry);
      return true;
    }
    return false;
  }

  const handleNavigationKey = (event: KeyboardEvent) => {
    if (!navigationKeys.has(event.key)) return false;
    event.preventDefault();
    closeContextMenu();
    const mode: ExplorerFocusMoveMode = event.shiftKey
        ? "extendSelection"
        : event.ctrlKey || event.metaKey
          ? "moveFocusOnly"
          : "replaceSelection";
    moveFocus(event.key, mode);
    return true;
  }

  const handleKeyDown = async (event: KeyboardEvent) => {
    if (event.defaultPrevented) return;
    if (!isViewportActive()) return;
    if (isRenaming()) {
      await handleRenameKey(event);
      return;
    }
    if (handleEscape(event)) return;
    if (handleContextMenuKey(event)) return;
    if (handlePropertiesShortcut(event)) return;
    if (handleSpaceShortcut(event)) return;
    if (handleClipboardShortcut(event)) return;
    if (handleSelectionShortcut(event)) return;
    if (handleViewShortcut(event)) return;
    if (await handleOpenShortcut(event)) return;
    if (handleActionShortcut(event)) return;
    if (handleTypeahead(event)) return;
    handleNavigationKey(event);
  }

  return {
    handleKeyDown
  };
}
