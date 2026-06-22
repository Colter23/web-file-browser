import type {ComputedRef} from "vue";
import type {FileInfo} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {isEditableEntry, isImageEntry} from "../utils/file-entry.ts";

type ImageViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type CopyPathPayload = {
  paths: string[];
}

type ExplorerEntryActionOptions = {
  currentPath: () => string;
  editableExtensions: () => readonly string[];
  selectedEntries: ComputedRef<ExplorerEntry[]>;
  imageEntries: ComputedRef<ExplorerEntry[]>;
  isRenaming: (entry: ExplorerEntry) => boolean;
  requestEditorLeave: () => Promise<boolean>;
  openEditor: (file: FileInfo) => void;
  loadFolder: (path: string) => Promise<unknown>;
  previewEntry: (entry: ExplorerEntry) => void;
  openImageViewer: (payload: ImageViewerPayload) => void;
  openNewTab: (entry: ExplorerEntry) => void;
  copyPath: (payload: CopyPathPayload) => void;
  closeContextMenu: () => void;
}

const entryFileInfo = (entry: ExplorerEntry): FileInfo => entry.file ?? {
  path: entry.path,
  name: entry.name,
  size: entry.size ?? 0,
  extension: entry.extension ?? "",
  modified: entry.modified ?? ""
};

export const useExplorerEntryActions = ({
  currentPath,
  editableExtensions,
  selectedEntries,
  imageEntries,
  isRenaming,
  requestEditorLeave,
  openEditor,
  loadFolder,
  previewEntry,
  openImageViewer,
  openNewTab,
  copyPath,
  closeContextMenu
}: ExplorerEntryActionOptions) => {
  const canEditEntry = (entry: ExplorerEntry | null) => {
    return isEditableEntry(entry, editableExtensions());
  }

  const editEntry = async (entry: ExplorerEntry) => {
    if (!canEditEntry(entry)) return;
    if (!await requestEditorLeave()) return;
    openEditor(entryFileInfo(entry));
  }

  const openEntry = async (entry: ExplorerEntry) => {
    if (isRenaming(entry)) return;
    if (entry.type === "folder") {
      if (!await requestEditorLeave()) return;
      await loadFolder(entry.path);
      return;
    }
    if (isImageEntry(entry)) {
      openImageViewer({entry, entries: imageEntries.value});
      return;
    }
    if (canEditEntry(entry)) {
      await editEntry(entry);
      return;
    }
    previewEntry(entry);
  }

  const openEntryInNewTab = (entry: ExplorerEntry) => {
    if (entry.type !== "folder") return;
    closeContextMenu();
    openNewTab(entry);
  }

  const copySelectedPaths = (fallbackEntry?: ExplorerEntry | null) => {
    const paths = selectedEntries.value.length
        ? selectedEntries.value.map(entry => entry.path)
        : fallbackEntry ? [fallbackEntry.path] : [currentPath()];
    copyPath({paths});
  }

  return {
    canEditEntry,
    editEntry,
    openEntry,
    openEntryInNewTab,
    copySelectedPaths
  };
}
