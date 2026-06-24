<script setup lang="ts">
import {computed, nextTick, ref, watch} from "vue";
import type {TrashRecord} from "../../class.ts";
import {formatEntryDate, formatEntrySize} from "../../utils/file-entry.ts";
import {parentPath} from "../../utils/file-path.ts";
import FileTypeIcon from "../FileTypeIcon.vue";
import Icon from "../Icon.vue";

type TrashSelectOptions = {
  range?: boolean;
  toggle?: boolean;
}

const props = defineProps<{
  records: TrashRecord[];
  selectedId: string;
  selectedIds: string[];
  selectedRecord: TrashRecord | null;
  loading: boolean;
  actionLoading: boolean;
  message: string;
}>();

const emit = defineEmits<{
  (e: "select", id: string, options?: TrashSelectOptions): void;
  (e: "select-all"): void;
  (e: "refresh"): void;
  (e: "restore"): void;
  (e: "delete"): void;
  (e: "empty"): void;
  (e: "cleanup"): void;
  (e: "close"): void;
}>();

const panelRef = ref<HTMLElement | null>(null);

const selectedIdSet = computed(() => new Set(props.selectedIds));
const selectedCount = computed(() => props.selectedIds.length);
const selectedCountText = computed(() => selectedCount.value ? `已选择 ${selectedCount.value} 项` : "未选择项目");
const restoreText = computed(() => selectedCount.value > 1 ? `恢复 ${selectedCount.value} 项` : "恢复");
const deleteText = computed(() => selectedCount.value > 1 ? `永久删除 ${selectedCount.value} 项` : "永久删除");
const canAct = computed(() => selectedCount.value > 0 && !props.loading && !props.actionLoading);
const canBulkAct = computed(() => props.records.length > 0 && !props.loading && !props.actionLoading);

const recordName = (record: TrashRecord) => {
  const parts = record.originalVirtualPath.split("/").filter(Boolean);
  return parts[parts.length - 1] || record.originalVirtualPath || "未知项目";
}

const recordKind = (record: TrashRecord) => record.kind === "folder" ? "folder" : "file";

const recordLocation = (record: TrashRecord) => parentPath(record.originalVirtualPath);

const recordSize = (record: TrashRecord) => record.kind === "folder" ? "-" : formatEntrySize(record.sizeBytes, "-");

const selectedDetails = computed(() => {
  const record = props.selectedRecord;
  if (!record) return [];
  return [
    {label: "原位置", value: record.originalVirtualPath},
    {label: "删除时间", value: formatEntryDate(record.deletedAt)},
    {label: "大小", value: recordSize(record)},
    {label: "类型", value: record.kind === "folder" ? "文件夹" : "文件"},
    {label: "操作者", value: record.actor || "-"}
  ];
});

const handleRowClick = (event: MouseEvent, record: TrashRecord) => {
  emit("select", record.id, {
    range: event.shiftKey,
    toggle: event.ctrlKey || event.metaKey
  });
}

const handleKeydown = (event: KeyboardEvent) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
    event.preventDefault();
    emit("select-all");
    return;
  }
  if (event.key === "Delete" && canAct.value) {
    event.preventDefault();
    emit("delete");
  }
}

watch(() => props.records.length, async () => {
  await nextTick();
  panelRef.value?.focus({preventScroll: true});
}, {once: true});

defineExpose({
  focus: () => panelRef.value?.focus({preventScroll: true})
});
</script>

<template>
  <section
      ref="panelRef"
      class="trash-panel"
      aria-label="回收站"
      tabindex="-1"
      @keydown="handleKeydown"
      @keydown.esc.prevent.stop="emit('close')">
    <div class="trash-panel-header">
      <div class="trash-title">
        <span class="trash-icon"><icon icon="action.trash" /></span>
        <div>
          <p>回收站</p>
          <small>{{ records.length ? `${records.length} 项 · ${selectedCountText}` : "暂无可恢复项目" }}</small>
        </div>
      </div>
      <div class="trash-actions">
        <button class="trash-icon-button" :disabled="loading" title="刷新回收站" @click="emit('refresh')">
          <icon icon="action.refresh" />
        </button>
        <button class="trash-icon-button" title="关闭回收站" @click="emit('close')">
          <icon icon="action.close" />
        </button>
      </div>
    </div>

    <div class="trash-toolbar">
      <button type="button" class="trash-primary" :disabled="!canAct" @click="emit('restore')">
        <icon icon="action.restore" />
        <span>{{ restoreText }}</span>
      </button>
      <button type="button" class="trash-secondary" :disabled="!canAct" @click="emit('delete')">
        <icon icon="action.delete" />
        <span>{{ deleteText }}</span>
      </button>
      <button type="button" class="trash-secondary" :disabled="!canBulkAct" @click="emit('cleanup')">
        <icon icon="action.clean" />
        <span>按策略清理</span>
      </button>
      <button type="button" class="trash-danger" :disabled="!canBulkAct" @click="emit('empty')">
        <icon icon="action.delete" />
        <span>清空</span>
      </button>
    </div>

    <p v-if="message" class="trash-message">{{ message }}</p>

    <div v-if="loading" class="trash-empty">正在加载回收站...</div>
    <div v-else-if="!records.length" class="trash-empty">回收站为空</div>
    <div v-else class="trash-content">
      <div class="trash-list" role="listbox" aria-label="回收站项目" aria-multiselectable="true">
        <button
            v-for="record in records"
            :key="record.id"
            type="button"
            class="trash-row"
            :class="{active: selectedIdSet.has(record.id), focused: selectedId === record.id}"
            role="option"
            :aria-selected="selectedIdSet.has(record.id)"
            :title="record.originalVirtualPath"
            @click="event => handleRowClick(event, record)">
          <span class="trash-row-check" aria-hidden="true">
            <icon v-if="selectedIdSet.has(record.id)" icon="action.check" />
          </span>
          <file-type-icon :kind="recordKind(record)" />
          <span class="trash-row-main">
            <strong>{{ recordName(record) }}</strong>
            <small>{{ recordLocation(record) }}</small>
          </span>
          <span class="trash-row-meta">
            <span>{{ recordSize(record) }}</span>
            <span>{{ formatEntryDate(record.deletedAt) }}</span>
          </span>
        </button>
      </div>

      <div class="trash-detail">
        <div v-if="selectedRecord" class="trash-detail-card">
          <strong>{{ recordName(selectedRecord) }}</strong>
          <div v-for="item in selectedDetails" :key="item.label" :title="item.value">
            <span>{{ item.label }}</span>
            <small>{{ item.value }}</small>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.trash-panel {
  @apply absolute right-3 top-3 z-20 flex w-[min(56rem,calc(100%-1.5rem))] flex-col gap-3 overflow-hidden rounded-lg border p-3 shadow-2xl outline-none backdrop-blur;
  max-height: min(36rem, calc(100% - 1.5rem));
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 96%, transparent);
  box-shadow: var(--app-menu-shadow);
}

.trash-panel:focus-visible {
  box-shadow: var(--app-menu-shadow), 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.trash-panel-header,
.trash-toolbar {
  @apply flex flex-wrap items-center justify-between gap-2;
}

.trash-title {
  @apply flex min-w-0 items-center gap-3;
}

.trash-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg;
  background: var(--app-control);
  color: var(--app-text-muted);
}

.trash-title p {
  @apply text-sm font-semibold;
  color: var(--app-text);
}

.trash-title small {
  @apply text-xs;
  color: var(--app-text-subtle);
}

.trash-actions {
  @apply flex shrink-0 items-center gap-1;
}

.trash-icon-button {
  @apply inline-flex h-8 w-8 items-center justify-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-50;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.trash-icon-button:hover:not(:disabled) {
  background: var(--app-control-hover);
}

.trash-toolbar {
  @apply justify-start;
}

.trash-primary,
.trash-secondary,
.trash-danger {
  @apply inline-flex h-8 items-center gap-1.5 rounded-md border px-3 text-xs font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.trash-primary {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.trash-primary:hover:not(:disabled) {
  background: var(--app-accent-strong);
}

.trash-secondary {
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.trash-secondary:hover:not(:disabled) {
  background: var(--app-control-hover);
}

.trash-danger {
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.trash-danger:hover:not(:disabled) {
  background: color-mix(in srgb, var(--app-danger) 16%, var(--app-panel-solid));
}

.trash-icon-button:focus-visible,
.trash-primary:focus-visible,
.trash-secondary:focus-visible,
.trash-danger:focus-visible,
.trash-row:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.trash-message {
  @apply rounded-md border px-3 py-2 text-xs;
  border-color: var(--app-success-border);
  background: var(--app-success-soft);
  color: var(--app-success-text);
}

.trash-empty {
  @apply flex h-20 items-center justify-center rounded-md border border-dashed text-sm;
  border-color: var(--app-border-soft);
  color: var(--app-text-subtle);
}

.trash-content {
  @apply grid min-h-0 gap-3;
  grid-template-columns: minmax(0, 1fr) minmax(14rem, 18rem);
}

.trash-list {
  @apply flex min-h-0 flex-col gap-1 overflow-auto pr-1;
}

.trash-row {
  @apply grid min-h-14 w-full grid-cols-[1rem_1.25rem_minmax(0,1fr)_auto] items-center gap-3 rounded-md border px-3 py-2 text-left;
  border-color: transparent;
  color: var(--app-text-muted);
}

.trash-row:hover,
.trash-row.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.trash-row.focused {
  box-shadow: inset 0 0 0 1px var(--app-accent, #2563eb);
}

.trash-row-check {
  @apply flex h-4 w-4 items-center justify-center rounded border text-[0.625rem];
  border-color: var(--app-border);
  background: var(--app-control-solid);
  color: var(--app-accent, #2563eb);
}

.trash-row.active .trash-row-check {
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent-soft, #eff6ff);
}

.trash-row-main {
  @apply flex min-w-0 flex-col gap-0.5;
}

.trash-row-main strong,
.trash-detail-card strong {
  @apply truncate text-sm font-medium;
  color: var(--app-text);
}

.trash-row-main small {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.trash-row-meta {
  @apply flex shrink-0 flex-col items-end gap-0.5 text-xs tabular-nums;
  color: var(--app-text-subtle);
}

.trash-detail {
  @apply min-h-0;
}

.trash-detail-card {
  @apply flex min-h-0 flex-col gap-2 rounded-md border p-3 text-xs;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.trash-detail-card div {
  @apply grid min-w-0 gap-1;
}

.trash-detail-card span {
  color: var(--app-text-subtle);
}

.trash-detail-card small {
  @apply min-w-0 break-all;
  color: var(--app-text-muted);
}

@media (max-width: 760px) {
  .trash-content {
    grid-template-columns: 1fr;
  }

  .trash-row {
    grid-template-columns: 1rem 1.25rem minmax(0, 1fr);
  }

  .trash-row-meta {
    @apply col-start-3 items-start;
  }
}
</style>
