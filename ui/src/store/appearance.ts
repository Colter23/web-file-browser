import {defineStore} from "pinia";
import type {AppAccentColor, AppColorMode, AppIconStyle, FileIconPalette, FileIconStyle} from "../class.ts";
import {readStorageItem, writeStorageItem} from "../utils/safe-storage.ts";

const storageKeys = {
  iconStyle: "appearance.iconStyle",
  fileIconStyle: "appearance.fileIconStyle",
  fileIconPalette: "appearance.fileIconPalette",
  accentColor: "appearance.accentColor",
  colorMode: "appearance.colorMode"
}

export type ResolvedColorMode = "light" | "dark";

export const iconStyleOptions: {value: AppIconStyle; label: string}[] = [
  {value: "lucide", label: "线性"},
  {value: "fluent", label: "填充"},
  {value: "solar", label: "双色"}
];

export const fileIconStyleOptions: {value: FileIconStyle; label: string}[] = [
  {value: "inherit", label: "跟随样式"},
  {value: "vscode-icons", label: "VSCode"},
  {value: "catppuccin", label: "Catppuccin"}
];

export const fileIconPaletteOptions: {value: FileIconPalette; label: string}[] = [
  {value: "category", label: "按类型着色"},
  {value: "accent", label: "跟随主题色"}
];

export const accentColorOptions: {value: AppAccentColor; label: string; color: string; contrast: string; soft: string; border: string}[] = [
  {value: "blue", label: "蓝色", color: "#2563eb", contrast: "#ffffff", soft: "#eff6ff", border: "#bfdbfe"},
  {value: "teal", label: "青绿", color: "#0f766e", contrast: "#ffffff", soft: "#f0fdfa", border: "#99f6e4"},
  {value: "violet", label: "紫色", color: "#7c3aed", contrast: "#ffffff", soft: "#f5f3ff", border: "#ddd6fe"},
  {value: "rose", label: "玫红", color: "#e11d48", contrast: "#ffffff", soft: "#fff1f2", border: "#fecdd3"},
  {value: "slate", label: "灰蓝", color: "#475569", contrast: "#ffffff", soft: "#f8fafc", border: "#cbd5e1"}
];

export const colorModeOptions: {value: AppColorMode; label: string}[] = [
  {value: "system", label: "跟随系统"},
  {value: "light", label: "亮色"},
  {value: "dark", label: "暗色"}
];

const iconStyles = iconStyleOptions.map(option => option.value);
const fileIconStyles = fileIconStyleOptions.map(option => option.value);
const fileIconPalettes = fileIconPaletteOptions.map(option => option.value);
const accentColors = accentColorOptions.map(option => option.value);
const colorModes = colorModeOptions.map(option => option.value);

const readIconStyle = (): AppIconStyle => {
  const value = readStorageItem(storageKeys.iconStyle);
  if (value === "classic" || value === "material") return "fluent";
  if (value === "phosphor") return "solar";
  if (value === "vscode-icons" || value === "catppuccin") return "lucide";
  return iconStyles.includes(value as AppIconStyle) ? value as AppIconStyle : "lucide";
}

const readFileIconStyle = (): FileIconStyle => {
  const value = readStorageItem(storageKeys.fileIconStyle);
  if (fileIconStyles.includes(value as FileIconStyle)) return value as FileIconStyle;
  const legacyValue = readStorageItem(storageKeys.iconStyle);
  return legacyValue === "vscode-icons" || legacyValue === "catppuccin" ? legacyValue : "inherit";
}

const readFileIconPalette = (): FileIconPalette => {
  const value = readStorageItem(storageKeys.fileIconPalette);
  return fileIconPalettes.includes(value as FileIconPalette) ? value as FileIconPalette : "category";
}

const readAccentColor = (): AppAccentColor => {
  const value = readStorageItem(storageKeys.accentColor);
  return accentColors.includes(value as AppAccentColor) ? value as AppAccentColor : "blue";
}

const readColorMode = (): AppColorMode => {
  const value = readStorageItem(storageKeys.colorMode);
  return colorModes.includes(value as AppColorMode) ? value as AppColorMode : "system";
}

export const resolveSystemColorMode = (): ResolvedColorMode => {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export const useAppearanceStore = defineStore("appearance", {
  state: () => ({
    iconStyle: readIconStyle(),
    fileIconStyle: readFileIconStyle(),
    fileIconPalette: readFileIconPalette(),
    accentColor: readAccentColor(),
    colorMode: readColorMode(),
    systemColorMode: resolveSystemColorMode()
  }),
  getters: {
    accentTheme: state => accentColorOptions.find(option => option.value === state.accentColor) ?? accentColorOptions[0],
    resolvedColorMode: state => state.colorMode === "system" ? state.systemColorMode : state.colorMode,
    cssVars(): Record<string, string> {
      const theme = this.accentTheme;
      const dark = this.resolvedColorMode === "dark";
      return {
        "--app-accent": theme.color,
        "--app-accent-contrast": theme.contrast,
        "--app-accent-soft": dark ? `color-mix(in srgb, ${theme.color} 24%, #111827)` : theme.soft,
        "--app-accent-border": dark ? `color-mix(in srgb, ${theme.color} 46%, #334155)` : theme.border,
        "--app-accent-hover": dark ? `color-mix(in srgb, ${theme.color} 16%, #1e293b)` : `color-mix(in srgb, ${theme.color} 8%, white)`,
        "--app-accent-selected": dark ? `color-mix(in srgb, ${theme.color} 28%, #172033)` : `color-mix(in srgb, ${theme.color} 18%, white)`,
        "--app-accent-strong": `color-mix(in srgb, ${theme.color} 88%, black)`,
        "--app-accent-ring": `color-mix(in srgb, ${theme.color} ${dark ? 34 : 24}%, transparent)`,
        "--app-accent-tint": `color-mix(in srgb, ${theme.color} ${dark ? 16 : 10}%, transparent)`
      };
    }
  },
  actions: {
    setIconStyle(style: AppIconStyle) {
      if (!iconStyles.includes(style)) return;
      this.iconStyle = style;
      writeStorageItem(storageKeys.iconStyle, style);
    },

    setFileIconStyle(style: FileIconStyle) {
      if (!fileIconStyles.includes(style)) return;
      this.fileIconStyle = style;
      writeStorageItem(storageKeys.fileIconStyle, style);
    },

    setFileIconPalette(palette: FileIconPalette) {
      if (!fileIconPalettes.includes(palette)) return;
      this.fileIconPalette = palette;
      writeStorageItem(storageKeys.fileIconPalette, palette);
    },

    setAccentColor(color: AppAccentColor) {
      if (!accentColors.includes(color)) return;
      this.accentColor = color;
      writeStorageItem(storageKeys.accentColor, color);
    },

    setColorMode(mode: AppColorMode) {
      if (!colorModes.includes(mode)) return;
      this.colorMode = mode;
      writeStorageItem(storageKeys.colorMode, mode);
    },

    setSystemColorMode(mode: ResolvedColorMode) {
      this.systemColorMode = mode;
    }
  }
});
