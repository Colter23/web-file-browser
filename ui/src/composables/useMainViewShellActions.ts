import type {Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type MainViewShellActionsOptions = {
  previewPanelVisible: Ref<boolean>;
  currentSelection: Ref<ExplorerEntry[]>;
  editorVisible: () => boolean;
  currentFolder: () => string;
  loadRoot: () => Promise<unknown>;
  refreshExplorer: (path: string) => Promise<unknown>;
  selectPaths: (paths: string[]) => Promise<boolean | undefined>;
  clearPersistedSelection: () => void;
  closePreviewPanel: () => void;
  clearPreviewContent: () => void;
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
  clearPreviewContent,
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
    clearPreviewContent();
    resetOperationPanel();
    resetDeleteConfirm();
    resetTaskCancelConfirm();
    closeImageViewer();
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
    if (path === "/") await loadRoot();
    await refreshExplorer(path);
    if (!selectedPaths.length) return;
    const restored = await selectPaths(selectedPaths);
    if (!restored) clearPersistedSelection();
  }

  return {
    closePanels,
    closeOperationShellPanels,
    closePreview,
    refreshCurrent
  };
}
