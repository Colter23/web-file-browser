import {computed, nextTick, ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import type {DeleteConfirmState, OperationPanelState, PropertiesPanelState} from "../components/operations/types.ts";
import {useI18n} from "../i18n";

type FileOperationPanelsOptions = {
  closeShellPanels: () => void;
  focusOperationPanel: () => void;
  focusDeleteConfirm: () => void;
  focusPropertiesPanel: () => void;
}

type OperationPanelDraft = Omit<OperationPanelState, "visible" | "submitting">;

const emptyOperationPanel = (): OperationPanelState => ({
  visible: false,
  kind: null,
  title: "",
  message: "",
  primaryText: "",
  name: "",
  format: "zip",
  entries: [],
  sourceEntry: null,
  submitting: false
});

const emptyDeleteConfirm = (): DeleteConfirmState => ({
  visible: false,
  entries: [],
  permanent: false,
  submitting: false,
  error: ""
});

const emptyPropertiesPanel = (): PropertiesPanelState => ({
  visible: false,
  entries: []
});

export const useFileOperationPanels = ({
  closeShellPanels,
  focusOperationPanel,
  focusDeleteConfirm,
  focusPropertiesPanel
}: FileOperationPanelsOptions) => {
  const {t} = useI18n();
  const operationPanel = ref<OperationPanelState>(emptyOperationPanel());
  const deleteConfirm = ref<DeleteConfirmState>(emptyDeleteConfirm());
  const propertiesPanel = ref<PropertiesPanelState>(emptyPropertiesPanel());

  const operationPanelNameLabel = computed(() => {
    switch (operationPanel.value.kind) {
      case "createFile":
        return t("operation.fileName");
      case "createFolder":
        return t("operation.folderName");
      case "archive":
        return t("operation.archiveName");
      case "extract":
        return t("operation.extractFolder");
      default:
        return t("operation.name");
    }
  });

  const resetOperationPanel = () => {
    operationPanel.value = emptyOperationPanel();
  }

  const resetDeleteConfirm = () => {
    deleteConfirm.value = emptyDeleteConfirm();
  }

  const closePropertiesPanel = () => {
    propertiesPanel.value = emptyPropertiesPanel();
  }

  const closePanels = () => {
    closeShellPanels();
    resetOperationPanel();
    resetDeleteConfirm();
    closePropertiesPanel();
  }

  const openOperationPanel = async (next: OperationPanelDraft) => {
    closePanels();
    operationPanel.value = {
      ...next,
      visible: true,
      submitting: false
    };
    await nextTick();
    focusOperationPanel();
  }

  const closeOperationPanel = () => {
    if (operationPanel.value.submitting) return;
    resetOperationPanel();
  }

  const openDeleteConfirm = async (entries: ExplorerEntry[]) => {
    closePanels();
    deleteConfirm.value = {
      visible: true,
      entries,
      permanent: false,
      submitting: false,
      error: ""
    };
    await nextTick();
    focusDeleteConfirm();
  }

  const closeDeleteConfirm = () => {
    if (deleteConfirm.value.submitting) return;
    resetDeleteConfirm();
  }

  const openPropertiesPanel = async (entries: ExplorerEntry[]) => {
    closePanels();
    propertiesPanel.value = {
      visible: true,
      entries
    };
    await nextTick();
    focusPropertiesPanel();
  }

  return {
    operationPanel,
    deleteConfirm,
    propertiesPanel,
    operationPanelNameLabel,
    resetOperationPanel,
    resetDeleteConfirm,
    closePropertiesPanel,
    closePanels,
    openOperationPanel,
    closeOperationPanel,
    openDeleteConfirm,
    closeDeleteConfirm,
    openPropertiesPanel
  };
}
