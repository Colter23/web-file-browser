import {defineStore} from "pinia";
import type {AppAccentColor, AppColorMode, AppIconStyle, FileIconPalette, FileIconStyle} from "../class.ts";
import type {MessageKey} from "../i18n";
import {readStorageItem, writeStorageItem} from "../utils/safe-storage.ts";

const storageKeys = {
  iconStyle: "appearance.iconStyle",
  fileIconStyle: "appearance.fileIconStyle",
  fileIconPalette: "appearance.fileIconPalette",
  accentColor: "appearance.accentColor",
  colorMode: "appearance.colorMode"
}

export type ResolvedColorMode = "light" | "dark";

export const iconStyleOptions: {value: AppIconStyle; labelKey: MessageKey}[] = [
  {value: "lucide", labelKey: "appearance.iconLucide"},
  {value: "fluent", labelKey: "appearance.iconFluent"},
  {value: "solar", labelKey: "appearance.iconSolar"}
];

export const fileIconStyleOptions: {value: FileIconStyle; labelKey: MessageKey}[] = [
  {value: "inherit", labelKey: "appearance.fileIconInherit"},
  {value: "vscode-icons", labelKey: "appearance.fileIconVscode"},
  {value: "catppuccin", labelKey: "appearance.fileIconCatppuccin"}
];

export const fileIconPaletteOptions: {value: FileIconPalette; labelKey: MessageKey}[] = [
  {value: "category", labelKey: "appearance.paletteCategory"},
  {value: "accent", labelKey: "appearance.paletteAccent"}
];

export const accentColorOptions: {
  value: AppAccentColor;
  labelKey: MessageKey;
  color: string;
  contrast: string;
  soft: string;
  border: string;
}[] = [
  {value: "blue", labelKey: "appearance.accentBlue", color: "#2563eb", contrast: "#ffffff", soft: "#eff6ff", border: "#bfdbfe"},
  {value: "sky", labelKey: "appearance.accentSky", color: "#0369a1", contrast: "#ffffff", soft: "#f0f9ff", border: "#bae6fd"},
  {value: "teal", labelKey: "appearance.accentTeal", color: "#0f766e", contrast: "#ffffff", soft: "#f0fdfa", border: "#99f6e4"},
  {value: "emerald", labelKey: "appearance.accentEmerald", color: "#15803d", contrast: "#ffffff", soft: "#f0fdf4", border: "#bbf7d0"},
  {value: "slate", labelKey: "appearance.accentSlate", color: "#475569", contrast: "#ffffff", soft: "#f8fafc", border: "#cbd5e1"},
  {value: "violet", labelKey: "appearance.accentViolet", color: "#7c3aed", contrast: "#ffffff", soft: "#f5f3ff", border: "#ddd6fe"},
  {value: "pink", labelKey: "appearance.accentPink", color: "#db2777", contrast: "#ffffff", soft: "#fdf2f8", border: "#fbcfe8"},
  {value: "rose", labelKey: "appearance.accentRose", color: "#be123c", contrast: "#ffffff", soft: "#fff1f2", border: "#fecdd3"},
  {value: "orange", labelKey: "appearance.accentOrange", color: "#c2410c", contrast: "#ffffff", soft: "#fff7ed", border: "#fed7aa"},
  {value: "amber", labelKey: "appearance.accentAmber", color: "#a16207", contrast: "#ffffff", soft: "#fffbeb", border: "#fde68a"}
];

export const colorModeOptions: {value: AppColorMode; labelKey: MessageKey}[] = [
  {value: "system", labelKey: "appearance.system"},
  {value: "light", labelKey: "appearance.light"},
  {value: "dark", labelKey: "appearance.dark"}
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
