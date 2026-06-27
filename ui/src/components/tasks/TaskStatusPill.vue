<script setup lang="ts">
import {computed} from "vue";
import type {TaskStatus} from "../../class.ts";
import {
  formatTaskBytes,
  taskCurrentPath,
  taskKindText,
  taskProgress,
  taskStateClass,
  taskStateText
} from "../../utils/task-status.ts";
import Icon from "../Icon.vue";

const props = defineProps<{
  visible: boolean;
  task: TaskStatus | null;
  activeCount: number;
  failed: boolean;
  panelOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "open"): void;
  (e: "dismiss"): void;
}>();

const statusClass = computed(() => props.failed ? "failed" : props.task ? taskStateClass(props.task.state) : "idle");
const progressText = computed(() => props.task ? taskProgress(props.task) : "0%");
const titleText = computed(() => {
  if (!props.task) return "后台任务";
  if (props.activeCount > 1) return `${props.activeCount} 个后台任务`;
  return `${taskKindText(props.task.kind)}任务`;
});
const detailText = computed(() => {
  if (!props.task) return "点击查看任务详情";
  const currentPath = taskCurrentPath(props.task);
  if (currentPath && props.task.state === "running") return currentPath;
  const speed = props.task.speedBytesPerSec > 0 ? ` · ${formatTaskBytes(props.task.speedBytesPerSec)}/s` : "";
  return `${taskStateText(props.task.state)} · ${progressText.value}${speed}`;
});
</script>

<template>
  <Transition name="task-pill">
    <div
        v-if="visible && task && !panelOpen"
        class="task-status-pill"
        :class="statusClass"
        role="status"
        aria-live="polite">
      <button class="task-pill-main" type="button" title="打开后台任务" @click="emit('open')">
        <span class="task-pill-icon">
          <icon :class="{'icon-motion-spin is-spinning': task.state === 'running'}" :icon="task.state === 'running' ? 'action.refresh' : 'view.details'" />
        </span>
        <span class="task-pill-copy">
          <strong>{{ titleText }}</strong>
          <small>{{ detailText }}</small>
        </span>
        <span class="task-pill-percent">{{ progressText }}</span>
        <span class="task-pill-track" aria-hidden="true">
          <span :style="{width: progressText}"></span>
        </span>
      </button>
      <button class="task-pill-close" type="button" title="隐藏任务提示" @click="emit('dismiss')">
        <icon icon="action.close" />
      </button>
    </div>
  </Transition>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.task-status-pill {
  @apply pointer-events-auto flex min-w-0 items-center overflow-hidden rounded-lg border shadow-xl backdrop-blur-xl;
  width: min(25rem, calc(100vw - 2rem));
  border-color: color-mix(in srgb, var(--app-border) 62%, transparent);
  background: color-mix(in srgb, var(--app-panel-solid) 92%, transparent);
  color: var(--app-text-muted);
  box-shadow: var(--app-menu-shadow);
}

.task-status-pill.failed {
  border-color: var(--app-danger-border);
}

.task-pill-main {
  @apply relative grid min-w-0 grow grid-cols-[1.75rem_minmax(0,1fr)_3rem] items-center gap-2 overflow-hidden px-2.5 py-2 text-left;
}

.task-pill-main:hover {
  background: var(--app-control-hover);
}

.task-pill-main:focus-visible,
.task-pill-close:focus-visible {
  @apply outline-none;
  box-shadow: inset 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.task-pill-icon {
  @apply grid size-7 place-items-center rounded-md;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.task-status-pill.failed .task-pill-icon {
  background: var(--app-danger-soft);
  color: var(--app-danger);
}

.task-pill-copy {
  @apply flex min-w-0 flex-col;
}

.task-pill-copy strong {
  @apply truncate text-sm font-semibold;
  color: var(--app-text);
}

.task-pill-copy small {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.task-pill-percent {
  @apply justify-self-end text-xs font-semibold tabular-nums;
  color: var(--app-accent, #2563eb);
}

.task-status-pill.failed .task-pill-percent {
  color: var(--app-danger);
}

.task-pill-track {
  @apply absolute inset-x-0 bottom-0 h-0.5 overflow-hidden;
  background: color-mix(in srgb, var(--app-border-soft) 74%, transparent);
}

.task-pill-track span {
  @apply block h-full;
  background: var(--app-accent, #2563eb);
  transition: width 0.18s ease;
}

.task-status-pill.failed .task-pill-track span {
  background: var(--app-danger);
}

.task-pill-close {
  @apply mr-1 grid size-7 shrink-0 place-items-center rounded-md;
  color: var(--app-text-subtle);
}

.task-pill-close:hover {
  background: var(--app-control-hover);
  color: var(--app-text);
}

.task-pill-enter-active,
.task-pill-leave-active {
  transition: opacity 0.14s ease, transform 0.16s cubic-bezier(0.2, 0, 0, 1);
}

.task-pill-enter-from,
.task-pill-leave-to {
  opacity: 0;
  transform: translateY(0.45rem) scale(0.98);
}

@media (prefers-reduced-motion: reduce) {
  .task-pill-enter-active,
  .task-pill-leave-active,
  .task-pill-track span {
    transition: none;
  }
}
</style>
