import {computed, ref} from "vue";
import type {
  RuntimeSettings,
  TrashBatchPurgeResponse,
  TrashBatchRestoreResponse,
  TrashRecord,
  TrashRestoreResponse
} from "../class.ts";

type ConflictPolicy = RuntimeSettings["conflictPolicy"];
type TrashSelectOptions = {
  range?: boolean;
  toggle?: boolean;
}

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
  const visible = ref(false);
  const loading = ref(false);
  const actionLoading = ref(false);
  const records = ref<TrashRecord[]>([]);
  const message = ref("");
  const selectedId = ref("");
  const selectedIds = ref<string[]>([]);
  const selectionAnchorId = ref("");

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
      showError(error, "加载回收站失败", "回收站加载失败");
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
      showError(error, "恢复回收站项目失败", "恢复失败");
    } finally {
      actionLoading.value = false;
    }
  }

  const deleteSelected = async () => {
    const recordsToDelete = [...selectedRecords.value];
    if (!recordsToDelete.length || actionLoading.value) return;
    const confirmText = recordsToDelete.length === 1
        ? `永久删除 ${recordsToDelete[0].originalVirtualPath}？此操作无法撤销。`
        : `永久删除选中的 ${recordsToDelete.length} 项？此操作无法撤销。`;
    if (!window.confirm(confirmText)) return;
    actionLoading.value = true;
    message.value = "";
    try {
      const response = recordsToDelete.length === 1
          ? await deleteSingleRecord(recordsToDelete[0])
          : await deleteTrashRecords(recordsToDelete.map(record => record.id));
      message.value = trashBatchPurgeMessage(response);
      await load(true);
    } catch (error) {
      showError(error, "永久删除回收站项目失败", "永久删除失败");
    } finally {
      actionLoading.value = false;
    }
  }

  const empty = async () => {
    if (!hasRecords.value || actionLoading.value) return;
    if (!window.confirm("清空回收站？此操作无法撤销。")) return;
    actionLoading.value = true;
    message.value = "";
    try {
      const response = await emptyTrash();
      message.value = response.removed > 0 ? `已清空 ${response.removed} 项` : "回收站已经是空的";
      await load(true);
    } catch (error) {
      showError(error, "清空回收站失败", "清空失败");
    } finally {
      actionLoading.value = false;
    }
  }

  const cleanup = async () => {
    if (actionLoading.value) return;
    actionLoading.value = true;
    message.value = "";
    try {
      const response = await cleanupTrash();
      message.value = response.removed > 0 ? `已按策略清理 ${response.removed} 项` : "没有需要清理的项目";
      await load(true);
    } catch (error) {
      showError(error, "按策略清理回收站失败", "清理失败");
    } finally {
      actionLoading.value = false;
    }
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
      return `已恢复到 ${response.restored[0]?.restoredVirtualPath ?? "-"}`;
    }
    if (response.failed > 0) return `已恢复 ${response.success} 项，${response.failed} 项失败`;
    return `已恢复 ${response.success} 项`;
  }

  const trashBatchPurgeMessage = (response: TrashBatchPurgeResponse) => {
    if (response.failed > 0) return `已永久删除 ${response.success} 项，${response.failed} 项失败`;
    return `已永久删除 ${response.success} 项`;
  }

  return {
    visible,
    loading,
    actionLoading,
    records,
    message,
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
    selectAllRecords,
    clearSelection,
    restoreSelected,
    deleteSelected,
    empty,
    cleanup
  };
}
