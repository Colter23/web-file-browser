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

const normalizedExtensionSet = (extensions: readonly string[]) => new Set(extensions.map(extension => extension.toLowerCase()));

const imageExtensionSet = normalizedExtensionSet(imageFileExtensions);
const textLikeExtensionSet = normalizedExtensionSet(textLikeFileExtensions);
const audioExtensionSet = normalizedExtensionSet(audioFileExtensions);
const videoExtensionSet = normalizedExtensionSet(videoFileExtensions);
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
