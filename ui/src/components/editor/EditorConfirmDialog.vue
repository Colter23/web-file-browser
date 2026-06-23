<script setup lang="ts">
import {nextTick, ref, watch} from "vue";
import Icon from "../Icon.vue";

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

const confirmRef = ref<HTMLElement | null>(null);

watch(() => props.visible, async visible => {
  if (!visible) return;
  await nextTick();
  confirmRef.value?.focus();
});
</script>

<template>
  <div v-if="visible" class="editor-confirm-mask" @click.stop>
    <section ref="confirmRef" class="editor-confirm" tabindex="-1" @keydown.esc.prevent.stop="emit('cancel')">
      <div class="confirm-icon">
        <icon icon="action.edit" color="var(--app-accent, #2563eb)" />
      </div>
      <div class="confirm-content">
        <h3>{{ title }}</h3>
        <p>{{ description }}</p>
      </div>
      <div class="confirm-actions">
        <button class="confirm-secondary" :disabled="busy" @click="emit('cancel')">取消</button>
        <button class="confirm-danger" :disabled="busy" @click="emit('discard')">{{ discardText }}</button>
        <button class="confirm-primary" :disabled="!canSave || busy" @click="emit('save')">{{ saveText }}</button>
      </div>
    </section>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-confirm-mask {
  @apply absolute inset-2 z-20 flex items-center justify-center rounded-md px-4 backdrop-blur-sm;
  background: color-mix(in srgb, var(--app-bg) 26%, transparent);
}

.editor-confirm {
  @apply grid w-full max-w-lg grid-cols-[2rem_1fr] gap-3 rounded-md border p-4 shadow-2xl outline-none;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  color: var(--app-text-muted);
}

.editor-confirm:focus-visible {
  box-shadow: inset 0 0 0 2px var(--app-accent-border, #bfdbfe);
}

.confirm-icon {
  @apply flex h-8 w-8 items-center justify-center rounded-md;
  background: var(--app-accent-soft, #eff6ff);
}

.confirm-content {
  @apply min-w-0;
}

.confirm-content h3 {
  @apply text-sm font-semibold;
  color: var(--app-text);
}

.confirm-content p {
  @apply mt-1 text-xs leading-5;
  color: var(--app-text-subtle);
}

.confirm-actions {
  @apply col-span-2 mt-1 flex justify-end gap-2;
}

.confirm-primary,
.confirm-secondary,
.confirm-danger {
  @apply h-8 rounded-md border px-3 text-xs font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.confirm-primary {
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.confirm-primary:hover:not(:disabled) {
  background: var(--app-accent-strong);
}

.confirm-secondary {
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.confirm-secondary:hover:not(:disabled) {
  background: var(--app-control-hover);
}

.confirm-danger {
  border-color: var(--app-danger-border);
  background: var(--app-control-solid);
  color: var(--app-danger);
}

.confirm-danger:hover:not(:disabled) {
  background: var(--app-danger-soft);
}
</style>
