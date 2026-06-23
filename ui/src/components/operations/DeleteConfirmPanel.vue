<script setup lang="ts">
import {computed, ref} from "vue";
import FileTypeIcon from "../FileTypeIcon.vue";
import OperationPanelShell from "./OperationPanelShell.vue";
import type {DeleteConfirmState} from "./types.ts";

type OperationPanelShellExpose = {
  focus: () => void;
}

const props = defineProps<{
  state: DeleteConfirmState;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "submit"): void;
}>();

const panelRef = ref<OperationPanelShellExpose | null>(null);

const title = computed(() => {
  const count = props.state.entries.length;
  return count > 1 ? `删除 ${count} 项？` : `删除 ${props.state.entries[0]?.name ?? "所选项目"}？`;
});

const message = computed(() => {
  const count = props.state.entries.length;
  return count > 1 ? "这些项目会被移动到回收站，之后可从回收站恢复。" : "该项目会被移动到回收站，之后可从回收站恢复。";
});

const visibleItems = computed(() => props.state.entries.slice(0, 5));
const extraCount = computed(() => Math.max(0, props.state.entries.length - visibleItems.value.length));

defineExpose({
  focus: () => panelRef.value?.focus()
});
</script>

<template>
  <operation-panel-shell
      v-if="state.visible"
      ref="panelRef"
      width="delete"
      variant="red"
      icon="action.delete"
      :title="title"
      :subtitle="message"
      :tabindex="-1"
      @close="emit('close')">
    <div class="delete-confirm-list">
      <div v-for="item in visibleItems" :key="item.path" :title="item.path">
        <file-type-icon :kind="item.type === 'folder' ? 'folder' : 'file'" />
        <span>{{ item.name }}</span>
      </div>
      <div v-if="extraCount" class="delete-confirm-more">
        另有 {{ extraCount }} 项
      </div>
    </div>
    <p v-if="state.error" class="delete-confirm-error">{{ state.error }}</p>
    <template #actions>
      <button type="button" class="operation-secondary" :disabled="state.submitting" @click="emit('close')">取消</button>
      <button type="button" class="delete-confirm-primary" :disabled="state.submitting" @click="emit('submit')">
        {{ state.submitting ? "创建任务中..." : "移动到回收站" }}
      </button>
    </template>
  </operation-panel-shell>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.delete-confirm-list {
  @apply flex max-h-40 flex-col gap-1 overflow-auto rounded-md border border-slate-100 bg-slate-50 p-2;
}

.delete-confirm-list div {
  @apply flex min-h-7 min-w-0 items-center gap-2 rounded px-2 text-xs text-slate-600;
}

.delete-confirm-list span {
  @apply min-w-0 truncate;
}

.delete-confirm-more {
  @apply text-slate-400;
}

.delete-confirm-error {
  @apply rounded-md border border-red-100 bg-red-50 px-3 py-2 text-xs text-red-600;
}

.operation-secondary,
.delete-confirm-primary {
  @apply h-9 rounded-md px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.operation-secondary {
  @apply border border-slate-200 bg-white text-slate-700 hover:bg-slate-50;
}

.delete-confirm-primary {
  @apply bg-red-600 text-white hover:bg-red-700;
}
</style>
