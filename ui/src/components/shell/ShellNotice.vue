<script setup lang="ts">
import {computed} from "vue";
import {useI18n} from "../../i18n";
import Icon from "../Icon.vue";
import type {ShellNoticeKind} from "./types.ts";

const props = defineProps<{
  kind: ShellNoticeKind;
  message: string;
  title?: string;
}>();

const {t} = useI18n();
const emit = defineEmits<{
  (e: "close"): void;
  (e: "pause"): void;
  (e: "resume"): void;
}>();

const shellNoticeLabel = computed(() => ({
  info: t("notice.info"),
  success: t("notice.success"),
  warning: t("notice.warning"),
  error: t("notice.error")
}[props.kind]));

const noticeIcon = computed(() => ({
  info: "action.properties",
  success: "action.check",
  warning: "action.warning",
  error: "action.close"
}[props.kind]));
</script>

<template>
  <section
      :class="['shell-notice', kind]"
      role="status"
      aria-live="polite"
      @mouseenter="emit('pause')"
      @mouseleave="emit('resume')"
      @focusin="emit('pause')"
      @focusout="emit('resume')">
    <div class="shell-notice-mark" aria-hidden="true">
      <icon :icon="noticeIcon" size="small" />
    </div>
    <div class="shell-notice-body">
      <strong>{{ title || shellNoticeLabel }}</strong>
      <span>{{ message }}</span>
    </div>
    <button type="button" class="shell-notice-close" :title="t('notice.close')" @click="emit('close')">
      <icon icon="action.close" />
    </button>
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.shell-notice {
  @apply flex w-[min(26rem,calc(100vw-3rem))] items-start gap-2.5 rounded-lg border px-3 py-2 text-sm backdrop-blur-xl;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 94%, transparent);
  color: var(--app-text-muted);
  box-shadow: var(--app-menu-shadow);
}

.shell-notice-mark {
  @apply mt-0.5 inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full;
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(59, 130, 246, 0.15));
}

.shell-notice-body {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.shell-notice-body strong {
  @apply text-sm font-semibold leading-5;
  color: var(--app-text);
}

.shell-notice-body span {
  @apply line-clamp-3 break-words text-xs leading-5;
  color: var(--app-text-muted);
}

.shell-notice-close {
  @apply -mr-1 flex h-7 w-7 shrink-0 items-center justify-center rounded-md transition;
  color: var(--app-text-subtle);
}

.shell-notice-close:hover {
  background: var(--app-control-hover);
  color: var(--app-text-muted);
}

.shell-notice.success {
  border-color: var(--app-success-border);
}

.shell-notice.success .shell-notice-mark {
  background: var(--app-success);
  color: var(--app-success-contrast);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-success) 18%, transparent);
}

.shell-notice.warning {
  border-color: var(--app-warning-border);
}

.shell-notice.warning .shell-notice-mark {
  background: var(--app-warning);
  color: var(--app-warning-contrast);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-warning) 20%, transparent);
}

.shell-notice.error {
  border-color: var(--app-danger-border);
}

.shell-notice.error .shell-notice-mark {
  background: var(--app-danger);
  color: var(--app-danger-contrast);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-danger) 20%, transparent);
}
</style>
