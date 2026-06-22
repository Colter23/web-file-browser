<script setup lang="ts">
import {computed} from "vue";
import Icon from "../Icon.vue";

type ShellNoticeKind = "info" | "success" | "warning" | "error";

const props = defineProps<{
  kind: ShellNoticeKind;
  message: string;
  title?: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const shellNoticeLabel = computed(() => ({
  info: "提示",
  success: "完成",
  warning: "需要注意",
  error: "操作失败"
}[props.kind]));
</script>

<template>
  <section :class="['shell-notice', kind]" role="status" aria-live="polite">
    <div class="shell-notice-mark" aria-hidden="true"></div>
    <div class="shell-notice-body">
      <strong>{{ title || shellNoticeLabel }}</strong>
      <span>{{ message }}</span>
    </div>
    <button type="button" class="shell-notice-close" title="关闭提示" @click="emit('close')">
      <icon icon="icon-close" />
    </button>
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.shell-notice {
  @apply absolute right-4 top-4 z-20 flex w-[min(24rem,calc(100%-2rem))] items-start gap-3 rounded-lg border bg-white/95 px-3 py-2 text-sm text-slate-700 shadow-xl backdrop-blur;
}

.shell-notice-mark {
  @apply mt-1 h-2.5 w-2.5 shrink-0 rounded-full bg-blue-500 shadow-[0_0_0_3px_rgba(59,130,246,0.15)];
}

.shell-notice-body {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.shell-notice-body strong {
  @apply truncate text-sm font-semibold text-slate-900;
}

.shell-notice-body span {
  @apply break-words text-xs leading-5 text-slate-600;
}

.shell-notice-close {
  @apply -mr-1 flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-slate-400 hover:bg-slate-100 hover:text-slate-700;
}

.shell-notice.success {
  @apply border-emerald-100;
}

.shell-notice.success .shell-notice-mark {
  @apply bg-emerald-500 shadow-[0_0_0_3px_rgba(16,185,129,0.15)];
}

.shell-notice.warning {
  @apply border-amber-100;
}

.shell-notice.warning .shell-notice-mark {
  @apply bg-amber-500 shadow-[0_0_0_3px_rgba(245,158,11,0.16)];
}

.shell-notice.error {
  @apply border-red-100;
}

.shell-notice.error .shell-notice-mark {
  @apply bg-red-500 shadow-[0_0_0_3px_rgba(239,68,68,0.16)];
}
</style>
