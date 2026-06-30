<script setup lang="ts">
import {Compartment, EditorSelection, EditorState, Prec, RangeSetBuilder} from "@codemirror/state";
import type {Extension} from "@codemirror/state";
import {Decoration, EditorView, keymap, ViewPlugin} from "@codemirror/view";
import type {DecorationSet, ViewUpdate} from "@codemirror/view";
import {basicSetup} from "codemirror";
import {indentWithTab} from "@codemirror/commands";
import {SearchQuery, highlightSelectionMatches, search, setSearchQuery} from "@codemirror/search";
import {indentUnit} from "@codemirror/language";
import {onBeforeUnmount, onMounted, ref, watch} from "vue";
import {loadCodeMirrorLanguage} from "./codemirror-languages.ts";
import {createCodeMirrorTheme} from "./codemirror-theme.ts";
import type {EditorCursorStatus, EditorSearchOptions} from "./types.ts";
import {useAppearanceStore} from "../../store/appearance.ts";

interface CodeEditorProps {
  mode: string;
  theme: string;
  highlight: string;
  content: string;
  fontSize?: number;
  wrap?: boolean;
  tabSize?: number;
  showWhitespace?: boolean;
  readOnly?: boolean;
}

type SearchMatch = {
  from: number;
  to: number;
}

const props = withDefaults(defineProps<CodeEditorProps>(), {
  mode: "text",
  theme: "app",
  highlight: "default",
  content: "",
  fontSize: 18,
  wrap: true,
  tabSize: 2,
  showWhitespace: false,
  readOnly: false
})

const emit = defineEmits<{
  (e: "change", content: string): void;
  (e: "save"): void;
  (e: "find"): void;
  (e: "replace"): void;
  (e: "goto-line"): void;
  (e: "cursor-change", status: EditorCursorStatus): void;
  (e: "zoom-font", step: number): void;
}>()

const appearanceStore = useAppearanceStore();
const editorRef = ref<HTMLElement | null>(null);
const languageCompartment = new Compartment();
const themeCompartment = new Compartment();
const wrapCompartment = new Compartment();
const tabCompartment = new Compartment();
const whitespaceCompartment = new Compartment();
const readOnlyCompartment = new Compartment();
const fontSizeCompartment = new Compartment();
let view: EditorView | null = null;
let syncing = false;
let disposed = false;
let languageLoadToken = 0;
let lastSearchOptions: EditorSearchOptions | null = null;

const tabExtensions = (tabSize: number) => [
  EditorState.tabSize.of(tabSize),
  indentUnit.of(" ".repeat(tabSize))
];

const readOnlyExtensions = (readOnly: boolean) => [
  EditorState.readOnly.of(readOnly),
  EditorView.editable.of(!readOnly)
];

const fontSizeExtension = (fontSize: number) => EditorView.theme({
  "&": {
    fontSize: `${fontSize}px`
  }
});

const wrapExtension = (wrap: boolean) => wrap ? EditorView.lineWrapping : [];

const visibleSpaceDecoration = Decoration.mark({class: "cm-visible-space"});
const visibleTabDecoration = Decoration.mark({class: "cm-visible-tab"});

const buildWhitespaceDecorations = (targetView: EditorView): DecorationSet => {
  const builder = new RangeSetBuilder<Decoration>();
  for (const {from, to} of targetView.visibleRanges) {
    const text = targetView.state.doc.sliceString(from, to);
    for (let index = 0; index < text.length; index++) {
      const code = text.charCodeAt(index);
      if (code === 32 || code === 9) {
        const position = from + index;
        builder.add(position, position + 1, code === 9 ? visibleTabDecoration : visibleSpaceDecoration);
      }
    }
  }
  return builder.finish();
}

const visibleWhitespacePlugin = ViewPlugin.fromClass(class {
  decorations: DecorationSet;

  constructor(targetView: EditorView) {
    this.decorations = buildWhitespaceDecorations(targetView);
  }

  update(update: ViewUpdate) {
    if (update.docChanged || update.viewportChanged) {
      this.decorations = buildWhitespaceDecorations(update.view);
    }
  }
}, {
  decorations: value => value.decorations
});

const visibleWhitespaceTheme = EditorView.baseTheme({
  "&": {
    "--cm-visible-whitespace": "color-mix(in srgb, var(--app-text-subtle) 50%, transparent)"
  },
  ".cm-visible-space": {
    backgroundImage: "radial-gradient(circle, var(--cm-visible-whitespace) 1px, transparent 1.3px)",
    backgroundPosition: "center",
    backgroundRepeat: "no-repeat"
  },
  ".cm-visible-tab": {
    backgroundImage: [
      "linear-gradient(to right, var(--cm-visible-whitespace), var(--cm-visible-whitespace))",
      "linear-gradient(45deg, transparent 45%, var(--cm-visible-whitespace) 46% 54%, transparent 55%)",
      "linear-gradient(-45deg, transparent 45%, var(--cm-visible-whitespace) 46% 54%, transparent 55%)"
    ].join(", "),
    backgroundPosition: "0.15em 55%, calc(100% - 0.45em) 55%, calc(100% - 0.45em) 55%",
    backgroundRepeat: "no-repeat",
    backgroundSize: "calc(100% - 0.55em) 1px, 0.35em 0.35em, 0.35em 0.35em"
  }
});

const whitespaceExtension = (showWhitespace: boolean): Extension => showWhitespace ? [
  visibleWhitespaceTheme,
  visibleWhitespacePlugin
] : [];

const customKeymap = () => Prec.highest(keymap.of([
  indentWithTab,
  {
    key: "Mod-s",
    run: () => {
      emit("save");
      return true;
    }
  },
  {
    key: "Mod-f",
    run: () => {
      emit("find");
      return true;
    }
  },
  {
    key: "Mod-h",
    run: () => {
      emit("replace");
      return true;
    }
  },
  {
    key: "Mod-g",
    run: () => {
      emit("goto-line");
      return true;
    }
  }
]));

const createExtensions = (languageExtension: Extension): Extension[] => [
  basicSetup,
  search({top: true}),
  highlightSelectionMatches(),
  customKeymap(),
  EditorView.updateListener.of(handleEditorUpdate),
  languageCompartment.of(languageExtension),
  themeCompartment.of(createCodeMirrorTheme(props.theme, appearanceStore.resolvedColorMode, props.highlight)),
  wrapCompartment.of(wrapExtension(props.wrap)),
  tabCompartment.of(tabExtensions(props.tabSize)),
  whitespaceCompartment.of(whitespaceExtension(props.showWhitespace)),
  readOnlyCompartment.of(readOnlyExtensions(props.readOnly)),
  fontSizeCompartment.of(fontSizeExtension(props.fontSize))
];

const createSearchQuery = (options: EditorSearchOptions, replacement = "") => new SearchQuery({
  search: options.needle,
  caseSensitive: Boolean(options.caseSensitive),
  literal: !options.regex,
  regexp: Boolean(options.regex),
  replace: replacement,
  wholeWord: Boolean(options.wholeWord)
});

const collectMatches = (query: SearchQuery, from = 0, to = view?.state.doc.length ?? 0) => {
  if (!view || !query.valid) return [];
  const matches: SearchMatch[] = [];
  const cursor = query.getCursor(view.state, from, to);
  for (let next = cursor.next(); !next.done; next = cursor.next()) {
    const match = next.value;
    if (match.to < match.from) continue;
    matches.push({from: match.from, to: match.to});
  }
  return matches;
}

const findMatch = (options: EditorSearchOptions) => {
  if (!view || !options.needle) return null;
  const query = createSearchQuery(options);
  if (!query.valid) return null;
  const selection = view.state.selection.main;
  const docLength = view.state.doc.length;

  if (options.backwards) {
    const before = collectMatches(query, 0, selection.from);
    if (before.length) return before[before.length - 1];
    const wrapped = collectMatches(query, selection.to, docLength);
    return wrapped.length ? wrapped[wrapped.length - 1] : null;
  }

  const start = selection.empty ? selection.from : selection.to;
  const after = collectMatches(query, start, docLength);
  if (after.length) return after[0];
  const wrapped = collectMatches(query, 0, start);
  return wrapped.length ? wrapped[0] : null;
}

const selectRange = (from: number, to: number, query?: SearchQuery) => {
  if (!view) return;
  view.dispatch({
    selection: EditorSelection.range(from, to),
    effects: [
      EditorView.scrollIntoView(from, {y: "center"}),
      ...(query ? [setSearchQuery.of(query)] : [])
    ]
  });
  view.focus();
  emitCursorStatus();
}

const findNeedle = (options: EditorSearchOptions) => {
  if (!view || !options.needle) return false;
  lastSearchOptions = options;
  const match = findMatch(options);
  if (!match) return false;
  selectRange(match.from, match.to, createSearchQuery(options));
  return true;
}

const isCurrentSelectionMatch = (query: SearchQuery) => {
  if (!view || !query.valid) return false;
  const selection = view.state.selection.main;
  if (selection.empty) return false;
  const match = collectMatches(query, selection.from, selection.to)[0];
  return Boolean(match && match.from === selection.from && match.to === selection.to);
}

const replacementFor = (text: string, options: EditorSearchOptions, replacement: string) => {
  if (!options.regex) return replacement;
  try {
    const flags = options.caseSensitive ? "" : "i";
    return text.replace(new RegExp(options.needle, flags), replacement);
  } catch {
    return replacement;
  }
}

const replaceCurrent = (replacement: string) => {
  if (!view || props.readOnly || !lastSearchOptions) return false;
  const query = createSearchQuery(lastSearchOptions, replacement);
  if (!isCurrentSelectionMatch(query)) return false;
  const selection = view.state.selection.main;
  const selectedText = view.state.sliceDoc(selection.from, selection.to);
  const nextText = replacementFor(selectedText, lastSearchOptions, replacement);
  view.dispatch({
    changes: {from: selection.from, to: selection.to, insert: nextText},
    selection: EditorSelection.range(selection.from, selection.from + nextText.length),
    effects: EditorView.scrollIntoView(selection.from, {y: "center"})
  });
  view.focus();
  emitCursorStatus();
  return true;
}

const replaceEverywhere = (replacement: string) => {
  if (!view || props.readOnly || !lastSearchOptions) return false;
  const query = createSearchQuery(lastSearchOptions, replacement);
  const matches = collectMatches(query);
  if (!matches.length) return false;
  const changes = matches.map(match => ({
    from: match.from,
    to: match.to,
    insert: replacementFor(view!.state.sliceDoc(match.from, match.to), lastSearchOptions!, replacement)
  }));
  view.dispatch({changes});
  view.focus();
  emitCursorStatus();
  return true;
}

const gotoLine = (line: number, column = 0) => {
  if (!view || !Number.isFinite(line)) return false;
  const targetLine = Math.max(1, Math.min(Math.round(line), view.state.doc.lines));
  const lineInfo = view.state.doc.line(targetLine);
  const targetColumn = Math.max(0, Math.round(column));
  const position = Math.min(lineInfo.from + targetColumn, lineInfo.to);
  view.dispatch({
    selection: EditorSelection.cursor(position),
    effects: EditorView.scrollIntoView(position, {y: "center"})
  });
  view.focus();
  emitCursorStatus();
  return true;
}

const lineCount = () => view?.state.doc.lines ?? 0;

const selectedRange = () => view?.state.selection.main;

const emitCursorStatus = () => {
  if (!view) return;
  const selection = view.state.selection.main;
  const cursor = view.state.doc.lineAt(selection.head);
  const from = Math.min(selection.from, selection.to);
  const to = Math.max(selection.from, selection.to);
  const selectedCharacters = to - from;
  const selectedRows = selectedCharacters > 0
      ? view.state.doc.lineAt(to).number - view.state.doc.lineAt(from).number + 1
      : 0;
  emit("cursor-change", {
    line: cursor.number,
    column: selection.head - cursor.from + 1,
    selectedRows,
    selectedCharacters
  });
}

const handleEditorUpdate = (update: ViewUpdate) => {
  if (update.docChanged && !syncing) {
    emit("change", update.state.doc.toString());
  }
  if (update.docChanged || update.selectionSet) {
    emitCursorStatus();
  }
}

const handleWheel = (event: WheelEvent) => {
  if (!event.ctrlKey || event.deltaY === 0) return;
  event.preventDefault();
  event.stopPropagation();
  emit("zoom-font", event.deltaY < 0 ? 1 : -1);
}

watch(() => [props.theme, appearanceStore.resolvedColorMode, props.highlight] as const, ([theme, appColorMode, highlight]) => {
  view?.dispatch({effects: themeCompartment.reconfigure(createCodeMirrorTheme(theme, appColorMode, highlight))});
});

watch(() => props.mode, async mode => {
  const token = ++languageLoadToken;
  const languageExtension = await loadCodeMirrorLanguage(mode);
  if (!view || disposed || token !== languageLoadToken) return;
  view.dispatch({effects: languageCompartment.reconfigure(languageExtension)});
});

watch(() => props.content, content => {
  if (!view || view.state.doc.toString() === content) return;
  syncing = true;
  view.dispatch({
    changes: {from: 0, to: view.state.doc.length, insert: content}
  });
  syncing = false;
  emitCursorStatus();
});

watch(() => props.fontSize, fontSize => {
  view?.dispatch({effects: fontSizeCompartment.reconfigure(fontSizeExtension(fontSize))});
});

watch(() => props.wrap, wrap => {
  view?.dispatch({effects: wrapCompartment.reconfigure(wrapExtension(wrap))});
});

watch(() => props.tabSize, tabSize => {
  view?.dispatch({effects: tabCompartment.reconfigure(tabExtensions(tabSize))});
});

watch(() => props.showWhitespace, showWhitespace => {
  view?.dispatch({effects: whitespaceCompartment.reconfigure(whitespaceExtension(showWhitespace))});
});

watch(() => props.readOnly, readOnly => {
  view?.dispatch({effects: readOnlyCompartment.reconfigure(readOnlyExtensions(readOnly))});
});

const initializeEditor = async () => {
  const token = ++languageLoadToken;
  const languageExtension = await loadCodeMirrorLanguage(props.mode);
  if (disposed || token !== languageLoadToken || !editorRef.value || view) return;
  view = new EditorView({
    state: EditorState.create({
      doc: props.content,
      extensions: createExtensions(languageExtension)
    }),
    parent: editorRef.value
  });
  emitCursorStatus();
}

onMounted(() => {
  void initializeEditor();
});

onBeforeUnmount(() => {
  disposed = true;
  view?.destroy();
  view = null;
});

defineExpose({
  focus: () => view?.focus(),
  getSelectedText: () => {
    if (!view) return "";
    const range = selectedRange();
    return range ? view.state.sliceDoc(range.from, range.to) : "";
  },
  getLineCount: lineCount,
  gotoLine,
  find: findNeedle,
  replaceCurrent,
  replaceAll: replaceEverywhere
})
</script>

<template>
  <div ref="editorRef" class="code-editor" @wheel.capture="handleWheel"></div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.code-editor {
  @apply h-full w-full;
}

.code-editor :deep(.cm-editor) {
  @apply h-full;
}
</style>
