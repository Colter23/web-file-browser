import {ref, watch} from "vue";
import editorConfig from "../assets/editor-config.json";

const storageKeys = {
  theme: "editor.theme",
  fontSize: "editor.fontSize",
  tabSize: "editor.tabSize",
  wrap: "editor.wrap"
};

const allThemeKeys = [...editorConfig.theme.light, ...editorConfig.theme.dark].map(theme => theme.key);

const readStorageItem = (key: string): string | null => {
  if (typeof localStorage === "undefined") return null;
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

const writeStorageItem = (key: string, value: string) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(key, value);
  } catch {
    // 本地存储不可用时，仍保留当前会话内的编辑设置。
  }
}

const normalizeNumberPreference = (value: unknown, fallback: number, min: number, max: number) => {
  const numeric = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(numeric)) return fallback;
  return Math.min(max, Math.max(min, Math.round(numeric)));
}

const readThemePreference = () => {
  const theme = readStorageItem(storageKeys.theme);
  return theme && allThemeKeys.includes(theme) ? theme : "github";
}

const readNumberPreference = (key: string, fallback: number, min: number, max: number) => {
  return normalizeNumberPreference(readStorageItem(key), fallback, min, max);
}

const readBooleanPreference = (key: string, fallback: boolean) => {
  const value = readStorageItem(key);
  if (value === "true") return true;
  if (value === "false") return false;
  return fallback;
}

export const useEditorPreferences = () => {
  const currentTheme = ref(readThemePreference());
  const fontSize = ref(readNumberPreference(storageKeys.fontSize, 16, 12, 28));
  const tabSize = ref(readNumberPreference(storageKeys.tabSize, 2, 2, 8));
  const wrap = ref(readBooleanPreference(storageKeys.wrap, true));

  watch(currentTheme, theme => {
    if (allThemeKeys.includes(theme)) writeStorageItem(storageKeys.theme, theme);
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
    writeStorageItem(storageKeys.wrap, String(Boolean(value)));
  });

  return {
    currentTheme,
    fontSize,
    tabSize,
    wrap
  };
}
