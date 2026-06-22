import {computed, nextTick, ref, watch} from "vue";
import type {FileInfo} from "../class.ts";
import type {EditorCursorStatus, PendingEditorAction} from "../components/editor/types.ts";
import {isApiError} from "../network";
import {getFile, saveFile} from "../network/file-api.ts";
import {useFileStore} from "../store";
import {checkFileLanguageMode} from "../utils/common.ts";

type EditorFileSessionOptions = {
  closeMenus: () => void;
  resetSearchState: () => void;
  focusEditor: () => void;
}

const defaultCursorStatus = (): EditorCursorStatus => ({line: 1, column: 1, selectedRows: 0, selectedCharacters: 0});

export const useEditorFileSession = ({
  closeMenus,
  resetSearchState,
  focusEditor
}: EditorFileSessionOptions) => {
  const fileStore = useFileStore();
  const fileInfo = ref<FileInfo | null>(null);
  const currentMode = ref("text");
  const content = ref("");
  const contentEtag = ref("");
  const isChange = ref(false);
  const loading = ref(false);
  const saving = ref(false);
  const statusText = ref("");
  const errorText = ref("");
  const saveConflict = ref(false);
  const pendingAction = ref<PendingEditorAction>("");
  const pendingBusy = ref(false);
  const cursorStatus = ref<EditorCursorStatus>(defaultCursorStatus());
  let loadVersion = 0;

  const canSave = computed(() => Boolean(fileInfo.value && isChange.value && contentEtag.value && !saveConflict.value && !loading.value && !saving.value));
  const editorReadOnly = computed(() => loading.value || saving.value || Boolean(pendingAction.value));

  const loadCurrentFile = async () => {
    if (!fileStore.showEditor || fileStore.currentFile == null) return;
    const version = ++loadVersion;
    const target = fileStore.currentFile;
    fileInfo.value = target;
    currentMode.value = checkFileLanguageMode(target.extension);
    statusText.value = "";
    errorText.value = "";
    saveConflict.value = false;
    cursorStatus.value = defaultCursorStatus();
    loading.value = true;
    try {
      const file = await getFile(target.path);
      if (version !== loadVersion) return;
      content.value = file.content;
      contentEtag.value = file.etag;
      isChange.value = false;
      statusText.value = "已打开";
      await nextTick();
      focusEditor();
    } catch (error) {
      if (version !== loadVersion) return;
      errorText.value = error instanceof Error ? error.message : "打开文件失败";
      content.value = "";
      contentEtag.value = "";
      isChange.value = false;
    } finally {
      if (version === loadVersion) loading.value = false;
    }
  }

  const resetEditorState = () => {
    closeMenus();
    resetSearchState();
    pendingAction.value = "";
    pendingBusy.value = false;
    fileStore.closeEditor();
    fileInfo.value = null;
    isChange.value = false;
    content.value = "";
    contentEtag.value = "";
    statusText.value = "";
    errorText.value = "";
    saveConflict.value = false;
    cursorStatus.value = defaultCursorStatus();
  }

  watch(() => [fileStore.showEditor, fileStore.currentFile?.path], loadCurrentFile);

  watch(isChange, value => {
    fileStore.setEditorDirty(value);
  });

  watch(() => fileStore.editorLeaveRequestId, requestId => {
    if (!requestId || !fileStore.showEditor) return;
    if (!isChange.value) {
      fileStore.resolveEditorLeave(true);
      return;
    }
    pendingAction.value = "external";
    closeMenus();
  });

  const onContentChange = (value: string) => {
    if (loading.value) return;
    pendingAction.value = "";
    content.value = value;
    isChange.value = true;
    statusText.value = "";
    errorText.value = "";
    if (saveConflict.value) saveConflict.value = false;
  }

  const onCursorChange = (status: EditorCursorStatus) => {
    cursorStatus.value = status;
  }

  const save = async () => {
    if (!fileInfo.value || saving.value || loading.value || saveConflict.value) return;
    saving.value = true;
    errorText.value = "";
    saveConflict.value = false;
    try {
      if (!contentEtag.value) {
        throw new Error("文件版本信息缺失，请重新打开文件后再保存");
      }
      const saved = await saveFile(fileInfo.value.path, content.value, contentEtag.value);
      contentEtag.value = saved.etag;
      isChange.value = false;
      statusText.value = "已保存";
    } catch (error) {
      if (isApiError(error) && (error.status === 412 || error.status === 428 || error.code === "PRECONDITION_FAILED" || error.code === "PRECONDITION_REQUIRED")) {
        saveConflict.value = true;
        errorText.value = error.status === 428
            ? "缺少文件版本信息，请重新载入后再保存"
            : "文件已被外部修改，请重新载入最新版本后再保存";
      } else {
        errorText.value = error instanceof Error ? error.message : "保存失败";
      }
    } finally {
      saving.value = false;
    }
  }

  const reload = async () => {
    if (isChange.value) {
      pendingAction.value = "reload";
      closeMenus();
      return;
    }
    await loadCurrentFile();
  }

  const close = () => {
    if (isChange.value) {
      pendingAction.value = "close";
      closeMenus();
      return;
    }
    resetEditorState();
  }

  const cancelPendingAction = () => {
    if (pendingBusy.value) return;
    if (pendingAction.value === "external") fileStore.resolveEditorLeave(false);
    pendingAction.value = "";
    nextTick(() => focusEditor());
  }

  const finishPendingAction = async () => {
    const action = pendingAction.value;
    pendingAction.value = "";
    if (action === "reload") {
      await loadCurrentFile();
      return;
    }
    if (action === "external") {
      fileStore.resolveEditorLeave(true);
      return;
    }
    if (action === "close") resetEditorState();
  }

  const discardPendingAction = async () => {
    if (pendingBusy.value) return;
    pendingBusy.value = true;
    isChange.value = false;
    await finishPendingAction();
    pendingBusy.value = false;
  }

  const savePendingAction = async () => {
    if (pendingBusy.value || !canSave.value) return;
    pendingBusy.value = true;
    await save();
    pendingBusy.value = false;
    if (!isChange.value && !saveConflict.value && !errorText.value) {
      await finishPendingAction();
    }
  }

  const handleBeforeUnload = (event: BeforeUnloadEvent) => {
    if (!fileStore.showEditor || !isChange.value) return;
    event.preventDefault();
    event.returnValue = "";
  }

  const dispose = () => {
    fileStore.resolveEditorLeave(false);
  }

  return {
    fileInfo,
    currentMode,
    content,
    isChange,
    loading,
    saving,
    statusText,
    errorText,
    saveConflict,
    pendingAction,
    pendingBusy,
    cursorStatus,
    canSave,
    editorReadOnly,
    loadCurrentFile,
    onContentChange,
    onCursorChange,
    save,
    reload,
    close,
    cancelPendingAction,
    discardPendingAction,
    savePendingAction,
    handleBeforeUnload,
    dispose
  };
}
