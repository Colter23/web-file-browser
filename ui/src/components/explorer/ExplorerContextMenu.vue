<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "./types.ts";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import {useViewportMenuPosition} from "../../composables/useViewportMenuPosition.ts";
import type {MessageKey} from "../../i18n";
import {useI18n} from "../../i18n";
import Icon from "../Icon.vue";

const props = defineProps<{
  background: boolean;
  x: number;
  y: number;
  canPaste: boolean;
  hasEntries: boolean;
  hasSelection: boolean;
  primaryEntry: ExplorerEntry | null;
  selectionCount: number;
  canViewImage: boolean;
  canPlayAudio: boolean;
  canViewVideo: boolean;
  canViewPdf: boolean;
  canEdit: boolean;
  canExtract: boolean;
  canFavorite: boolean;
  favorite: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "escape"): void;
  (e: "open"): void;
  (e: "open-new-tab"): void;
  (e: "view-image"): void;
  (e: "play-audio"): void;
  (e: "view-video"): void;
  (e: "view-pdf"): void;
  (e: "edit"): void;
  (e: "preview"): void;
  (e: "cut"): void;
  (e: "copy"): void;
  (e: "copy-path"): void;
  (e: "paste"): void;
  (e: "download"): void;
  (e: "archive"): void;
  (e: "extract"): void;
  (e: "rename"): void;
  (e: "delete"): void;
  (e: "properties"): void;
  (e: "add-favorite"): void;
  (e: "remove-favorite"): void;
  (e: "create-file"): void;
  (e: "create-folder"): void;
  (e: "select-all"): void;
  (e: "invert-selection"): void;
  (e: "clear-selection"): void;
}>();

const {t} = useI18n();
const menuRef = ref<HTMLElement | null>(null);
const {menuPosition, placeMenu} = useViewportMenuPosition({menuRef});

const isMultiSelect = computed(() => props.selectionCount > 1);
const canOpenNewTab = computed(() => Boolean(props.primaryEntry?.type === "folder"));
const canPreviewEntry = computed(() => Boolean(props.primaryEntry?.type === "file"));
const canDownloadEntry = computed(() => Boolean(props.primaryEntry?.type === "file"));
const hasFavoriteAction = computed(() => props.canFavorite);
const hasViewerAction = computed(() => props.canViewImage || props.canPlayAudio || props.canViewVideo || props.canViewPdf || props.canEdit || canPreviewEntry.value);
const hasTransferAction = computed(() => props.canPaste || props.selectionCount > 0 || props.canExtract || canDownloadEntry.value);

const contextLabel = (singleKey: MessageKey, multipleKey: MessageKey) => {
  return isMultiSelect.value
      ? t("context.actionWithCount", {action: t(multipleKey), count: props.selectionCount})
      : t(singleKey);
}

const {
  focusFirstMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef,
  onEscape: () => emit("escape")
});

const refreshMenu = async () => {
  await placeMenu({x: props.x, y: props.y});
  await focusFirstMenuButton();
}

useOutsidePointerDown({
  refs: [menuRef],
  onOutsidePointerDown: () => emit("close")
});

onMounted(() => {
  void refreshMenu();
  window.addEventListener("resize", refreshMenu);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", refreshMenu);
});

watch(() => [props.background, props.x, props.y, props.primaryEntry?.path, props.selectionCount] as const, () => {
  void refreshMenu();
}, {flush: "post"});
</script>

<template>
  <Teleport to="body">
    <div
        ref="menuRef"
        class="context-menu"
        role="menu"
        :aria-label="t('context.menu')"
        :style="{left: `${menuPosition.x}px`, top: `${menuPosition.y}px`}"
        @click.stop
        @contextmenu.prevent.stop
        @keydown="handleMenuKeyDown">
      <template v-if="background">
        <button class="context-row" @click="emit('create-file')">
          <span class="context-row-icon"><icon icon="action.new-file" /></span>
          <span class="context-row-label">{{ t("common.createFile") }}</span>
        </button>
        <button class="context-row" @click="emit('create-folder')">
          <span class="context-row-icon"><icon icon="action.new-folder" /></span>
          <span class="context-row-label">{{ t("common.createFolder") }}</span>
        </button>
        <div class="context-separator"></div>
        <button class="context-row" :disabled="!canPaste" @click="emit('paste')">
          <span class="context-row-icon"><icon icon="action.paste" /></span>
          <span class="context-row-label">{{ t("common.paste") }}</span>
          <span class="context-row-shortcut">Ctrl+V</span>
        </button>
        <button class="context-row" @click="emit('copy-path')">
          <span class="context-row-icon"><icon icon="action.copy-path" /></span>
          <span class="context-row-label">{{ t("context.copyCurrentPath") }}</span>
        </button>
        <div class="context-separator"></div>
        <button class="context-row" :disabled="!hasEntries" @click="emit('select-all')">
          <span class="context-row-icon"><icon icon="action.select-all" /></span>
          <span class="context-row-label">{{ t("context.selectAll") }}</span>
          <span class="context-row-shortcut">Ctrl+A</span>
        </button>
        <button class="context-row" :disabled="!hasEntries" @click="emit('invert-selection')">
          <span class="context-row-icon"><icon icon="action.invert-selection" /></span>
          <span class="context-row-label">{{ t("context.invertSelection") }}</span>
        </button>
        <button class="context-row" :disabled="!hasSelection" @click="emit('clear-selection')">
          <span class="context-row-icon"><icon icon="action.clear-selection" /></span>
          <span class="context-row-label">{{ t("context.clearSelection") }}</span>
        </button>
      </template>

      <template v-else>
        <div class="context-quick-actions" :aria-label="t('context.quickActions')">
          <button
              class="context-quick-button"
              :disabled="!selectionCount"
              :title="`${contextLabel('common.cut', 'context.cutSelected')} (Ctrl+X)`"
              @click="emit('cut')">
            <icon icon="action.cut" />
            <span>{{ t("common.cut") }}</span>
          </button>
          <button
              class="context-quick-button"
              :disabled="!selectionCount"
              :title="`${contextLabel('common.copy', 'context.copySelected')} (Ctrl+C)`"
              @click="emit('copy')">
            <icon icon="action.copy" />
            <span>{{ t("common.copy") }}</span>
          </button>
          <button
              class="context-quick-button"
              :disabled="!primaryEntry || isMultiSelect"
              :title="t('common.rename')"
              @click="emit('rename')">
            <icon icon="action.rename" />
            <span>{{ t("common.rename") }}</span>
          </button>
          <button
              class="context-quick-button danger"
              :disabled="!primaryEntry"
              :title="contextLabel('common.delete', 'context.deleteSelected')"
              @click="emit('delete')">
            <icon icon="action.delete" />
            <span>{{ t("common.delete") }}</span>
          </button>
        </div>
        <div class="context-separator"></div>
        <button class="context-row" @click="emit('open')">
          <span class="context-row-icon"><icon icon="action.open" /></span>
          <span class="context-row-label">{{ t("context.open") }}</span>
          <span class="context-row-shortcut">Enter</span>
        </button>
        <button v-if="canOpenNewTab" class="context-row" @click="emit('open-new-tab')">
          <span class="context-row-icon"><icon icon="action.open-new-tab" /></span>
          <span class="context-row-label">{{ t("context.openNewTab") }}</span>
        </button>
        <button v-if="favorite && canFavorite" class="context-row" @click="emit('remove-favorite')">
          <span class="context-row-icon favorite"><icon icon="action.favorite-filled" /></span>
          <span class="context-row-label">{{ t("context.removeFavorite") }}</span>
        </button>
        <button v-else-if="hasFavoriteAction" class="context-row" @click="emit('add-favorite')">
          <span class="context-row-icon favorite"><icon icon="action.favorite" /></span>
          <span class="context-row-label">{{ t("context.addFavorite") }}</span>
        </button>
        <template v-if="hasViewerAction">
          <div class="context-separator"></div>
          <button v-if="canViewImage" class="context-row" @click="emit('view-image')">
            <span class="context-row-icon"><icon icon="view.image" /></span>
            <span class="context-row-label">{{ t("context.viewImage") }}</span>
          </button>
          <button v-if="canPlayAudio" class="context-row" @click="emit('play-audio')">
            <span class="context-row-icon"><icon icon="view.audio" /></span>
            <span class="context-row-label">{{ t("context.playAudio") }}</span>
          </button>
          <button v-if="canViewVideo" class="context-row" @click="emit('view-video')">
            <span class="context-row-icon"><icon icon="view.video" /></span>
            <span class="context-row-label">{{ t("context.playVideo") }}</span>
          </button>
          <button v-if="canViewPdf" class="context-row" @click="emit('view-pdf')">
            <span class="context-row-icon"><icon icon="view.pdf" /></span>
            <span class="context-row-label">{{ t("context.viewPdf") }}</span>
          </button>
          <button v-if="canEdit" class="context-row" @click="emit('edit')">
            <span class="context-row-icon"><icon icon="action.edit" /></span>
            <span class="context-row-label">{{ t("context.edit") }}</span>
          </button>
          <button v-if="canPreviewEntry" class="context-row" @click="emit('preview')">
            <span class="context-row-icon"><icon icon="action.preview" /></span>
            <span class="context-row-label">{{ t("common.preview") }}</span>
            <span class="context-row-shortcut">Space</span>
          </button>
        </template>
        <template v-if="hasTransferAction">
          <div class="context-separator"></div>
          <button v-if="canPaste" class="context-row" @click="emit('paste')">
            <span class="context-row-icon"><icon icon="action.paste" /></span>
            <span class="context-row-label">{{ t("common.paste") }}</span>
            <span class="context-row-shortcut">Ctrl+V</span>
          </button>
          <button v-if="selectionCount" class="context-row" @click="emit('copy-path')">
            <span class="context-row-icon"><icon icon="action.copy-path" /></span>
            <span class="context-row-label">{{ contextLabel("context.copyPath", "context.copySelectedPaths") }}</span>
          </button>
          <button v-if="canDownloadEntry" class="context-row" @click="emit('download')">
            <span class="context-row-icon"><icon icon="action.download" /></span>
            <span class="context-row-label">{{ t("common.download") }}</span>
          </button>
          <button v-if="selectionCount" class="context-row" @click="emit('archive')">
            <span class="context-row-icon"><icon icon="action.archive" /></span>
            <span class="context-row-label">{{ contextLabel("common.archive", "context.archiveSelected") }}</span>
          </button>
          <button v-if="canExtract" class="context-row" @click="emit('extract')">
            <span class="context-row-icon"><icon icon="action.extract" /></span>
            <span class="context-row-label">{{ t("common.extract") }}</span>
          </button>
        </template>
        <div v-if="selectionCount" class="context-separator"></div>
        <button v-if="selectionCount" class="context-row" @click="emit('properties')">
          <span class="context-row-icon"><icon icon="action.properties" /></span>
          <span class="context-row-label">{{ t("context.properties") }}</span>
          <span class="context-row-shortcut">Alt+Enter</span>
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.context-menu {
  @apply fixed z-50 w-[17.5rem] overflow-hidden rounded-md border py-1 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.context-menu button {
  @apply border-0 bg-transparent;
  color: var(--app-text-muted);
}

.context-menu button:disabled {
  @apply cursor-default;
  color: var(--app-text-disabled);
}

.context-menu button:disabled:hover {
  background: var(--app-panel-solid);
}

.context-menu button:hover:not(:disabled),
.context-menu button:focus-visible {
  color: var(--app-text);
}

.context-menu button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.context-menu button:focus-visible {
  @apply outline-none;
  background: var(--app-accent-soft, #eff6ff);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.context-quick-actions {
  @apply grid grid-cols-4 px-1;
}

.context-quick-button {
  @apply relative flex h-[3.25rem] min-w-0 flex-col items-center justify-center gap-1 rounded px-1 text-xs;
}

.context-quick-button:not(:last-child)::after {
  @apply absolute right-0 top-2 h-9 w-px content-[''];
  background: var(--app-divider);
}

.context-quick-button :deep(.icon) {
  @apply text-[1rem];
}

.context-quick-button span {
  @apply max-w-full truncate;
}

.context-row {
  @apply grid h-8 w-full items-center gap-2 px-3 text-left;
  grid-template-columns: 1rem minmax(0, 1fr) auto;
}

.context-row-icon {
  @apply inline-flex items-center justify-center text-[0.95rem];
  color: var(--app-accent, #2563eb);
}

.context-row-icon.favorite {
  color: color-mix(in srgb, var(--app-warning) 88%, var(--app-accent, #2563eb));
}

.context-row-label {
  @apply min-w-0 truncate;
}

.context-row-shortcut {
  @apply pl-4 text-xs;
  color: var(--app-text-subtle);
}

.context-row:disabled .context-row-icon,
.context-row:disabled .context-row-shortcut {
  color: var(--app-text-disabled);
}

.context-separator {
  @apply my-1 border-t;
  border-color: var(--app-border-soft);
}

.context-menu .danger,
.context-menu .danger .context-row-icon {
  color: var(--app-danger);
}

.context-menu .danger:hover:not(:disabled) {
  background: var(--app-danger-soft);
}

.context-menu .danger:disabled {
  color: color-mix(in srgb, var(--app-danger) 38%, var(--app-text-disabled));
}

.context-menu .danger:focus-visible {
  background: var(--app-danger-soft);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-danger) 38%, transparent);
}
</style>
