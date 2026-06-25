<script setup lang="ts">
import {computed, nextTick, ref, watch} from "vue";
import type {TaskKind, TaskState, TaskStatus} from "../../class.ts";
import type {TaskCancelConfirmState} from "../../composables/useTaskPanel.ts";
import {useDraggablePanel} from "../../composables/useDraggablePanel.ts";
import Icon from "../Icon.vue";

const props = defineProps<{
  tasks: TaskStatus[];
  loading: boolean;
  cleanupLoading: boolean;
  cleanupTaskCount: number;
  message: string;
  lastUpdatedAt: string;
  cancelConfirm: TaskCancelConfirmState;
}>();

const emit = defineEmits<{
  (e: "refresh"): void;
  (e: "cleanup-finished"): void;
  (e: "close"): void;
  (e: "cancel", task: TaskStatus): void;
  (e: "close-cancel"): void;
  (e: "confirm-cancel"): void;
}>();

const panelRef = ref<HTMLElement | null>(null);
const cancelConfirmRef = ref<HTMLElement | null>(null);
const {
  dragging,
  panelStyle,
  resetPosition,
  startDrag
} = useDraggablePanel({panelRef});

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

defineExpose({
  focus: () => panelRef.value?.focus({preventScroll: true})
});
</script>

<template>
  <section ref="panelRef" class="task-panel" :class="{'is-dragging': dragging}" :style="panelStyle" aria-label="后台任务" tabindex="-1">
    <div class="task-panel-header" title="拖动移动任务面板" @pointerdown="startDrag" @dblclick="resetPosition">
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
          <icon icon="action.refresh" size="normal" />
        </button>
        <button
            class="task-icon-button"
            :disabled="loading || cleanupLoading || cleanupTaskCount === 0"
            title="清理已结束任务"
            @click="emit('cleanup-finished')">
          <icon icon="action.clean" size="normal" />
        </button>
        <button class="task-icon-button" title="关闭任务面板" @click="emit('close')">
          <icon icon="action.close" size="normal" />
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
        @keydown.esc.prevent.stop="emit('close-cancel')">
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
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.task-panel {
  @apply fixed left-1/2 top-1/2 z-40 flex -translate-x-1/2 -translate-y-1/2 flex-col gap-2 overflow-hidden rounded-lg border px-3 py-2 shadow-2xl outline-none backdrop-blur;
  width: min(38rem, calc(100vw - 1.5rem));
  max-height: min(32rem, calc(100vh - 1.5rem));
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 96%, transparent);
  box-shadow: var(--app-menu-shadow);
}

.task-panel.is-dragging {
  @apply select-none;
}

.task-panel:focus-visible {
  box-shadow: var(--app-menu-shadow), 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.task-panel-header {
  @apply flex cursor-move select-none items-center justify-between gap-3;
}

.task-panel-title {
  @apply text-sm font-semibold;
  color: var(--app-text);
}

.task-panel-message {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.task-panel-badges {
  @apply ml-auto hidden shrink-0 items-center gap-1 md:flex;
}

.task-badge {
  @apply rounded px-2 py-0.5 text-xs font-medium;
}

.task-badge.running {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.task-badge.queued {
  background: var(--app-panel-muted);
  color: var(--app-text-muted);
}

.task-badge.failed {
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.task-panel-actions {
  @apply flex shrink-0 items-center gap-1;
}

.task-icon-button {
  @apply inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-50;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.task-icon-button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.task-icon-button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.task-empty {
  @apply flex h-16 items-center justify-center rounded border border-dashed text-sm;
  border-color: var(--app-border-soft);
  color: var(--app-text-subtle);
}

.task-list {
  @apply flex min-h-0 flex-col gap-2 overflow-auto pr-1;
}

.task-cancel-confirm {
  @apply flex shrink-0 items-center justify-between gap-3 rounded border px-3 py-2 text-sm outline-none;
  border-color: var(--app-warning-border);
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.task-cancel-confirm:focus-visible {
  @apply ring-2 ring-inset;
  --tw-ring-color: var(--app-warning-ring);
}

.task-cancel-confirm-main {
  @apply flex min-w-0 flex-col gap-0.5;
}

.task-cancel-confirm-main strong,
.task-cancel-confirm-main span {
  @apply truncate;
}

.task-cancel-confirm-main span {
  @apply text-xs;
  color: var(--app-warning-text);
}

.task-cancel-error {
  color: var(--app-danger);
}

.task-cancel-actions {
  @apply flex shrink-0 items-center gap-2;
}

.task-cancel-secondary,
.task-cancel-primary {
  @apply h-8 rounded border px-3 text-xs font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.task-cancel-secondary {
  border-color: var(--app-warning-border);
  background: var(--app-control-solid);
  color: var(--app-warning-text);
}

.task-cancel-secondary:hover {
  background: var(--app-warning-soft);
}

.task-cancel-primary {
  border-color: var(--app-warning);
  background: var(--app-warning);
  color: var(--app-warning-contrast);
}

.task-cancel-primary:hover:not(:disabled) {
  background: var(--app-warning-strong);
}

.task-cancel-secondary:focus-visible,
.task-cancel-primary:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 3px var(--app-warning-ring);
}

.task-row {
  @apply grid min-h-16 grid-cols-[minmax(0,1fr)_auto] items-start gap-x-3 gap-y-2 rounded border px-3 py-2 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.task-row-main {
  @apply flex min-w-0 items-center gap-2;
}

.task-kind {
  @apply shrink-0 font-medium;
  color: var(--app-text);
}

.task-id {
  @apply min-w-0 truncate text-xs;
  color: var(--app-text-subtle);
}

.task-state {
  @apply shrink-0 rounded px-2 py-0.5 text-xs;
}

.task-state.queued {
  background: var(--app-panel-solid);
  color: var(--app-text-muted);
}

.task-state.running {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.task-state.completed {
  background: var(--app-success-soft);
  color: var(--app-success-text);
}

.task-state.failed {
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.task-state.cancelled {
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.task-progress {
  @apply col-span-2 flex min-w-0 items-center gap-2;
}

.task-progress-track {
  @apply h-2 min-w-0 grow overflow-hidden rounded;
  background: var(--app-border-soft);
}

.task-progress-track span {
  @apply block h-full rounded;
  background: var(--app-accent, #2563eb);
}

.task-progress-text {
  @apply w-10 shrink-0 text-right text-xs tabular-nums;
  color: var(--app-text-muted);
}

.task-current {
  @apply col-span-2 truncate text-xs;
  color: var(--app-text-subtle);
}

.task-meta {
  @apply col-span-2 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1 text-xs;
  color: var(--app-text-muted);
}

.task-errors {
  color: var(--app-danger);
}

.task-error-list {
  @apply col-span-2 flex min-w-0 flex-col gap-1 rounded border px-2 py-1 text-xs;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.task-error-list div {
  @apply truncate;
}

.task-cancel {
  @apply col-start-2 row-start-1 h-8 justify-self-end rounded border px-2 text-sm disabled:cursor-not-allowed;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.task-cancel:hover:not(:disabled) {
  background: var(--app-danger-soft);
  color: var(--app-danger);
}

.task-cancel:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.task-cancel:disabled {
  color: var(--app-text-disabled);
}

.task-cancel:disabled:hover {
  background: var(--app-control-solid);
}
</style>
