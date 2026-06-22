import {computed, ref} from "vue";
import type {TaskStatus} from "../class.ts";

export type TaskCancelConfirmState = {
  visible: boolean;
  task: TaskStatus | null;
  submitting: boolean;
  error: string;
}

type TaskPanelOptions = {
  listTasks: () => Promise<TaskStatus[]>;
  cancelTask: (id: string) => Promise<unknown>;
  showError: (error: unknown, fallback: string, title?: string) => void;
}

const emptyCancelConfirm = (): TaskCancelConfirmState => ({
  visible: false,
  task: null,
  submitting: false,
  error: ""
});

const canCancelTask = (task: TaskStatus) => task.state === "queued" || task.state === "running";

const shortTaskId = (id: string) => id.slice(0, 8);

export const useTaskPanel = ({listTasks, cancelTask, showError}: TaskPanelOptions) => {
  const visible = ref(false);
  const loading = ref(false);
  const tasks = ref<TaskStatus[]>([]);
  const message = ref("");
  const lastUpdatedAt = ref("");
  const cancelConfirm = ref<TaskCancelConfirmState>(emptyCancelConfirm());
  let pollTimer: number | undefined;

  const activeTaskCount = computed(() => tasks.value.filter(task => task.state === "running" || task.state === "queued").length);
  const hasActiveTasks = computed(() => activeTaskCount.value > 0);
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
  }

  const load = async (silent = false) => {
    if (!silent) loading.value = true;
    try {
      tasks.value = await listTasks();
      updateRefreshTime();
      schedulePolling();
    } catch (error) {
      stopPolling();
      showError(error, "加载任务失败", "任务加载失败");
    } finally {
      if (!silent) loading.value = false;
    }
  }

  const schedulePolling = () => {
    stopPolling();
    if (!visible.value || !hasActiveTasks.value) return;
    pollTimer = window.setTimeout(() => {
      void load(true);
    }, 1500);
  }

  const toggle = async () => {
    visible.value = !visible.value;
    if (visible.value) {
      await load();
    } else {
      stopPolling();
    }
  }

  const close = () => {
    visible.value = false;
    stopPolling();
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

  const markStarted = async (id: string, label = "后台任务") => {
    message.value = `${label}已创建：${shortTaskId(id)}`;
    visible.value = true;
    await load();
  }

  return {
    visible,
    loading,
    tasks,
    message,
    lastUpdatedAt,
    cancelConfirm,
    buttonText,
    load,
    toggle,
    close,
    stopPolling,
    resetCancelConfirm,
    requestCancel,
    closeCancelConfirm,
    submitCancelConfirm,
    markStarted
  };
}
