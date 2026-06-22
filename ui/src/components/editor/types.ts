import type {ComponentPublicInstance} from "vue";

export type EditorCursorStatus = {
  line: number;
  column: number;
  selectedRows: number;
  selectedCharacters: number;
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
