import type {Ref} from "vue";
import type {DeleteConfirmState, OperationPanelState, PropertiesPanelState} from "../components/operations/types.ts";
import {usePanelFocusRestore} from "./usePanelFocusRestore.ts";

type MainViewPanelClosersOptions = {
  editorVisible: () => boolean;
  focusExplorer: () => Promise<void>;
  taskPanelVisible: Ref<boolean>;
  operationPanel: Ref<OperationPanelState>;
  deleteConfirm: Ref<DeleteConfirmState>;
  propertiesPanel: Ref<PropertiesPanelState>;
  previewPanelVisible: Ref<boolean>;
  imageViewerVisible: Ref<boolean>;
  closeTaskPanel: () => void;
  closeOperationPanel: () => void;
  closeDeleteConfirm: () => void;
  closePropertiesPanel: () => void;
  closePreview: () => void;
  closeImageViewer: () => void;
}

export const useMainViewPanelClosers = ({
  editorVisible,
  focusExplorer,
  taskPanelVisible,
  operationPanel,
  deleteConfirm,
  propertiesPanel,
  previewPanelVisible,
  imageViewerVisible,
  closeTaskPanel,
  closeOperationPanel,
  closeDeleteConfirm,
  closePropertiesPanel,
  closePreview,
  closeImageViewer
}: MainViewPanelClosersOptions) => {
  const {closeAndFocusExplorer} = usePanelFocusRestore({
    editorVisible,
    focusExplorer
  });

  return {
    closeTaskPanelAndFocus: () => closeAndFocusExplorer(() => taskPanelVisible.value, closeTaskPanel),
    closeOperationPanelAndFocus: () => closeAndFocusExplorer(() => operationPanel.value.visible, closeOperationPanel),
    closeDeleteConfirmAndFocus: () => closeAndFocusExplorer(() => deleteConfirm.value.visible, closeDeleteConfirm),
    closePropertiesPanelAndFocus: () => closeAndFocusExplorer(() => propertiesPanel.value.visible, closePropertiesPanel),
    closePreviewAndFocus: () => closeAndFocusExplorer(() => previewPanelVisible.value, closePreview),
    closeImageViewerAndFocus: () => closeAndFocusExplorer(() => imageViewerVisible.value, closeImageViewer)
  };
}
