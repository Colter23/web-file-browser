export type FileEntryKind = "folder" | "file";

export type FileEntryLike = {
  type: FileEntryKind;
  name: string;
  modified?: string;
  size?: number;
  extension?: string;
}

export const imageFileExtensions = ["apng", "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "webp"];
export const textLikeFileExtensions = ["txt", "log", "md", "json", "yaml", "yml", "toml", "xml", "csv"];
export const archiveFileExtensions = ["zip", "rar", "7z", "tar", "gz", "tgz"];

const normalizedExtensionSet = (extensions: readonly string[]) => new Set(extensions.map(extension => extension.toLowerCase()));

const imageExtensionSet = normalizedExtensionSet(imageFileExtensions);
const textLikeExtensionSet = normalizedExtensionSet(textLikeFileExtensions);
const archiveExtensionSet = normalizedExtensionSet(archiveFileExtensions);

const hasExtension = (extensions: readonly string[], extension: string) => extensions.some(item => item.toLowerCase() === extension);

export const normalizeEntryExtension = (entry: FileEntryLike | null | undefined) => entry?.extension?.toLowerCase() ?? "";

export const isImageEntry = (entry: FileEntryLike | null | undefined) => {
  if (!entry || entry.type !== "file") return false;
  return imageExtensionSet.has(normalizeEntryExtension(entry));
}

export const isTextLikeEntry = (entry: FileEntryLike | null | undefined, editableExtensions: readonly string[] = []) => {
  if (!entry || entry.type !== "file") return false;
  const extension = normalizeEntryExtension(entry);
  return textLikeExtensionSet.has(extension) || hasExtension(editableExtensions, extension);
}

export const isEditableEntry = (entry: FileEntryLike | null | undefined, editableExtensions: readonly string[]) => {
  if (!entry || entry.type !== "file") return false;
  return hasExtension(editableExtensions, normalizeEntryExtension(entry));
}

export const isArchiveEntry = (entry: FileEntryLike | null | undefined) => {
  if (!entry || entry.type !== "file") return false;
  const name = entry.name.toLowerCase();
  return archiveExtensionSet.has(normalizeEntryExtension(entry)) || name.endsWith(".tar.gz");
}

export const isExtractableArchiveEntry = (entry: FileEntryLike | null | undefined) => {
  if (!entry || entry.type !== "file") return false;
  const name = entry.name.toLowerCase();
  return name.endsWith(".zip") || name.endsWith(".tar.gz") || name.endsWith(".tgz");
}

export const fileEntryIcon = (entry: FileEntryLike, editableExtensions: readonly string[] = []) => {
  if (entry.type === "folder") return "icon-folder-fill";
  if (isArchiveEntry(entry)) return "icon-file-zip-fill";
  if (isImageEntry(entry)) return "icon-file-image-fill";
  if (isTextLikeEntry(entry, editableExtensions)) return "icon-file-common-filling";
  return "icon-file-fill";
}

export const formatEntryDate = (srcDate?: string) => {
  if (!srcDate) return "-";
  const date = new Date(srcDate);
  if (Number.isNaN(date.getTime())) return srcDate;
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit"
  }).format(date);
}

export const formatEntrySize = (size?: number, missingText = "-") => {
  if (!Number.isFinite(size)) return missingText;
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = size ?? 0;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

export const entryTypeText = (entry: FileEntryLike) => {
  if (entry.type === "folder") return "文件夹";
  return entry.extension ? `${entry.extension.toUpperCase()} 文件` : "文件";
}
