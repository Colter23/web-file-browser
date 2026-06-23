import type {AppIconStyle} from "../class.ts";
import type {AppIconDefinition, AppIconPack} from "./icon-packs/types.ts";

const fallbackIconStyle: AppIconStyle = "lucide";

const iconPackLoaders: Record<AppIconStyle, () => Promise<AppIconPack>> = {
  lucide: async () => (await import("./icon-packs/lucide-pack.ts")).lucideIconPack,
  classic: async () => (await import("./icon-packs/classic-pack.ts")).classicIconPack
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

  if (style !== "classic") {
    const classicPack = await loadIconPack("classic");
    return classicPack.resolve(icon);
  }

  return undefined;
};
