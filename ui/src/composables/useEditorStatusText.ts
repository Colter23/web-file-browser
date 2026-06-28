import {computed} from "vue";
import type {Ref} from "vue";
import type {FileInfo} from "../class.ts";
import type {EditorCursorStatus, EditorHighlightOption, EditorModeOption, EditorThemeGroups, PendingEditorAction} from "../components/editor/types.ts";
import {useI18n} from "../i18n";
import type {MessageKey} from "../i18n";
import {formatEntrySize} from "../utils/file-entry.ts";

type EditorStatusTextOptions = {
  fileInfo: Ref<FileInfo | null>;
  currentMode: Ref<string>;
  currentTheme: Ref<string>;
  currentHighlight: Ref<string>;
  modes: readonly EditorModeOption[];
  themes: EditorThemeGroups;
  highlights: readonly EditorHighlightOption[];
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
  currentHighlight,
  modes,
  themes,
  highlights,
  wrap,
  cursorStatus,
  saveConflict,
  errorText,
  pendingAction,
  pendingBusy
}: EditorStatusTextOptions) => {
  const {t} = useI18n();
  const localizedOptionName = (prefix: "theme" | "highlight", key: string, fallback: string) => {
    const messageKey = `editor.${prefix}.${key}` as MessageKey;
    const translated = t(messageKey);
    return translated === messageKey ? fallback : translated;
  }

  const fileTitle = computed(() => fileInfo.value?.name ?? t("editor.noFile"));
  const filePathText = computed(() => fileInfo.value?.path ?? "");

  const selectedModeName = computed(() => modes.find(mode => mode.key === currentMode.value)?.name ?? currentMode.value);

  const selectedThemeName = computed(() => {
    const allThemes = Object.values(themes).flat();
    const theme = allThemes.find(theme => theme.key === currentTheme.value);
    return localizedOptionName("theme", currentTheme.value, theme?.name ?? currentTheme.value);
  });

  const selectedHighlightName = computed(() => {
    const highlight = highlights.find(highlight => highlight.key === currentHighlight.value);
    return localizedOptionName("highlight", currentHighlight.value, highlight?.name ?? currentHighlight.value);
  });

  const wrapText = computed(() => wrap.value ? t("editor.wrap") : t("editor.noWrap"));

  const fileSizeText = computed(() => formatEntrySize(fileInfo.value?.size, "0 B"));
  const cursorStatusText = computed(() => t("editor.cursor", {line: cursorStatus.value.line, column: cursorStatus.value.column}));

  const selectionStatusText = computed(() => {
    if (!cursorStatus.value.selectedCharacters) return "";
    const rows = cursorStatus.value.selectedRows > 1 ? t("editor.selectionRows", {count: cursorStatus.value.selectedRows}) : "";
    return t("editor.selection", {rows, chars: cursorStatus.value.selectedCharacters});
  });

  const editorMessageText = computed(() => errorText.value || (saveConflict.value ? t("editor.versionChanged") : ""));

  const confirmTitle = computed(() => {
    if (pendingAction.value === "reload") return t("editor.confirmReloadTitle");
    if (pendingAction.value === "external") return t("editor.confirmExternalTitle");
    return t("editor.confirmCloseTitle");
  });

  const confirmDescription = computed(() => {
    if (pendingAction.value === "reload") return t("editor.confirmReloadDescription");
    if (pendingAction.value === "external") return t("editor.confirmExternalDescription");
    return t("editor.confirmCloseDescription");
  });

  const confirmSaveText = computed(() => {
    if (pendingBusy.value) return t("editor.processing");
    if (pendingAction.value === "external") return t("editor.saveAndLeave");
    return pendingAction.value === "reload" ? t("editor.saveAndReload") : t("editor.saveAndClose");
  });

  const confirmDiscardText = computed(() => {
    if (pendingAction.value === "reload") return t("editor.discardAndReload");
    if (pendingAction.value === "external") return t("editor.discardAndLeave");
    return t("editor.discardAndClose");
  });

  return {
    fileTitle,
    filePathText,
    selectedModeName,
    selectedThemeName,
    selectedHighlightName,
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
