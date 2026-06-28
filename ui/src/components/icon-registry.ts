import type {AppIconStyle} from "../class.ts";
import type {AppIconDefinition, AppIconPack} from "./icon-packs/types.ts";

const fallbackIconStyle: AppIconStyle = "lucide";

const iconPackLoaders: Record<AppIconStyle, () => Promise<AppIconPack>> = {
  lucide: async () => (await import("./icon-packs/lucide-pack.ts")).lucideIconPack,
  fluent: async () => (await import("./icon-packs/fluent-pack.ts")).fluentIconPack,
  solar: async () => (await import("./icon-packs/solar-pack.ts")).solarIconPack,
  "fluent-color": async () => (await import("./icon-packs/fluent-color-pack.ts")).fluentColorIconPack,
  "vscode-icons": async () => (await import("./icon-packs/vscode-icons-pack.ts")).vscodeIconsPack,
  catppuccin: async () => (await import("./icon-packs/catppuccin-icons-pack.ts")).catppuccinIconsPack
};

const iconPackPromises: Partial<Record<AppIconStyle, Promise<AppIconPack>>> = {};

const loadIconPack = (style: AppIconStyle) => {
  iconPackPromises[style] ??= iconPackLoaders[style]();
  return iconPackPromises[style];
};

export const resolveAppIcon = async (style: AppIconStyle, icon: string): Promise<AppIconDefinition | undefined> => {
  const preferredPack = await loadIconPack(style);
  const preferredIcon = preferredPack.resolve(icon);
  if (preferredIcon) return preferredIcon;

  if (style !== fallbackIconStyle) {
    const fallbackPack = await loadIconPack(fallbackIconStyle);
    const fallbackIcon = fallbackPack.resolve(icon);
    if (fallbackIcon) return fallbackIcon;
  }

  return undefined;
};
