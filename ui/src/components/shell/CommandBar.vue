<script setup lang="ts">
import Icon from "../Icon.vue";

defineProps<{
  hasSelection: boolean;
  canPasteSelection: boolean;
  canDownloadSelection: boolean;
  canPreviewSelection: boolean;
  canArchiveSelection: boolean;
  canExtractSelection: boolean;
  canRenameSelection: boolean;
  canDeleteSelection: boolean;
  selectionStatusText: string;
  taskPanelVisible: boolean;
  taskButtonText: string;
}>();

const emit = defineEmits<{
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
  (e: "toggle-tasks"): void;
}>();
</script>

<template>
  <div class="command-bar">
    <button class="command-button" @click="emit('create-file')">
      <icon icon="icon-file-add-fill" />
      <span>新建文件</span>
    </button>
    <button class="command-button" title="新建文件夹 (Ctrl+Shift+N)" @click="emit('create-folder')">
      <icon icon="icon-folder-add-fill" />
      <span>新建文件夹</span>
    </button>
    <span class="command-separator"></span>
    <button class="command-button" :disabled="!hasSelection" title="剪切 (Ctrl+X)" @click="emit('cut')">
      <icon icon="icon-scissors" />
      <span>剪切</span>
    </button>
    <button class="command-button" :disabled="!hasSelection" title="复制 (Ctrl+C)" @click="emit('copy')">
      <icon icon="icon-copy" />
      <span>复制</span>
    </button>
    <button class="command-button" :disabled="!canPasteSelection" title="粘贴 (Ctrl+V)" @click="emit('paste')">
      <icon icon="icon-paste" />
      <span>粘贴</span>
    </button>
    <span class="command-separator"></span>
    <button class="command-button" :disabled="!canDownloadSelection" @click="emit('download')">
      <icon icon="icon-download" />
      <span>下载</span>
    </button>
    <button class="command-button" :disabled="!canPreviewSelection" title="预览 (Space / Ctrl+Enter)" @click="emit('preview')">
      <icon icon="icon-file-image-fill" />
      <span>预览</span>
    </button>
    <button class="command-button" :disabled="!canArchiveSelection" @click="emit('archive')">
      <icon icon="icon-file-zip-fill" />
      <span>压缩</span>
    </button>
    <button class="command-button" :disabled="!canExtractSelection" @click="emit('extract')">
      <icon icon="icon-file-zip" />
      <span>解压</span>
    </button>
    <button class="command-button" :disabled="!canRenameSelection" @click="emit('rename')">
      <icon icon="icon-rename" />
      <span>重命名</span>
    </button>
    <button class="command-button danger" :disabled="!canDeleteSelection" @click="emit('delete')">
      <icon icon="icon-delete-fill" />
      <span>删除</span>
    </button>
    <span class="command-status" :title="`${selectionStatusText} · Ctrl+A 全选`">{{ selectionStatusText }}</span>
    <button :class="['command-button', {active: taskPanelVisible}]" @click="emit('toggle-tasks')">
      <icon icon="icon-file-common-filling" />
      <span>{{ taskButtonText }}</span>
    </button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.command-bar {
  @apply flex h-11 shrink-0 items-center gap-1 overflow-x-auto border-b border-slate-200 bg-slate-50/70 px-3;
}

.command-button {
  @apply inline-flex h-8 shrink-0 items-center justify-center gap-1.5 rounded-lg border border-transparent bg-transparent px-2.5 text-sm text-slate-700 shadow-none hover:border-slate-200 hover:bg-white;
}

.command-button.active {
  @apply border-slate-200 bg-white;
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
  @apply mx-1 h-5 w-px shrink-0 bg-slate-200;
}

.command-status {
  @apply ml-auto min-w-32 truncate pl-3 text-right text-xs text-slate-500;
}
</style>
