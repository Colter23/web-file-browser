import type {TaskKind, TaskState, TaskStatus} from "../class.ts";
import {translate} from "../i18n";

export const canCancelTask = (task: TaskStatus) => task.state === "queued" || task.state === "running";

export const canCleanupTask = (task: TaskStatus) => !canCancelTask(task);

export const taskKindText = (kind: TaskKind) => ({
  copy: translate("tasks.kind.copy"),
  move: translate("tasks.kind.move"),
  delete: translate("tasks.kind.delete"),
  archive: translate("tasks.kind.archive"),
  extract: translate("tasks.kind.extract")
}[kind] ?? kind);

export const taskStateText = (state: TaskState) => ({
  queued: translate("tasks.state.queued"),
  running: translate("tasks.state.running"),
  completed: translate("tasks.state.completed"),
  failed: translate("tasks.state.failed"),
  cancelled: translate("tasks.state.cancelled")
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
  const total = task.totalBytes > 0 ? formatTaskBytes(task.totalBytes) : translate("tasks.totalUnknown");
  return `${processed} / ${total}`;
}

export const taskItemsText = (task: TaskStatus) => {
  const total = task.totalItems > 0 ? task.totalItems : "?";
  return translate("tasks.itemsProgress", {processed: task.processedItems, total});
}
