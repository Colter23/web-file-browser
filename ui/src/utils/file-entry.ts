import type {ArchiveFormat, FileInfo} from "../class.ts";
import {parentPath} from "./file-path.ts";

export type FileEntryKind = "folder" | "file";

export type FileEntryLike = {
  type: FileEntryKind;
  name: string;
  path?: string;
  modified?: string;
  size?: number;
  extension?: string;
  file?: FileInfo;
}

export type EntryPreviewKind = "image" | "text" | "audio" | "video" | "unknown";

export type FileEntryIconKind =
  | "folder"
  | "folder-open"
  | "image"
  | "text"
  | "code"
  | "config"
  | "archive"
  | "audio"
  | "video"
  | "pdf"
  | "spreadsheet"
  | "document"
  | "presentation"
  | "executable"
  | "shortcut"
  | "database"
  | "font"
  | "package"
  | "markup"
  | "unknown"
  | "file";

export type FileEntryMetaRow = {
  label: string;
  value: string;
}

export type FileEntryMetaOptions = {
  typeText?: string;
  sizeText?: string;
  includeLocation?: boolean;
  includePath?: boolean;
  pathBeforeStats?: boolean;
  modifiedLabel?: string;
}

export const imageFileExtensions = ["apng", "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "webp"];
export const textLikeFileExtensions = ["txt", "log", "md", "json", "yaml", "yml", "toml", "xml", "csv"];
export const audioFileExtensions = ["mp3", "wav", "ogg", "flac", "m4a", "aac"];
export const videoFileExtensions = ["mp4", "webm", "mov", "mkv", "avi"];
export const archiveFileExtensions = ["zip", "rar", "7z", "tar", "gz", "tgz"];
export const codeFileExtensions = ["c", "cc", "cpp", "cs", "css", "go", "h", "hpp", "html", "java", "js", "jsx", "kt", "kts", "php", "py", "rs", "scss", "sh", "sql", "swift", "ts", "tsx", "vue"];
export const configFileExtensions = ["conf", "config", "env", "ini", "properties", "toml", "yaml", "yml"];
export const spreadsheetFileExtensions = ["csv", "ods", "xls", "xlsm", "xlsx"];
export const documentFileExtensions = ["doc", "docx", "odt", "rtf"];
export const presentationFileExtensions = ["odp", "ppt", "pptx"];
export const executableFileExtensions = ["app", "bat", "cmd", "com", "exe", "msi", "ps1"];
export const shortcutFileExtensions = ["lnk", "url", "webloc"];
export const databaseFileExtensions = ["db", "mdb", "sqlite", "sqlite3"];
export const fontFileExtensions = ["eot", "otf", "ttf", "woff", "woff2"];
export const packageFileExtensions = ["apk", "deb", "dmg", "jar", "rpm"];

const normalizedExtensionSet = (extensions: readonly string[]) => new Set(extensions.map(extension => extension.toLowerCase()));

const imageExtensionSet = normalizedExtensionSet(imageFileExtensions);
const textLikeExtensionSet = normalizedExtensionSet(textLikeFileExtensions);
const audioExtensionSet = normalizedExtensionSet(audioFileExtensions);
const videoExtensionSet = normalizedExtensionSet(videoFileExtensions);
const archiveExtensionSet = normalizedExtensionSet(archiveFileExtensions);
const codeExtensionSet = normalizedExtensionSet(codeFileExtensions);
const configExtensionSet = normalizedExtensionSet(configFileExtensions);
const spreadsheetExtensionSet = normalizedExtensionSet(spreadsheetFileExtensions);
const documentExtensionSet = normalizedExtensionSet(documentFileExtensions);
const presentationExtensionSet = normalizedExtensionSet(presentationFileExtensions);
const executableExtensionSet = normalizedExtensionSet(executableFileExtensions);
const shortcutExtensionSet = normalizedExtensionSet(shortcutFileExtensions);
const databaseExtensionSet = normalizedExtensionSet(databaseFileExtensions);
const fontExtensionSet = normalizedExtensionSet(fontFileExtensions);
const packageExtensionSet = normalizedExtensionSet(packageFileExtensions);

const hasExtension = (extensions: readonly string[], extension: string) => extensions.some(item => item.toLowerCase() === extension);

const extensionFromName = (name?: string) => {
  if (!name || name.startsWith(".") && !name.slice(1).includes(".")) return "";
  const dotIndex = name.lastIndexOf(".");
  return dotIndex > 0 ? name.slice(dotIndex + 1).toLowerCase() : "";
}

export const normalizeEntryExtension = (entry: FileEntryLike | null | undefined) => entry?.extension?.toLowerCase() || extensionFromName(entry?.name);

export const isImageEntry = (entry: FileEntryLike | null | undefined) => {
  if (!entry || entry.type !== "file") return false;
  return imageExtensionSet.has(normalizeEntryExtension(entry));
}

export const isTextLikeEntry = (entry: FileEntryLike | null | undefined, editableExtensions: readonly string[] = []) => {
  if (!entry || entry.type !== "file") return false;
  const extension = normalizeEntryExtension(entry);
  return textLikeExtensionSet.has(extension) || hasExtension(editableExtensions, extension);
}

export const isAudioEntry = (entry: FileEntryLike | null | undefined) => {
  if (!entry || entry.type !== "file") return false;
  return audioExtensionSet.has(normalizeEntryExtension(entry));
}

export const isVideoEntry = (entry: FileEntryLike | null | undefined) => {
  if (!entry || entry.type !== "file") return false;
  return videoExtensionSet.has(normalizeEntryExtension(entry));
}

export const entryPreviewKind = (entry: FileEntryLike | null | undefined, editableExtensions: readonly string[] = []): EntryPreviewKind => {
  if (!entry || entry.type !== "file") return "unknown";
  if (isImageEntry(entry)) return "image";
  if (isAudioEntry(entry)) return "audio";
  if (isVideoEntry(entry)) return "video";
  if (isTextLikeEntry(entry, editableExtensions)) return "text";
  return "unknown";
}

export const entryPreviewTypeText = (kind: EntryPreviewKind) => ({
  image: "图片",
  text: "文本",
  audio: "音频",
  video: "视频",
  unknown: "文件"
}[kind]);

export const isEditableEntry = (entry: FileEntryLike | null | undefined, editableExtensions: readonly string[]) => {
  if (!entry || entry.type !== "file") return false;
  return hasExtension(editableExtensions, normalizeEntryExtension(entry));
}

export const entryFileInfo = (entry: FileEntryLike): FileInfo => entry.file ?? {
  path: entry.path ?? "",
  name: entry.name,
  size: entry.size ?? 0,
  extension: entry.extension ?? "",
  modified: entry.modified ?? ""
};

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

export const archiveStem = (name: string) => {
  const lower = name.toLowerCase();
  if (lower.endsWith(".tar.gz")) return name.slice(0, -7);
  if (lower.endsWith(".tgz")) return name.slice(0, -4);
  if (lower.endsWith(".zip")) return name.slice(0, -4);
  return name;
}

export const archiveFormatExtension = (format: ArchiveFormat) => format === "tarGz" ? ".tar.gz" : ".zip";

export const fileEntryIconKind = (entry: FileEntryLike, editableExtensions: readonly string[] = []): FileEntryIconKind => {
  if (entry.type === "folder") return "folder";
  if (isArchiveEntry(entry)) return "archive";
  const extension = normalizeEntryExtension(entry);
  const name = entry.name.toLowerCase();
  if (imageExtensionSet.has(extension)) return "image";
  if (audioExtensionSet.has(extension)) return "audio";
  if (videoExtensionSet.has(extension)) return "video";
  if (extension === "pdf") return "pdf";
  if (spreadsheetExtensionSet.has(extension)) return "spreadsheet";
  if (presentationExtensionSet.has(extension)) return "presentation";
  if (documentExtensionSet.has(extension)) return "document";
  if (shortcutExtensionSet.has(extension)) return "shortcut";
  if (executableExtensionSet.has(extension)) return "executable";
  if (databaseExtensionSet.has(extension)) return "database";
  if (fontExtensionSet.has(extension)) return "font";
  if (packageExtensionSet.has(extension)) return "package";
  if (configExtensionSet.has(extension) || name.startsWith(".") && ["env", "gitignore", "npmrc", "yarnrc"].some(item => name.endsWith(item))) return "config";
  if (extension === "json" || extension === "xml") return "markup";
  if (codeExtensionSet.has(extension)) return "code";
  if (isTextLikeEntry(entry, editableExtensions)) return "text";
  return "file";
}

export const fileEntryIconName = (entry: FileEntryLike, editableExtensions: readonly string[] = []) => `file.${fileEntryIconKind(entry, editableExtensions)}`;

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

export const entryMetaRows = (entry: FileEntryLike, options: FileEntryMetaOptions = {}): FileEntryMetaRow[] => {
  const rows: FileEntryMetaRow[] = [
    {label: "类型", value: options.typeText ?? entryTypeText(entry)}
  ];
  if (entry.path && options.includeLocation) rows.push({label: "位置", value: parentPath(entry.path)});
  if (entry.path && options.includePath && options.pathBeforeStats) rows.push({label: "路径", value: entry.path});
  rows.push(
    {label: "大小", value: options.sizeText ?? (entry.type === "file" ? formatEntrySize(entry.size) : "-")},
    {label: options.modifiedLabel ?? "修改", value: formatEntryDate(entry.modified)}
  );
  if (entry.path && options.includePath && !options.pathBeforeStats) rows.push({label: "路径", value: entry.path});
  return rows;
}

export const entryTypeText = (entry: FileEntryLike) => {
  if (entry.type === "folder") return "文件夹";
  return entry.extension ? `${entry.extension.toUpperCase()} 文件` : "文件";
}
