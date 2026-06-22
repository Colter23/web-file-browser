import {ref} from "vue";
import type {ShellNoticeKind} from "../components/shell/types.ts";

type ShellNoticeState = {
  visible: boolean;
  kind: ShellNoticeKind;
  title: string;
  message: string;
}

const defaultTitle: Record<ShellNoticeKind, string> = {
  info: "提示",
  success: "完成",
  warning: "需要注意",
  error: "操作失败"
};

export const useShellNotice = () => {
  const notice = ref<ShellNoticeState>({
    visible: false,
    kind: "info",
    title: "提示",
    message: ""
  });
  let timer: number | undefined;

  const stopTimer = () => {
    if (!timer) return;
    window.clearTimeout(timer);
    timer = undefined;
  }

  const close = () => {
    stopTimer();
    notice.value.visible = false;
  }

  const show = (message: string, kind: ShellNoticeKind = "info", title?: string, timeoutMs?: number) => {
    stopTimer();
    notice.value = {
      visible: true,
      kind,
      title: title ?? defaultTitle[kind],
      message
    };
    const duration = timeoutMs ?? (kind === "error" ? 7000 : 3500);
    if (duration > 0) {
      timer = window.setTimeout(close, duration);
    }
  }

  const showError = (error: unknown, fallback: string, title = "操作失败") => {
    show(error instanceof Error && error.message ? error.message : fallback, "error", title);
  }

  return {
    notice,
    show,
    showError,
    close,
    stopTimer
  };
}
