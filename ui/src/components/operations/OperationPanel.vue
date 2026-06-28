<script setup lang="ts">
import {computed, nextTick, ref, watch} from "vue";
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
      return "action.new-file";
    case "createFolder":
      return "action.new-folder";
    case "archive":
      return "action.archive";
    case "extract":
      return "action.extract";
    default:
      return "file.text";
  }
});

const focusNameInput = async () => {
  await nextTick();
  const selectInput = () => {
    const input = nameInputRef.value;
    if (!input || props.state.submitting) return;
    input.focus({preventScroll: true});
    input.select();
  }
  if (typeof window.requestAnimationFrame === "function") {
    window.requestAnimationFrame(selectInput);
    return;
  }
  selectInput();
}

watch(() => props.state.visible, visible => {
  if (visible) void focusNameInput();
}, {flush: "post"});

defineExpose({
  focus: () => {
    void focusNameInput();
  }
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
  @apply flex flex-col gap-1.5 text-xs font-medium;
  color: var(--app-text-subtle);
}

.operation-field input {
  @apply h-9 rounded-md border px-3 text-sm font-normal outline-none disabled:cursor-not-allowed;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text);
}

.operation-field input:disabled {
  background: var(--app-control);
  color: var(--app-text-disabled);
}

.operation-field input:focus {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.operation-segmented {
  @apply inline-flex w-fit overflow-hidden rounded-md border;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.operation-segmented button {
  @apply h-8 border-r px-3 text-xs font-semibold last:border-r-0;
  border-color: var(--app-border-soft);
  color: var(--app-text-muted);
}

.operation-segmented button:hover {
  background: var(--app-control-hover);
}

.operation-segmented button:focus-visible {
  @apply relative z-10 outline-none;
  box-shadow: inset 0 0 0 1px var(--app-accent, #2563eb), 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.operation-segmented button.active {
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.operation-segmented button.active:hover {
  background: var(--app-accent, #2563eb);
}

.operation-hint {
  @apply rounded-md px-3 py-2 text-xs;
  border: 1px solid var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.operation-secondary,
.operation-primary {
  @apply h-9 rounded-md px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.operation-secondary {
  @apply border;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.operation-secondary:hover:not(:disabled) {
  background: var(--app-control-hover);
}

.operation-primary {
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.operation-primary:hover:not(:disabled) {
  background: var(--app-accent-strong);
}

.operation-secondary:focus-visible,
.operation-primary:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}
</style>
