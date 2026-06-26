import type {ComponentPublicInstance} from "vue";

export type EditorCursorStatus = {
  line: number;
  column: number;
  selectedRows: number;
  selectedCharacters: number;
}

export type EditorMenuName = "language" | "theme" | "settings" | "";

export type EditorMenuAnchor = {
  left: number;
  right: number;
  bottom: number;
  align?: "start" | "end";
}

export type PendingEditorAction = "close" | "reload" | "external" | "";

export type EditorInputRefSetter = (element: Element | ComponentPublicInstance | null) => void;

export type EditorSearchOptionName = "case" | "word" | "regex";

export type EditorModeOption = {
  name: string;
  key: string;
}

export type EditorThemeOption = {
  name: string;
  key: string;
  icon?: string;
}

export type EditorThemeGroups = {
  automatic: EditorThemeOption[];
  light: EditorThemeOption[];
  dark: EditorThemeOption[];
}

export type EditorSearchOptions = {
  needle: string;
  backwards?: boolean;
  caseSensitive?: boolean;
  wholeWord?: boolean;
  regex?: boolean;
}

export type CodeEditorExpose = ComponentPublicInstance & {
  focus?: () => void;
  getSelectedText?: () => string;
  getLineCount?: () => number;
  gotoLine?: (line: number, column?: number) => boolean;
  find?: (options: EditorSearchOptions) => boolean;
  replaceCurrent?: (replacement: string) => boolean;
  replaceAll?: (replacement: string) => boolean;
}
