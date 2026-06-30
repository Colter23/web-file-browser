<script setup lang="ts">
import {nextTick, ref, watch} from "vue";
import {useI18n} from "../../i18n";
import OperationPanelShell from "../operations/OperationPanelShell.vue";

type OperationPanelShellExpose = {
  focus: () => void;
}

const props = defineProps<{
  visible: boolean;
  title: string;
  description: string;
  saveText: string;
  discardText: string;
  canSave: boolean;
  busy: boolean;
}>();

const emit = defineEmits<{
  (e: "cancel"): void;
  (e: "discard"): void;
  (e: "save"): void;
}>();

const {t} = useI18n();
const confirmRef = ref<OperationPanelShellExpose | null>(null);

watch(() => props.visible, async visible => {
  if (!visible) return;
  await nextTick();
  confirmRef.value?.focus();
});
</script>

<template>
  <operation-panel-shell
      v-if="visible"
      ref="confirmRef"
      width="operation"
      variant="blue"
      icon="action.save"
      :title="title"
      :subtitle="description"
      :tabindex="-1"
      @close="emit('cancel')">
    <template #actions>
      <button type="button" class="confirm-secondary" :disabled="busy" @click="emit('cancel')">{{ t("editor.cancel") }}</button>
      <button type="button" class="confirm-danger" :disabled="busy" @click="emit('discard')">{{ discardText }}</button>
      <button type="button" class="confirm-primary" :disabled="!canSave || busy" @click="emit('save')">{{ saveText }}</button>
    </template>
  </operation-panel-shell>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.confirm-primary {
  @apply h-9 rounded-md px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.confirm-primary:hover:not(:disabled) {
  background: var(--app-accent-strong);
}

.confirm-secondary {
  @apply h-9 rounded-md border px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.confirm-secondary:hover:not(:disabled) {
  background: var(--app-control-hover);
}

.confirm-danger {
  @apply h-9 rounded-md border px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
  border-color: var(--app-danger-border);
  background: var(--app-control-solid);
  color: var(--app-danger);
}

.confirm-danger:hover:not(:disabled) {
  background: var(--app-danger-soft);
}

.confirm-primary:focus-visible,
.confirm-secondary:focus-visible,
.confirm-danger:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}
</style>
