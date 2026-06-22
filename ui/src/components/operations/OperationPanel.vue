<script setup lang="ts">
import {computed, ref} from "vue";
import type {ArchiveFormat} from "../../class";
import OperationPanelShell from "./OperationPanelShell.vue";
import type {OperationPanelState} from "./types.ts";

const props = defineProps<{
  state: OperationPanelState;
}>();

const emit = defineEmits<{
  (e: "update:name", name: string): void;
  (e: "update:format", format: ArchiveFormat): void;
  (e: "close"): void;
  (e: "submit"): void;
}>();

const nameInputRef = ref<HTMLInputElement | null>(null);

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

defineExpose({
  focus: () => nameInputRef.value?.focus()
});
</script>

<template>
  <operation-panel-shell
      v-if="state.visible"
      as="form"
      width="operation"
      variant="blue"
      :icon="panelIcon"
      :title="state.title"
      :subtitle="state.message"
      @close="emit('close')"
      @submit="emit('submit')">
    <label class="operation-field">
      <span>{{ nameLabel }}</span>
      <input
          ref="nameInputRef"
          v-model="nameModel"
          type="text"
          autocomplete="off"
          :disabled="state.submitting">
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
    <template #actions>
      <button type="button" class="operation-secondary" :disabled="state.submitting" @click="emit('close')">取消</button>
      <button type="submit" class="operation-primary" :disabled="state.submitting || !state.name.trim()">
        {{ state.submitting ? "处理中..." : state.primaryText }}
      </button>
    </template>
  </operation-panel-shell>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

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
