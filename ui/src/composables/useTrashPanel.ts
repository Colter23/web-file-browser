import {computed, ref} from "vue";
import type {RuntimeSettings, TrashRecord, TrashRestoreResponse} from "../class.ts";

type ConflictPolicy = RuntimeSettings["conflictPolicy"];

type TrashPanelOptions = {
  listTrashRecords: () => Promise<TrashRecord[]>;
  restoreTrashRecord: (id: string, conflictPolicy?: ConflictPolicy) => Promise<TrashRestoreResponse>;
  deleteTrashRecord: (id: string) => Promise<void>;
  emptyTrash: () => Promise<{removed: number}>;
  cleanupTrash: () => Promise<{removed: number}>;
  showError: (error: unknown, fallback: string, title?: string) => void;
  onRestored?: (response: TrashRestoreResponse) => void | Promise<void>;
}

export const useTrashPanel = ({
  listTrashRecords,
  restoreTrashRecord,
  deleteTrashRecord,
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

  const selectedRecord = computed(() => records.value.find(record => record.id === selectedId.value) ?? records.value[0] ?? null);
  const hasRecords = computed(() => records.value.length > 0);

  const ensureSelection = () => {
    if (!records.value.length) {
      selectedId.value = "";
      return;
    }
    if (!records.value.some(record => record.id === selectedId.value)) {
      selectedId.value = records.value[0].id;
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

  const selectRecord = (id: string) => {
    selectedId.value = id;
  }

  const restoreSelected = async () => {
    const record = selectedRecord.value;
    if (!record || actionLoading.value) return;
    actionLoading.value = true;
    message.value = "";
    try {
      const response = await restoreTrashRecord(record.id);
      message.value = `已恢复到 ${response.restoredVirtualPath}`;
      await onRestored?.(response);
      await load(true);
    } catch (error) {
      showError(error, "恢复回收站项目失败", "恢复失败");
    } finally {
      actionLoading.value = false;
    }
  }

  const deleteSelected = async () => {
    const record = selectedRecord.value;
    if (!record || actionLoading.value) return;
    if (!window.confirm(`永久删除 ${record.originalVirtualPath}？此操作无法撤销。`)) return;
    actionLoading.value = true;
    message.value = "";
    try {
      await deleteTrashRecord(record.id);
      message.value = "已永久删除 1 项";
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

  return {
    visible,
    loading,
    actionLoading,
    records,
    message,
    selectedId,
    selectedRecord,
    hasRecords,
    load,
    open,
    close,
    toggle,
    selectRecord,
    restoreSelected,
    deleteSelected,
    empty,
    cleanup
  };
}
