import type {Extension} from "@codemirror/state";
import {EditorView} from "@codemirror/view";
import {HighlightStyle, syntaxHighlighting} from "@codemirror/language";
import {tags} from "@lezer/highlight";
import type {ResolvedColorMode} from "../../store/appearance.ts";

type EditorThemePalette = {
  dark: boolean;
  background: string;
  gutterBackground: string;
  text: string;
  textMuted: string;
  textFaint: string;
  border: string;
  activeLine: string;
  activeGutter: string;
}

const appPalette = (dark: boolean): EditorThemePalette => ({
  dark,
  background: "var(--app-panel-solid)",
  gutterBackground: "color-mix(in srgb, var(--app-panel-muted) 72%, transparent)",
  text: "var(--app-text)",
  textMuted: "var(--app-text-subtle)",
  textFaint: "var(--app-text-faint)",
  border: "var(--app-border-soft)",
  activeLine: "color-mix(in srgb, var(--app-accent) 8%, transparent)",
  activeGutter: "color-mix(in srgb, var(--app-accent) 10%, transparent)"
});

const lightPalette: EditorThemePalette = {
  dark: false,
  background: "#ffffff",
  gutterBackground: "#f8fafc",
  text: "#0f172a",
  textMuted: "#64748b",
  textFaint: "#94a3b8",
  border: "#e2e8f0",
  activeLine: "color-mix(in srgb, var(--app-accent) 7%, transparent)",
  activeGutter: "color-mix(in srgb, var(--app-accent) 10%, white)"
};

const softLightPalette: EditorThemePalette = {
  dark: false,
  background: "color-mix(in srgb, var(--app-accent) 2%, #ffffff)",
  gutterBackground: "color-mix(in srgb, var(--app-accent) 4%, #f8fafc)",
  text: "#172033",
  textMuted: "#64748b",
  textFaint: "#94a3b8",
  border: "color-mix(in srgb, var(--app-accent) 12%, #e2e8f0)",
  activeLine: "color-mix(in srgb, var(--app-accent) 9%, transparent)",
  activeGutter: "color-mix(in srgb, var(--app-accent) 13%, white)"
};

const darkPalette: EditorThemePalette = {
  dark: true,
  background: "#111827",
  gutterBackground: "#0f172a",
  text: "#e5e7eb",
  textMuted: "#94a3b8",
  textFaint: "#64748b",
  border: "#263244",
  activeLine: "color-mix(in srgb, var(--app-accent) 14%, transparent)",
  activeGutter: "color-mix(in srgb, var(--app-accent) 18%, #0f172a)"
};

const softDarkPalette: EditorThemePalette = {
  dark: true,
  background: "#172033",
  gutterBackground: "#121b2b",
  text: "#e2e8f0",
  textMuted: "#a5b4c8",
  textFaint: "#718096",
  border: "#2b3a52",
  activeLine: "color-mix(in srgb, var(--app-accent) 13%, transparent)",
  activeGutter: "color-mix(in srgb, var(--app-accent) 17%, #121b2b)"
};

const resolvePalette = (theme: string, appColorMode: ResolvedColorMode): EditorThemePalette => {
  if (theme === "light") return lightPalette;
  if (theme === "soft_light") return softLightPalette;
  if (theme === "dark") return darkPalette;
  if (theme === "soft_dark") return softDarkPalette;
  return appPalette(appColorMode === "dark");
}

const editorTheme = (palette: EditorThemePalette) => EditorView.theme({
  "&": {
    height: "100%",
    color: palette.text,
    backgroundColor: palette.background
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
    color: palette.textFaint,
    backgroundColor: palette.gutterBackground,
    borderRight: `1px solid ${palette.border}`
  },
  ".cm-activeLine": {
    backgroundColor: palette.activeLine
  },
  ".cm-activeLineGutter": {
    color: palette.text,
    backgroundColor: palette.activeGutter
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
    color: palette.text,
    backgroundColor: palette.background,
    border: `1px solid ${palette.border}`
  },
  ".cm-tooltip-autocomplete ul li[aria-selected]": {
    color: "var(--app-text)",
    backgroundColor: "color-mix(in srgb, var(--app-accent) 18%, transparent)"
  }
}, {dark: palette.dark});

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

export const createCodeMirrorTheme = (theme: string, appColorMode: ResolvedColorMode): Extension => {
  const palette = resolvePalette(theme, appColorMode);
  return [
    editorTheme(palette),
    syntaxHighlighting(palette.dark ? darkHighlightStyle : lightHighlightStyle)
  ];
}
