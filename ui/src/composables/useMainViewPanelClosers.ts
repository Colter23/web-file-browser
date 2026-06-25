import type {Ref} from "vue";
import type {DeleteConfirmState, OperationPanelState, PropertiesPanelState} from "../components/operations/types.ts";
import {usePanelFocusRestore} from "./usePanelFocusRestore.ts";

type MainViewPanelClosersOptions = {
  editorVisible: () => boolean;
  focusExplorer: () => Promise<void>;
  taskPanelVisible: Ref<boolean>;
  trashPanelVisible: Ref<boolean>;
  operationPanel: Ref<OperationPanelState>;
  deleteConfirm: Ref<DeleteConfirmState>;
  propertiesPanel: Ref<PropertiesPanelState>;
  previewPanelVisible: Ref<boolean>;
  imageViewerVisible: Ref<boolean>;
  videoViewerVisible: Ref<boolean>;
  pdfViewerVisible: Ref<boolean>;
  closeTaskPanel: () => void;
  closeTrashPanel: () => void;
  closeOperationPanel: () => void;
  closeDeleteConfirm: () => void;
  closePropertiesPanel: () => void;
  closePreview: () => void;
  closeImageViewer: () => void;
  closeVideoViewer: () => void;
  closePdfViewer: () => void;
}

export const useMainViewPanelClosers = ({
  editorVisible,
  focusExplorer,
  taskPanelVisible,
  trashPanelVisible,
  operationPanel,
  deleteConfirm,
  propertiesPanel,
  previewPanelVisible,
  imageViewerVisible,
  videoViewerVisible,
  pdfViewerVisible,
  closeTaskPanel,
  closeTrashPanel,
  closeOperationPanel,
  closeDeleteConfirm,
  closePropertiesPanel,
  closePreview,
  closeImageViewer,
  closeVideoViewer,
  closePdfViewer
}: MainViewPanelClosersOptions) => {
  const {closeAndFocusExplorer} = usePanelFocusRestore({
    editorVisible,
    focusExplorer
  });

  return {
    closeTaskPanelAndFocus: () => closeAndFocusExplorer(() => taskPanelVisible.value, closeTaskPanel),
    closeTrashPanelAndFocus: () => closeAndFocusExplorer(() => trashPanelVisible.value, closeTrashPanel),
    closeOperationPanelAndFocus: () => closeAndFocusExplorer(() => operationPanel.value.visible, closeOperationPanel),
    closeDeleteConfirmAndFocus: () => closeAndFocusExplorer(() => deleteConfirm.value.visible, closeDeleteConfirm),
    closePropertiesPanelAndFocus: () => closeAndFocusExplorer(() => propertiesPanel.value.visible, closePropertiesPanel),
    closePreviewAndFocus: () => closeAndFocusExplorer(() => previewPanelVisible.value, closePreview),
    closeImageViewerAndFocus: () => closeAndFocusExplorer(() => imageViewerVisible.value, closeImageViewer),
    closeVideoViewerAndFocus: () => closeAndFocusExplorer(() => videoViewerVisible.value, closeVideoViewer),
    closePdfViewerAndFocus: () => closeAndFocusExplorer(() => pdfViewerVisible.value, closePdfViewer)
  };
}
