<script setup lang="ts">
import Icon from "../Icon.vue";
import type {EditorInputRefSetter} from "./types.ts";

defineProps<{
  visible: boolean;
  lineText: string;
  status: string;
  lineCount: number;
  placeholder: string;
  canGotoLine: boolean;
  setGotoInputRef: EditorInputRefSetter;
}>();

const emit = defineEmits<{
  (e: "update:lineText", value: string): void;
  (e: "clear-status"): void;
  (e: "submit"): void;
  (e: "close"): void;
}>();

const updateLineText = (event: Event) => {
  const input = event.target as HTMLInputElement | null;
  emit("update:lineText", input?.value.trim() ?? "");
  emit("clear-status");
}
</script>

<template>
  <div v-if="visible" class="goto-bar" @click.stop @keydown.esc.prevent.stop="emit('close')">
    <div class="goto-fields">
      <span>行</span>
      <input
          :ref="setGotoInputRef"
          :value="lineText"
          class="goto-input"
          type="number"
          min="1"
          :max="Math.max(1, lineCount)"
          :placeholder="placeholder"
          @keydown.enter.prevent.stop="emit('submit')"
          @input="updateLineText">
      <span class="goto-range">/ {{ Math.max(1, lineCount) }}</span>
    </div>
    <div class="goto-actions">
      <span v-if="status" class="goto-status">{{ status }}</span>
      <button class="text-tool" title="跳转到行" :disabled="!canGotoLine" @click="emit('submit')">跳转</button>
      <button title="关闭跳转" @click="emit('close')">
        <icon icon="icon-close" />
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.goto-bar {
  @apply relative z-20 flex shrink-0 items-center justify-between gap-2 rounded-md border border-slate-200 bg-white px-2 py-1.5 text-xs shadow-sm;
}

.goto-fields {
  @apply flex min-w-0 grow items-center gap-2;
}

.goto-input {
  @apply h-8 min-w-0 max-w-28 flex-1 rounded-md border border-slate-200 bg-white px-2 text-right text-sm tabular-nums text-slate-900 outline-none placeholder:text-slate-400 focus:border-blue-500 focus:ring-2 focus:ring-blue-100;
}

.goto-actions {
  @apply flex shrink-0 items-center gap-1 text-slate-600;
}

.goto-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-slate-200 bg-white px-2 text-xs font-medium text-slate-600 hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-white;
}

.goto-actions .text-tool {
  @apply min-w-9;
}

.goto-status {
  @apply max-w-28 truncate px-1 text-amber-600;
}

.goto-range {
  @apply shrink-0 text-slate-400;
}
</style>
