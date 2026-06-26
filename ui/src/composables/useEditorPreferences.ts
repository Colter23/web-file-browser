import {ref, watch} from "vue";
import editorConfig from "../assets/editor-config.json";
import {readBooleanStorage, readStorageItem, writeBooleanStorage, writeStorageItem} from "../utils/safe-storage.ts";
import type {EditorHighlightOption, EditorThemeOption} from "../components/editor/types.ts";

const storageKeys = {
  theme: "editor.theme",
  highlight: "editor.highlight",
  fontSize: "editor.fontSize",
  tabSize: "editor.tabSize",
  wrap: "editor.wrap"
};

const allThemes = Object.values(editorConfig.theme).flat() as EditorThemeOption[];
const allThemeKeys = allThemes.map(theme => theme.key);
const allHighlights = editorConfig.highlight as EditorHighlightOption[];
const allHighlightKeys = allHighlights.map(highlight => highlight.key);
const legacyThemeMap: Record<string, string> = {
  soft_light: "paper_light",
  soft_dark: "midnight_dark"
};

const normalizeNumberPreference = (value: unknown, fallback: number, min: number, max: number) => {
  const numeric = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(numeric)) return fallback;
  return Math.min(max, Math.max(min, Math.round(numeric)));
}

const readThemePreference = () => {
  const theme = readStorageItem(storageKeys.theme);
  if (theme && legacyThemeMap[theme]) return legacyThemeMap[theme];
  return theme && allThemeKeys.includes(theme) ? theme : "app";
}

const readHighlightPreference = () => {
  const highlight = readStorageItem(storageKeys.highlight);
  return highlight && allHighlightKeys.includes(highlight) ? highlight : "default";
}

const readNumberPreference = (key: string, fallback: number, min: number, max: number) => {
  return normalizeNumberPreference(readStorageItem(key), fallback, min, max);
}

const readBooleanPreference = (key: string, fallback: boolean) => {
  return readBooleanStorage(key, fallback);
}

export const useEditorPreferences = () => {
  const currentTheme = ref(readThemePreference());
  const currentHighlight = ref(readHighlightPreference());
  const fontSize = ref(readNumberPreference(storageKeys.fontSize, 16, 12, 28));
  const tabSize = ref(readNumberPreference(storageKeys.tabSize, 2, 2, 8));
  const wrap = ref(readBooleanPreference(storageKeys.wrap, true));

  watch(currentTheme, theme => {
    if (allThemeKeys.includes(theme)) writeStorageItem(storageKeys.theme, theme);
  });

  watch(currentHighlight, highlight => {
    if (allHighlightKeys.includes(highlight)) writeStorageItem(storageKeys.highlight, highlight);
  });

  watch(fontSize, value => {
    const normalized = normalizeNumberPreference(value, 16, 12, 28);
    if (value !== normalized) {
      fontSize.value = normalized;
      return;
    }
    writeStorageItem(storageKeys.fontSize, String(normalized));
  });

  watch(tabSize, value => {
    const normalized = normalizeNumberPreference(value, 2, 2, 8);
    if (value !== normalized) {
      tabSize.value = normalized;
      return;
    }
    writeStorageItem(storageKeys.tabSize, String(normalized));
  });

  watch(wrap, value => {
    writeBooleanStorage(storageKeys.wrap, Boolean(value));
  });

  return {
    currentTheme,
    currentHighlight,
    fontSize,
    tabSize,
    wrap
  };
}
