<script setup lang="ts">
import {computed, ref} from "vue";
import Icon from "../Icon.vue";

type ExplorerEntry = {
  type: "folder" | "file";
  name: string;
  path: string;
}

type DeleteConfirmState = {
  visible: boolean;
  entries: ExplorerEntry[];
  submitting: boolean;
  error: string;
}

const props = defineProps<{
  state: DeleteConfirmState;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "submit"): void;
}>();

const panelRef = ref<HTMLElement | null>(null);

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
  <section
      v-if="state.visible"
      ref="panelRef"
      class="delete-confirm-panel"
      tabindex="-1"
      @keydown.esc.prevent="emit('close')">
    <div class="delete-confirm-header">
      <div class="delete-confirm-icon">
        <icon icon="icon-delete-fill" />
      </div>
      <div class="delete-confirm-title">
        <strong>{{ title }}</strong>
        <span>{{ message }}</span>
      </div>
      <button type="button" class="operation-panel-close" title="关闭" @click="emit('close')">
        <icon icon="icon-close" />
      </button>
    </div>
    <div class="delete-confirm-list">
      <div v-for="item in visibleItems" :key="item.path" :title="item.path">
        <icon :icon="item.type === 'folder' ? 'icon-folder-fill' : 'icon-file-fill'" />
        <span>{{ item.name }}</span>
      </div>
      <div v-if="extraCount" class="delete-confirm-more">
        另有 {{ extraCount }} 项
      </div>
    </div>
    <p v-if="state.error" class="delete-confirm-error">{{ state.error }}</p>
    <div class="delete-confirm-actions">
      <button type="button" class="operation-secondary" :disabled="state.submitting" @click="emit('close')">取消</button>
      <button type="button" class="delete-confirm-primary" :disabled="state.submitting" @click="emit('submit')">
        {{ state.submitting ? "创建任务中..." : "移动到回收站" }}
      </button>
    </div>
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.delete-confirm-panel {
  @apply absolute left-1/2 top-6 z-30 flex w-[min(30rem,calc(100%-2rem))] -translate-x-1/2 flex-col gap-3 rounded-lg border border-red-100 bg-white p-4 text-sm text-slate-700 shadow-2xl outline-none;
}

.delete-confirm-header {
  @apply flex items-start gap-3;
}

.delete-confirm-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-red-50 text-xl text-red-600;
}

.delete-confirm-title {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.delete-confirm-title strong {
  @apply truncate text-base font-semibold text-slate-900;
}

.delete-confirm-title span {
  @apply text-xs leading-5 text-slate-500;
}

.operation-panel-close {
  @apply flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-slate-500 hover:bg-slate-100;
}

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

.delete-confirm-actions {
  @apply flex justify-end gap-2 pt-1;
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
