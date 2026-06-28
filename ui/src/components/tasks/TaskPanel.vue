<script setup lang="ts">
import {computed, nextTick, ref, watch} from "vue";
import type {TaskKind, TaskStatus} from "../../class.ts";
import type {TaskCancelConfirmState} from "../../composables/useTaskPanel.ts";
import {useDraggablePanel} from "../../composables/useDraggablePanel.ts";
import {
  canCancelTask,
  formatTaskBytes,
  shortTaskId,
  taskBytesText,
  taskCurrentPath,
  taskItemsText,
  taskKindText,
  taskProgress,
  taskStateClass,
  taskStateText
} from "../../utils/task-status.ts";
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

type TaskOverviewItem = {
  key: "running" | "queued" | "failed" | "completed" | "cancelled";
  label: string;
  value: number;
}

const taskStats = computed(() => {
  const stats = {
    running: 0,
    queued: 0,
    failed: 0,
    completed: 0,
    cancelled: 0,
    totalErrors: 0
  };
  props.tasks.forEach(task => {
    if (task.state === "running") stats.running += 1;
    if (task.state === "queued") stats.queued += 1;
    if (task.state === "failed") stats.failed += 1;
    if (task.state === "completed") stats.completed += 1;
    if (task.state === "cancelled") stats.cancelled += 1;
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
  if (parts.length) return parts.join(" · ");
  if (taskStats.value.cancelled && taskStats.value.completed) return `完成 ${taskStats.value.completed} · 取消 ${taskStats.value.cancelled}`;
  if (taskStats.value.cancelled) return `已取消 ${taskStats.value.cancelled}/${props.tasks.length}`;
  return `已完成 ${taskStats.value.completed}/${props.tasks.length}`;
});

const taskRefreshText = computed(() => props.lastUpdatedAt ? `上次刷新：${props.lastUpdatedAt}` : "打开后自动刷新任务状态");
const taskPanelSubtitle = computed(() => props.message || taskRefreshText.value);
const taskOverviewItems = computed<TaskOverviewItem[]>(() => {
  const items: TaskOverviewItem[] = [];
  if (taskStats.value.running) items.push({key: "running", label: "运行", value: taskStats.value.running});
  if (taskStats.value.queued) items.push({key: "queued", label: "排队", value: taskStats.value.queued});
  if (taskStats.value.failed) items.push({key: "failed", label: "失败", value: taskStats.value.failed});
  if (taskStats.value.completed) items.push({key: "completed", label: "完成", value: taskStats.value.completed});
  if (taskStats.value.cancelled) items.push({key: "cancelled", label: "取消", value: taskStats.value.cancelled});
  return items;
});

const taskKindIcon = (kind: TaskKind) => ({
  copy: "action.copy",
  move: "action.cut",
  delete: "action.delete",
  archive: "action.archive",
  extract: "action.extract"
}[kind] ?? "view.details");
const taskKindClass = (kind: TaskKind) => ({
  copy: "kind-copy",
  move: "kind-move",
  delete: "kind-delete",
  archive: "kind-archive",
  extract: "kind-extract"
}[kind] ?? "kind-unknown");

const taskSortTime = (task: TaskStatus) => Date.parse(task.finishedAt ?? task.startedAt ?? task.createdAt) || 0;
const taskStateWeight = (task: TaskStatus) => ({
  running: 0,
  queued: 1,
  failed: 2,
  cancelled: 3,
  completed: 4
}[task.state] ?? 5);
const orderedTasks = computed(() => {
  return [...props.tasks].sort((left, right) => {
    const stateDelta = taskStateWeight(left) - taskStateWeight(right);
    if (stateDelta) return stateDelta;
    return taskSortTime(right) - taskSortTime(left);
  });
});

const taskCancelTitle = computed(() => props.cancelConfirm.task ? `取消${taskKindText(props.cancelConfirm.task.kind)}任务？` : "取消任务？");

const taskCancelMessage = computed(() => {
  const task = props.cancelConfirm.task;
  if (!task) return "任务取消请求会发送给后端。";
  return `#${shortTaskId(task.id)} · ${taskStateText(task.state)} · ${taskProgress(task)}`;
});
const taskErrorsTitle = (task: TaskStatus) => task.errors.map(error => `${error.path}：${error.message}`).join("\n");

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
  <teleport to="body">
    <section ref="panelRef" class="task-panel" :class="{'is-dragging': dragging}" :style="panelStyle" aria-label="后台任务" tabindex="-1">
      <div class="task-panel-header" title="拖动移动任务面板，双击回到默认位置" @pointerdown="startDrag" @dblclick="resetPosition">
        <div class="task-panel-heading">
          <div class="task-panel-icon">
            <icon icon="view.details" />
          </div>
          <div class="task-panel-copy">
            <p class="task-panel-title">后台任务</p>
            <p class="task-panel-message">{{ taskPanelSubtitle }}</p>
          </div>
        </div>
        <div class="task-panel-actions" @pointerdown.stop @dblclick.stop>
          <button type="button" class="task-icon-button" :disabled="loading" title="刷新任务" @click="emit('refresh')">
            <icon class="icon-motion-spin" :class="{'is-spinning': loading}" icon="action.refresh" size="normal" />
          </button>
          <button
              type="button"
              class="task-icon-button"
              :disabled="loading || cleanupLoading || cleanupTaskCount === 0"
              :title="cleanupTaskCount > 0 ? `清理 ${cleanupTaskCount} 条已结束任务` : '没有可清理的已结束任务'"
              @click="emit('cleanup-finished')">
            <icon class="icon-motion-brush" :class="{'is-brushing': cleanupLoading}" icon="action.clean" size="normal" />
          </button>
          <button type="button" class="task-icon-button" title="关闭任务面板" @click="emit('close')">
            <icon icon="action.close" size="normal" />
          </button>
        </div>
      </div>

      <div v-if="tasks.length" class="task-overview">
        <div class="task-overview-summary">
          <strong>{{ taskSummaryText }}</strong>
          <span>{{ taskRefreshText }}</span>
        </div>
        <div class="task-overview-metrics">
          <span v-for="item in taskOverviewItems" :key="item.key" :class="['task-metric', item.key]">
            <small>{{ item.label }}</small>
            <strong>{{ item.value }}</strong>
          </span>
          <span v-if="taskStats.totalErrors" class="task-metric failed">
            <small>错误</small>
            <strong>{{ taskStats.totalErrors }}</strong>
          </span>
        </div>
      </div>

      <div v-if="loading" class="task-empty">正在加载任务...</div>
      <div v-else-if="!tasks.length" class="task-empty">暂无后台任务</div>
      <div v-else class="task-list" role="list" aria-label="后台任务列表">
        <article v-for="task in orderedTasks" :key="task.id" :class="['task-row', taskStateClass(task.state), taskKindClass(task.kind)]" role="listitem">
          <div class="task-row-accent" aria-hidden="true"></div>
          <span class="task-kind-icon">
            <icon :icon="taskKindIcon(task.kind)" />
          </span>
          <div class="task-row-content">
            <div class="task-row-main">
              <div class="task-row-name">
                <span class="task-kind">{{ taskKindText(task.kind) }}</span>
                <span :class="['task-state', taskStateClass(task.state)]">{{ taskStateText(task.state) }}</span>
                <span class="task-id">#{{ shortTaskId(task.id) }}</span>
                <span v-if="taskCurrentPath(task)" class="task-current" :title="taskCurrentPath(task)">
                  当前：{{ taskCurrentPath(task) }}
                </span>
              </div>
              <div class="task-meta">
                <span>{{ taskBytesText(task) }}</span>
                <span>{{ formatTaskBytes(task.speedBytesPerSec) }}/s</span>
                <span>{{ taskItemsText(task) }}</span>
                <span v-if="task.errors.length" class="task-errors" :title="taskErrorsTitle(task)">错误 {{ task.errors.length }}</span>
              </div>
            </div>
            <div class="task-progress">
              <div class="task-progress-track">
                <span :style="{ width: taskProgress(task) }"></span>
              </div>
              <span class="task-progress-text">{{ taskProgress(task) }}</span>
            </div>
          </div>
          <button v-if="canCancelTask(task)" type="button" class="task-cancel" @click="emit('cancel', task)">
            <icon icon="action.close" size="small" />
            <span>取消</span>
          </button>
        </article>
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
  </teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.task-panel {
  @apply fixed z-40 flex flex-col gap-2.5 overflow-hidden rounded-lg border p-3 shadow-2xl outline-none backdrop-blur;
  left: 0;
  top: 0;
  width: min(40rem, calc(100vw - 1.5rem));
  max-height: min(34rem, calc(100vh - 1.5rem));
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
  @apply flex cursor-move select-none items-center justify-between gap-3 rounded-md;
}

.task-panel-heading {
  @apply flex min-w-0 items-center gap-2.5;
}

.task-panel-icon {
  @apply grid size-9 shrink-0 place-items-center rounded-md;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.task-panel-copy {
  @apply flex min-w-0 flex-col;
}

.task-panel-title {
  @apply text-sm font-semibold leading-5;
  color: var(--app-text);
}

.task-panel-message {
  @apply truncate text-xs leading-5;
  color: var(--app-text-subtle);
}

.task-panel-actions {
  @apply flex shrink-0 items-center gap-1;
}

.task-icon-button {
  @apply inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-md border disabled:cursor-not-allowed disabled:opacity-50;
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

.icon-motion-brush.is-brushing {
  animation: task-brush 0.7s ease-in-out infinite;
}

.task-overview {
  @apply grid gap-2 rounded-md border px-3 py-2;
  grid-template-columns: minmax(0, 1fr) auto;
  border-color: color-mix(in srgb, var(--app-border-soft) 78%, transparent);
  background: color-mix(in srgb, var(--app-panel-muted) 76%, transparent);
}

.task-overview-summary {
  @apply flex min-w-0 flex-col;
}

.task-overview-summary strong {
  @apply truncate text-sm font-semibold;
  color: var(--app-text);
}

.task-overview-summary span {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.task-overview-metrics {
  @apply flex shrink-0 items-center gap-1;
}

.task-metric {
  @apply flex min-w-12 flex-col items-center rounded border px-2 py-1;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
}

.task-metric small {
  @apply text-[0.68rem] leading-3;
  color: var(--app-text-subtle);
}

.task-metric strong {
  @apply text-sm font-semibold leading-4 tabular-nums;
  color: var(--app-text-muted);
}

.task-metric.running {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
}

.task-metric.running strong {
  color: var(--app-accent, #2563eb);
}

.task-metric.failed {
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
}

.task-metric.failed strong {
  color: var(--app-danger);
}

.task-metric.completed strong {
  color: var(--app-success-text);
}

.task-metric.cancelled strong {
  color: var(--app-warning-text);
}

.task-empty {
  @apply flex h-20 items-center justify-center rounded-md border border-dashed text-sm;
  border-color: var(--app-border-soft);
  color: var(--app-text-subtle);
}

.task-list {
  @apply flex min-h-0 flex-col gap-2 overflow-y-auto overflow-x-hidden py-0.5 pr-1;
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
  @apply relative grid min-h-0 shrink-0 items-center gap-x-2 rounded-md border py-2 pr-3 pl-4 text-sm;
  grid-template-columns: 2rem minmax(0, 1fr) auto;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
  --task-kind-color: var(--app-accent, #2563eb);
  --task-kind-soft: var(--app-accent-soft, #eff6ff);
  --task-kind-border: var(--app-accent-border, #bfdbfe);
}

.task-row.kind-copy {
  --task-kind-color: #2563eb;
  --task-kind-soft: color-mix(in srgb, #2563eb 12%, var(--app-panel-muted));
  --task-kind-border: color-mix(in srgb, #60a5fa 58%, var(--app-border-soft));
}

.task-row.kind-move {
  --task-kind-color: #7c3aed;
  --task-kind-soft: color-mix(in srgb, #7c3aed 12%, var(--app-panel-muted));
  --task-kind-border: color-mix(in srgb, #a78bfa 58%, var(--app-border-soft));
}

.task-row.kind-delete {
  --task-kind-color: var(--app-danger);
  --task-kind-soft: color-mix(in srgb, var(--app-danger-soft) 70%, var(--app-panel-muted));
  --task-kind-border: color-mix(in srgb, var(--app-danger-border) 68%, var(--app-border-soft));
}

.task-row.kind-archive {
  --task-kind-color: #d97706;
  --task-kind-soft: color-mix(in srgb, #d97706 13%, var(--app-panel-muted));
  --task-kind-border: color-mix(in srgb, #f59e0b 58%, var(--app-border-soft));
}

.task-row.kind-extract {
  --task-kind-color: #059669;
  --task-kind-soft: color-mix(in srgb, #059669 12%, var(--app-panel-muted));
  --task-kind-border: color-mix(in srgb, #34d399 58%, var(--app-border-soft));
}

.task-row.failed {
  border-color: color-mix(in srgb, var(--app-danger-border) 78%, var(--app-border-soft));
  background: color-mix(in srgb, var(--app-danger-soft) 34%, var(--app-panel-muted));
}

.task-row-accent {
  @apply absolute inset-y-0 left-0 w-1 rounded-l-md;
  background: var(--task-kind-color);
}

.task-kind-icon {
  @apply grid size-8 place-items-center rounded-md border;
  border-color: var(--task-kind-border);
  background: var(--task-kind-soft);
  color: var(--task-kind-color);
}

.task-row-content {
  @apply grid min-w-0 gap-1;
}

.task-row-main {
  @apply grid min-w-0 items-center gap-2;
  grid-template-columns: minmax(0, 1fr) auto;
}

.task-row-name {
  @apply flex min-w-0 items-center gap-1.5;
}

.task-kind {
  @apply shrink-0 rounded px-1.5 py-0.5 text-xs font-medium;
  background: color-mix(in srgb, var(--task-kind-soft) 72%, transparent);
  color: var(--task-kind-color);
}

.task-id {
  @apply shrink-0 text-xs leading-5;
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
  @apply flex min-w-0 items-center gap-2;
}

.task-progress-track {
  @apply h-2 min-w-0 grow overflow-hidden rounded-full p-px;
  background: color-mix(in srgb, var(--task-kind-border) 42%, var(--app-border-soft));
}

.task-progress-track span {
  @apply block h-full rounded-full;
  background: linear-gradient(90deg, var(--task-kind-color), color-mix(in srgb, var(--task-kind-color) 78%, var(--app-control-solid)));
  transition: width 0.18s ease;
}

.task-row.failed .task-progress-track span {
  background: var(--app-danger);
}

.task-row.completed .task-progress-track span {
  background: var(--app-success);
}

.task-progress-text {
  @apply w-10 shrink-0 text-right text-xs tabular-nums;
  color: var(--app-text-muted);
}

.task-current {
  @apply min-w-0 truncate text-xs;
  color: var(--app-text-subtle);
}

.task-meta {
  @apply flex min-w-0 shrink-0 items-center gap-x-2 text-xs whitespace-nowrap;
  color: var(--app-text-muted);
}

.task-errors {
  color: var(--app-danger);
}

.task-cancel {
  @apply inline-flex shrink-0 items-center justify-center gap-1 self-stretch rounded border px-2 text-xs font-medium;
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

@keyframes task-brush {
  0%,
  100% {
    transform: rotate(0deg);
  }
  50% {
    transform: rotate(-12deg);
  }
}

@media (max-width: 640px) {
  .task-overview {
    grid-template-columns: minmax(0, 1fr);
  }

  .task-overview-metrics {
    @apply flex-wrap;
  }

  .task-row {
    @apply items-start;
    grid-template-columns: 2rem minmax(0, 1fr);
  }

  .task-cancel {
    @apply col-span-2 h-7 self-auto justify-self-start;
  }

  .task-row-main {
    grid-template-columns: minmax(0, 1fr);
  }

  .task-meta {
    @apply flex-wrap;
  }
}

@media (prefers-reduced-motion: reduce) {
  .icon-motion-brush.is-brushing {
    animation: none;
  }

  .task-progress-track span {
    transition: none;
  }
}
</style>
