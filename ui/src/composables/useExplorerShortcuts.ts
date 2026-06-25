import type {Ref} from "vue";
import {useFileStore} from "../store";

type MaybePromise<T = unknown> = T | Promise<T>;

type ExplorerShortcutsOptions = {
  imageViewerVisible: Ref<boolean>;
  videoViewerVisible: Ref<boolean>;
  pdfViewerVisible: Ref<boolean>;
  previewPanelVisible: Ref<boolean>;
  hasPreviewableSelection: () => boolean;
  focusSearchInput: () => void;
  focusBreadcrumbInput: () => void;
  selectAllEntries: () => void;
  applyViewShortcut: (code: string) => boolean;
  tabShortcutTargetId: (code: string) => string;
  switchTab: (tabId: string) => MaybePromise;
  openTab: () => MaybePromise;
  reopenClosedTab: () => MaybePromise;
  closeActiveTab: () => MaybePromise;
  switchRelativeTab: (offset: number) => MaybePromise;
  createFolderFromShortcut: () => MaybePromise;
  refreshCurrent: (keepSelection?: boolean) => MaybePromise;
  copySelected: () => void;
  cutSelected: () => void;
  pasteSelected: () => MaybePromise;
  closePreview: () => void;
  previewSelectedQuietly: () => Promise<boolean>;
  showEmptyPreviewPane: () => void;
  handleBackspaceNavigation: () => void;
  navigateBack: () => MaybePromise;
  navigateForward: () => MaybePromise;
  navigateUp: () => MaybePromise;
}

const formControlSelector = "input, textarea, select, [contenteditable='true']";
const operationOverlaySelector = ".operation-shell, .operation-panel, .delete-confirm-panel, .properties-panel";
const shellOverlaySelector = `.ace_editor, ${operationOverlaySelector}, .context-menu, .tab-context-menu, .view-menu-panel, .task-panel, .trash-panel`;

export const shouldIgnoreNavigationShortcut = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return Boolean(target.closest(`${formControlSelector}, ${shellOverlaySelector}`));
}

const shouldIgnoreActionShortcut = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return Boolean(target.closest(`button, a, ${formControlSelector}, ${shellOverlaySelector}`));
}

const shouldKeepEditorFindShortcut = (showEditor: boolean, target: EventTarget | null) => {
  if (showEditor) return true;
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(`.ace_editor, ${operationOverlaySelector}`));
}

const shouldIgnoreAddressShortcut = (showEditor: boolean, target: EventTarget | null) => {
  if (showEditor) return true;
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return Boolean(target.closest(shellOverlaySelector));
}

const hasPageTextSelection = () => {
  const selection = window.getSelection();
  return Boolean(selection && !selection.isCollapsed && selection.toString().trim());
}

const isExplorerShortcutTarget = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".explorer-viewport"));
}

export const useExplorerShortcuts = ({
  imageViewerVisible,
  videoViewerVisible,
  pdfViewerVisible,
  previewPanelVisible,
  hasPreviewableSelection,
  focusSearchInput,
  focusBreadcrumbInput,
  selectAllEntries,
  applyViewShortcut,
  tabShortcutTargetId,
  switchTab,
  openTab,
  reopenClosedTab,
  closeActiveTab,
  switchRelativeTab,
  createFolderFromShortcut,
  refreshCurrent,
  copySelected,
  cutSelected,
  pasteSelected,
  closePreview,
  previewSelectedQuietly,
  showEmptyPreviewPane,
  handleBackspaceNavigation,
  navigateBack,
  navigateForward,
  navigateUp
}: ExplorerShortcutsOptions) => {
  const fileStore = useFileStore();

  const shouldIgnoreShellShortcut = (target: EventTarget | null) => {
    return fileStore.showEditor || shouldIgnoreNavigationShortcut(target);
  }

  const handleClipboardShortcut = (key: string, event: KeyboardEvent) => {
    if (event.shiftKey || shouldIgnoreShellShortcut(event.target) || isExplorerShortcutTarget(event.target)) return false;
    if ((key === "c" || key === "x") && hasPageTextSelection()) return false;
    if (key === "c") {
      event.preventDefault();
      copySelected();
      return true;
    }
    if (key === "x") {
      event.preventDefault();
      cutSelected();
      return true;
    }
    if (key === "v") {
      event.preventDefault();
      void pasteSelected();
      return true;
    }
    return false;
  }

  const handleSelectAllShortcut = (key: string, event: KeyboardEvent) => {
    if (key !== "a" || event.shiftKey || shouldIgnoreShellShortcut(event.target) || isExplorerShortcutTarget(event.target)) return false;
    if (hasPageTextSelection()) return false;
    event.preventDefault();
    selectAllEntries();
    return true;
  }

  const focusSearch = () => {
    if (fileStore.showEditor) return;
    focusSearchInput();
  }

  const focusBreadcrumb = () => {
    if (fileStore.showEditor) return;
    focusBreadcrumbInput();
  }

  const togglePreviewFromShortcut = async () => {
    if (previewPanelVisible.value) {
      closePreview();
      return true;
    }
    const previewed = await previewSelectedQuietly();
    if (previewed) return true;
    if (fileStore.showEditor) return false;
    showEmptyPreviewPane();
    return true;
  }

  const handleWindowKeyDown = (event: KeyboardEvent) => {
    if (imageViewerVisible.value || videoViewerVisible.value || pdfViewerVisible.value) return;
    if (event.defaultPrevented) return;
    const key = event.key.toLowerCase();
    const commandKey = event.ctrlKey || event.metaKey;

    if (commandKey && event.shiftKey && !event.altKey && !shouldIgnoreShellShortcut(event.target)) {
      if (applyViewShortcut(event.code)) {
        event.preventDefault();
        return;
      }
    }
    if ((commandKey && !event.altKey && key === "l") || (event.altKey && !event.ctrlKey && !event.metaKey && key === "d")) {
      if (shouldIgnoreAddressShortcut(fileStore.showEditor, event.target)) return;
      event.preventDefault();
      focusBreadcrumb();
      return;
    }
    if (commandKey && !event.altKey && !event.shiftKey && (key === "f" || key === "e")) {
      if (shouldKeepEditorFindShortcut(fileStore.showEditor, event.target)) return;
      event.preventDefault();
      focusSearch();
      return;
    }
    if (commandKey && !event.altKey && !shouldIgnoreShellShortcut(event.target)) {
      if (handleClipboardShortcut(key, event)) return;
      if (handleSelectAllShortcut(key, event)) return;
      const tabShortcutId = !event.shiftKey ? tabShortcutTargetId(event.code) : "";
      if (tabShortcutId) {
        event.preventDefault();
        if (tabShortcutId !== fileStore.activeTabId) void switchTab(tabShortcutId);
        return;
      }
      if (key === "t") {
        event.preventDefault();
        void (event.shiftKey ? reopenClosedTab() : openTab());
        return;
      }
      if (key === "w") {
        event.preventDefault();
        void closeActiveTab();
        return;
      }
      if (key === "pageup" || key === "pagedown") {
        event.preventDefault();
        void switchRelativeTab(key === "pageup" ? -1 : 1);
        return;
      }
      if (event.shiftKey && key === "n") {
        event.preventDefault();
        void createFolderFromShortcut();
        return;
      }
      if (!event.shiftKey && key === "r") {
        event.preventDefault();
        void refreshCurrent(true);
        return;
      }
    }
    if (event.key === "F5" && !event.altKey && !event.ctrlKey && !event.metaKey && !shouldIgnoreShellShortcut(event.target)) {
      event.preventDefault();
      void refreshCurrent(true);
      return;
    }
    if (event.key === "Backspace" && !event.altKey && !event.ctrlKey && !event.metaKey && !shouldIgnoreNavigationShortcut(event.target)) {
      event.preventDefault();
      handleBackspaceNavigation();
      return;
    }
    if (event.altKey && !event.ctrlKey && !event.metaKey && event.key.toLowerCase() === "p" && !shouldIgnoreNavigationShortcut(event.target)) {
      event.preventDefault();
      void togglePreviewFromShortcut();
      return;
    }
    if ((event.key === " " || event.code === "Space")
        && !event.altKey
        && !event.ctrlKey
        && !event.metaKey
        && !isExplorerShortcutTarget(event.target)
        && !shouldIgnoreActionShortcut(event.target)) {
      if (hasPreviewableSelection()) {
        event.preventDefault();
        void previewSelectedQuietly();
      }
      return;
    }
    if (!event.altKey || event.ctrlKey || event.metaKey || shouldIgnoreNavigationShortcut(event.target)) return;
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      void navigateBack();
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      void navigateForward();
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      void navigateUp();
    }
  }

  return {
    togglePreviewFromShortcut,
    handleWindowKeyDown
  };
}
