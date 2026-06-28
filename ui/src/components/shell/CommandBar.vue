<script setup lang="ts">
import {nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {CSSProperties} from "vue";
import type {DirSortKey, DirSortOrder, ExplorerIconSize, ExplorerViewMode} from "../../class";
import type {ExplorerViewModeSelection} from "../../composables/useExplorerViewMode.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import {useI18n} from "../../i18n";
import {scrollHorizontallyWithWheel} from "../../utils/wheel.ts";
import Icon from "../Icon.vue";
import SortMenu from "./SortMenu.vue";
import ViewModeMenu from "./ViewModeMenu.vue";

defineProps<{
  hasSelection: boolean;
  canPasteSelection: boolean;
  canDownloadSelection: boolean;
  canPreviewSelection: boolean;
  canArchiveSelection: boolean;
  canExtractSelection: boolean;
  canRenameSelection: boolean;
  canDeleteSelection: boolean;
  viewModeIcon: string;
  viewModeLabel: string;
  viewModeButtonTitle: string;
  viewMode: ExplorerViewMode;
  iconSize: ExplorerIconSize;
  sortKey: DirSortKey;
  sortOrder: DirSortOrder;
  previewPanelVisible: boolean;
  canTogglePreviewPane: boolean;
}>();

const emit = defineEmits<{
  (e: "upload"): void;
  (e: "create-file"): void;
  (e: "create-folder"): void;
  (e: "cut"): void;
  (e: "copy"): void;
  (e: "paste"): void;
  (e: "download"): void;
  (e: "preview"): void;
  (e: "archive"): void;
  (e: "extract"): void;
  (e: "rename"): void;
  (e: "delete"): void;
  (e: "select-view-mode", selection: ExplorerViewModeSelection): void;
  (e: "set-sort-key", key: DirSortKey): void;
  (e: "set-sort-order", order: DirSortOrder): void;
  (e: "toggle-preview"): void;
}>();

const {t} = useI18n();
const commandBarRef = ref<HTMLElement | null>(null);
const commandActionsRef = ref<HTMLElement | null>(null);
const createButtonRef = ref<HTMLButtonElement | null>(null);
const createMenuRef = ref<HTMLElement | null>(null);
const createMenuOpen = ref(false);
const createMenuStyle = ref<CSSProperties>({});

const closeCreateMenu = () => {
  createMenuOpen.value = false;
}

const updateCreateMenuPosition = () => {
  const bar = commandBarRef.value;
  const button = createButtonRef.value;
  if (!bar || !button) return;
  const barRect = bar.getBoundingClientRect();
  const buttonRect = button.getBoundingClientRect();
  const menuWidth = 224;
  const left = Math.min(Math.max(8, buttonRect.left - barRect.left), Math.max(8, barRect.width - menuWidth - 8));
  createMenuStyle.value = {
    left: `${left}px`,
    top: `${buttonRect.bottom - barRect.top + 6}px`
  };
}

const focusCreateButton = async () => {
  await nextTick();
  createButtonRef.value?.focus({preventScroll: true});
}

const {
  focusFirstMenuButton,
  handleMenuKeyDown: handleCreateMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef: createMenuRef,
  onEscape: () => {
    closeCreateMenu();
    void focusCreateButton();
  }
});

const openCreateMenu = async () => {
  updateCreateMenuPosition();
  createMenuOpen.value = true;
  await focusFirstMenuButton();
}

const toggleCreateMenu = async () => {
  if (createMenuOpen.value) {
    closeCreateMenu();
    return;
  }
  await openCreateMenu();
}

const handleCreateButtonKeyDown = (event: KeyboardEvent) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (!createMenuOpen.value) void openCreateMenu();
  else void focusFirstMenuButton();
}

const emitCreateAction = (type: "file" | "folder") => {
  closeCreateMenu();
  if (type === "file") emit("create-file");
  else emit("create-folder");
}

useOutsidePointerDown({
  refs: [createButtonRef, createMenuRef],
  enabled: () => createMenuOpen.value,
  onOutsidePointerDown: closeCreateMenu
});

onMounted(() => {
  commandActionsRef.value?.addEventListener("wheel", scrollHorizontallyWithWheel, {passive: false});
  commandActionsRef.value?.addEventListener("scroll", updateCreateMenuPosition, {passive: true});
  window.addEventListener("resize", updateCreateMenuPosition);
});

onBeforeUnmount(() => {
  commandActionsRef.value?.removeEventListener("wheel", scrollHorizontallyWithWheel);
  commandActionsRef.value?.removeEventListener("scroll", updateCreateMenuPosition);
  window.removeEventListener("resize", updateCreateMenuPosition);
});
</script>

<template>
  <div ref="commandBarRef" class="command-bar">
    <div ref="commandActionsRef" class="command-actions" :aria-label="t('command.fileOperations')">
      <div class="command-group command-group-primary" role="group" :aria-label="t('command.createAndUpload')">
        <button class="command-button strong" :title="t('command.uploadTitle')" @click="emit('upload')">
          <icon icon="action.upload" />
          <span>{{ t("common.upload") }}</span>
        </button>
        <button
            ref="createButtonRef"
            class="command-button"
            :class="{active: createMenuOpen}"
            :title="t('command.createTitle')"
            aria-haspopup="menu"
            :aria-expanded="createMenuOpen"
            @click="toggleCreateMenu"
            @keydown="handleCreateButtonKeyDown">
          <icon icon="action.new-file" />
          <span>{{ t("common.create") }}</span>
          <icon class="command-button-caret icon-motion-caret" :class="{'is-open': createMenuOpen}" icon="action.down" />
        </button>
      </div>
      <div class="command-group" role="group" :aria-label="t('command.clipboard')">
        <button class="command-button" :disabled="!hasSelection" :title="t('command.cutTitle')" @click="emit('cut')">
          <icon icon="action.cut" />
          <span>{{ t("common.cut") }}</span>
        </button>
        <button class="command-button" :disabled="!hasSelection" :title="t('command.copyTitle')" @click="emit('copy')">
          <icon icon="action.copy" />
          <span>{{ t("common.copy") }}</span>
        </button>
        <button class="command-button" :disabled="!canPasteSelection" :title="t('command.pasteTitle')" @click="emit('paste')">
          <icon icon="action.paste" />
          <span>{{ t("common.paste") }}</span>
        </button>
      </div>
      <div class="command-group" role="group" :aria-label="t('command.selectionActions')">
        <button class="command-button" :disabled="!canDownloadSelection" :title="t('common.download')" @click="emit('download')">
          <icon icon="action.download" />
          <span>{{ t("common.download") }}</span>
        </button>
        <button class="command-button" :disabled="!canPreviewSelection" :title="t('command.previewTitle')" @click="emit('preview')">
          <icon icon="action.preview" />
          <span>{{ t("common.preview") }}</span>
        </button>
        <button class="command-button" :disabled="!canArchiveSelection" :title="t('common.archive')" @click="emit('archive')">
          <icon icon="action.archive" />
          <span>{{ t("common.archive") }}</span>
        </button>
        <button class="command-button" :disabled="!canExtractSelection" :title="t('common.extract')" @click="emit('extract')">
          <icon icon="action.extract" />
          <span>{{ t("common.extract") }}</span>
        </button>
        <button class="command-button" :disabled="!canRenameSelection" :title="t('common.rename')" @click="emit('rename')">
          <icon icon="action.rename" />
          <span>{{ t("common.rename") }}</span>
        </button>
        <button class="command-button danger" :disabled="!canDeleteSelection" :title="t('common.delete')" @click="emit('delete')">
          <icon icon="action.delete" />
          <span>{{ t("common.delete") }}</span>
        </button>
      </div>
    </div>
    <div class="command-view-tools" role="group" :aria-label="t('command.viewAndSort')">
      <sort-menu
          :sort-key="sortKey"
          :sort-order="sortOrder"
          @set-sort-key="key => emit('set-sort-key', key)"
          @set-sort-order="order => emit('set-sort-order', order)" />
      <view-mode-menu
          :icon="viewModeIcon"
          :label="viewModeLabel"
          :title="viewModeButtonTitle"
          :view-mode="viewMode"
          :icon-size="iconSize"
          @select="selection => emit('select-view-mode', selection)" />
      <button
          class="view-button"
          :class="{active: previewPanelVisible}"
          :disabled="!canTogglePreviewPane"
          :title="previewPanelVisible ? t('command.closePreviewPane') : t('command.openPreviewPane')"
          @click="emit('toggle-preview')">
        <icon icon="view.preview-pane" />
        <span>{{ t("command.previewPane") }}</span>
      </button>
    </div>
    <div
        v-if="createMenuOpen"
        ref="createMenuRef"
        class="create-menu-panel"
        :style="createMenuStyle"
        role="menu"
        :aria-label="t('common.create')"
        @keydown="handleCreateMenuKeyDown">
      <button class="create-menu-item" type="button" role="menuitem" tabindex="-1" @click="emitCreateAction('file')">
        <span class="create-menu-icon"><icon icon="action.new-file" /></span>
        <span class="create-menu-copy">
          <strong>{{ t("common.createFile") }}</strong>
          <small>{{ t("command.createFileHint") }}</small>
        </span>
      </button>
      <button class="create-menu-item" type="button" role="menuitem" tabindex="-1" @click="emitCreateAction('folder')">
        <span class="create-menu-icon"><icon icon="action.new-folder" /></span>
        <span class="create-menu-copy">
          <strong>{{ t("common.createFolder") }}</strong>
          <small>{{ t("command.createFolderHint") }}</small>
        </span>
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.command-bar {
  @apply relative z-30 flex h-11 shrink-0 items-center gap-2 overflow-visible border-b px-2;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

.command-actions {
  @apply flex h-full min-w-0 grow items-center gap-1 overflow-x-auto overflow-y-hidden pr-1;
  scrollbar-width: none;
}

.command-actions::-webkit-scrollbar {
  display: none;
}

.command-group {
  @apply flex h-8 shrink-0 items-center gap-0.5 rounded-md border border-transparent px-0.5;
}

.command-group + .command-group {
  @apply relative ml-1 border-l border-transparent pl-2;
  border-radius: 0;
}

.command-group + .command-group::before {
  content: "";
  @apply absolute left-0 top-1/2 h-4 w-px -translate-y-1/2;
  background: color-mix(in srgb, var(--app-divider) 58%, transparent);
}

.command-group-primary {
  background: color-mix(in srgb, var(--app-control-solid) 48%, transparent);
}

.command-button {
  @apply inline-flex h-8 shrink-0 items-center justify-center gap-1.5 rounded-md border border-transparent bg-transparent px-2.5 text-sm shadow-none;
  color: var(--app-text-muted);
}

.command-button:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
}

.command-button:focus-visible,
.view-button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.command-button.active {
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
}

.command-button-caret {
  @apply -mr-0.5 text-[0.65rem];
  color: var(--app-text-subtle);
}

.command-button.strong {
  color: var(--app-accent, #2563eb);
}

.command-button.strong:hover {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
}

.command-button:disabled {
  @apply cursor-not-allowed hover:border-transparent hover:bg-transparent;
  color: var(--app-text-disabled);
}

.command-button.danger {
  color: var(--app-danger);
}

.command-button.danger:hover {
  background: var(--app-danger-soft);
}

.command-button.danger:disabled {
  @apply hover:bg-transparent;
  color: color-mix(in srgb, var(--app-danger) 38%, var(--app-text-disabled));
}

.command-view-tools {
  @apply flex h-8 shrink-0 items-center gap-0 overflow-visible rounded-md border;
  border-color: color-mix(in srgb, var(--app-border-soft) 78%, transparent);
  background: color-mix(in srgb, var(--app-control-solid) 54%, transparent);
}

.command-view-tools :deep(.sort-button),
.command-view-tools :deep(.view-button) {
  @apply h-[1.875rem] rounded-none border-transparent bg-transparent px-2 shadow-none;
}

.command-view-tools :deep(.sort-button),
.command-view-tools :deep(.view-menu .view-button) {
  @apply border-r;
  border-right-color: color-mix(in srgb, var(--app-divider) 62%, transparent);
}

.command-view-tools :deep(.sort-button) {
  @apply rounded-l-md;
}

.command-view-tools > .view-button {
  @apply rounded-r-md;
}

.command-view-tools :deep(.sort-button:hover),
.command-view-tools :deep(.view-button:hover),
.command-view-tools > .view-button:hover:not(:disabled) {
  border-color: transparent;
  background: var(--app-control-hover);
}

.command-view-tools :deep(.sort-button:hover),
.command-view-tools :deep(.view-menu .view-button:hover) {
  border-right-color: color-mix(in srgb, var(--app-divider) 62%, transparent);
}

.command-view-tools :deep(.sort-button:focus-visible),
.command-view-tools :deep(.view-button:focus-visible),
.command-view-tools > .view-button:focus-visible {
  @apply relative z-10;
}

.command-view-tools :deep(.sort-button.active),
.command-view-tools :deep(.view-button.active),
.command-view-tools > .view-button.active {
  border-color: transparent;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, var(--app-panel-solid));
  color: var(--app-accent, #2563eb);
}

.command-view-tools :deep(.sort-button.active),
.command-view-tools :deep(.view-menu .view-button.active) {
  border-right-color: color-mix(in srgb, var(--app-divider) 62%, transparent);
}

.create-menu-panel {
  @apply absolute z-50 w-56 overflow-hidden rounded-md border p-1.5;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.create-menu-item {
  @apply grid w-full grid-cols-[1.75rem_minmax(0,1fr)] items-center gap-2 rounded-md border border-transparent px-1.5 py-1.5 text-left text-sm;
  color: var(--app-text-muted);
}

.create-menu-item:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
}

.create-menu-item:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.create-menu-icon {
  @apply grid size-7 place-items-center rounded;
  background: var(--app-control);
  color: var(--app-text-muted);
}

.create-menu-item:hover .create-menu-icon,
.create-menu-item:focus-visible .create-menu-icon {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 14%, transparent);
  color: var(--app-accent, #2563eb);
}

.create-menu-copy {
  @apply flex min-w-0 flex-col;
}

.create-menu-copy strong {
  @apply truncate text-sm font-medium;
}

.create-menu-copy small {
  @apply text-xs leading-snug;
  color: var(--app-text-subtle);
}

.view-button {
  @apply inline-flex h-8 shrink-0 items-center justify-center gap-1.5 rounded-md border px-2 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.view-button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.view-button.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.view-button:disabled {
  @apply cursor-not-allowed;
  color: var(--app-text-disabled);
}

.view-button:disabled:hover {
  background: var(--app-control-solid);
}

@media (max-width: 1180px) {
  .command-button {
    @apply px-2;
  }

  .command-button span,
  .view-button span {
    @apply sr-only;
  }

  .command-view-tools :deep(.sort-button-label),
  .command-view-tools :deep(.view-button > span) {
    @apply sr-only;
  }
}

</style>
