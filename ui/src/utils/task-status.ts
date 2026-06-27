import type {TaskKind, TaskState, TaskStatus} from "../class.ts";

export const canCancelTask = (task: TaskStatus) => task.state === "queued" || task.state === "running";

export const canCleanupTask = (task: TaskStatus) => !canCancelTask(task);

export const taskKindText = (kind: TaskKind) => ({
  copy: "复制",
  move: "移动",
  delete: "删除",
  archive: "压缩",
  extract: "解压"
}[kind] ?? kind);

export const taskStateText = (state: TaskState) => ({
  queued: "排队中",
  running: "运行中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消"
}[state] ?? state);

export const taskStateClass = (state: TaskState) => ({
  queued: "queued",
  running: "running",
  completed: "completed",
  failed: "failed",
  cancelled: "cancelled"
}[state] ?? "queued");

export const shortTaskId = (id: string) => id.slice(0, 8);

export const taskProgress = (task: TaskStatus) => `${Math.round((task.progress || 0) * 100)}%`;

export const taskCurrentPath = (task: TaskStatus) => task.currentPath?.trim();

export const formatTaskBytes = (bytes?: number) => {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

export const taskBytesText = (task: TaskStatus) => {
  const processed = formatTaskBytes(task.processedBytes);
  const total = task.totalBytes > 0 ? formatTaskBytes(task.totalBytes) : "未知总量";
  return `${processed} / ${total}`;
}

export const taskItemsText = (task: TaskStatus) => {
  const total = task.totalItems > 0 ? task.totalItems : "?";
  return `${task.processedItems} / ${total} 项`;
}
