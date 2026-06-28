<script setup lang="ts">
import {computed, ref} from "vue";
import {formatEntrySize} from "../../utils/file-entry.ts";
import {parentPath} from "../../utils/file-path.ts";
import FileTypeIcon from "../FileTypeIcon.vue";
import OperationPanelShell from "../operations/OperationPanelShell.vue";
import type {TrashConfirmState} from "./types.ts";

type OperationPanelShellExpose = {
  focus: () => void;
}

const props = defineProps<{
  state: TrashConfirmState;
  totalCount: number;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "submit"): void;
}>();

const panelRef = ref<OperationPanelShellExpose | null>(null);

const isEmptyConfirm = computed(() => props.state.kind === "empty");
const isCleanupConfirm = computed(() => props.state.kind === "cleanup");
const selectedCount = computed(() => props.state.records.length);
const affectedCount = computed(() => isEmptyConfirm.value ? props.totalCount : selectedCount.value);
const visibleRecords = computed(() => props.state.records.slice(0, 5));
const extraCount = computed(() => Math.max(0, props.state.records.length - visibleRecords.value.length));

const title = computed(() => {
  if (isEmptyConfirm.value) return `清空回收站？`;
  if (isCleanupConfirm.value) return "按策略清理回收站？";
  return selectedCount.value > 1 ? `永久删除 ${selectedCount.value} 项？` : `永久删除 ${recordName(props.state.records[0])}？`;
});

const message = computed(() => {
  if (isEmptyConfirm.value) return `回收站中的 ${affectedCount.value} 项会被永久删除，此操作无法撤销。`;
  if (isCleanupConfirm.value) return "将根据保留天数和容量上限永久清理符合策略的项目。";
  return selectedCount.value > 1
      ? "这些项目会从回收站永久删除，之后无法从应用内恢复。"
      : "该项目会从回收站永久删除，之后无法从应用内恢复。";
});

const submitText = computed(() => {
  if (props.state.submitting) return "处理中...";
  if (isCleanupConfirm.value) return "按策略清理";
  return isEmptyConfirm.value ? "清空回收站" : "永久删除";
});
const panelIcon = computed(() => isCleanupConfirm.value ? "action.clean" : "action.delete");

const recordName = (record = props.state.records[0]) => {
  if (!record) return "所选项目";
  const parts = record.originalVirtualPath.split("/").filter(Boolean);
  return parts[parts.length - 1] || record.originalVirtualPath || "未知项目";
}

const recordKind = (record = props.state.records[0]) => record?.kind === "folder" ? "folder" : "file";
const recordLocation = (record = props.state.records[0]) => record ? parentPath(record.originalVirtualPath) : "";
const recordSize = (record = props.state.records[0]) => record?.kind === "folder" ? "-" : formatEntrySize(record?.sizeBytes, "-");

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
      :icon="panelIcon"
      :title="title"
      :subtitle="message"
      :tabindex="-1"
      @close="emit('close')">
    <div v-if="isEmptyConfirm || isCleanupConfirm" class="trash-confirm-warning">
      <strong>{{ isCleanupConfirm ? `当前 ${totalCount} 项` : `${affectedCount} 项` }}</strong>
      <span>{{ isCleanupConfirm ? "符合清理策略的项目会被永久删除。" : "将会被直接清理，清空后无法通过回收站恢复。" }}</span>
    </div>
    <div v-else class="trash-confirm-list">
      <div v-for="record in visibleRecords" :key="record.id" :title="record.originalVirtualPath">
        <file-type-icon :kind="recordKind(record)" :name="recordName(record)" />
        <span class="trash-confirm-name">{{ recordName(record) }}</span>
        <small>{{ recordLocation(record) }}</small>
        <small>{{ recordSize(record) }}</small>
      </div>
      <div v-if="extraCount" class="trash-confirm-more">
        另有 {{ extraCount }} 项
      </div>
    </div>
    <p v-if="state.error" class="trash-confirm-error">{{ state.error }}</p>
    <template #actions>
      <button type="button" class="trash-confirm-secondary" :disabled="state.submitting" @click="emit('close')">取消</button>
      <button type="button" class="trash-confirm-primary" :disabled="state.submitting" @click="emit('submit')">
        {{ submitText }}
      </button>
    </template>
  </operation-panel-shell>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.trash-confirm-warning,
.trash-confirm-list {
  @apply rounded-md border;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
}

.trash-confirm-warning {
  @apply grid gap-1 px-3 py-2 text-sm;
  color: var(--app-danger-text);
}

.trash-confirm-warning strong {
  @apply font-semibold;
}

.trash-confirm-warning span {
  @apply text-xs leading-5;
}

.trash-confirm-list {
  @apply flex max-h-48 flex-col gap-1 overflow-auto p-2;
  background: var(--app-panel-muted);
}

.trash-confirm-list div {
  @apply grid min-h-8 min-w-0 grid-cols-[1.25rem_minmax(7rem,1fr)_minmax(5rem,0.75fr)_auto] items-center gap-2 rounded px-2 text-xs;
  color: var(--app-text-muted);
}

.trash-confirm-list .trash-confirm-more {
  @apply block min-h-0 px-2 py-1;
  color: var(--app-text-disabled);
}

.trash-confirm-name,
.trash-confirm-list small {
  @apply min-w-0 truncate;
}

.trash-confirm-list small {
  color: var(--app-text-subtle);
}

.trash-confirm-error {
  @apply rounded-md border px-3 py-2 text-xs;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger);
}

.trash-confirm-secondary,
.trash-confirm-primary {
  @apply h-9 rounded-md px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.trash-confirm-secondary {
  @apply border;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.trash-confirm-secondary:hover:not(:disabled) {
  background: var(--app-control-hover);
}

.trash-confirm-primary {
  background: var(--app-danger-strong);
  color: var(--app-danger-contrast);
}

.trash-confirm-primary:hover:not(:disabled) {
  background: var(--app-danger);
}

.trash-confirm-secondary:focus-visible,
.trash-confirm-primary:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

@media (max-width: 640px) {
  .trash-confirm-list div {
    grid-template-columns: 1.25rem minmax(0, 1fr);
  }

  .trash-confirm-list small {
    @apply col-start-2;
  }
}
</style>
