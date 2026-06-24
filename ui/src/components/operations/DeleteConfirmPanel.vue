<script setup lang="ts">
import {computed, ref} from "vue";
import FileTypeIcon from "../FileTypeIcon.vue";
import OperationPanelShell from "./OperationPanelShell.vue";
import type {DeleteConfirmState} from "./types.ts";

type OperationPanelShellExpose = {
  focus: () => void;
}

const props = defineProps<{
  state: DeleteConfirmState;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "submit"): void;
  (e: "update:permanent", value: boolean): void;
}>();

const panelRef = ref<OperationPanelShellExpose | null>(null);

const title = computed(() => {
  const count = props.state.entries.length;
  const action = props.state.permanent ? "永久删除" : "删除";
  return count > 1 ? `${action} ${count} 项？` : `${action} ${props.state.entries[0]?.name ?? "所选项目"}？`;
});

const message = computed(() => {
  const count = props.state.entries.length;
  if (props.state.permanent) return count > 1
      ? "这些项目会被直接删除，无法从回收站恢复。"
      : "该项目会被直接删除，无法从回收站恢复。";
  return count > 1 ? "这些项目会被移动到回收站，之后可从回收站恢复。" : "该项目会被移动到回收站，之后可从回收站恢复。";
});

const visibleItems = computed(() => props.state.entries.slice(0, 5));
const extraCount = computed(() => Math.max(0, props.state.entries.length - visibleItems.value.length));
const submitText = computed(() => {
  if (props.state.submitting) return "创建任务中...";
  return props.state.permanent ? "永久删除" : "移动到回收站";
});

defineExpose({
  focus: () => panelRef.value?.focus()
});
</script>

<template>
  <operation-panel-shell
      v-if="state.visible"
      ref="panelRef"
      width="delete"
      variant="red"
      icon="action.delete"
      :title="title"
      :subtitle="message"
      :tabindex="-1"
      @close="emit('close')">
    <div class="delete-confirm-list">
      <div v-for="item in visibleItems" :key="item.path" :title="item.path">
        <file-type-icon :kind="item.type === 'folder' ? 'folder' : 'file'" />
        <span>{{ item.name }}</span>
      </div>
      <div v-if="extraCount" class="delete-confirm-more">
        另有 {{ extraCount }} 项
      </div>
    </div>
    <label class="delete-confirm-permanent" :class="{active: state.permanent}">
      <input
          type="checkbox"
          :checked="state.permanent"
          :disabled="state.submitting"
          @change="event => emit('update:permanent', (event.target as HTMLInputElement).checked)">
      <span>
        <strong>永久删除</strong>
        <small>跳过回收站，删除后无法从应用内恢复。</small>
      </span>
    </label>
    <p v-if="state.error" class="delete-confirm-error">{{ state.error }}</p>
    <template #actions>
      <button type="button" class="operation-secondary" :disabled="state.submitting" @click="emit('close')">取消</button>
      <button
          type="button"
          class="delete-confirm-primary"
          :class="{permanent: state.permanent}"
          :disabled="state.submitting"
          @click="emit('submit')">
        {{ submitText }}
      </button>
    </template>
  </operation-panel-shell>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.delete-confirm-list {
  @apply flex max-h-40 flex-col gap-1 overflow-auto rounded-md border p-2;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.delete-confirm-list div {
  @apply flex min-h-7 min-w-0 items-center gap-2 rounded px-2 text-xs;
  color: var(--app-text-muted);
}

.delete-confirm-list span {
  @apply min-w-0 truncate;
}

.delete-confirm-more {
  color: var(--app-text-disabled);
}

.delete-confirm-permanent {
  @apply flex cursor-pointer items-start gap-3 rounded-md border p-3 text-xs transition;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.delete-confirm-permanent:hover {
  background: var(--app-control-hover);
}

.delete-confirm-permanent.active {
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.delete-confirm-permanent input {
  @apply mt-0.5 h-4 w-4 shrink-0;
  accent-color: var(--app-danger);
}

.delete-confirm-permanent span {
  @apply flex min-w-0 flex-col gap-1;
}

.delete-confirm-permanent strong {
  @apply text-sm font-semibold;
  color: var(--app-text);
}

.delete-confirm-permanent.active strong {
  color: var(--app-danger-text);
}

.delete-confirm-permanent small {
  @apply leading-5;
  color: var(--app-text-subtle);
}

.delete-confirm-permanent.active small {
  color: var(--app-danger-text);
}

.delete-confirm-permanent:focus-within {
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.delete-confirm-error {
  @apply rounded-md border px-3 py-2 text-xs;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger);
}

.operation-secondary,
.delete-confirm-primary {
  @apply h-9 rounded-md px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.operation-secondary {
  @apply border;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.operation-secondary:hover:not(:disabled) {
  background: var(--app-control-hover);
}

.delete-confirm-primary {
  background: var(--app-danger);
  color: var(--app-danger-contrast);
}

.delete-confirm-primary.permanent {
  background: var(--app-danger-strong);
}

.delete-confirm-primary:hover:not(:disabled) {
  background: var(--app-danger-strong);
}

.operation-secondary:focus-visible,
.delete-confirm-primary:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}
</style>
