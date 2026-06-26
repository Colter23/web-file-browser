import type {Extension} from "@codemirror/state";
import {EditorView} from "@codemirror/view";
import {HighlightStyle, syntaxHighlighting} from "@codemirror/language";
import {tags} from "@lezer/highlight";

const darkThemeKeys = new Set([
  "chaos",
  "cobalt",
  "dracula",
  "github_dark",
  "merbivore",
  "merbivore_soft",
  "monokai",
  "nord_dark",
  "one_dark",
  "pastel_on_dark",
  "solarized_dark",
  "tomorrow_night",
  "tomorrow_night_blue",
  "vibrant_ink"
]);

const editorTheme = (dark: boolean) => EditorView.theme({
  "&": {
    height: "100%",
    color: "var(--app-text)",
    backgroundColor: "var(--app-panel-solid)"
  },
  "&.cm-focused": {
    outline: "none"
  },
  ".cm-scroller": {
    fontFamily: "\"Cascadia Mono\", Consolas, \"SFMono-Regular\", monospace",
    lineHeight: "1.58"
  },
  ".cm-content": {
    minHeight: "100%",
    padding: "12px 0",
    caretColor: "var(--app-accent)"
  },
  ".cm-line": {
    padding: "0 14px"
  },
  ".cm-gutters": {
    color: "var(--app-text-faint)",
    backgroundColor: "color-mix(in srgb, var(--app-panel-muted) 72%, transparent)",
    borderRight: "1px solid var(--app-border-soft)"
  },
  ".cm-activeLine": {
    backgroundColor: "color-mix(in srgb, var(--app-accent) 8%, transparent)"
  },
  ".cm-activeLineGutter": {
    color: "var(--app-text)",
    backgroundColor: "color-mix(in srgb, var(--app-accent) 10%, transparent)"
  },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground, ::selection": {
    backgroundColor: "color-mix(in srgb, var(--app-accent) 28%, transparent)"
  },
  ".cm-cursor": {
    borderLeftColor: "var(--app-accent)"
  },
  ".cm-searchMatch": {
    backgroundColor: "color-mix(in srgb, #facc15 42%, transparent)",
    outline: "1px solid color-mix(in srgb, #ca8a04 70%, transparent)"
  },
  ".cm-searchMatch.cm-searchMatch-selected": {
    backgroundColor: "color-mix(in srgb, #fb923c 54%, transparent)"
  },
  ".cm-tooltip, .cm-tooltip-autocomplete": {
    color: "var(--app-text)",
    backgroundColor: "var(--app-panel-solid)",
    border: "1px solid var(--app-border)"
  },
  ".cm-tooltip-autocomplete ul li[aria-selected]": {
    color: "var(--app-text)",
    backgroundColor: "color-mix(in srgb, var(--app-accent) 18%, transparent)"
  }
}, {dark});

const lightHighlightStyle = HighlightStyle.define([
  {tag: tags.keyword, color: "#7c3aed"},
  {tag: [tags.atom, tags.bool, tags.number], color: "#0f766e"},
  {tag: [tags.string, tags.special(tags.string)], color: "#b45309"},
  {tag: tags.comment, color: "#64748b", fontStyle: "italic"},
  {tag: [tags.variableName, tags.propertyName], color: "#2563eb"},
  {tag: [tags.function(tags.variableName), tags.function(tags.propertyName)], color: "#be123c"},
  {tag: [tags.typeName, tags.className], color: "#9333ea"},
  {tag: tags.operator, color: "#475569"},
  {tag: tags.link, color: "#2563eb", textDecoration: "underline"},
  {tag: tags.heading, color: "#0f172a", fontWeight: "600"},
  {tag: tags.invalid, color: "#dc2626"}
]);

const darkHighlightStyle = HighlightStyle.define([
  {tag: tags.keyword, color: "#c4b5fd"},
  {tag: [tags.atom, tags.bool, tags.number], color: "#67e8f9"},
  {tag: [tags.string, tags.special(tags.string)], color: "#fde68a"},
  {tag: tags.comment, color: "#94a3b8", fontStyle: "italic"},
  {tag: [tags.variableName, tags.propertyName], color: "#93c5fd"},
  {tag: [tags.function(tags.variableName), tags.function(tags.propertyName)], color: "#fda4af"},
  {tag: [tags.typeName, tags.className], color: "#ddd6fe"},
  {tag: tags.operator, color: "#cbd5e1"},
  {tag: tags.link, color: "#93c5fd", textDecoration: "underline"},
  {tag: tags.heading, color: "#f8fafc", fontWeight: "600"},
  {tag: tags.invalid, color: "#f87171"}
]);

export const createCodeMirrorTheme = (theme: string): Extension => {
  const dark = darkThemeKeys.has(theme);
  return [
    editorTheme(dark),
    syntaxHighlighting(dark ? darkHighlightStyle : lightHighlightStyle)
  ];
}
