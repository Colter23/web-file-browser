import {nextTick, watch} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {ExplorerTab} from "../class.ts";
import {useFileStore} from "../store";

type ExplorerTabContextOptions = {
  activeTab: ComputedRef<ExplorerTab | undefined>;
  searchText: Ref<string>;
  selectPaths: (paths: string[], scrollToSelection?: boolean) => Promise<boolean | undefined>;
  getScrollTop: () => number;
  setScrollTop: (scrollTop: number) => Promise<void | undefined>;
}

export const useExplorerTabContext = ({
  activeTab,
  searchText,
  selectPaths,
  getScrollTop,
  setScrollTop
}: ExplorerTabContextOptions) => {
  const fileStore = useFileStore();
  let suppressSelectionPersistence = false;
  let suppressScrollPersistence = false;
  let tabContextRestoreToken = 0;
  let scrollPersistTimer: number | undefined;

  const shouldPersistSelection = () => !fileStore.showEditor && !suppressSelectionPersistence;

  const persistSelectedPaths = (paths: string[]) => {
    fileStore.setActiveTabSelectedPaths(paths);
  }

  const finishSelectionRestore = (token: number) => {
    if (token !== tabContextRestoreToken) return;
    suppressSelectionPersistence = false;
  }

  const restoreActiveTabSelection = async (paths: string[], token: number, attempt = 0) => {
    if (!paths.length || token !== tabContextRestoreToken) {
      finishSelectionRestore(token);
      return;
    }
    const restored = await selectPaths(paths, false);
    if (restored || token !== tabContextRestoreToken || attempt >= 6) {
      finishSelectionRestore(token);
      return;
    }
    window.setTimeout(() => {
      void restoreActiveTabSelection(paths, token, attempt + 1);
    }, 80);
  }

  const finishScrollRestore = (token: number) => {
    if (token !== tabContextRestoreToken) return;
    suppressScrollPersistence = false;
  }

  const persistActiveTabScrollTop = (scrollTop: number, tabId = fileStore.activeTabId, path = fileStore.currentPath) => {
    if (suppressScrollPersistence || fileStore.showEditor) return;
    if (scrollPersistTimer) window.clearTimeout(scrollPersistTimer);
    scrollPersistTimer = window.setTimeout(() => {
      scrollPersistTimer = undefined;
      if (suppressScrollPersistence || fileStore.showEditor || tabId !== fileStore.activeTabId || path !== fileStore.currentPath) return;
      fileStore.setActiveTabScrollTop(scrollTop);
    }, 120);
  }

  const persistCurrentExplorerScrollTop = () => {
    if (suppressScrollPersistence || fileStore.showEditor) return;
    if (scrollPersistTimer) {
      window.clearTimeout(scrollPersistTimer);
      scrollPersistTimer = undefined;
    }
    fileStore.setActiveTabScrollTop(getScrollTop());
  }

  const syncActiveTabContext = async () => {
    const tab = activeTab.value;
    const selectedPaths = [...(tab?.selectedPaths ?? [])];
    const scrollTop = tab?.scrollTop ?? 0;
    const token = ++tabContextRestoreToken;
    suppressSelectionPersistence = true;
    suppressScrollPersistence = true;
    searchText.value = tab?.filterText ?? "";
    await nextTick();
    await restoreActiveTabSelection(selectedPaths, token);
    await setScrollTop(scrollTop);
    finishScrollRestore(token);
  }

  const stopScrollPersistence = () => {
    if (!scrollPersistTimer) return;
    window.clearTimeout(scrollPersistTimer);
    scrollPersistTimer = undefined;
  }

  watch(searchText, text => {
    if (fileStore.showEditor) return;
    fileStore.setActiveTabFilterText(text);
  });

  return {
    shouldPersistSelection,
    persistSelectedPaths,
    persistActiveTabScrollTop,
    persistCurrentExplorerScrollTop,
    syncActiveTabContext,
    stopScrollPersistence
  };
}
