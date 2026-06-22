export type ShellNoticeKind = "info" | "success" | "warning" | "error";

export type ShellNoticePayload = {
  kind: ShellNoticeKind;
  title: string;
  message: string;
}
