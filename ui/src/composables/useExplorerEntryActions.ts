import type {ComputedRef} from "vue";
import type {FileInfo} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {entryFileInfo, isAudioEntry, isEditableEntry, isImageEntry, isPdfEntry, isVideoEntry} from "../utils/file-entry.ts";

type ImageViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type VideoViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type AudioPlayerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type PdfViewerPayload = {
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
  audioEntries: ComputedRef<ExplorerEntry[]>;
  videoEntries: ComputedRef<ExplorerEntry[]>;
  pdfEntries: ComputedRef<ExplorerEntry[]>;
  isRenaming: (entry: ExplorerEntry) => boolean;
  requestEditorLeave: () => Promise<boolean>;
  openEditor: (file: FileInfo) => void;
  loadFolder: (path: string) => Promise<unknown>;
  previewEntry: (entry: ExplorerEntry) => void;
  openImageViewer: (payload: ImageViewerPayload) => void;
  openAudioPlayer: (payload: AudioPlayerPayload) => void;
  openVideoViewer: (payload: VideoViewerPayload) => void;
  openPdfViewer: (payload: PdfViewerPayload) => void;
  openNewTab: (entry: ExplorerEntry) => void;
  copyPath: (payload: CopyPathPayload) => void;
  closeContextMenu: () => void;
}

export const useExplorerEntryActions = ({
  currentPath,
  editableExtensions,
  selectedEntries,
  imageEntries,
  audioEntries,
  videoEntries,
  pdfEntries,
  isRenaming,
  requestEditorLeave,
  openEditor,
  loadFolder,
  previewEntry,
  openImageViewer,
  openAudioPlayer,
  openVideoViewer,
  openPdfViewer,
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
    if (isAudioEntry(entry)) {
      openAudioPlayer({entry, entries: audioEntries.value});
      return;
    }
    if (isVideoEntry(entry)) {
      openVideoViewer({entry, entries: videoEntries.value});
      return;
    }
    if (isPdfEntry(entry)) {
      openPdfViewer({entry, entries: pdfEntries.value});
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
