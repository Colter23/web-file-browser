import {computed, ref} from "vue";
import type {ComputedRef} from "vue";
import type {ExplorerTab} from "../class.ts";
import {useFileStore} from "../store";
import {normalizePathText, parentPath} from "../utils/file-path.ts";

type NavigateToPathOptions = {
  skipEditorLeave?: boolean;
  focusExplorer?: boolean;
}

type NavigateComplete = (navigated: boolean) => void;

type ExplorerNavigationOptions = {
  activeTab: ComputedRef<ExplorerTab | undefined>;
  refreshExplorer: (path: string) => Promise<boolean>;
  focusExplorer: () => Promise<void>;
  closePanels: () => void;
  syncActiveTabContext: () => Promise<void>;
  persistCurrentExplorerScrollTop: () => void;
  shouldIgnoreNavigationShortcut: (target: EventTarget | null) => boolean;
}

export const useExplorerNavigation = ({
  activeTab,
  refreshExplorer,
  focusExplorer,
  closePanels,
  syncActiveTabContext,
  persistCurrentExplorerScrollTop,
  shouldIgnoreNavigationShortcut
}: ExplorerNavigationOptions) => {
  const fileStore = useFileStore();
  const historyMouseButton = ref(-1);

  const currentFolder = () => fileStore.currentPath || "/";
  const canNavigateBack = computed(() => Boolean(activeTab.value?.backStack?.length));
  const canNavigateForward = computed(() => Boolean(activeTab.value?.forwardStack?.length));
  const canNavigateUp = computed(() => currentFolder() !== "/");
  const navigateBackTarget = computed(() => {
    const stack = activeTab.value?.backStack ?? [];
    return stack[stack.length - 1] ?? "";
  });
  const navigateForwardTarget = computed(() => activeTab.value?.forwardStack?.[0] ?? "");
  const navigateUpTarget = computed(() => canNavigateUp.value ? parentPath(currentFolder()) : "");
  const navigateBackTitle = computed(() => navigateBackTarget.value ? `后退到 ${navigateBackTarget.value} (Alt+← / 鼠标后退键)` : "后退 (Alt+← / 鼠标后退键)");
  const navigateForwardTitle = computed(() => navigateForwardTarget.value ? `前进到 ${navigateForwardTarget.value} (Alt+→ / 鼠标前进键)` : "前进 (Alt+→ / 鼠标前进键)");
  const navigateUpTitle = computed(() => navigateUpTarget.value ? `返回上级 ${navigateUpTarget.value} (Alt+↑)` : "返回上级 (Alt+↑)");

  const finishPathNavigation = async (path: string, focusAfterNavigation = true) => {
    closePanels();
    const loaded = await refreshExplorer(path);
    if (!loaded) {
      if (focusAfterNavigation) await focusExplorer();
      return false;
    }
    await syncActiveTabContext();
    if (focusAfterNavigation) await focusExplorer();
    return true;
  }

  const navigateToPath = async (path: string, options: NavigateToPathOptions = {}) => {
    const targetPath = normalizePathText(path);
    if (!options.skipEditorLeave && !await fileStore.requestEditorLeave()) return false;
    persistCurrentExplorerScrollTop();
    return finishPathNavigation(targetPath, options.focusExplorer ?? true);
  }

  const navigateBack = async () => {
    if (!await fileStore.requestEditorLeave()) return;
    persistCurrentExplorerScrollTop();
    const path = fileStore.goBack();
    if (!path) return;
    await finishPathNavigation(path);
  }

  const navigateForward = async () => {
    if (!await fileStore.requestEditorLeave()) return;
    persistCurrentExplorerScrollTop();
    const path = fileStore.goForward();
    if (!path) return;
    await finishPathNavigation(path);
  }

  const navigateUp = async () => {
    if (!canNavigateUp.value) return;
    await navigateToPath(parentPath(currentFolder()));
  }

  const handleBreadcrumbNavigate = async (path: string, complete?: NavigateComplete) => {
    const navigated = await navigateToPath(path);
    complete?.(navigated);
  }

  const handleBackspaceNavigation = () => {
    if (canNavigateBack.value) {
      void navigateBack();
      return;
    }
    void navigateUp();
  }

  const handleHistoryMouseButton = (event: MouseEvent) => {
    if (fileStore.showEditor || shouldIgnoreNavigationShortcut(event.target)) return false;
    if (event.button === 3 && canNavigateBack.value) {
      event.preventDefault();
      void navigateBack();
      return true;
    }
    if (event.button === 4 && canNavigateForward.value) {
      event.preventDefault();
      void navigateForward();
      return true;
    }
    return false;
  }

  const handleHistoryMouseDown = (event: MouseEvent) => {
    historyMouseButton.value = handleHistoryMouseButton(event) ? event.button : -1;
  }

  const handleHistoryMouseUp = (event: MouseEvent) => {
    if (historyMouseButton.value >= 0 && event.button === historyMouseButton.value) {
      event.preventDefault();
      historyMouseButton.value = -1;
    }
  }

  const handleHistoryAuxClick = (event: MouseEvent) => {
    if (event.button === 3 || event.button === 4) event.preventDefault();
  }

  return {
    currentFolder,
    canNavigateBack,
    canNavigateForward,
    canNavigateUp,
    navigateBackTitle,
    navigateForwardTitle,
    navigateUpTitle,
    navigateToPath,
    navigateBack,
    navigateForward,
    navigateUp,
    handleBreadcrumbNavigate,
    handleBackspaceNavigation,
    handleHistoryMouseDown,
    handleHistoryMouseUp,
    handleHistoryAuxClick
  };
}
