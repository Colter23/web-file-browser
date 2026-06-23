import type {ExplorerEntry} from "../components/explorer/types.ts";

export const INTERNAL_ENTRY_DRAG_MIME = "application/x-web-file-browser-entries";

let activeInternalEntryDragEntries: ExplorerEntry[] = [];

const normalizeEntry = (entry: ExplorerEntry): ExplorerEntry => ({
  type: entry.type,
  name: entry.name,
  path: entry.path,
  modified: entry.modified ?? "",
  size: entry.size,
  extension: entry.extension,
  file: entry.file
});

const isExplorerEntry = (value: unknown): value is ExplorerEntry => {
  if (!value || typeof value !== "object") return false;
  const entry = value as Partial<ExplorerEntry>;
  return (entry.type === "folder" || entry.type === "file")
      && typeof entry.name === "string"
      && typeof entry.path === "string";
}

export const hasInternalEntryDragData = (dataTransfer: DataTransfer | null) => {
  return activeInternalEntryDragEntries.length > 0 || Array.from(dataTransfer?.types ?? []).includes(INTERNAL_ENTRY_DRAG_MIME);
}

export const getActiveInternalEntryDragEntries = () => activeInternalEntryDragEntries;

export const setActiveInternalEntryDragEntries = (entries: ExplorerEntry[]) => {
  activeInternalEntryDragEntries = entries.map(normalizeEntry);
}

export const clearActiveInternalEntryDragEntries = () => {
  activeInternalEntryDragEntries = [];
}

export const writeInternalEntryDragData = (dataTransfer: DataTransfer | null, entries: ExplorerEntry[]) => {
  if (!dataTransfer) return;
  const normalizedEntries = entries.map(normalizeEntry);
  setActiveInternalEntryDragEntries(normalizedEntries);
  dataTransfer.setData(INTERNAL_ENTRY_DRAG_MIME, JSON.stringify(normalizedEntries));
  dataTransfer.setData("text/plain", normalizedEntries.map(entry => entry.path).join("\n"));
}

export const readInternalEntryDragData = (dataTransfer: DataTransfer | null): ExplorerEntry[] => {
  if (!hasInternalEntryDragData(dataTransfer)) return [];
  try {
    const payload = dataTransfer?.getData(INTERNAL_ENTRY_DRAG_MIME) ?? "";
    const parsed = JSON.parse(payload) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isExplorerEntry).map(entry => normalizeEntry(entry));
  } catch {
    return getActiveInternalEntryDragEntries();
  }
}
