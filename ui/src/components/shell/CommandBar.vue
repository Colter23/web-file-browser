<script setup lang="ts">
import type {DirSortKey, DirSortOrder, ExplorerIconSize, ExplorerViewMode} from "../../class";
import type {ExplorerViewModeSelection} from "../../composables/useExplorerViewMode.ts";
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
</script>

<template>
  <div class="command-bar">
    <div class="command-actions" aria-label="文件操作">
      <button class="command-button strong" title="上传" @click="emit('upload')">
        <icon icon="action.upload" />
        <span>上传</span>
      </button>
      <button class="command-button" @click="emit('create-file')">
        <icon icon="action.new-file" />
        <span>新建文件</span>
      </button>
      <button class="command-button" title="新建文件夹 (Ctrl+Shift+N)" @click="emit('create-folder')">
        <icon icon="action.new-folder" />
        <span>新建文件夹</span>
      </button>
      <span class="command-separator"></span>
      <button class="command-button" :disabled="!hasSelection" title="剪切 (Ctrl+X)" @click="emit('cut')">
        <icon icon="action.cut" />
        <span>剪切</span>
      </button>
      <button class="command-button" :disabled="!hasSelection" title="复制 (Ctrl+C)" @click="emit('copy')">
        <icon icon="action.copy" />
        <span>复制</span>
      </button>
      <button class="command-button" :disabled="!canPasteSelection" title="粘贴 (Ctrl+V)" @click="emit('paste')">
        <icon icon="action.paste" />
        <span>粘贴</span>
      </button>
      <span class="command-separator"></span>
      <button class="command-button" :disabled="!canDownloadSelection" @click="emit('download')">
        <icon icon="action.download" />
        <span>下载</span>
      </button>
      <button class="command-button" :disabled="!canPreviewSelection" title="预览 (Space / Ctrl+Enter)" @click="emit('preview')">
        <icon icon="action.preview" />
        <span>预览</span>
      </button>
      <button class="command-button" :disabled="!canArchiveSelection" @click="emit('archive')">
        <icon icon="action.archive" />
        <span>压缩</span>
      </button>
      <button class="command-button" :disabled="!canExtractSelection" @click="emit('extract')">
        <icon icon="action.extract" />
        <span>解压</span>
      </button>
      <button class="command-button" :disabled="!canRenameSelection" @click="emit('rename')">
        <icon icon="action.rename" />
        <span>重命名</span>
      </button>
      <button class="command-button danger" :disabled="!canDeleteSelection" @click="emit('delete')">
        <icon icon="action.delete" />
        <span>删除</span>
      </button>
    </div>
    <div class="command-view-tools">
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
          title="预览窗格 (Alt+P)"
          @click="emit('toggle-preview')">
        <icon icon="view.preview-pane" />
        <span>{{ previewPanelVisible ? "关闭预览" : "预览窗格" }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.command-bar {
  @apply relative z-30 flex h-11 shrink-0 items-center gap-2 overflow-visible border-b border-[#e3ebf5] bg-[#fbfdff] px-2.5;
}

.command-actions {
  @apply flex h-full min-w-0 grow items-center gap-0.5 overflow-x-auto overflow-y-hidden;
  scrollbar-width: none;
}

.command-actions::-webkit-scrollbar {
  display: none;
}

.command-button {
  @apply inline-flex h-8 shrink-0 items-center justify-center gap-1.5 rounded-md border border-transparent bg-transparent px-2.5 text-sm text-slate-700 shadow-none hover:border-slate-200 hover:bg-white;
}

.command-button.active {
  @apply border-slate-200 bg-white;
}

.command-button.strong {
  @apply text-blue-700 hover:border-blue-200 hover:bg-blue-50;
}

.command-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:border-transparent hover:bg-transparent;
}

.command-button.danger {
  @apply text-red-600 hover:bg-red-50;
}

.command-button.danger:disabled {
  @apply text-red-200 hover:bg-transparent;
}

.command-separator {
  @apply mx-1 h-5 w-px shrink-0 bg-[#dbe6f1];
}

.command-view-tools {
  @apply flex shrink-0 items-center gap-1.5 border-l border-[#dbe6f1] pl-2.5;
}

.command-view-tools :deep(.sort-button),
.command-view-tools :deep(.view-button) {
  @apply h-8 px-2.5;
}

.view-button {
  @apply inline-flex h-8 shrink-0 items-center justify-center gap-1.5 rounded-md border border-slate-200 bg-white px-2.5 text-sm text-slate-700 hover:bg-blue-50;
}

.view-button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.view-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:bg-white;
}

</style>
