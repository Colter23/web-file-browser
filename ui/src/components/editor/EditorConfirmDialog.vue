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
        <icon icon="icon-edit-filling" color="#2563eb" />
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
  @apply absolute inset-2 z-20 flex items-center justify-center rounded-md bg-slate-900/15 px-4 backdrop-blur-sm;
}

.editor-confirm {
  @apply grid w-full max-w-lg grid-cols-[2rem_1fr] gap-3 rounded-md border border-slate-200 bg-white p-4 text-slate-700 shadow-2xl outline-none;
}

.editor-confirm:focus-visible {
  @apply ring-2 ring-inset ring-blue-300;
}

.confirm-icon {
  @apply flex h-8 w-8 items-center justify-center rounded-md bg-blue-50;
}

.confirm-content {
  @apply min-w-0;
}

.confirm-content h3 {
  @apply text-sm font-semibold text-slate-900;
}

.confirm-content p {
  @apply mt-1 text-xs leading-5 text-slate-500;
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
  @apply border-blue-600 bg-blue-600 text-white hover:bg-blue-700;
}

.confirm-secondary {
  @apply border-slate-200 bg-white text-slate-700 hover:bg-slate-50;
}

.confirm-danger {
  @apply border-red-200 bg-white text-red-600 hover:bg-red-50;
}
</style>
