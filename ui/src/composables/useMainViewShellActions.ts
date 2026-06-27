import type {Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type MainViewShellActionsOptions = {
  previewPanelVisible: Ref<boolean>;
  currentSelection: Ref<ExplorerEntry[]>;
  editorVisible: () => boolean;
  currentFolder: () => string;
  refreshExplorer: (path: string, options?: {forceRefresh?: boolean}) => Promise<unknown>;
  refreshTreePath: (path: string) => Promise<unknown>;
  selectPaths: (paths: string[]) => Promise<boolean | undefined>;
  clearPersistedSelection: () => void;
  closePreviewPanel: () => void;
  resetPreviewContext: () => void;
  resetImageViewer: () => void;
  closeImageViewer: () => void;
  resetVideoViewer: () => void;
  closeVideoViewer: () => void;
  resetPdfViewer: () => void;
  closePdfViewer: () => void;
  hideOperationPanel: () => void;
  resetOperationPanel: () => void;
  resetDeleteConfirm: () => void;
  closePropertiesPanel: () => void;
  resetTaskCancelConfirm: () => void;
  closeTrashPanel: () => void;
}

export const useMainViewShellActions = ({
  previewPanelVisible,
  currentSelection,
  editorVisible,
  currentFolder,
  refreshExplorer,
  refreshTreePath,
  selectPaths,
  clearPersistedSelection,
  closePreviewPanel,
  resetPreviewContext,
  resetImageViewer,
  closeImageViewer,
  resetVideoViewer,
  closeVideoViewer,
  resetPdfViewer,
  closePdfViewer,
  hideOperationPanel,
  resetOperationPanel,
  resetDeleteConfirm,
  closePropertiesPanel,
  resetTaskCancelConfirm,
  closeTrashPanel
}: MainViewShellActionsOptions) => {
  const closePanels = () => {
    closePreviewPanel();
    hideOperationPanel();
    resetDeleteConfirm();
    closePropertiesPanel();
    resetTaskCancelConfirm();
    closeTrashPanel();
    resetImageViewer();
    resetVideoViewer();
    resetPdfViewer();
  }

  const closeTransientPanels = () => {
    resetPreviewContext();
    hideOperationPanel();
    resetDeleteConfirm();
    closePropertiesPanel();
    resetTaskCancelConfirm();
    closeTrashPanel();
    closeImageViewer();
    closeVideoViewer();
    closePdfViewer();
  }

  const closeOperationShellPanels = () => {
    closePreviewPanel();
    resetTaskCancelConfirm();
    closeTrashPanel();
  }

  const closePreview = () => {
    closePanels();
  }

  const resetPanelsForKeptPreview = () => {
    resetPreviewContext();
    resetOperationPanel();
    resetDeleteConfirm();
    resetTaskCancelConfirm();
    closeTrashPanel();
    resetImageViewer();
    resetVideoViewer();
    resetPdfViewer();
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
    await Promise.all([
      refreshExplorer(path, {forceRefresh: true}),
      refreshTreePath(path)
    ]);
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
