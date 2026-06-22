import {computed, ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import type {TabContextMenuState, TabDropPlacement} from "../components/tabs/types.ts";
import {useFileStore} from "../store";

type ExplorerTabsOptions = {
  currentFolder: () => string;
  closePanels: () => void;
  syncActiveTabContext: () => Promise<void>;
  persistCurrentExplorerScrollTop: () => void;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
}

export const useExplorerTabs = ({
  currentFolder,
  closePanels,
  syncActiveTabContext,
  persistCurrentExplorerScrollTop,
  showNotice
}: ExplorerTabsOptions) => {
  const fileStore = useFileStore();
  const tabContextMenu = ref<TabContextMenuState>({
    visible: false,
    x: 0,
    y: 0,
    tabId: ""
  });
  const draggingTabId = ref("");
  const tabDropTargetId = ref("");
  const tabDropPlacement = ref<TabDropPlacement | "">("");
  const tabContextMenuWidth = 184;
  const tabContextMenuHeight = 252;

  const tabContextTarget = computed(() => fileStore.tabs.find(tab => tab.id === tabContextMenu.value.tabId) ?? null);
  const tabContextIndex = computed(() => fileStore.tabs.findIndex(tab => tab.id === tabContextMenu.value.tabId));
  const canCloseTabContext = computed(() => fileStore.tabs.length > 1);
  const canCloseOtherTabsContext = computed(() => fileStore.tabs.length > 1 && Boolean(tabContextTarget.value));
  const canCloseRightTabsContext = computed(() => tabContextIndex.value >= 0 && tabContextIndex.value < fileStore.tabs.length - 1);
  const canReopenClosedTab = computed(() => fileStore.closedTabs.length > 0);

  const closeTabContextMenu = () => {
    tabContextMenu.value.visible = false;
  }

  const openTabContextMenu = (event: MouseEvent, tabId: string) => {
    event.preventDefault();
    event.stopPropagation();
    const x = Math.min(Math.max(8, event.clientX), Math.max(8, window.innerWidth - tabContextMenuWidth - 8));
    const y = Math.min(Math.max(8, event.clientY), Math.max(8, window.innerHeight - tabContextMenuHeight - 8));
    tabContextMenu.value = {visible: true, x, y, tabId};
  }

  const openTab = async () => {
    if (!await fileStore.requestEditorLeave()) return;
    persistCurrentExplorerScrollTop();
    closeTabContextMenu();
    fileStore.openTab(currentFolder());
    closePanels();
    await syncActiveTabContext();
  }

  const openEntryInNewTab = async (entry: ExplorerEntry) => {
    if (entry.type !== "folder") return;
    if (!await fileStore.requestEditorLeave()) return;
    persistCurrentExplorerScrollTop();
    closeTabContextMenu();
    fileStore.openPathInNewTab(entry.path);
    closePanels();
    await syncActiveTabContext();
  }

  const switchTab = async (tabId: string) => {
    closeTabContextMenu();
    if (tabId !== fileStore.activeTabId && !await fileStore.requestEditorLeave()) return;
    if (tabId !== fileStore.activeTabId) persistCurrentExplorerScrollTop();
    fileStore.switchTab(tabId);
    closePanels();
    await syncActiveTabContext();
  }

  const switchRelativeTab = async (offset: number) => {
    if (fileStore.tabs.length <= 1) return false;
    const currentIndex = fileStore.tabs.findIndex(tab => tab.id === fileStore.activeTabId);
    const startIndex = currentIndex >= 0 ? currentIndex : 0;
    const nextIndex = (startIndex + offset + fileStore.tabs.length) % fileStore.tabs.length;
    const nextTab = fileStore.tabs[nextIndex];
    if (!nextTab || nextTab.id === fileStore.activeTabId) return false;
    await switchTab(nextTab.id);
    return true;
  }

  const tabShortcutTargetId = (code: string) => {
    const match = /^(?:Digit|Numpad)([1-9])$/.exec(code);
    if (!match) return "";
    const shortcutNumber = Number(match[1]);
    const index = shortcutNumber === 9 ? fileStore.tabs.length - 1 : shortcutNumber - 1;
    const nextTab = fileStore.tabs[index];
    return nextTab?.id ?? "";
  }

  const closeTabById = async (tabId: string) => {
    if (fileStore.tabs.length <= 1) return false;
    const wasActive = fileStore.activeTabId === tabId;
    if (wasActive && !await fileStore.requestEditorLeave()) return false;
    if (wasActive) persistCurrentExplorerScrollTop();
    fileStore.closeTab(tabId);
    if (wasActive) {
      closePanels();
      await syncActiveTabContext();
    }
    return true;
  }

  const closeActiveTab = async () => {
    if (fileStore.tabs.length <= 1) return false;
    if (!await fileStore.requestEditorLeave()) return false;
    persistCurrentExplorerScrollTop();
    fileStore.closeTab(fileStore.activeTabId);
    closePanels();
    await syncActiveTabContext();
    return true;
  }

  const closeTab = (event: MouseEvent, tabId: string) => {
    event.stopPropagation();
    closeTabContextMenu();
    void closeTabById(tabId);
  }

  const handleTabAuxClick = (event: MouseEvent, tabId: string) => {
    if (event.button !== 1) return;
    event.preventDefault();
    event.stopPropagation();
    closeTabContextMenu();
    void closeTabById(tabId);
  }

  const duplicateTabFromMenu = async () => {
    const tabId = tabContextMenu.value.tabId;
    if (!await fileStore.requestEditorLeave()) return;
    persistCurrentExplorerScrollTop();
    closeTabContextMenu();
    fileStore.duplicateTab(tabId);
    closePanels();
    await syncActiveTabContext();
  }

  const closeTabFromMenu = async () => {
    const tabId = tabContextMenu.value.tabId;
    closeTabContextMenu();
    await closeTabById(tabId);
  }

  const reopenClosedTab = async () => {
    if (!canReopenClosedTab.value) return false;
    if (!await fileStore.requestEditorLeave()) return false;
    persistCurrentExplorerScrollTop();
    closeTabContextMenu();
    const tab = fileStore.reopenClosedTab();
    if (!tab) return false;
    closePanels();
    await syncActiveTabContext();
    showNotice(`已重新打开：${tab.title}`, "info", "标签页", 1600);
    return true;
  }

  const closeOtherTabsFromMenu = async () => {
    const tabId = tabContextMenu.value.tabId;
    const changesActiveTab = fileStore.activeTabId !== tabId;
    if (changesActiveTab && !await fileStore.requestEditorLeave()) return;
    if (changesActiveTab) persistCurrentExplorerScrollTop();
    closeTabContextMenu();
    fileStore.closeOtherTabs(tabId);
    if (changesActiveTab) {
      closePanels();
      await syncActiveTabContext();
    }
  }

  const closeRightTabsFromMenu = async () => {
    const tabId = tabContextMenu.value.tabId;
    const closesActiveTab = tabContextIndex.value >= 0 && fileStore.tabs.findIndex(tab => tab.id === fileStore.activeTabId) > tabContextIndex.value;
    if (closesActiveTab && !await fileStore.requestEditorLeave()) return;
    if (closesActiveTab) persistCurrentExplorerScrollTop();
    closeTabContextMenu();
    fileStore.closeTabsToRight(tabId);
    if (closesActiveTab) {
      closePanels();
      await syncActiveTabContext();
    }
  }

  const startTabDrag = (event: DragEvent, tabId: string) => {
    if (event.target instanceof HTMLElement && event.target.closest(".tab-close")) {
      event.preventDefault();
      return;
    }
    draggingTabId.value = tabId;
    tabDropTargetId.value = "";
    tabDropPlacement.value = "";
    closeTabContextMenu();
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.dropEffect = "move";
      event.dataTransfer.setData("text/plain", tabId);
    }
  }

  const dragOverTab = (event: DragEvent, tabId: string) => {
    if (!draggingTabId.value || draggingTabId.value === tabId) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    const target = event.currentTarget instanceof HTMLElement ? event.currentTarget : null;
    const rect = target?.getBoundingClientRect();
    tabDropTargetId.value = tabId;
    tabDropPlacement.value = rect && event.clientX > rect.left + rect.width / 2 ? "after" : "before";
  }

  const leaveTabDropTarget = (event: DragEvent, tabId: string) => {
    if (tabDropTargetId.value !== tabId) return;
    const related = event.relatedTarget;
    if (related instanceof Node && event.currentTarget instanceof HTMLElement && event.currentTarget.contains(related)) return;
    tabDropTargetId.value = "";
    tabDropPlacement.value = "";
  }

  const dropTab = (event: DragEvent, tabId: string) => {
    if (!draggingTabId.value || draggingTabId.value === tabId || !tabDropPlacement.value) return;
    event.preventDefault();
    event.stopPropagation();
    fileStore.reorderTab(draggingTabId.value, tabId, tabDropPlacement.value);
    draggingTabId.value = "";
    tabDropTargetId.value = "";
    tabDropPlacement.value = "";
  }

  const finishTabDrag = () => {
    draggingTabId.value = "";
    tabDropTargetId.value = "";
    tabDropPlacement.value = "";
  }

  return {
    tabContextMenu,
    tabContextTarget,
    canCloseTabContext,
    canCloseOtherTabsContext,
    canCloseRightTabsContext,
    canReopenClosedTab,
    draggingTabId,
    tabDropTargetId,
    tabDropPlacement,
    closeTabContextMenu,
    openTabContextMenu,
    openTab,
    openEntryInNewTab,
    switchTab,
    switchRelativeTab,
    tabShortcutTargetId,
    closeActiveTab,
    closeTab,
    handleTabAuxClick,
    duplicateTabFromMenu,
    closeTabFromMenu,
    reopenClosedTab,
    closeOtherTabsFromMenu,
    closeRightTabsFromMenu,
    startTabDrag,
    dragOverTab,
    leaveTabDropTarget,
    dropTab,
    finishTabDrag
  };
}
