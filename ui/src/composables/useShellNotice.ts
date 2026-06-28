import {ref} from "vue";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useI18n} from "../i18n";
import type {MessageKey} from "../i18n";

type ShellNoticeState = {
  id: number;
  visible: boolean;
  kind: ShellNoticeKind;
  title: string;
  message: string;
  durationMs: number;
}

const defaultTitleKey: Record<ShellNoticeKind, MessageKey> = {
  info: "notice.info",
  success: "notice.success",
  warning: "notice.warning",
  error: "notice.error"
};

export const useShellNotice = () => {
  const {t} = useI18n();
  const notice = ref<ShellNoticeState>({
    id: 0,
    visible: false,
    kind: "info",
    title: t("notice.info"),
    message: "",
    durationMs: 3500
  });
  let timer: number | undefined;
  let noticeId = 0;
  let timerStartedAt = 0;
  let remainingMs = 0;

  const stopTimer = () => {
    if (!timer) return;
    window.clearTimeout(timer);
    timer = undefined;
    const elapsed = Date.now() - timerStartedAt;
    remainingMs = Math.max(0, remainingMs - elapsed);
  }

  const close = () => {
    stopTimer();
    notice.value.visible = false;
  }

  const startTimer = (duration: number) => {
    if (duration > 0) {
      timerStartedAt = Date.now();
      remainingMs = duration;
      timer = window.setTimeout(close, duration);
    }
  }

  const show = (message: string, kind: ShellNoticeKind = "info", title?: string, timeoutMs?: number) => {
    stopTimer();
    const duration = timeoutMs ?? (kind === "error" ? 7000 : 3500);
    notice.value = {
      id: ++noticeId,
      visible: true,
      kind,
      title: title ?? t(defaultTitleKey[kind]),
      message,
      durationMs: duration
    };
    startTimer(duration);
  }

  const showError = (error: unknown, fallback: string, title = t("notice.error")) => {
    show(error instanceof Error && error.message ? error.message : fallback, "error", title);
  }

  const resumeTimer = () => {
    if (!notice.value.visible) return;
    stopTimer();
    startTimer(remainingMs || notice.value.durationMs);
  }

  return {
    notice,
    show,
    showError,
    close,
    stopTimer,
    resumeTimer
  };
}
