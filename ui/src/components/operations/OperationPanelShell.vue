<script setup lang="ts">
import {computed, ref} from "vue";
import Icon from "../Icon.vue";

type OperationPanelShellTag = "form" | "section";
type OperationPanelShellVariant = "blue" | "red" | "neutral";
type OperationPanelShellWidth = "operation" | "delete" | "properties";

const props = withDefaults(defineProps<{
  as?: OperationPanelShellTag;
  icon: string;
  title: string;
  subtitle: string;
  variant?: OperationPanelShellVariant;
  width?: OperationPanelShellWidth;
  tabindex?: number;
}>(), {
  as: "section",
  variant: "blue",
  width: "operation",
  tabindex: undefined
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "submit"): void;
}>();

const shellRef = ref<HTMLElement | null>(null);

const shellClass = computed(() => ["operation-shell", `width-${props.width}`]);
const iconClass = computed(() => ["operation-shell-icon", props.variant]);

defineExpose({
  focus: () => shellRef.value?.focus()
});
</script>

<template>
  <component
      :is="as"
      ref="shellRef"
      :class="shellClass"
      :tabindex="tabindex"
      @submit.prevent="emit('submit')"
      @keydown.esc.prevent.stop="emit('close')">
    <div class="operation-shell-header">
      <div :class="iconClass">
        <icon :icon="icon" />
      </div>
      <div class="operation-shell-title">
        <strong>{{ title }}</strong>
        <span>{{ subtitle }}</span>
      </div>
      <button type="button" class="operation-shell-close" title="关闭" @click="emit('close')">
        <icon icon="action.close" />
      </button>
    </div>
    <slot />
    <div v-if="$slots.actions" class="operation-shell-actions">
      <slot name="actions" />
    </div>
  </component>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.operation-shell {
  @apply absolute left-1/2 top-6 z-30 flex -translate-x-1/2 flex-col gap-3 rounded-lg border bg-white p-4 text-sm text-slate-700 shadow-2xl outline-none;
}

.operation-shell.width-operation {
  @apply w-[min(28rem,calc(100%-2rem))] border-slate-200;
}

.operation-shell.width-delete {
  @apply w-[min(30rem,calc(100%-2rem))] border-red-100;
}

.operation-shell.width-properties {
  @apply w-[min(32rem,calc(100%-2rem))] border-slate-200;
}

.operation-shell-header {
  @apply flex items-start gap-3;
}

.operation-shell-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-xl;
}

.operation-shell-icon.blue {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.operation-shell-icon.red {
  @apply bg-red-50 text-red-600;
}

.operation-shell-icon.neutral {
  @apply bg-slate-100 text-slate-600;
}

.operation-shell-title {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.operation-shell-title strong {
  @apply truncate text-base font-semibold text-slate-900;
}

.operation-shell-title span {
  @apply truncate text-xs leading-5 text-slate-500;
}

.operation-shell-close {
  @apply flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-slate-500 hover:bg-slate-100;
}

.operation-shell-actions {
  @apply flex justify-end gap-2 pt-1;
}
</style>
