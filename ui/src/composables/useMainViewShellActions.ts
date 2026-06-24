import type {Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type MainViewShellActionsOptions = {
  previewPanelVisible: Ref<boolean>;
  currentSelection: Ref<ExplorerEntry[]>;
  editorVisible: () => boolean;
  currentFolder: () => string;
  loadRoot: (options?: {forceRefresh?: boolean}) => Promise<unknown>;
  refreshExplorer: (path: string, options?: {forceRefresh?: boolean}) => Promise<unknown>;
  selectPaths: (paths: string[]) => Promise<boolean | undefined>;
  clearPersistedSelection: () => void;
  closePreviewPanel: () => void;
  resetPreviewContext: () => void;
  resetImageViewer: () => void;
  closeImageViewer: () => void;
  hideOperationPanel: () => void;
  resetOperationPanel: () => void;
  resetDeleteConfirm: () => void;
  closePropertiesPanel: () => void;
  resetTaskCancelConfirm: () => void;
}

export const useMainViewShellActions = ({
  previewPanelVisible,
  currentSelection,
  editorVisible,
  currentFolder,
  loadRoot,
  refreshExplorer,
  selectPaths,
  clearPersistedSelection,
  closePreviewPanel,
  resetPreviewContext,
  resetImageViewer,
  closeImageViewer,
  hideOperationPanel,
  resetOperationPanel,
  resetDeleteConfirm,
  closePropertiesPanel,
  resetTaskCancelConfirm
}: MainViewShellActionsOptions) => {
  const closePanels = () => {
    closePreviewPanel();
    hideOperationPanel();
    resetDeleteConfirm();
    closePropertiesPanel();
    resetTaskCancelConfirm();
    resetImageViewer();
  }

  const closeTransientPanels = () => {
    resetPreviewContext();
    hideOperationPanel();
    resetDeleteConfirm();
    closePropertiesPanel();
    resetTaskCancelConfirm();
    closeImageViewer();
  }

  const closeOperationShellPanels = () => {
    closePreviewPanel();
    resetTaskCancelConfirm();
  }

  const closePreview = () => {
    closePanels();
  }

  const resetPanelsForKeptPreview = () => {
    resetPreviewContext();
    resetOperationPanel();
    resetDeleteConfirm();
    resetTaskCancelConfirm();
    resetImageViewer();
  }

  const refreshCurrent = async (keepSelection = false) => {
    const keepPreview = keepSelection && previewPanelVisible.value && !editorVisible();
    const selectedPaths = keepSelection ? currentSelection.value.map(entry => entry.path) : [];
    if (keepPreview) {
      resetPanelsForKeptPreview();
    } else {
      closePanels();
    }
    const path = currentFolder();
    if (path === "/") await loadRoot({forceRefresh: true});
    await refreshExplorer(path, {forceRefresh: true});
    if (!selectedPaths.length) return;
    const restored = await selectPaths(selectedPaths);
    if (!restored) clearPersistedSelection();
  }

  return {
    closePanels,
    closeTransientPanels,
    closeOperationShellPanels,
    closePreview,
    refreshCurrent
  };
}
