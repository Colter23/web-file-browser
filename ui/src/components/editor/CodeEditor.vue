<script setup lang="ts">
import ace from "ace-builds";
import {onBeforeUnmount, onMounted, ref, watch} from "vue";
import {aceModePath, aceThemePath, loadAceMode, loadAceTheme, registerAceResources} from "./ace-resources.ts";
import type {EditorCursorStatus, EditorSearchOptions} from "./types.ts";

let aceReady: Promise<void> | null = null;

const exposeAceGlobal = () => {
  (globalThis as typeof globalThis & {ace?: typeof ace}).ace = ace;
}

const ensureAceReady = async () => {
  if (!aceReady) {
    exposeAceGlobal();
    aceReady = Promise.all([
      import("ace-builds/src-noconflict/ext-language_tools")
    ]).then(() => {
      exposeAceGlobal();
      registerAceResources();
    });
  }
  await aceReady;
}

interface CodeEditorProps {
  mode: string;
  theme: string;
  content: string;
  fontSize?: number;
  wrap?: boolean;
  tabSize?: number;
  readOnly?: boolean;
}

const props = withDefaults(defineProps<CodeEditorProps>(), {
  mode: "text",
  theme: "dracula",
  content: "",
  fontSize: 16,
  wrap: true,
  tabSize: 2,
  readOnly: false
})

const emit = defineEmits<{
  (e: "change", content: string): void;
  (e: "save"): void;
  (e: "find"): void;
  (e: "replace"): void;
  (e: "goto-line"): void;
  (e: "cursor-change", status: EditorCursorStatus): void;
}>()

const editorRef = ref<HTMLElement | null>(null);
let editor: ReturnType<typeof ace.edit> | null = null;
let syncing = false;
let disposed = false;
const editorVerticalInset = 12;
const editorScrollMargin = 8;
let themeLoadToken = 0;
let modeLoadToken = 0;

const findNeedle = (options: EditorSearchOptions) => {
  if (!editor || !options.needle) return false;
  try {
    const range = editor.find(options.needle, {
      backwards: Boolean(options.backwards),
      wrap: true,
      caseSensitive: Boolean(options.caseSensitive),
      wholeWord: Boolean(options.wholeWord),
      regExp: Boolean(options.regex),
      preventScroll: false
    });
    editor.focus();
    return Boolean(range);
  } catch {
    return false;
  }
}

const replaceCurrent = (replacement: string) => {
  if (!editor) return false;
  try {
    const before = editor.getValue();
    editor.replace(replacement);
    const changed = editor.getValue() !== before;
    if (changed) emitCursorStatus();
    editor.focus();
    return changed;
  } catch {
    return false;
  }
}

const replaceEverywhere = (replacement: string) => {
  if (!editor) return false;
  try {
    const before = editor.getValue();
    editor.replaceAll(replacement);
    const changed = editor.getValue() !== before;
    if (changed) emitCursorStatus();
    editor.focus();
    return changed;
  } catch {
    return false;
  }
}

const gotoLine = (line: number, column = 0) => {
  if (!editor || !Number.isFinite(line)) return false;
  const targetLine = Math.max(1, Math.min(Math.round(line), editor.session.getLength()));
  const targetColumn = Math.max(0, Math.round(column));
  editor.gotoLine(targetLine, targetColumn, true);
  editor.focus();
  emitCursorStatus();
  return true;
}

const lineCount = () => editor?.session.getLength() ?? 0;

const emitCursorStatus = () => {
  if (!editor) return;
  const cursor = editor.getCursorPosition();
  const range = editor.getSelectionRange();
  const selectedCharacters = editor.getSelectedText().length;
  const selectedRows = selectedCharacters > 0 ? Math.abs(range.end.row - range.start.row) + 1 : 0;
  emit("cursor-change", {
    line: cursor.row + 1,
    column: cursor.column + 1,
    selectedRows,
    selectedCharacters
  });
}

const applyEditorSpacing = () => {
  if (!editor) return;
  editor.renderer.setMargin(editorVerticalInset, editorVerticalInset, 0, 0);
  editor.renderer.setScrollMargin(editorScrollMargin, editorScrollMargin);
}

watch(() => props.theme, async (theme: string) => {
  const token = ++themeLoadToken;
  await loadAceTheme(theme);
  if (!editor || disposed || token !== themeLoadToken) return;
  editor.setTheme(aceThemePath(theme));
  requestAnimationFrame(applyEditorSpacing);
});

watch(() => props.mode, async (mode: string) => {
  const token = ++modeLoadToken;
  await loadAceMode(mode);
  if (!editor || disposed || token !== modeLoadToken) return;
  editor.session.setMode(aceModePath(mode));
});

watch(() => props.content, (content: string) => {
  if (!editor || editor.getValue() === content) return;
  syncing = true;
  editor.session.setValue(content);
  syncing = false;
  emitCursorStatus();
});

watch(() => props.fontSize, (fontSize: number) => {
  editor?.setOption("fontSize", fontSize);
});

watch(() => props.wrap, (wrap: boolean) => {
  editor?.session.setUseWrapMode(wrap);
});

watch(() => props.tabSize, (tabSize: number) => {
  editor?.session.setTabSize(tabSize);
});

watch(() => props.readOnly, (readOnly: boolean) => {
  editor?.setReadOnly(readOnly);
});

onMounted(() => {
  if (!editorRef.value) return;
  void initializeEditor();
})

const initializeEditor = async () => {
  await ensureAceReady();
  await Promise.all([loadAceTheme(props.theme), loadAceMode(props.mode)]);
  if (disposed || !editorRef.value || editor) return;
  editor = ace.edit(editorRef.value);

  editor.setOptions({
    theme: aceThemePath(props.theme),
    fontSize: props.fontSize,
    mode: aceModePath(props.mode),
    value: props.content,
    readOnly: props.readOnly,

    enableBasicAutocompletion: true,
    enableSnippets: true,
    enableLiveAutocompletion: true
  });
  applyEditorSpacing();
  editor.session.setUseWrapMode(props.wrap);
  editor.session.setTabSize(props.tabSize);
  editor.commands.addCommand({
    name: "saveFile",
    bindKey: {win: "Ctrl-S", mac: "Command-S"},
    exec: () => emit("save")
  });
  editor.commands.addCommand({
    name: "openFindBar",
    bindKey: {win: "Ctrl-F", mac: "Command-F"},
    exec: () => emit("find")
  });
  editor.commands.addCommand({
    name: "openReplaceBar",
    bindKey: {win: "Ctrl-H", mac: "Command-Option-F"},
    exec: () => emit("replace")
  });
  editor.commands.addCommand({
    name: "openGotoLineBar",
    bindKey: {win: "Ctrl-G", mac: "Command-L"},
    exec: () => emit("goto-line")
  });
  editor.selection.on("changeCursor", emitCursorStatus);
  editor.selection.on("changeSelection", emitCursorStatus);
  emitCursorStatus();

  editor.session.on("change", () => {
    if (editor && !syncing) {
      emit("change", editor.getValue());
      emitCursorStatus();
    }
  });
}

onBeforeUnmount(() => {
  disposed = true;
  editor?.destroy();
  editor = null;
});

defineExpose({
  focus: () => editor?.focus(),
  getSelectedText: () => editor?.getSelectedText() ?? "",
  getLineCount: lineCount,
  gotoLine,
  find: findNeedle,
  replaceCurrent,
  replaceAll: replaceEverywhere
})
</script>

<template>
  <div ref="editorRef" class="code-editor"></div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
.code-editor {
  @apply w-full h-full
}
</style>
