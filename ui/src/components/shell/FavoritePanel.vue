<script setup lang="ts">
import {computed, nextTick, ref} from "vue";
import type {FavoriteItem} from "../../class.ts";
import {readBooleanStorage, writeBooleanStorage} from "../../utils/safe-storage.ts";
import {normalizePathText} from "../../utils/file-path.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import Icon from "../Icon.vue";
import FileTypeIcon from "../FileTypeIcon.vue";
import type {ShellNoticeKind} from "./types.ts";
import FavoriteContextMenu from "./FavoriteContextMenu.vue";

type FavoriteDropPlacement = "before" | "after";

const collapsedStorageKey = "explorer.favoritePanelCollapsed";

const props = defineProps<{
  favorites: FavoriteItem[];
  loading: boolean;
  currentPath: string;
}>();

const emit = defineEmits<{
  (e: "open", favorite: FavoriteItem): void;
  (e: "open-new-tab", favorite: FavoriteItem): void;
  (e: "rename", payload: {favorite: FavoriteItem; name: string}): void;
  (e: "reorder", payload: {source: FavoriteItem; target: FavoriteItem; placement: FavoriteDropPlacement}): void;
  (e: "remove", favorite: FavoriteItem): void;
  (e: "refresh"): void;
  (e: "notice", payload: {message: string; kind?: ShellNoticeKind; title?: string}): void;
}>();

const collapsed = ref(readBooleanStorage(collapsedStorageKey, false));
const renamingFavoriteId = ref("");
const renameDraft = ref("");
const renameEditRef = ref<HTMLElement | null>(null);
const renameInputRef = ref<HTMLInputElement | null>(null);
const removingFavoriteId = ref("");
const removeConfirmRef = ref<HTMLElement | null>(null);
const suppressedFavoriteActionsId = ref("");
const draggingFavoriteId = ref("");
const dropTargetId = ref("");
const dropPlacement = ref<FavoriteDropPlacement | "">("");
const favoriteButtonRefs = new Map<string, HTMLElement>();
const removeConfirmButtonRefs = new Map<string, HTMLButtonElement>();
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  favoriteId: ""
});

const isActive = (favorite: FavoriteItem) => {
  return normalizePathText(favorite.path) === normalizePathText(props.currentPath || "/");
}

const contextFavorite = computed(() => {
  return props.favorites.find(favorite => favorite.id === contextMenu.value.favoriteId) ?? null;
});

const toggleCollapsed = () => {
  collapsed.value = !collapsed.value;
  writeBooleanStorage(collapsedStorageKey, collapsed.value);
  clearRemoveConfirm();
  closeContextMenu();
}

const setFavoriteButtonRef = (favoriteId: string, element: unknown) => {
  if (element instanceof HTMLElement) {
    favoriteButtonRefs.set(favoriteId, element);
  } else {
    favoriteButtonRefs.delete(favoriteId);
  }
}

const setRenameEditRef = (element: unknown) => {
  renameEditRef.value = element instanceof HTMLElement ? element : null;
}

const setRenameInputRef = (element: unknown) => {
  renameInputRef.value = element instanceof HTMLInputElement ? element : null;
}

const setRemoveConfirmRef = (element: unknown) => {
  removeConfirmRef.value = element instanceof HTMLElement ? element : null;
}

const focusFavoriteButton = async (favoriteId: string) => {
  await nextTick();
  favoriteButtonRefs.get(favoriteId)?.focus({preventScroll: true});
}

const focusRemoveConfirmButton = async (favoriteId: string) => {
  await nextTick();
  removeConfirmButtonRefs.get(favoriteId)?.focus({preventScroll: true});
}

const emitNotice = (message: string, kind: ShellNoticeKind = "info", title = "收藏夹") => {
  emit("notice", {message, kind, title});
}

const isRemoveConfirming = (favorite: FavoriteItem) => removingFavoriteId.value === favorite.id;

const shouldShowFavoriteActions = (favorite: FavoriteItem) => {
  return renamingFavoriteId.value !== favorite.id
      && !isRemoveConfirming(favorite)
      && suppressedFavoriteActionsId.value !== favorite.id;
}

const clearSuppressedFavoriteActions = (favoriteId?: string) => {
  if (!favoriteId || suppressedFavoriteActionsId.value === favoriteId) {
    suppressedFavoriteActionsId.value = "";
  }
}

const clearRemoveConfirm = () => {
  removingFavoriteId.value = "";
}

const setRemoveConfirmButtonRef = (favoriteId: string, element: unknown) => {
  if (element instanceof HTMLButtonElement) {
    removeConfirmButtonRefs.set(favoriteId, element);
  } else {
    removeConfirmButtonRefs.delete(favoriteId);
  }
}

const startRemoveConfirm = async (favorite: FavoriteItem) => {
  clearSuppressedFavoriteActions();
  clearRemoveConfirm();
  removingFavoriteId.value = favorite.id;
  await focusRemoveConfirmButton(favorite.id);
}

const cancelRemoveConfirm = (favoriteId = removingFavoriteId.value) => {
  clearRemoveConfirm();
  suppressedFavoriteActionsId.value = favoriteId;
}

const confirmRemove = (favorite: FavoriteItem) => {
  if (!isRemoveConfirming(favorite)) return;
  clearRemoveConfirm();
  emit("remove", favorite);
}

const handleOpen = (favorite: FavoriteItem) => {
  clearRemoveConfirm();
  if (favorite.missing) {
    emitNotice("该收藏目录已经缺失，请检查或移除收藏项。", "warning");
    return;
  }
  emit("open", favorite);
}

const handleOpenNewTab = (favorite: FavoriteItem) => {
  clearRemoveConfirm();
  if (favorite.missing) {
    emitNotice("该收藏目录已经缺失，请检查或移除收藏项。", "warning");
    return;
  }
  emit("open-new-tab", favorite);
}

const startRename = async (favorite: FavoriteItem) => {
  clearRemoveConfirm();
  closeContextMenu();
  renamingFavoriteId.value = favorite.id;
  renameDraft.value = favorite.name;
  await nextTick();
  renameInputRef.value?.focus({preventScroll: true});
  renameInputRef.value?.select();
}

const cancelRename = () => {
  renamingFavoriteId.value = "";
  renameDraft.value = "";
}

const commitRename = (favorite: FavoriteItem) => {
  if (renamingFavoriteId.value !== favorite.id) return;
  const nextName = renameDraft.value.trim();
  if (!nextName) {
    emitNotice("收藏夹名称不能为空", "warning");
    void nextTick(() => {
      renameInputRef.value?.focus({preventScroll: true});
      renameInputRef.value?.select();
    });
    return;
  }
  if (nextName !== favorite.name) {
    emit("rename", {favorite, name: nextName});
  }
  cancelRename();
}

useOutsidePointerDown({
  refs: [renameEditRef],
  enabled: () => Boolean(renamingFavoriteId.value),
  onOutsidePointerDown: () => {
    const favorite = props.favorites.find(item => item.id === renamingFavoriteId.value);
    if (favorite) {
      commitRename(favorite);
    } else {
      cancelRename();
    }
  }
});

useOutsidePointerDown({
  refs: [removeConfirmRef],
  enabled: () => Boolean(removingFavoriteId.value),
  onOutsidePointerDown: () => cancelRemoveConfirm()
});

const closeContextMenu = () => {
  const favoriteId = contextMenu.value.favoriteId;
  contextMenu.value.visible = false;
  if (favoriteId) void focusFavoriteButton(favoriteId);
}

const openContextMenuAt = (favorite: FavoriteItem, x: number, y: number) => {
  clearRemoveConfirm();
  contextMenu.value = {
    visible: true,
    x,
    y,
    favoriteId: favorite.id
  };
}

const openContextMenu = (favorite: FavoriteItem, event: MouseEvent) => {
  event.preventDefault();
  event.stopPropagation();
  openContextMenuAt(favorite, event.clientX, event.clientY);
}

const openKeyboardContextMenu = (favorite: FavoriteItem) => {
  const button = favoriteButtonRefs.get(favorite.id);
  const rect = button?.getBoundingClientRect();
  openContextMenuAt(
      favorite,
      rect ? rect.left + Math.min(rect.width - 8, 28) : window.innerWidth / 2,
      rect ? rect.bottom - 4 : window.innerHeight / 2
  );
}

const handleFavoriteKeyDown = (event: KeyboardEvent, favorite: FavoriteItem) => {
  if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
    event.preventDefault();
    openKeyboardContextMenu(favorite);
    return;
  }
  if (event.key === "F2") {
    event.preventDefault();
    void startRename(favorite);
  }
}

const copyFavoritePath = async (favorite: FavoriteItem) => {
  try {
    await navigator.clipboard.writeText(favorite.path);
    emitNotice("收藏路径已复制", "success");
  } catch {
    emitNotice("浏览器未允许写入剪贴板，请手动复制路径。", "error", "复制路径失败");
  }
}

const contextOpen = () => {
  if (!contextFavorite.value) return;
  clearRemoveConfirm();
  closeContextMenu();
  handleOpen(contextFavorite.value);
}

const contextOpenNewTab = () => {
  if (!contextFavorite.value) return;
  clearRemoveConfirm();
  closeContextMenu();
  handleOpenNewTab(contextFavorite.value);
}

const contextRename = () => {
  if (!contextFavorite.value) return;
  clearRemoveConfirm();
  void startRename(contextFavorite.value);
}

const contextCopyPath = () => {
  if (!contextFavorite.value) return;
  clearRemoveConfirm();
  const favorite = contextFavorite.value;
  closeContextMenu();
  void copyFavoritePath(favorite);
}

const contextRemove = () => {
  if (!contextFavorite.value) return;
  const favorite = contextFavorite.value;
  contextMenu.value.visible = false;
  void startRemoveConfirm(favorite);
}

const clearDropState = () => {
  draggingFavoriteId.value = "";
  dropTargetId.value = "";
  dropPlacement.value = "";
}

const handleDragStart = (event: DragEvent, favorite: FavoriteItem) => {
  if (isRemoveConfirming(favorite)) {
    event.preventDefault();
    return;
  }
  const target = event.target;
  if (target instanceof HTMLElement && target.closest(".favorite-action, .favorite-rename-input")) {
    event.preventDefault();
    return;
  }
  clearRemoveConfirm();
  closeContextMenu();
  draggingFavoriteId.value = favorite.id;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", favorite.path);
  }
}

const handleDragOver = (event: DragEvent, favorite: FavoriteItem) => {
  if (!draggingFavoriteId.value || draggingFavoriteId.value === favorite.id) return;
  event.preventDefault();
  event.stopPropagation();
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  dropTargetId.value = favorite.id;
  dropPlacement.value = event.clientY > rect.top + rect.height / 2 ? "after" : "before";
  if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
}

const handleDrop = (event: DragEvent, favorite: FavoriteItem) => {
  if (!draggingFavoriteId.value || draggingFavoriteId.value === favorite.id || !dropPlacement.value) {
    clearDropState();
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  const source = props.favorites.find(item => item.id === draggingFavoriteId.value);
  if (source) {
    emit("reorder", {source, target: favorite, placement: dropPlacement.value});
  }
  clearDropState();
}
</script>

<template>
  <section class="favorite-panel" :class="{collapsed}" aria-label="收藏夹">
    <div class="favorite-header">
      <button
          class="favorite-collapse"
          type="button"
          :aria-expanded="!collapsed"
          :title="collapsed ? '展开收藏夹' : '折叠收藏夹'"
          @click="toggleCollapsed">
        <span class="favorite-title-spacer" aria-hidden="true"></span>
        <span class="favorite-caret-slot" aria-hidden="true">
          <icon icon="action.down" class="favorite-caret icon-motion-caret" :class="{'is-collapsed': collapsed}" size="0.86rem" />
        </span>
        <span class="favorite-header-icon" aria-hidden="true">
          <icon icon="action.favorite" size="1.05rem" />
        </span>
        <span class="favorite-title">收藏夹</span>
      </button>
      <button class="favorite-refresh" :disabled="loading" title="刷新收藏夹" @click="emit('refresh')">
        <icon class="icon-motion-spin" :class="{'is-spinning': loading}" icon="action.refresh" />
      </button>
    </div>

    <Transition name="sidebar-section">
      <div v-if="!collapsed" class="favorite-body-shell">
        <div class="favorite-body">
          <div v-if="loading && !favorites.length" class="favorite-empty">正在加载...</div>
          <div v-else-if="!favorites.length" class="favorite-empty">暂无收藏</div>
          <div v-else class="favorite-list" role="list" @dragend="clearDropState">
            <div
                v-for="favorite in favorites"
                :key="favorite.id"
                class="favorite-row"
                :class="{
                  active: isActive(favorite),
                  missing: favorite.missing,
                  confirmingRemove: isRemoveConfirming(favorite),
                  dragging: draggingFavoriteId === favorite.id,
                  dropBefore: dropTargetId === favorite.id && dropPlacement === 'before',
                  dropAfter: dropTargetId === favorite.id && dropPlacement === 'after'
                }"
                role="listitem"
                :draggable="renamingFavoriteId !== favorite.id && !isRemoveConfirming(favorite)"
                @dragstart="handleDragStart($event, favorite)"
                @dragover="handleDragOver($event, favorite)"
                @drop="handleDrop($event, favorite)"
                @pointerleave="clearSuppressedFavoriteActions(favorite.id)"
                @contextmenu.prevent.stop="openContextMenu(favorite, $event)">
              <button
                  v-if="renamingFavoriteId !== favorite.id"
                  :ref="element => setFavoriteButtonRef(favorite.id, element)"
                  class="favorite-open"
                  :disabled="isRemoveConfirming(favorite)"
                  :title="favorite.path"
                  @click="handleOpen(favorite)"
                  @auxclick.middle.prevent="handleOpenNewTab(favorite)"
                  @keydown="handleFavoriteKeyDown($event, favorite)">
                <span class="favorite-icon" aria-hidden="true">
                  <icon v-if="favorite.missing" icon="action.warning" />
                  <file-type-icon v-else kind="folder" :name="favorite.name" />
                </span>
                <span class="favorite-copy">
                  <span class="favorite-name">{{ favorite.name }}</span>
                  <small>{{ favorite.missing ? "目录缺失" : favorite.path }}</small>
                </span>
              </button>
              <div v-else :ref="setRenameEditRef" class="favorite-edit">
                <span class="favorite-icon" aria-hidden="true">
                  <icon v-if="favorite.missing" icon="action.warning" />
                  <file-type-icon v-else kind="folder" :name="favorite.name" />
                </span>
                <input
                    :ref="setRenameInputRef"
                    v-model="renameDraft"
                    class="favorite-rename-input"
                    type="text"
                    spellcheck="false"
                    @click.stop
                    @keydown.enter.prevent="commitRename(favorite)"
                    @keydown.esc.prevent.stop="cancelRename"
                    @blur="commitRename(favorite)">
              </div>
              <span v-if="shouldShowFavoriteActions(favorite)" class="favorite-actions">
                <button class="favorite-action" title="在新标签页中打开" @click.stop="handleOpenNewTab(favorite)">
                  <icon icon="action.open-new-tab" />
                </button>
                <button class="favorite-action" title="重命名收藏项" @click.stop="startRename(favorite)">
                  <icon icon="action.rename" />
                </button>
                <button class="favorite-action danger" title="从收藏夹移除" @click.stop="startRemoveConfirm(favorite)">
                  <icon icon="action.close" />
                </button>
              </span>
              <span v-else-if="isRemoveConfirming(favorite)" :ref="setRemoveConfirmRef" class="favorite-remove-confirm" @keydown.esc.prevent.stop="cancelRemoveConfirm(favorite.id)">
                <button
                    :ref="element => setRemoveConfirmButtonRef(favorite.id, element)"
                    class="favorite-action confirm"
                    title="确认移除收藏项"
                    @click.stop="confirmRemove(favorite)">
                  <icon icon="action.trash" />
                </button>
                <button class="favorite-action cancel" title="取消移除" @click.stop="cancelRemoveConfirm(favorite.id)">
                  <icon icon="action.close" />
                </button>
              </span>
            </div>
          </div>
        </div>
      </div>
    </Transition>
    <favorite-context-menu
        v-if="contextMenu.visible && contextFavorite"
        :x="contextMenu.x"
        :y="contextMenu.y"
        :favorite="contextFavorite"
        :loading="loading"
        @close="closeContextMenu"
        @escape="closeContextMenu"
        @open="contextOpen"
        @open-new-tab="contextOpenNewTab"
        @rename="contextRename"
        @refresh="emit('refresh'); closeContextMenu()"
        @copy-path="contextCopyPath"
        @remove="contextRemove" />
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.favorite-panel {
  @apply flex shrink-0 flex-col border-b pb-2;
  border-color: var(--app-border-soft);
}

.favorite-panel.collapsed {
  @apply pb-1;
}

.favorite-header {
  @apply flex h-7 w-full items-center rounded-sm border border-transparent pr-1;
  color: var(--app-text-muted);
}

.favorite-collapse {
  @apply flex h-full min-w-0 flex-1 items-center rounded-sm border border-transparent bg-transparent pr-2 text-[13px] font-medium text-left;
  padding-left: 0.125rem;
  color: inherit;
}

.favorite-title {
  @apply truncate;
}

.favorite-title-spacer {
  @apply block h-full w-0.5 shrink-0;
}

.favorite-caret-slot,
.favorite-header-icon {
  @apply inline-flex h-5 w-5 shrink-0 items-center justify-center;
}

.favorite-caret-slot {
  @apply mr-0.5 rounded-sm;
}

.favorite-header-icon {
  @apply mr-1.5;
  color: var(--app-accent, #2563eb);
}

.favorite-caret {
  color: var(--app-text-subtle);
}

.favorite-refresh {
  @apply inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-sm border border-transparent;
  color: var(--app-text-subtle);
}

.favorite-header:hover,
.favorite-header:focus-within,
.favorite-refresh:hover:not(:disabled),
.favorite-refresh:focus-visible {
  background: var(--app-control-hover);
  color: var(--app-text);
}

.favorite-collapse:focus-visible,
.favorite-refresh:focus-visible,
.favorite-action:focus-visible,
.favorite-open:focus-visible,
.favorite-rename-input:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent-border, #bfdbfe);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.favorite-refresh:disabled {
  @apply cursor-wait;
  color: var(--app-text-disabled);
}

.favorite-body-shell {
  display: grid;
  grid-template-rows: 1fr;
  overflow: hidden;
}

.favorite-body {
  min-height: 0;
}

.favorite-list {
  @apply flex max-h-40 flex-col gap-0.5 overflow-auto pr-1;
}

.favorite-list::-webkit-scrollbar {
  width: 8px;
}

.favorite-list::-webkit-scrollbar-thumb {
  @apply rounded-full;
  background: var(--app-border);
}

.favorite-row {
  @apply relative grid h-8 min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-1.5 rounded-sm border border-transparent px-1.5 text-left;
  color: var(--app-text-muted);
}

.favorite-row.dragging {
  @apply opacity-50;
}

.favorite-row.dropBefore {
  box-shadow: inset 0 2px 0 var(--app-accent, #2563eb);
}

.favorite-row.dropAfter {
  box-shadow: inset 0 -2px 0 var(--app-accent, #2563eb);
}

.favorite-open {
  @apply grid h-full min-w-0 grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-1.5 rounded-sm border border-transparent bg-transparent text-left;
  color: inherit;
}

.favorite-open:disabled {
  @apply cursor-default opacity-100;
  color: inherit;
}

.favorite-edit {
  @apply grid h-full min-w-0 grid-cols-[1.25rem_minmax(0,1fr)] items-center gap-1.5;
}

.favorite-row:hover {
  background: var(--app-accent-hover, #eaf4ff);
}

.favorite-row.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-selected, #dceeff);
  color: color-mix(in srgb, var(--app-accent, #2563eb) 62%, var(--app-text));
}

.favorite-row.confirmingRemove {
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
}

.favorite-row.missing {
  color: var(--app-text-subtle);
}

.favorite-icon {
  @apply inline-flex items-center justify-center;
  color: color-mix(in srgb, var(--app-accent, #2563eb) 78%, var(--app-text-muted));
}

.favorite-row.missing .favorite-icon {
  color: color-mix(in srgb, var(--app-warning) 88%, var(--app-text-muted));
}

.favorite-row.active .favorite-icon {
  color: var(--app-accent, #2563eb);
}

.favorite-copy {
  @apply flex min-w-0 flex-col gap-0.5;
}

.favorite-name {
  @apply truncate text-[13px] leading-none;
}

.favorite-copy small {
  @apply truncate text-[0.66rem] leading-none;
  color: var(--app-text-subtle);
}

.favorite-rename-input {
  @apply h-6 min-w-0 rounded-sm border px-1.5 text-[13px];
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-control-solid);
  color: var(--app-text);
}

.favorite-actions {
  @apply hidden shrink-0 items-center gap-0.5;
}

.favorite-remove-confirm {
  @apply inline-flex shrink-0 items-center gap-0.5;
}

.favorite-row:hover .favorite-actions,
.favorite-row:focus-within .favorite-actions {
  @apply inline-flex;
}

.favorite-action {
  @apply inline-flex h-5 w-5 items-center justify-center rounded-sm border border-transparent;
  color: var(--app-text-subtle);
}

.favorite-action:hover {
  background: var(--app-control-hover);
  color: var(--app-accent, #2563eb);
}

.favorite-action.danger:hover {
  background: var(--app-danger-soft);
  color: var(--app-danger);
}

.favorite-action.confirm {
  border-color: var(--app-danger-border);
  background: color-mix(in srgb, var(--app-danger) 12%, transparent);
  color: var(--app-danger);
}

.favorite-action.confirm:hover,
.favorite-action.confirm:focus-visible {
  background: var(--app-danger);
  color: var(--app-danger-contrast);
}

.favorite-action.cancel:hover {
  color: var(--app-text);
}

.favorite-empty {
  @apply flex h-8 items-center px-2 text-xs;
  color: var(--app-text-disabled);
}

.sidebar-section-enter-active {
  transition:
      grid-template-rows 0.16s cubic-bezier(0.2, 0, 0, 1),
      opacity 0.12s ease,
      transform 0.16s cubic-bezier(0.2, 0, 0, 1);
}

.sidebar-section-leave-active {
  transition:
      grid-template-rows 0.13s cubic-bezier(0.4, 0, 1, 1),
      opacity 0.1s ease,
      transform 0.13s cubic-bezier(0.4, 0, 1, 1);
}

.sidebar-section-enter-from,
.sidebar-section-leave-to {
  grid-template-rows: 0fr;
  opacity: 0;
  transform: translateY(-2px);
}

@media (prefers-reduced-motion: reduce) {
  .sidebar-section-enter-active,
  .sidebar-section-leave-active {
    transition: none;
  }
}
</style>
