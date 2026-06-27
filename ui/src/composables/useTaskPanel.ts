import {computed, ref} from "vue";
import type {TaskStatus} from "../class.ts";
import {canCancelTask, canCleanupTask, shortTaskId} from "../utils/task-status.ts";

export type TaskCancelConfirmState = {
  visible: boolean;
  task: TaskStatus | null;
  submitting: boolean;
  error: string;
}

type TaskPanelOptions = {
  listTasks: () => Promise<TaskStatus[]>;
  cancelTask: (id: string) => Promise<unknown>;
  cleanupTasks: () => Promise<{removed: number}>;
  showNotice: (message: string, kind?: "info" | "success" | "warning" | "error", title?: string, timeoutMs?: number) => void;
  showError: (error: unknown, fallback: string, title?: string) => void;
  onTaskSettled?: (tasks: TaskStatus[]) => void | Promise<void>;
}

const emptyCancelConfirm = (): TaskCancelConfirmState => ({
  visible: false,
  task: null,
  submitting: false,
  error: ""
});

export const useTaskPanel = ({listTasks, cancelTask, cleanupTasks, showNotice, showError, onTaskSettled}: TaskPanelOptions) => {
  const visible = ref(false);
  const summaryVisible = ref(false);
  const loading = ref(false);
  const cleanupLoading = ref(false);
  const tasks = ref<TaskStatus[]>([]);
  const message = ref("");
  const lastUpdatedAt = ref("");
  const cancelConfirm = ref<TaskCancelConfirmState>(emptyCancelConfirm());
  const summaryDismissed = ref(false);
  const recentSummaryTaskId = ref("");
  const trackedActiveTaskIds = new Set<string>();
  const pendingTaskIds = new Set<string>();
  let pollTimer: number | undefined;
  let summaryHideTimer: number | undefined;

  const activeTaskCount = computed(() => tasks.value.filter(task => task.state === "running" || task.state === "queued").length);
  const cleanupTaskCount = computed(() => tasks.value.filter(canCleanupTask).length);
  const hasActiveTasks = computed(() => activeTaskCount.value > 0);
  const summaryTask = computed(() => {
    const activeRecentTask = recentSummaryTaskId.value
        ? tasks.value.find(task => task.id === recentSummaryTaskId.value && canCancelTask(task))
        : null;
    return activeRecentTask
        ?? tasks.value.find(task => task.state === "running")
        ?? tasks.value.find(task => task.state === "queued")
        ?? (recentSummaryTaskId.value ? tasks.value.find(task => task.id === recentSummaryTaskId.value) : null)
        ?? null;
  });
  const summaryFailed = computed(() => summaryTask.value?.state === "failed" || Boolean(summaryTask.value?.errors.length));
  const buttonText = computed(() => hasActiveTasks.value ? `任务 ${activeTaskCount.value}` : "任务");

  const updateRefreshTime = () => {
    lastUpdatedAt.value = new Intl.DateTimeFormat("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit"
    }).format(new Date());
  }

  const stopPolling = () => {
    if (!pollTimer) return;
    window.clearTimeout(pollTimer);
    pollTimer = undefined;
    stopSummaryHideTimer();
  }

  const stopSummaryHideTimer = () => {
    if (!summaryHideTimer) return;
    window.clearTimeout(summaryHideTimer);
    summaryHideTimer = undefined;
  }

  const scheduleSummaryHide = () => {
    stopSummaryHideTimer();
    if (hasActiveTasks.value || pendingTaskIds.size) {
      summaryVisible.value = !summaryDismissed.value;
      return;
    }
    summaryDismissed.value = false;
    if (!summaryVisible.value) return;
    summaryHideTimer = window.setTimeout(() => {
      summaryVisible.value = false;
      recentSummaryTaskId.value = "";
      summaryHideTimer = undefined;
    }, 3600);
  }

  const syncTaskTransitions = (nextTasks: TaskStatus[]) => {
    const nextTaskIds = new Set(nextTasks.map(task => task.id));
    const settledTasks = nextTasks.filter(task => {
      if (canCancelTask(task)) return false;
      return trackedActiveTaskIds.has(task.id) || pendingTaskIds.has(task.id);
    });
    nextTasks.forEach(task => pendingTaskIds.delete(task.id));
    const removedActiveTaskSettled = Array.from(trackedActiveTaskIds).some(id => !pendingTaskIds.has(id) && !nextTaskIds.has(id));
    trackedActiveTaskIds.clear();
    nextTasks.filter(canCancelTask).forEach(task => trackedActiveTaskIds.add(task.id));
    if (settledTasks.length) {
      const failedCount = settledTasks.filter(task => task.state === "failed").length;
      const summarySettledTask = settledTasks.find(task => task.state === "failed") ?? settledTasks[0];
      if (summarySettledTask) recentSummaryTaskId.value = summarySettledTask.id;
      if (failedCount) showNotice(`${failedCount} 个后台任务失败，请打开任务查看详情`, "error", "任务失败", 5000);
    }
    if (settledTasks.length || removedActiveTaskSettled) void onTaskSettled?.(settledTasks);
  }

  const load = async (silent = false) => {
    if (!silent) loading.value = true;
    try {
      const nextTasks = await listTasks();
      syncTaskTransitions(nextTasks);
      tasks.value = nextTasks;
      updateRefreshTime();
      schedulePolling();
      scheduleSummaryHide();
    } catch (error) {
      stopPolling();
      showError(error, "加载任务失败", "任务加载失败");
    } finally {
      if (!silent) loading.value = false;
    }
  }

  const schedulePolling = () => {
    stopPolling();
    if (!hasActiveTasks.value && !pendingTaskIds.size) return;
    pollTimer = window.setTimeout(() => {
      void load(true);
    }, 1500);
  }

  const toggle = async () => {
    visible.value = !visible.value;
    if (visible.value) summaryVisible.value = false;
    if (visible.value) {
      await load();
    } else {
      close();
    }
  }

  const close = () => {
    visible.value = false;
    if (hasActiveTasks.value || pendingTaskIds.size) schedulePolling();
    else stopPolling();
    scheduleSummaryHide();
  }

  const open = async () => {
    visible.value = true;
    summaryVisible.value = false;
    stopSummaryHideTimer();
    await load();
  }

  const dismissSummary = () => {
    summaryDismissed.value = true;
    summaryVisible.value = false;
    recentSummaryTaskId.value = "";
    stopSummaryHideTimer();
  }

  const resetCancelConfirm = () => {
    cancelConfirm.value = emptyCancelConfirm();
  }

  const requestCancel = async (task: TaskStatus) => {
    if (!canCancelTask(task)) return;
    cancelConfirm.value = {
      visible: true,
      task,
      submitting: false,
      error: ""
    };
  }

  const closeCancelConfirm = () => {
    if (cancelConfirm.value.submitting) return;
    resetCancelConfirm();
  }

  const submitCancelConfirm = async () => {
    const task = cancelConfirm.value.task;
    if (!task || !canCancelTask(task) || cancelConfirm.value.submitting) return;
    cancelConfirm.value.submitting = true;
    cancelConfirm.value.error = "";
    try {
      await cancelTask(task.id);
      message.value = `已发送取消请求：${shortTaskId(task.id)}`;
      resetCancelConfirm();
      await load();
    } catch (error) {
      cancelConfirm.value.error = error instanceof Error ? error.message : "取消任务失败";
    } finally {
      if (cancelConfirm.value.visible) cancelConfirm.value.submitting = false;
    }
  }

  const cleanupFinishedTasks = async () => {
    if (cleanupLoading.value || loading.value) return;
    cleanupLoading.value = true;
    try {
      resetCancelConfirm();
      const result = await cleanupTasks();
      message.value = result.removed > 0 ? `已清理 ${result.removed} 条已结束任务` : "没有可清理的已结束任务";
      await load(true);
    } catch (error) {
      showError(error, "清理已结束任务失败", "任务清理失败");
    } finally {
      cleanupLoading.value = false;
    }
  }

  const markStarted = async (id: string, label = "后台任务") => {
    message.value = `${label}已创建：${shortTaskId(id)}`;
    pendingTaskIds.add(id);
    recentSummaryTaskId.value = id;
    summaryDismissed.value = false;
    summaryVisible.value = true;
    await load();
  }

  return {
    visible,
    summaryVisible,
    loading,
    cleanupLoading,
    tasks,
    message,
    lastUpdatedAt,
    cancelConfirm,
    buttonText,
    load,
    toggle,
    open,
    close,
    dismissSummary,
    stopPolling,
    resetCancelConfirm,
    requestCancel,
    closeCancelConfirm,
    submitCancelConfirm,
    cleanupFinishedTasks,
    cleanupTaskCount,
    activeTaskCount,
    hasActiveTasks,
    summaryTask,
    summaryFailed,
    markStarted
  };
}
