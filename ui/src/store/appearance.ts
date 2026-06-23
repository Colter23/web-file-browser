import {defineStore} from "pinia";
import type {AppAccentColor, AppColorMode, AppIconStyle, FileIconPalette} from "../class.ts";
import {readStorageItem, writeStorageItem} from "../utils/safe-storage.ts";

const storageKeys = {
  iconStyle: "appearance.iconStyle",
  fileIconPalette: "appearance.fileIconPalette",
  accentColor: "appearance.accentColor",
  colorMode: "appearance.colorMode"
}

export type ResolvedColorMode = "light" | "dark";

export const iconStyleOptions: {value: AppIconStyle; label: string}[] = [
  {value: "lucide", label: "线性"},
  {value: "classic", label: "填充"}
];

export const fileIconPaletteOptions: {value: FileIconPalette; label: string}[] = [
  {value: "category", label: "分类色"},
  {value: "accent", label: "主题色"}
];

export const accentColorOptions: {value: AppAccentColor; label: string; color: string; soft: string; border: string}[] = [
  {value: "blue", label: "蓝色", color: "#2563eb", soft: "#eff6ff", border: "#bfdbfe"},
  {value: "teal", label: "青绿", color: "#0f766e", soft: "#f0fdfa", border: "#99f6e4"},
  {value: "violet", label: "紫色", color: "#7c3aed", soft: "#f5f3ff", border: "#ddd6fe"},
  {value: "rose", label: "玫红", color: "#e11d48", soft: "#fff1f2", border: "#fecdd3"},
  {value: "slate", label: "灰蓝", color: "#475569", soft: "#f8fafc", border: "#cbd5e1"}
];

export const colorModeOptions: {value: AppColorMode; label: string}[] = [
  {value: "system", label: "跟随系统"},
  {value: "light", label: "亮色"},
  {value: "dark", label: "暗色"}
];

const iconStyles = iconStyleOptions.map(option => option.value);
const fileIconPalettes = fileIconPaletteOptions.map(option => option.value);
const accentColors = accentColorOptions.map(option => option.value);
const colorModes = colorModeOptions.map(option => option.value);

const readIconStyle = (): AppIconStyle => {
  const value = readStorageItem(storageKeys.iconStyle);
  return iconStyles.includes(value as AppIconStyle) ? value as AppIconStyle : "lucide";
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
        "--app-accent-soft": dark ? `color-mix(in srgb, ${theme.color} 24%, #111827)` : theme.soft,
        "--app-accent-border": dark ? `color-mix(in srgb, ${theme.color} 46%, #334155)` : theme.border,
        "--app-accent-hover": dark ? `color-mix(in srgb, ${theme.color} 16%, #1e293b)` : `color-mix(in srgb, ${theme.color} 8%, white)`,
        "--app-accent-selected": dark ? `color-mix(in srgb, ${theme.color} 28%, #172033)` : `color-mix(in srgb, ${theme.color} 18%, white)`,
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
