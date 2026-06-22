<script setup lang="ts">
import ace from "ace-builds";
import {onBeforeUnmount, onMounted, ref, watch} from "vue";

let aceReady: Promise<void> | null = null;

const exposeAceGlobal = () => {
  (globalThis as typeof globalThis & {ace?: typeof ace}).ace = ace;
}

const configureAce = () => {
  ace.config.setModuleUrl("ace/mode/base_worker", "/ace/worker-base.js");
  ace.config.setModuleUrl("ace/mode/json_worker", "/ace/worker-json.js");
  ace.config.setModuleUrl("ace/mode/xml_worker", "/ace/worker-xml.js");
  ace.config.setModuleUrl("ace/mode/yaml_worker", "/ace/worker-yaml.js");
}

const ensureAceReady = async () => {
  if (!aceReady) {
    exposeAceGlobal();
    aceReady = Promise.all([
      import("ace-builds/esm-resolver"),
      import("ace-builds/src-noconflict/ext-language_tools")
    ]).then(() => {
      exposeAceGlobal();
      configureAce();
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

type CodeEditorCursorStatus = {
  line: number;
  column: number;
  selectedRows: number;
  selectedCharacters: number;
}

type SearchOptions = {
  needle: string;
  backwards?: boolean;
  caseSensitive?: boolean;
  wholeWord?: boolean;
  regex?: boolean;
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
  (e: "cursor-change", status: CodeEditorCursorStatus): void;
}>()

const editorRef = ref<HTMLElement | null>(null);
let editor: ReturnType<typeof ace.edit> | null = null;
let syncing = false;
let disposed = false;

const findNeedle = (options: SearchOptions) => {
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

watch(() => props.theme, (theme: string) => {
  editor?.setTheme("ace/theme/" + theme);
});

watch(() => props.mode, (mode: string) => {
  editor?.session.setMode("ace/mode/" + mode);
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
  if (disposed || !editorRef.value || editor) return;
  editor = ace.edit(editorRef.value);

  editor.setOptions({
    theme: "ace/theme/" + props.theme,
    fontSize: props.fontSize,
    mode: "ace/mode/" + props.mode,
    value: props.content,
    readOnly: props.readOnly,

    enableBasicAutocompletion: true,
    enableSnippets: true,
    enableLiveAutocompletion: true
  });
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
