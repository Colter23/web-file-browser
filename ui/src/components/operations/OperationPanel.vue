<script setup lang="ts">
import {computed} from "vue";
import type {ArchiveFormat} from "../../class";
import Icon from "../Icon.vue";

type ExplorerEntry = {
  type: "folder" | "file";
  name: string;
  path: string;
  modified?: string;
  size?: number;
  extension?: string;
}

type OperationPanelKind = "createFile" | "createFolder" | "archive" | "extract";

type OperationPanelState = {
  visible: boolean;
  kind: OperationPanelKind | null;
  title: string;
  message: string;
  primaryText: string;
  name: string;
  format: ArchiveFormat;
  entries: ExplorerEntry[];
  sourceEntry: ExplorerEntry | null;
  submitting: boolean;
}

const props = defineProps<{
  state: OperationPanelState;
}>();

const emit = defineEmits<{
  (e: "update:name", name: string): void;
  (e: "update:format", format: ArchiveFormat): void;
  (e: "close"): void;
  (e: "submit"): void;
}>();

const nameModel = computed({
  get: () => props.state.name,
  set: value => emit("update:name", value)
});

const setFormat = (format: ArchiveFormat) => {
  emit("update:format", format);
}

const nameLabel = computed(() => {
  switch (props.state.kind) {
    case "createFile":
      return "文件名";
    case "createFolder":
      return "文件夹名";
    case "archive":
      return "压缩包名称";
    case "extract":
      return "解压到文件夹";
    default:
      return "名称";
  }
});

const panelIcon = computed(() => {
  switch (props.state.kind) {
    case "createFile":
      return "icon-file-add-fill";
    case "createFolder":
      return "icon-folder-add-fill";
    case "archive":
    case "extract":
      return "icon-file-zip-fill";
    default:
      return "icon-file-common-filling";
  }
});
</script>

<template>
  <form v-if="state.visible" class="operation-panel" @submit.prevent="emit('submit')">
    <div class="operation-panel-header">
      <div class="operation-panel-icon">
        <icon :icon="panelIcon" />
      </div>
      <div class="operation-panel-title">
        <strong>{{ state.title }}</strong>
        <span>{{ state.message }}</span>
      </div>
      <button type="button" class="operation-panel-close" title="关闭" @click="emit('close')">
        <icon icon="icon-close" />
      </button>
    </div>
    <label class="operation-field">
      <span>{{ nameLabel }}</span>
      <input
          v-model="nameModel"
          type="text"
          autocomplete="off"
          :disabled="state.submitting"
          @keydown.esc.prevent="emit('close')">
    </label>
    <div v-if="state.kind === 'archive'" class="operation-field">
      <span>压缩格式</span>
      <div class="operation-segmented">
        <button type="button" :class="{active: state.format === 'zip'}" @click="setFormat('zip')">ZIP</button>
        <button type="button" :class="{active: state.format === 'tarGz'}" @click="setFormat('tarGz')">TAR.GZ</button>
      </div>
    </div>
    <div v-if="state.kind === 'archive'" class="operation-hint">
      {{ state.entries.length }} 项将加入压缩包
    </div>
    <div v-else-if="state.kind === 'extract' && state.sourceEntry" class="operation-hint">
      源文件：{{ state.sourceEntry.name }}
    </div>
    <div class="operation-actions">
      <button type="button" class="operation-secondary" :disabled="state.submitting" @click="emit('close')">取消</button>
      <button type="submit" class="operation-primary" :disabled="state.submitting || !state.name.trim()">
        {{ state.submitting ? "处理中..." : state.primaryText }}
      </button>
    </div>
  </form>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.operation-panel {
  @apply absolute left-1/2 top-6 z-30 flex w-[min(28rem,calc(100%-2rem))] -translate-x-1/2 flex-col gap-3 rounded-lg border border-slate-200 bg-white p-4 text-sm shadow-2xl;
}

.operation-panel-header {
  @apply flex items-start gap-3;
}

.operation-panel-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-blue-50 text-xl text-blue-600;
}

.operation-panel-title {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.operation-panel-title strong {
  @apply truncate text-base font-semibold text-slate-900;
}

.operation-panel-title span {
  @apply truncate text-xs text-slate-500;
}

.operation-panel-close {
  @apply flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-slate-500 hover:bg-slate-100;
}

.operation-field {
  @apply flex flex-col gap-1.5 text-xs font-medium text-slate-500;
}

.operation-field input {
  @apply h-9 rounded-md border border-slate-200 bg-white px-3 text-sm font-normal text-slate-900 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100 disabled:bg-slate-50 disabled:text-slate-400;
}

.operation-segmented {
  @apply inline-flex w-fit overflow-hidden rounded-md border border-slate-200 bg-slate-50;
}

.operation-segmented button {
  @apply h-8 border-r border-slate-200 px-3 text-xs font-semibold text-slate-600 last:border-r-0 hover:bg-white;
}

.operation-segmented button.active {
  @apply bg-blue-600 text-white hover:bg-blue-600;
}

.operation-hint {
  @apply rounded-md border border-blue-100 bg-blue-50 px-3 py-2 text-xs text-blue-700;
}

.operation-actions {
  @apply flex justify-end gap-2 pt-1;
}

.operation-secondary,
.operation-primary {
  @apply h-9 rounded-md px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.operation-secondary {
  @apply border border-slate-200 bg-white text-slate-700 hover:bg-slate-50;
}

.operation-primary {
  @apply bg-blue-600 text-white hover:bg-blue-700;
}
</style>
