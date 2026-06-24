import {computed} from "vue";
import type {Ref} from "vue";
import type {FileInfo} from "../class.ts";
import type {EditorCursorStatus, EditorModeOption, EditorThemeGroups, PendingEditorAction} from "../components/editor/types.ts";
import {formatEntrySize} from "../utils/file-entry.ts";

type EditorStatusTextOptions = {
  fileInfo: Ref<FileInfo | null>;
  currentMode: Ref<string>;
  currentTheme: Ref<string>;
  modes: readonly EditorModeOption[];
  themes: EditorThemeGroups;
  wrap: Ref<boolean>;
  cursorStatus: Ref<EditorCursorStatus>;
  saveConflict: Ref<boolean>;
  errorText: Ref<string>;
  pendingAction: Ref<PendingEditorAction>;
  pendingBusy: Ref<boolean>;
}

export const useEditorStatusText = ({
  fileInfo,
  currentMode,
  currentTheme,
  modes,
  themes,
  wrap,
  cursorStatus,
  saveConflict,
  errorText,
  pendingAction,
  pendingBusy
}: EditorStatusTextOptions) => {
  const fileTitle = computed(() => fileInfo.value?.name ?? "未打开文件");
  const filePathText = computed(() => fileInfo.value?.path ?? "");

  const selectedModeName = computed(() => modes.find(mode => mode.key === currentMode.value)?.name ?? currentMode.value);

  const selectedThemeName = computed(() => {
    const allThemes = [...themes.light, ...themes.dark];
    return allThemes.find(theme => theme.key === currentTheme.value)?.name ?? currentTheme.value;
  });

  const wrapText = computed(() => wrap.value ? "自动换行" : "不换行");

  const fileSizeText = computed(() => formatEntrySize(fileInfo.value?.size, "0 B"));
  const cursorStatusText = computed(() => `第 ${cursorStatus.value.line} 行，第 ${cursorStatus.value.column} 列`);

  const selectionStatusText = computed(() => {
    if (!cursorStatus.value.selectedCharacters) return "";
    const rows = cursorStatus.value.selectedRows > 1 ? `${cursorStatus.value.selectedRows} 行，` : "";
    return `已选中 ${rows}${cursorStatus.value.selectedCharacters} 字符`;
  });

  const editorMessageText = computed(() => errorText.value || (saveConflict.value ? "文件版本已变化，请重新载入后再保存" : ""));

  const confirmTitle = computed(() => {
    if (pendingAction.value === "reload") return "重新载入文件？";
    if (pendingAction.value === "external") return "离开编辑器？";
    return "关闭编辑器？";
  });

  const confirmDescription = computed(() => {
    if (pendingAction.value === "reload") return "当前修改还没有保存，重新载入会用磁盘上的最新内容覆盖编辑区。";
    if (pendingAction.value === "external") return "当前修改还没有保存，继续操作会离开编辑器并丢弃未保存内容。";
    return "当前修改还没有保存，关闭后未保存的内容会被丢弃。";
  });

  const confirmSaveText = computed(() => {
    if (pendingBusy.value) return "处理中";
    if (pendingAction.value === "external") return "保存并离开";
    return pendingAction.value === "reload" ? "保存并重新载入" : "保存并关闭";
  });

  const confirmDiscardText = computed(() => {
    if (pendingAction.value === "reload") return "放弃并重新载入";
    if (pendingAction.value === "external") return "放弃并离开";
    return "放弃并关闭";
  });

  return {
    fileTitle,
    filePathText,
    selectedModeName,
    selectedThemeName,
    fileSizeText,
    wrapText,
    cursorStatusText,
    selectionStatusText,
    editorMessageText,
    confirmTitle,
    confirmDescription,
    confirmSaveText,
    confirmDiscardText
  };
}
