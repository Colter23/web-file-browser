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
  selection: string;
  searchMatch: string;
  searchMatchOutline: string;
  searchSelected: string;
}

type SyntaxHighlightColors = {
  keyword: string;
  atom: string;
  string: string;
  comment: string;
  variable: string;
  function: string;
  type: string;
  operator: string;
  link: string;
  heading: string;
  invalid: string;
}

type SyntaxHighlightPair = {
  light: HighlightStyle;
  dark: HighlightStyle;
}

const foldChevronMask = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2.4' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m9 18 6-6-6-6'/%3E%3C/svg%3E\")";

const appPalette = (dark: boolean): EditorThemePalette => ({
  dark,
  background: "var(--app-panel-solid)",
  gutterBackground: "color-mix(in srgb, var(--app-panel-muted) 72%, transparent)",
  text: "var(--app-text)",
  textMuted: "var(--app-text-subtle)",
  textFaint: "var(--app-text-faint)",
  border: "var(--app-border-soft)",
  activeLine: "color-mix(in srgb, var(--app-accent) 8%, transparent)",
  activeGutter: "color-mix(in srgb, var(--app-accent) 10%, transparent)",
  selection: "color-mix(in srgb, var(--app-accent) 28%, transparent)",
  searchMatch: "color-mix(in srgb, #facc15 42%, transparent)",
  searchMatchOutline: "color-mix(in srgb, #ca8a04 70%, transparent)",
  searchSelected: "color-mix(in srgb, #fb923c 54%, transparent)"
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
  activeGutter: "color-mix(in srgb, var(--app-accent) 10%, white)",
  selection: "color-mix(in srgb, var(--app-accent) 26%, transparent)",
  searchMatch: "color-mix(in srgb, #facc15 42%, transparent)",
  searchMatchOutline: "color-mix(in srgb, #ca8a04 70%, transparent)",
  searchSelected: "color-mix(in srgb, #fb923c 54%, transparent)"
};

const paperLightPalette: EditorThemePalette = {
  dark: false,
  background: "#fffaf0",
  gutterBackground: "#f6eddd",
  text: "#2c241d",
  textMuted: "#76685c",
  textFaint: "#a08f7d",
  border: "#eadbc7",
  activeLine: "color-mix(in srgb, #f59e0b 11%, transparent)",
  activeGutter: "color-mix(in srgb, #f59e0b 13%, #f6eddd)",
  selection: "color-mix(in srgb, #d97706 24%, transparent)",
  searchMatch: "color-mix(in srgb, #fbbf24 50%, transparent)",
  searchMatchOutline: "color-mix(in srgb, #b45309 62%, transparent)",
  searchSelected: "color-mix(in srgb, #ea580c 44%, transparent)"
};

const mistLightPalette: EditorThemePalette = {
  dark: false,
  background: "#f6f9fd",
  gutterBackground: "#edf4fb",
  text: "#152238",
  textMuted: "#586b82",
  textFaint: "#8ba0b6",
  border: "#d5e1ec",
  activeLine: "color-mix(in srgb, #0ea5e9 10%, transparent)",
  activeGutter: "color-mix(in srgb, #0ea5e9 13%, #edf4fb)",
  selection: "color-mix(in srgb, #0ea5e9 24%, transparent)",
  searchMatch: "color-mix(in srgb, #facc15 38%, transparent)",
  searchMatchOutline: "color-mix(in srgb, #0284c7 46%, transparent)",
  searchSelected: "color-mix(in srgb, #38bdf8 35%, transparent)"
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
  activeGutter: "color-mix(in srgb, var(--app-accent) 18%, #0f172a)",
  selection: "color-mix(in srgb, var(--app-accent) 32%, transparent)",
  searchMatch: "color-mix(in srgb, #facc15 34%, transparent)",
  searchMatchOutline: "color-mix(in srgb, #fde68a 45%, transparent)",
  searchSelected: "color-mix(in srgb, #fb923c 44%, transparent)"
};

const midnightDarkPalette: EditorThemePalette = {
  dark: true,
  background: "#08111f",
  gutterBackground: "#060d18",
  text: "#dbeafe",
  textMuted: "#93a8c6",
  textFaint: "#5f789a",
  border: "#18324f",
  activeLine: "color-mix(in srgb, #38bdf8 13%, transparent)",
  activeGutter: "color-mix(in srgb, #38bdf8 18%, #060d18)",
  selection: "color-mix(in srgb, #38bdf8 30%, transparent)",
  searchMatch: "color-mix(in srgb, #eab308 34%, transparent)",
  searchMatchOutline: "color-mix(in srgb, #7dd3fc 45%, transparent)",
  searchSelected: "color-mix(in srgb, #fb923c 42%, transparent)"
};

const graphiteDarkPalette: EditorThemePalette = {
  dark: true,
  background: "#18181b",
  gutterBackground: "#111113",
  text: "#e4e4e7",
  textMuted: "#a1a1aa",
  textFaint: "#71717a",
  border: "#303036",
  activeLine: "color-mix(in srgb, #a1a1aa 13%, transparent)",
  activeGutter: "color-mix(in srgb, #a1a1aa 16%, #111113)",
  selection: "color-mix(in srgb, #a1a1aa 28%, transparent)",
  searchMatch: "color-mix(in srgb, #facc15 33%, transparent)",
  searchMatchOutline: "color-mix(in srgb, #fef3c7 38%, transparent)",
  searchSelected: "color-mix(in srgb, #fb923c 38%, transparent)"
};

const resolvePalette = (theme: string, appColorMode: ResolvedColorMode): EditorThemePalette => {
  if (theme === "light") return lightPalette;
  if (theme === "paper_light" || theme === "soft_light") return paperLightPalette;
  if (theme === "mist_light") return mistLightPalette;
  if (theme === "dark") return darkPalette;
  if (theme === "midnight_dark" || theme === "soft_dark") return midnightDarkPalette;
  if (theme === "graphite_dark") return graphiteDarkPalette;
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
  ".cm-foldGutter": {
    minWidth: "28px"
  },
  ".cm-foldGutter .cm-gutterElement": {
    boxSizing: "border-box",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    minWidth: "28px",
    padding: "0 3px"
  },
  ".cm-foldGutter span": {
    boxSizing: "border-box",
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    width: "22px",
    height: "22px",
    margin: "0 auto",
    padding: "0",
    borderRadius: "6px",
    color: palette.textFaint,
    cursor: "pointer",
    fontSize: "0",
    lineHeight: "1",
    transition: "color 120ms ease, background-color 120ms ease"
  },
  ".cm-foldGutter span::before": {
    content: "\"\"",
    width: "14px",
    height: "14px",
    backgroundColor: "currentColor",
    maskImage: foldChevronMask,
    maskPosition: "center",
    maskRepeat: "no-repeat",
    maskSize: "14px 14px",
    WebkitMaskImage: foldChevronMask,
    WebkitMaskPosition: "center",
    WebkitMaskRepeat: "no-repeat",
    WebkitMaskSize: "14px 14px",
    transform: "rotate(0deg)",
    transition: "transform 120ms ease"
  },
  ".cm-foldGutter span[title='Fold line']::before": {
    transform: "rotate(90deg)"
  },
  ".cm-foldGutter span:hover": {
    color: palette.text,
    backgroundColor: "color-mix(in srgb, var(--app-accent) 12%, transparent)"
  },
  ".cm-activeLine": {
    backgroundColor: palette.activeLine
  },
  ".cm-activeLineGutter": {
    color: palette.text,
    backgroundColor: palette.activeGutter
  },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground, &.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, ::selection": {
    backgroundColor: palette.selection
  },
  ".cm-cursor": {
    borderLeftColor: "var(--app-accent)"
  },
  ".cm-searchMatch": {
    backgroundColor: palette.searchMatch,
    outline: `1px solid ${palette.searchMatchOutline}`
  },
  ".cm-searchMatch.cm-searchMatch-selected": {
    backgroundColor: palette.searchSelected
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

const createSyntaxHighlightStyle = (colors: SyntaxHighlightColors) => HighlightStyle.define([
  {tag: tags.keyword, color: colors.keyword},
  {tag: [tags.atom, tags.bool, tags.number], color: colors.atom},
  {tag: [tags.string, tags.special(tags.string)], color: colors.string},
  {tag: tags.comment, color: colors.comment, fontStyle: "italic"},
  {tag: [tags.variableName, tags.propertyName], color: colors.variable},
  {tag: [tags.function(tags.variableName), tags.function(tags.propertyName)], color: colors.function},
  {tag: [tags.typeName, tags.className], color: colors.type},
  {tag: tags.operator, color: colors.operator},
  {tag: [tags.punctuation, tags.bracket, tags.separator], color: colors.operator},
  {tag: tags.regexp, color: colors.atom},
  {tag: tags.escape, color: colors.operator, fontWeight: "650"},
  {tag: tags.color, color: colors.atom, fontWeight: "650"},
  {tag: [tags.macroName, tags.labelName], color: colors.function, fontWeight: "650"},
  {tag: [tags.namespace, tags.annotation], color: colors.type},
  {tag: [tags.meta, tags.documentMeta], color: colors.comment},
  {tag: [tags.heading1, tags.heading2], color: colors.heading, fontWeight: "750"},
  {tag: [tags.heading3, tags.heading4, tags.heading5, tags.heading6, tags.heading], color: colors.heading, fontWeight: "650"},
  {tag: [tags.link, tags.url], color: colors.link, textDecoration: "underline"},
  {tag: tags.processingInstruction, color: colors.operator},
  {tag: tags.contentSeparator, color: colors.operator, fontWeight: "650"},
  {tag: tags.list, color: colors.atom},
  {tag: tags.quote, color: colors.comment, fontStyle: "italic"},
  {tag: tags.emphasis, color: colors.variable, fontStyle: "italic"},
  {tag: tags.strong, color: colors.function, fontWeight: "750"},
  {
    tag: tags.monospace,
    color: colors.string,
    backgroundColor: "color-mix(in srgb, currentColor 12%, transparent)",
    borderRadius: "4px",
    padding: "0 0.16em"
  },
  {tag: tags.strikethrough, color: colors.comment, textDecoration: "line-through"},
  {tag: tags.inserted, color: colors.type},
  {tag: tags.deleted, color: colors.invalid, textDecoration: "line-through"},
  {tag: tags.changed, color: colors.heading},
  {tag: tags.invalid, color: colors.invalid}
]);

const syntaxHighlightStyles: Record<string, SyntaxHighlightPair> = {
  default: {
    light: createSyntaxHighlightStyle({
      keyword: "#7c3aed",
      atom: "#0d9488",
      string: "#d97706",
      comment: "#64748b",
      variable: "#2563eb",
      function: "#e11d48",
      type: "#16a34a",
      operator: "#475569",
      link: "#0284c7",
      heading: "#6d28d9",
      invalid: "#dc2626"
    }),
    dark: createSyntaxHighlightStyle({
      keyword: "#c084fc",
      atom: "#2dd4bf",
      string: "#fbbf24",
      comment: "#94a3b8",
      variable: "#60a5fa",
      function: "#fb7185",
      type: "#4ade80",
      operator: "#cbd5e1",
      link: "#38bdf8",
      heading: "#c084fc",
      invalid: "#f87171"
    })
  },
  vivid: {
    light: createSyntaxHighlightStyle({
      keyword: "#c026d3",
      atom: "#ea580c",
      string: "#16a34a",
      comment: "#64748b",
      variable: "#0891b2",
      function: "#dc2626",
      type: "#7c3aed",
      operator: "#111827",
      link: "#2563eb",
      heading: "#c026d3",
      invalid: "#dc2626"
    }),
    dark: createSyntaxHighlightStyle({
      keyword: "#ff5cdb",
      atom: "#fb923c",
      string: "#86efac",
      comment: "#94a3b8",
      variable: "#22d3ee",
      function: "#f87171",
      type: "#c4b5fd",
      operator: "#f8fafc",
      link: "#93c5fd",
      heading: "#ff5cdb",
      invalid: "#fb7185"
    })
  },
  muted: {
    light: createSyntaxHighlightStyle({
      keyword: "#4f46e5",
      atom: "#059669",
      string: "#ca8a04",
      comment: "#64748b",
      variable: "#0284c7",
      function: "#be123c",
      type: "#0d9488",
      operator: "#374151",
      link: "#2563eb",
      heading: "#4f46e5",
      invalid: "#b91c1c"
    }),
    dark: createSyntaxHighlightStyle({
      keyword: "#a5b4fc",
      atom: "#34d399",
      string: "#facc15",
      comment: "#9ca3af",
      variable: "#38bdf8",
      function: "#fda4af",
      type: "#2dd4bf",
      operator: "#d1d5db",
      link: "#93c5fd",
      heading: "#a5b4fc",
      invalid: "#f87171"
    })
  },
  cool: {
    light: createSyntaxHighlightStyle({
      keyword: "#0891b2",
      atom: "#ca8a04",
      string: "#65a30d",
      comment: "#64748b",
      variable: "#db2777",
      function: "#f97316",
      type: "#8b5cf6",
      operator: "#334155",
      link: "#2563eb",
      heading: "#0891b2",
      invalid: "#dc2626"
    }),
    dark: createSyntaxHighlightStyle({
      keyword: "#22d3ee",
      atom: "#fde047",
      string: "#a3e635",
      comment: "#94a3b8",
      variable: "#f472b6",
      function: "#fb923c",
      type: "#c084fc",
      operator: "#e2e8f0",
      link: "#7dd3fc",
      heading: "#22d3ee",
      invalid: "#f87171"
    })
  },
  warm: {
    light: createSyntaxHighlightStyle({
      keyword: "#e11d48",
      atom: "#2563eb",
      string: "#d97706",
      comment: "#78716c",
      variable: "#7c3aed",
      function: "#059669",
      type: "#db2777",
      operator: "#57534e",
      link: "#0284c7",
      heading: "#e11d48",
      invalid: "#dc2626"
    }),
    dark: createSyntaxHighlightStyle({
      keyword: "#fb7185",
      atom: "#93c5fd",
      string: "#fbbf24",
      comment: "#a8a29e",
      variable: "#c4b5fd",
      function: "#86efac",
      type: "#f0abfc",
      operator: "#e7e5e4",
      link: "#7dd3fc",
      heading: "#fb7185",
      invalid: "#fb7185"
    })
  }
};

const resolveSyntaxHighlightStyle = (highlight: string, dark: boolean) => {
  const pair = syntaxHighlightStyles[highlight] ?? syntaxHighlightStyles.default;
  return dark ? pair.dark : pair.light;
}

export const createCodeMirrorTheme = (theme: string, appColorMode: ResolvedColorMode, highlight = "default"): Extension => {
  const palette = resolvePalette(theme, appColorMode);
  return [
    editorTheme(palette),
    syntaxHighlighting(resolveSyntaxHighlightStyle(highlight, palette.dark))
  ];
}
