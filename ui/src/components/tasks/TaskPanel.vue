<script setup lang="ts">
import {computed, nextTick, ref, watch} from "vue";
import type {TaskKind, TaskState, TaskStatus} from "../../class.ts";
import type {TaskCancelConfirmState} from "../../composables/useTaskPanel.ts";
import Icon from "../Icon.vue";

const props = defineProps<{
  tasks: TaskStatus[];
  loading: boolean;
  message: string;
  lastUpdatedAt: string;
  cancelConfirm: TaskCancelConfirmState;
}>();

const emit = defineEmits<{
  (e: "refresh"): void;
  (e: "close"): void;
  (e: "cancel", task: TaskStatus): void;
  (e: "close-cancel"): void;
  (e: "confirm-cancel"): void;
}>();

const cancelConfirmRef = ref<HTMLElement | null>(null);

const taskKindText = (kind: TaskKind) => ({
  copy: "复制",
  move: "移动",
  delete: "删除",
  archive: "压缩",
  extract: "解压"
}[kind] ?? kind);

const taskStateText = (state: TaskState) => ({
  queued: "排队中",
  running: "运行中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消"
}[state] ?? state);

const taskStateClass = (state: TaskState) => ({
  queued: "queued",
  running: "running",
  completed: "completed",
  failed: "failed",
  cancelled: "cancelled"
}[state] ?? "queued");

const canCancelTask = (task: TaskStatus) => task.state === "queued" || task.state === "running";
const shortTaskId = (id: string) => id.slice(0, 8);
const taskProgress = (task: TaskStatus) => `${Math.round((task.progress || 0) * 100)}%`;
const taskCurrentPath = (task: TaskStatus) => task.currentPath?.trim();

const formatBytes = (bytes?: number) => {
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

const taskBytesText = (task: TaskStatus) => {
  const processed = formatBytes(task.processedBytes);
  const total = task.totalBytes > 0 ? formatBytes(task.totalBytes) : "未知总量";
  return `${processed} / ${total}`;
}

const taskItemsText = (task: TaskStatus) => {
  const total = task.totalItems > 0 ? task.totalItems : "?";
  return `${task.processedItems} / ${total} 项`;
}

const taskStats = computed(() => {
  const stats = {
    running: 0,
    queued: 0,
    failed: 0,
    completed: 0,
    totalErrors: 0
  };
  props.tasks.forEach(task => {
    if (task.state === "running") stats.running += 1;
    if (task.state === "queued") stats.queued += 1;
    if (task.state === "failed") stats.failed += 1;
    if (task.state === "completed") stats.completed += 1;
    stats.totalErrors += task.errors.length;
  });
  return stats;
});

const taskSummaryText = computed(() => {
  if (!props.tasks.length) return "暂无后台任务";
  const parts: string[] = [];
  if (taskStats.value.running) parts.push(`运行 ${taskStats.value.running}`);
  if (taskStats.value.queued) parts.push(`排队 ${taskStats.value.queued}`);
  if (taskStats.value.failed) parts.push(`失败 ${taskStats.value.failed}`);
  if (taskStats.value.totalErrors) parts.push(`错误 ${taskStats.value.totalErrors}`);
  return parts.length ? parts.join(" · ") : `已完成 ${taskStats.value.completed}/${props.tasks.length}`;
});

const taskRefreshText = computed(() => props.lastUpdatedAt ? `上次刷新：${props.lastUpdatedAt}` : "打开后自动刷新任务状态");

const taskCancelTitle = computed(() => props.cancelConfirm.task ? `取消${taskKindText(props.cancelConfirm.task.kind)}任务？` : "取消任务？");

const taskCancelMessage = computed(() => {
  const task = props.cancelConfirm.task;
  if (!task) return "任务取消请求会发送给后端。";
  return `#${shortTaskId(task.id)} · ${taskStateText(task.state)} · ${taskProgress(task)}`;
});

watch(() => props.cancelConfirm.visible, async visible => {
  if (!visible) return;
  await nextTick();
  cancelConfirmRef.value?.focus();
});
</script>

<template>
  <div class="task-panel">
    <div class="task-panel-header">
      <div class="min-w-0">
        <p class="task-panel-title">后台任务 · {{ taskSummaryText }}</p>
        <p class="task-panel-message">{{ message || taskRefreshText }}</p>
      </div>
      <div class="task-panel-badges">
        <span v-if="taskStats.running" class="task-badge running">运行 {{ taskStats.running }}</span>
        <span v-if="taskStats.queued" class="task-badge queued">排队 {{ taskStats.queued }}</span>
        <span v-if="taskStats.failed" class="task-badge failed">失败 {{ taskStats.failed }}</span>
        <span v-if="taskStats.totalErrors" class="task-badge failed">错误 {{ taskStats.totalErrors }}</span>
      </div>
      <div class="task-panel-actions">
        <button class="task-icon-button" :disabled="loading" title="刷新任务" @click="emit('refresh')">
          <icon icon="icon-refresh" size="normal" />
        </button>
        <button class="task-icon-button" title="关闭任务面板" @click="emit('close')">
          <icon icon="icon-close" size="normal" />
        </button>
      </div>
    </div>

    <div v-if="loading" class="task-empty">正在加载任务...</div>
    <div v-else-if="!tasks.length" class="task-empty">暂无后台任务</div>
    <div v-else class="task-list">
      <div v-for="task in tasks" :key="task.id" class="task-row">
        <div class="task-row-main">
          <span class="task-kind">{{ taskKindText(task.kind) }}</span>
          <span :class="['task-state', taskStateClass(task.state)]">{{ taskStateText(task.state) }}</span>
          <span class="task-id">#{{ shortTaskId(task.id) }}</span>
        </div>
        <div class="task-progress">
          <div class="task-progress-track">
            <span :style="{ width: taskProgress(task) }"></span>
          </div>
          <span class="task-progress-text">{{ taskProgress(task) }}</span>
        </div>
        <div v-if="taskCurrentPath(task)" class="task-current" :title="taskCurrentPath(task)">
          当前：{{ taskCurrentPath(task) }}
        </div>
        <div class="task-meta">
          <span>{{ taskBytesText(task) }}</span>
          <span>{{ formatBytes(task.speedBytesPerSec) }}/s</span>
          <span>{{ taskItemsText(task) }}</span>
          <span v-if="task.errors.length" class="task-errors">错误 {{ task.errors.length }}</span>
        </div>
        <div v-if="task.errors.length" class="task-error-list">
          <div v-for="error in task.errors.slice(0, 2)" :key="`${task.id}-${error.path}-${error.message}`" :title="`${error.path}：${error.message}`">
            {{ error.path }}：{{ error.message }}
          </div>
        </div>
        <button class="task-cancel" :disabled="!canCancelTask(task)" @click="emit('cancel', task)">取消</button>
      </div>
    </div>
    <section
        v-if="cancelConfirm.visible"
        ref="cancelConfirmRef"
        class="task-cancel-confirm"
        tabindex="-1"
        @keydown.esc.prevent="emit('close-cancel')">
      <div class="task-cancel-confirm-main">
        <strong>{{ taskCancelTitle }}</strong>
        <span>{{ taskCancelMessage }}</span>
        <span v-if="cancelConfirm.error" class="task-cancel-error">{{ cancelConfirm.error }}</span>
      </div>
      <div class="task-cancel-actions">
        <button type="button" class="task-cancel-secondary" :disabled="cancelConfirm.submitting" @click="emit('close-cancel')">保留任务</button>
        <button type="button" class="task-cancel-primary" :disabled="cancelConfirm.submitting" @click="emit('confirm-cancel')">
          {{ cancelConfirm.submitting ? "发送中..." : "确认取消" }}
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.task-panel {
  @apply flex max-h-72 shrink-0 flex-col gap-2 overflow-hidden border-b border-slate-200 bg-white px-3 py-2;
}

.task-panel-header {
  @apply flex items-center justify-between gap-3;
}

.task-panel-title {
  @apply text-sm font-semibold text-slate-900;
}

.task-panel-message {
  @apply truncate text-xs text-slate-500;
}

.task-panel-badges {
  @apply ml-auto hidden shrink-0 items-center gap-1 md:flex;
}

.task-badge {
  @apply rounded px-2 py-0.5 text-xs font-medium;
}

.task-badge.running {
  @apply bg-blue-100 text-blue-700;
}

.task-badge.queued {
  @apply bg-slate-200 text-slate-700;
}

.task-badge.failed {
  @apply bg-red-100 text-red-700;
}

.task-panel-actions {
  @apply flex shrink-0 items-center gap-1;
}

.task-icon-button {
  @apply inline-flex h-8 w-8 items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-700 hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-50;
}

.task-empty {
  @apply flex h-16 items-center justify-center rounded border border-dashed border-slate-200 text-sm text-slate-500;
}

.task-list {
  @apply flex flex-col gap-2 overflow-auto pr-1;
}

.task-cancel-confirm {
  @apply flex shrink-0 items-center justify-between gap-3 rounded border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-900 outline-none;
}

.task-cancel-confirm:focus-visible {
  @apply ring-2 ring-inset ring-amber-300;
}

.task-cancel-confirm-main {
  @apply flex min-w-0 flex-col gap-0.5;
}

.task-cancel-confirm-main strong,
.task-cancel-confirm-main span {
  @apply truncate;
}

.task-cancel-confirm-main span {
  @apply text-xs text-amber-700;
}

.task-cancel-error {
  @apply text-red-600;
}

.task-cancel-actions {
  @apply flex shrink-0 items-center gap-2;
}

.task-cancel-secondary,
.task-cancel-primary {
  @apply h-8 rounded border px-3 text-xs font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.task-cancel-secondary {
  @apply border-amber-200 bg-white text-amber-800 hover:bg-amber-100;
}

.task-cancel-primary {
  @apply border-amber-600 bg-amber-600 text-white hover:bg-amber-700;
}

.task-row {
  @apply grid min-h-16 grid-cols-[minmax(9rem,1.1fr)_minmax(10rem,1.2fr)_minmax(14rem,1.5fr)_4rem] items-center gap-x-3 gap-y-2 rounded border border-slate-100 bg-slate-50 px-3 py-2 text-sm;
}

.task-row-main {
  @apply flex min-w-0 items-center gap-2;
}

.task-kind {
  @apply shrink-0 font-medium text-slate-900;
}

.task-id {
  @apply truncate text-xs text-slate-500;
}

.task-state {
  @apply shrink-0 rounded px-2 py-0.5 text-xs;
}

.task-state.queued {
  @apply bg-slate-200 text-slate-700;
}

.task-state.running {
  @apply bg-blue-100 text-blue-700;
}

.task-state.completed {
  @apply bg-emerald-100 text-emerald-700;
}

.task-state.failed {
  @apply bg-red-100 text-red-700;
}

.task-state.cancelled {
  @apply bg-amber-100 text-amber-700;
}

.task-progress {
  @apply flex min-w-0 items-center gap-2;
}

.task-progress-track {
  @apply h-2 min-w-0 grow overflow-hidden rounded bg-slate-200;
}

.task-progress-track span {
  @apply block h-full rounded bg-blue-500;
}

.task-progress-text {
  @apply w-10 shrink-0 text-right text-xs tabular-nums text-slate-600;
}

.task-current {
  @apply truncate text-xs text-slate-500;
}

.task-meta {
  @apply flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1 text-xs text-slate-600;
}

.task-errors {
  @apply text-red-600;
}

.task-error-list {
  @apply col-span-3 flex min-w-0 flex-col gap-1 rounded border border-red-100 bg-red-50 px-2 py-1 text-xs text-red-700;
}

.task-error-list div {
  @apply truncate;
}

.task-cancel {
  @apply h-8 rounded border border-slate-200 bg-white px-2 text-sm text-slate-700 hover:bg-red-50 hover:text-red-600 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:bg-white;
}
</style>
