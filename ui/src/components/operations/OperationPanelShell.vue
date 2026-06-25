<script setup lang="ts">
import {computed, ref} from "vue";
import {useDraggablePanel} from "../../composables/useDraggablePanel.ts";
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
const {
  dragging,
  panelStyle,
  resetPosition,
  startDrag
} = useDraggablePanel({panelRef: shellRef});

const shellClass = computed(() => ["operation-shell", `width-${props.width}`, {"is-dragging": dragging.value}]);
const iconClass = computed(() => ["operation-shell-icon", props.variant]);

defineExpose({
  focus: () => shellRef.value?.focus()
});
</script>

<template>
  <teleport to="body">
    <component
        :is="as"
        ref="shellRef"
        :class="shellClass"
        :style="panelStyle"
        :tabindex="tabindex"
        @submit.prevent="emit('submit')"
        @keydown.esc.prevent.stop="emit('close')">
      <div class="operation-shell-header" title="拖动移动面板" @pointerdown="startDrag" @dblclick="resetPosition">
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
  </teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.operation-shell {
  @apply fixed z-50 flex flex-col gap-3 overflow-hidden rounded-lg border p-4 text-sm shadow-2xl outline-none;
  left: 0;
  top: 0;
  max-height: calc(100vh - 2rem);
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  color: var(--app-text-muted);
  box-shadow: var(--app-menu-shadow);
}

.operation-shell.is-dragging {
  @apply select-none;
}

.operation-shell.width-operation {
  width: min(28rem, calc(100vw - 2rem));
}

.operation-shell.width-delete {
  width: min(30rem, calc(100vw - 2rem));
  border-color: var(--app-danger-border);
}

.operation-shell.width-properties {
  width: min(32rem, calc(100vw - 2rem));
}

.operation-shell-header {
  @apply flex cursor-move select-none items-start gap-3;
}

.operation-shell-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-xl;
}

.operation-shell-icon.blue {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.operation-shell-icon.red {
  background: var(--app-danger-soft);
  color: var(--app-danger);
}

.operation-shell-icon.neutral {
  background: var(--app-control);
  color: var(--app-text-muted);
}

.operation-shell-title {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.operation-shell-title strong {
  @apply truncate text-base font-semibold;
  color: var(--app-text);
}

.operation-shell-title span {
  @apply truncate text-xs leading-5;
  color: var(--app-text-subtle);
}

.operation-shell-close {
  @apply flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md;
  color: var(--app-text-subtle);
}

.operation-shell-close:hover {
  background: var(--app-control-hover);
  color: var(--app-text);
}

.operation-shell-close:focus-visible {
  @apply outline-none;
  background: var(--app-control-hover);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.operation-shell-actions {
  @apply flex justify-end gap-2 pt-1;
}
</style>
