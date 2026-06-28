<script setup lang="ts">
import {computed, nextTick, ref, watch} from "vue";
import type {TrashRecord} from "../../class.ts";
import {useDraggablePanel} from "../../composables/useDraggablePanel.ts";
import {useI18n} from "../../i18n";
import {formatEntryDate, formatEntrySize} from "../../utils/file-entry.ts";
import {parentPath} from "../../utils/file-path.ts";
import FileTypeIcon from "../FileTypeIcon.vue";
import Icon from "../Icon.vue";

type TrashSelectOptions = {
  range?: boolean;
  toggle?: boolean;
}

type TrashMoveDirection = "next" | "previous" | "first" | "last";
type TrashFocusMoveMode = "replaceSelection" | "extendSelection" | "moveFocusOnly";

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
  (e: "move-selection", direction: TrashMoveDirection, mode?: TrashFocusMoveMode): void;
  (e: "select-all"): void;
  (e: "toggle-focused"): void;
  (e: "refresh"): void;
  (e: "restore"): void;
  (e: "delete"): void;
  (e: "empty"): void;
  (e: "cleanup"): void;
  (e: "close"): void;
}>();

const {t} = useI18n();
const panelRef = ref<HTMLElement | null>(null);
const {
  dragging,
  panelStyle,
  resetPosition,
  startDrag
} = useDraggablePanel({panelRef});

const selectedIdSet = computed(() => new Set(props.selectedIds));
const selectedCount = computed(() => props.selectedIds.length);
const totalSizeText = computed(() => {
  const totalSize = props.records.reduce((sum, record) => sum + (record.sizeBytes ?? 0), 0);
  return props.records.some(record => typeof record.sizeBytes === "number")
      ? formatEntrySize(totalSize, "-")
      : "";
});
const selectedCountText = computed(() => selectedCount.value ? t("trash.selected", {count: selectedCount.value}) : t("trash.noSelection"));
const summaryText = computed(() => {
  if (!props.records.length) return t("trash.emptyRecoverable");
  const parts = [t("trash.summaryCount", {count: props.records.length}), selectedCountText.value];
  if (totalSizeText.value) parts.push(totalSizeText.value);
  return parts.join(" · ");
});
const restoreText = computed(() => selectedCount.value > 1 ? t("trash.restoreCount", {count: selectedCount.value}) : t("trash.restore"));
const deleteText = computed(() => selectedCount.value > 1 ? t("trash.permanentDeleteCount", {count: selectedCount.value}) : t("trash.permanentDelete"));
const canAct = computed(() => selectedCount.value > 0 && !props.loading && !props.actionLoading);
const canBulkAct = computed(() => props.records.length > 0 && !props.loading && !props.actionLoading);

const recordName = (record: TrashRecord) => {
  const parts = record.originalVirtualPath.split("/").filter(Boolean);
  return parts[parts.length - 1] || record.originalVirtualPath || t("trash.unknownItem");
}

const recordKind = (record: TrashRecord) => record.kind === "folder" ? "folder" : "file";

const recordLocation = (record: TrashRecord) => parentPath(record.originalVirtualPath);

const recordSize = (record: TrashRecord) => record.kind === "folder" ? "-" : formatEntrySize(record.sizeBytes, "-");

const selectedDetails = computed(() => {
  const record = props.selectedRecord;
  if (!record) return [];
  return [
    {label: t("trash.deletedAt"), value: formatEntryDate(record.deletedAt)},
    {label: t("trash.size"), value: recordSize(record)},
    {label: t("trash.type"), value: record.kind === "folder" ? t("common.folder") : t("common.file")},
    {label: t("trash.actor"), value: record.actor || "-"}
  ];
});

const handleRowClick = (event: MouseEvent, record: TrashRecord) => {
  emit("select", record.id, {
    range: event.shiftKey,
    toggle: event.ctrlKey || event.metaKey
  });
}

const handleCheckClick = (record: TrashRecord) => {
  emit("select", record.id, {toggle: true});
  panelRef.value?.focus({preventScroll: true});
}

const navigationKeyMap: Record<string, TrashMoveDirection | undefined> = {
  ArrowDown: "next",
  ArrowUp: "previous",
  Home: "first",
  End: "last"
};

const isSpaceKey = (event: KeyboardEvent) => event.key === " " || event.code === "Space";

const isListKeyboardTarget = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  return target === panelRef.value || Boolean(target.closest(".trash-list, .trash-row"));
}

const handleKeydown = (event: KeyboardEvent) => {
  if (!isListKeyboardTarget(event.target)) return;
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
    event.preventDefault();
    emit("select-all");
    return;
  }
  if ((event.ctrlKey || event.metaKey) && !event.altKey && !event.shiftKey && isSpaceKey(event)) {
    event.preventDefault();
    emit("toggle-focused");
    return;
  }
  const moveDirection = navigationKeyMap[event.key];
  if (moveDirection) {
    event.preventDefault();
    const mode: TrashFocusMoveMode = event.shiftKey
        ? "extendSelection"
        : event.ctrlKey || event.metaKey
          ? "moveFocusOnly"
          : "replaceSelection";
    emit("move-selection", moveDirection, mode);
    return;
  }
  if (event.key === "Enter" && canAct.value) {
    event.preventDefault();
    emit("restore");
    return;
  }
  if (event.key === "Delete" && canAct.value) {
    event.preventDefault();
    emit("delete");
  }
}

const scrollFocusedRecordIntoView = async (id: string) => {
  if (!id) return;
  await nextTick();
  panelRef.value
      ?.querySelector<HTMLElement>(`[data-trash-id="${CSS.escape(id)}"]`)
      ?.scrollIntoView({block: "nearest"});
}

watch(() => props.records.length, async () => {
  await nextTick();
  panelRef.value?.focus({preventScroll: true});
}, {once: true});

watch(() => props.selectedId, id => {
  void scrollFocusedRecordIntoView(id);
});

defineExpose({
  focus: () => panelRef.value?.focus({preventScroll: true})
});
</script>

<template>
  <teleport to="body">
    <section
        ref="panelRef"
        class="trash-panel"
        :class="{'is-dragging': dragging}"
        :style="panelStyle"
        :aria-label="t('trash.title')"
        tabindex="-1"
        @keydown="handleKeydown"
        @keydown.esc.prevent.stop="emit('close')">
    <div class="trash-panel-header" :title="t('trash.dragTitle')" @pointerdown="startDrag" @dblclick="resetPosition">
      <div class="trash-title">
        <span class="trash-icon"><icon icon="action.trash" /></span>
        <div>
          <p>{{ t("trash.title") }}</p>
          <small>{{ summaryText }}</small>
        </div>
      </div>
      <div class="trash-actions">
        <button class="trash-icon-button" :disabled="loading" :title="t('trash.refresh')" @click="emit('refresh')">
          <icon class="icon-motion-spin" :class="{'is-spinning': loading}" icon="action.refresh" />
        </button>
        <button class="trash-icon-button" :title="t('trash.close')" @click="emit('close')">
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
        <span>{{ t("trash.cleanup") }}</span>
      </button>
      <button type="button" class="trash-danger" :disabled="!canBulkAct" @click="emit('empty')">
        <icon icon="action.delete" />
        <span>{{ t("trash.empty") }}</span>
      </button>
    </div>

    <p v-if="message" class="trash-message">{{ message }}</p>

    <div v-if="loading" class="trash-empty">
      <icon class="icon-motion-spin is-spinning" icon="action.refresh" />
      <span>{{ t("trash.loading") }}</span>
    </div>
    <div v-else-if="!records.length" class="trash-empty">
      <icon icon="action.trash" />
      <span>{{ t("trash.emptyState") }}</span>
    </div>
    <div v-else class="trash-content">
      <div class="trash-list" role="listbox" :aria-label="t('trash.list')" aria-multiselectable="true">
        <div
            v-for="record in records"
            :key="record.id"
            class="trash-row"
            :class="{active: selectedIdSet.has(record.id), focused: selectedId === record.id}"
            role="option"
            tabindex="-1"
            :aria-selected="selectedIdSet.has(record.id)"
            :data-trash-id="record.id"
            :title="record.originalVirtualPath"
            @click="event => handleRowClick(event, record)">
          <button
              type="button"
              class="trash-row-check"
              tabindex="-1"
              :aria-label="selectedIdSet.has(record.id) ? t('trash.unselect', {name: recordName(record)}) : t('trash.select', {name: recordName(record)})"
              :aria-pressed="selectedIdSet.has(record.id)"
              @click.stop="handleCheckClick(record)">
            <icon v-if="selectedIdSet.has(record.id)" icon="action.check" />
          </button>
          <file-type-icon class="trash-row-file-icon" :kind="recordKind(record)" :name="recordName(record)" size="1.35rem" />
          <span class="trash-row-main">
            <strong>{{ recordName(record) }}</strong>
            <small>{{ recordLocation(record) }}</small>
          </span>
          <span class="trash-row-meta">
            <span>{{ recordSize(record) }}</span>
            <span>{{ formatEntryDate(record.deletedAt) }}</span>
          </span>
        </div>
      </div>

      <div class="trash-detail">
        <div v-if="selectedRecord" class="trash-detail-card">
          <div class="trash-detail-head" :title="selectedRecord.originalVirtualPath">
            <file-type-icon :kind="recordKind(selectedRecord)" :name="recordName(selectedRecord)" size="1.4rem" />
            <span>
              <strong>{{ recordName(selectedRecord) }}</strong>
              <small>{{ recordLocation(selectedRecord) }}</small>
            </span>
          </div>
          <div class="trash-detail-fields">
            <div v-for="item in selectedDetails" :key="item.label" class="trash-detail-field" :title="item.value">
              <span>{{ item.label }}</span>
              <small>{{ item.value }}</small>
            </div>
          </div>
        </div>
      </div>
    </div>
    </section>
  </teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.trash-panel {
  @apply fixed z-40 flex flex-col gap-3 overflow-hidden rounded-lg border p-3 shadow-2xl outline-none backdrop-blur;
  left: 0;
  top: 0;
  width: min(56rem, calc(100vw - 1.5rem));
  max-height: min(36rem, calc(100vh - 1.5rem));
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 96%, transparent);
  box-shadow: var(--app-menu-shadow);
}

.trash-panel.is-dragging {
  @apply select-none;
}

.trash-panel:focus-visible {
  box-shadow: var(--app-menu-shadow), 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.trash-panel-header,
.trash-toolbar {
  @apply flex flex-wrap items-center justify-between gap-2;
}

.trash-panel-header {
  @apply cursor-move select-none;
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
  @apply inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-50;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.trash-icon-button:hover:not(:disabled) {
  background: var(--app-control-hover);
}

.trash-toolbar {
  @apply justify-start;
  border-radius: 0.5rem;
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
  @apply flex h-28 flex-col items-center justify-center gap-2 rounded-md border border-dashed text-sm;
  border-color: var(--app-border-soft);
  color: var(--app-text-subtle);
}

.trash-empty :deep(.app-icon) {
  @apply text-2xl;
}

.trash-content {
  @apply grid min-h-0 flex-1 gap-3 overflow-hidden;
  grid-template-columns: minmax(0, 1fr) minmax(14rem, 18rem);
}

.trash-list {
  @apply flex min-h-0 flex-col gap-1 overflow-auto pr-1;
}

.trash-row {
  @apply grid min-h-14 w-full cursor-pointer grid-cols-[1.125rem_1.25rem_minmax(0,1fr)_auto] items-center gap-3 rounded-md border px-3 py-2 text-left transition-colors duration-150;
  border-color: transparent;
  color: var(--app-text-muted);
}

.trash-row:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
}

.trash-row.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, var(--app-panel-solid));
  color: var(--app-accent, #2563eb);
}

.trash-row.focused {
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.trash-row-check {
  @apply grid size-[1.125rem] cursor-pointer place-items-center rounded border text-[0.625rem] shadow-sm transition-colors duration-150;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-accent-contrast);
}

.trash-row-check:hover {
  border-color: var(--app-accent, #2563eb);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 10%, var(--app-panel-solid));
}

.trash-row.active .trash-row-check {
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--app-accent, #2563eb) 14%, transparent);
}

.trash-row-check:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.trash-row.active .trash-row-main small,
.trash-row.active .trash-row-meta {
  color: color-mix(in srgb, var(--app-accent, #2563eb) 78%, var(--app-text-subtle));
}

.trash-row-main {
  @apply flex min-w-0 flex-col gap-0.5;
}

.trash-row-file-icon {
  justify-self: center;
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
  @apply flex min-h-0 flex-col gap-3 rounded-md border p-3 text-[0.8125rem];
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.trash-detail-head {
  @apply grid min-w-0 grid-cols-[1.4rem_minmax(0,1fr)] items-center gap-2 rounded-md px-1 py-0.5;
}

.trash-detail-head > span {
  @apply flex min-w-0 flex-col gap-0.5;
}

.trash-detail-card strong {
  @apply truncate text-sm font-semibold;
  color: var(--app-text);
}

.trash-detail-head small {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.trash-detail-fields {
  @apply grid gap-1.5;
}

.trash-detail-field {
  @apply grid min-w-0 grid-cols-[4rem_minmax(0,1fr)] gap-2 rounded-md px-2 py-1.5;
}

.trash-detail-field:nth-child(odd) {
  background: color-mix(in srgb, var(--app-control) 44%, transparent);
}

.trash-detail-field span {
  @apply text-xs leading-5;
  color: var(--app-text-subtle);
}

.trash-detail-field small {
  @apply min-w-0 break-all text-[0.8125rem] leading-5;
  color: var(--app-text);
}

@media (max-width: 760px) {
  .trash-content {
    grid-template-columns: 1fr;
  }

  .trash-row {
    grid-template-columns: 1.125rem 1.25rem minmax(0, 1fr);
  }

  .trash-row-meta {
    @apply col-start-3 items-start;
  }
}
</style>
