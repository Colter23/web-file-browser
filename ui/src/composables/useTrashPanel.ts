import {computed, ref} from "vue";
import type {
  RuntimeSettings,
  TrashBatchPurgeResponse,
  TrashBatchRestoreResponse,
  TrashRecord,
  TrashRestoreResponse
} from "../class.ts";
import type {TrashConfirmState} from "../components/trash/types.ts";
import {useI18n} from "../i18n";

type ConflictPolicy = RuntimeSettings["conflictPolicy"];
type TrashSelectOptions = {
  range?: boolean;
  toggle?: boolean;
}

type TrashFocusMoveMode = "replaceSelection" | "extendSelection" | "moveFocusOnly";

type TrashPanelOptions = {
  listTrashRecords: () => Promise<TrashRecord[]>;
  restoreTrashRecord: (id: string, conflictPolicy?: ConflictPolicy) => Promise<TrashRestoreResponse>;
  restoreTrashRecords: (ids: string[], conflictPolicy?: ConflictPolicy) => Promise<TrashBatchRestoreResponse>;
  deleteTrashRecord: (id: string) => Promise<void>;
  deleteTrashRecords: (ids: string[]) => Promise<TrashBatchPurgeResponse>;
  emptyTrash: () => Promise<{removed: number}>;
  cleanupTrash: () => Promise<{removed: number}>;
  showError: (error: unknown, fallback: string, title?: string) => void;
  onRestored?: (response: TrashRestoreResponse) => void | Promise<void>;
}

const emptyTrashConfirm = (): TrashConfirmState => ({
  visible: false,
  kind: null,
  records: [],
  submitting: false,
  error: ""
});

export const useTrashPanel = ({
  listTrashRecords,
  restoreTrashRecord,
  restoreTrashRecords,
  deleteTrashRecord,
  deleteTrashRecords,
  emptyTrash,
  cleanupTrash,
  showError,
  onRestored
}: TrashPanelOptions) => {
  const {t} = useI18n();
  const visible = ref(false);
  const loading = ref(false);
  const actionLoading = ref(false);
  const records = ref<TrashRecord[]>([]);
  const message = ref("");
  const selectedId = ref("");
  const selectedIds = ref<string[]>([]);
  const selectionAnchorId = ref("");
  const confirm = ref<TrashConfirmState>(emptyTrashConfirm());

  const selectedIdSet = computed(() => new Set(selectedIds.value));
  const selectedRecords = computed(() => records.value.filter(record => selectedIdSet.value.has(record.id)));
  const selectedRecord = computed(() => {
    return records.value.find(record => record.id === selectedId.value)
        ?? selectedRecords.value[0]
        ?? null;
  });
  const selectedCount = computed(() => selectedRecords.value.length);
  const hasRecords = computed(() => records.value.length > 0);

  const normalizedSelection = (ids: string[]) => {
    const requested = new Set(ids);
    return records.value
        .map(record => record.id)
        .filter(id => requested.has(id));
  }

  const setSelection = (ids: string[], activeId = ids[0] ?? "") => {
    const normalized = normalizedSelection(ids);
    selectedIds.value = normalized;
    selectedId.value = activeId && normalized.includes(activeId) ? activeId : normalized[0] ?? "";
  }

  const ensureSelection = () => {
    if (!records.value.length) {
      selectedId.value = "";
      selectedIds.value = [];
      selectionAnchorId.value = "";
      return;
    }
    setSelection(selectedIds.value, selectedId.value);
    if (!selectedIds.value.length) {
      setSelection([records.value[0].id], records.value[0].id);
    }
    if (!selectionAnchorId.value || !records.value.some(record => record.id === selectionAnchorId.value)) {
      selectionAnchorId.value = selectedId.value;
    }
  }

  const load = async (silent = false) => {
    if (!silent) loading.value = true;
    try {
      records.value = await listTrashRecords();
      ensureSelection();
    } catch (error) {
      showError(error, t("trash.loadFailed"), t("trash.loadFailedTitle"));
    } finally {
      if (!silent) loading.value = false;
    }
  }

  const open = async () => {
    visible.value = true;
    await load();
  }

  const close = () => {
    visible.value = false;
    message.value = "";
    resetConfirm();
  }

  const toggle = async () => {
    if (visible.value) {
      close();
      return;
    }
    await open();
  }

  const selectRecord = (id: string, options: TrashSelectOptions = {}) => {
    if (!records.value.some(record => record.id === id)) return;

    if (options.range) {
      const anchorId = selectionAnchorId.value || selectedId.value || selectedIds.value[0] || id;
      const startIndex = records.value.findIndex(record => record.id === anchorId);
      const endIndex = records.value.findIndex(record => record.id === id);
      if (startIndex >= 0 && endIndex >= 0) {
        const [from, to] = startIndex < endIndex ? [startIndex, endIndex] : [endIndex, startIndex];
        const rangeIds = records.value.slice(from, to + 1).map(record => record.id);
        setSelection(options.toggle ? [...selectedIds.value, ...rangeIds] : rangeIds, id);
        return;
      }
    }

    selectionAnchorId.value = id;
    if (options.toggle) {
      if (selectedIdSet.value.has(id)) {
        const nextIds = selectedIds.value.filter(selected => selected !== id);
        const nextActive = selectedId.value === id ? nextIds[nextIds.length - 1] ?? "" : selectedId.value;
        setSelection(nextIds, nextActive);
      } else {
        setSelection([...selectedIds.value, id], id);
      }
      return;
    }

    setSelection([id], id);
  }

  const moveSelection = (direction: "next" | "previous" | "first" | "last", mode: TrashFocusMoveMode = "replaceSelection") => {
    if (!records.value.length) return;
    const currentIndex = selectedId.value
        ? records.value.findIndex(record => record.id === selectedId.value)
        : -1;
    let nextIndex = currentIndex >= 0 ? currentIndex : 0;
    if (direction === "next") nextIndex = Math.min(records.value.length - 1, nextIndex + 1);
    if (direction === "previous") nextIndex = Math.max(0, nextIndex - 1);
    if (direction === "first") nextIndex = 0;
    if (direction === "last") nextIndex = records.value.length - 1;
    const nextRecord = records.value[nextIndex];
    if (!nextRecord) return;
    if (mode === "moveFocusOnly") {
      selectedId.value = nextRecord.id;
      return;
    }
    selectRecord(nextRecord.id, {range: mode === "extendSelection"});
  }

  const selectAllRecords = () => {
    const ids = records.value.map(record => record.id);
    setSelection(ids, selectedId.value || (ids[0] ?? ""));
    if (!selectionAnchorId.value) selectionAnchorId.value = selectedId.value;
  }

  const clearSelection = () => {
    selectedIds.value = [];
    selectedId.value = "";
    selectionAnchorId.value = "";
  }

  const toggleFocusedRecord = () => {
    const focusedId = selectedId.value || selectedIds.value[0];
    if (!focusedId) return;
    selectRecord(focusedId, {toggle: true});
  }

  const resetConfirm = () => {
    confirm.value = emptyTrashConfirm();
  }

  const closeConfirm = () => {
    if (confirm.value.submitting) return;
    resetConfirm();
  }

  const restoreSelected = async () => {
    const recordsToRestore = [...selectedRecords.value];
    if (!recordsToRestore.length || actionLoading.value) return;
    actionLoading.value = true;
    message.value = "";
    try {
      const response = recordsToRestore.length === 1
          ? await restoreSingleRecord(recordsToRestore[0])
          : await restoreTrashRecords(recordsToRestore.map(record => record.id));
      message.value = trashBatchRestoreMessage(response);
      const lastResponse = response.restored[response.restored.length - 1];
      if (lastResponse) await onRestored?.(lastResponse);
      await load(true);
    } catch (error) {
      showError(error, t("trash.restoreFailed"), t("trash.restoreFailedTitle"));
    } finally {
      actionLoading.value = false;
    }
  }

  const deleteSelected = async () => {
    const recordsToDelete = [...selectedRecords.value];
    if (!recordsToDelete.length || actionLoading.value) return;
    confirm.value = {
      visible: true,
      kind: "delete",
      records: recordsToDelete,
      submitting: false,
      error: ""
    };
  }

  const empty = async () => {
    if (!hasRecords.value || actionLoading.value) return;
    confirm.value = {
      visible: true,
      kind: "empty",
      records: [],
      submitting: false,
      error: ""
    };
  }

  const submitConfirm = async () => {
    if (!confirm.value.visible || !confirm.value.kind || confirm.value.submitting || actionLoading.value) return;
    confirm.value.submitting = true;
    confirm.value.error = "";
    actionLoading.value = true;
    message.value = "";
    try {
      if (confirm.value.kind === "delete") {
        const recordsToDelete = [...confirm.value.records];
        if (!recordsToDelete.length) return;
        const response = recordsToDelete.length === 1
            ? await deleteSingleRecord(recordsToDelete[0])
            : await deleteTrashRecords(recordsToDelete.map(record => record.id));
        message.value = trashBatchPurgeMessage(response);
      } else if (confirm.value.kind === "empty") {
        const response = await emptyTrash();
        message.value = response.removed > 0 ? t("trash.emptied", {count: response.removed}) : t("trash.alreadyEmpty");
      } else {
        const response = await cleanupTrash();
        message.value = response.removed > 0 ? t("trash.cleaned", {count: response.removed}) : t("trash.noCleanup");
      }
      resetConfirm();
      await load(true);
    } catch (error) {
      confirm.value.error = error instanceof Error ? error.message : confirmErrorMessage(confirm.value.kind);
    } finally {
      actionLoading.value = false;
      if (confirm.value.visible) confirm.value.submitting = false;
    }
  }

  const cleanup = async () => {
    if (!hasRecords.value || actionLoading.value) return;
    confirm.value = {
      visible: true,
      kind: "cleanup",
      records: [],
      submitting: false,
      error: ""
    };
  }

  const restoreSingleRecord = async (record: TrashRecord): Promise<TrashBatchRestoreResponse> => {
    const restored = await restoreTrashRecord(record.id);
    return {
      restored: [restored],
      errors: [],
      success: 1,
      failed: 0
    };
  }

  const deleteSingleRecord = async (record: TrashRecord): Promise<TrashBatchPurgeResponse> => {
    await deleteTrashRecord(record.id);
    return {
      purged: [record.id],
      errors: [],
      success: 1,
      failed: 0
    };
  }

  const trashBatchRestoreMessage = (response: TrashBatchRestoreResponse) => {
    if (response.success === 1 && response.failed === 0) {
      return t("trash.restoredTo", {path: response.restored[0]?.restoredVirtualPath ?? "-"});
    }
    if (response.failed > 0) return t("trash.restoredPartial", {success: response.success, failed: response.failed});
    return t("trash.restored", {count: response.success});
  }

  const trashBatchPurgeMessage = (response: TrashBatchPurgeResponse) => {
    if (response.failed > 0) return t("trash.purgedPartial", {success: response.success, failed: response.failed});
    return t("trash.purged", {count: response.success});
  }

  const confirmErrorMessage = (kind: TrashConfirmState["kind"]) => {
    if (kind === "delete") return t("trash.deleteFailed");
    if (kind === "cleanup") return t("trash.cleanupFailed");
    return t("trash.emptyFailed");
  }

  return {
    visible,
    loading,
    actionLoading,
    records,
    message,
    confirm,
    selectedId,
    selectedIds,
    selectedRecords,
    selectedRecord,
    selectedCount,
    hasRecords,
    load,
    open,
    close,
    toggle,
    selectRecord,
    moveSelection,
    selectAllRecords,
    clearSelection,
    toggleFocusedRecord,
    closeConfirm,
    submitConfirm,
    restoreSelected,
    deleteSelected,
    empty,
    cleanup
  };
}
