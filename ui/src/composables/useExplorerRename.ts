import {nextTick, ref} from "vue";
import type {ComponentPublicInstance} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type RenamePayload = {
  entry: ExplorerEntry;
  name: string;
}

type ExplorerRenameOptions = {
  entryByPath: (path: string) => ExplorerEntry | undefined;
  ensureEntrySelected: (entry: ExplorerEntry) => void;
  closeContextMenu: () => void;
  focusViewport: () => void;
  submitRename: (payload: RenamePayload) => void;
}

const selectRenameText = (input: HTMLInputElement, entry: ExplorerEntry) => {
  if (entry.type === "folder") {
    input.select();
    return;
  }
  const suffix = entry.extension ? `.${entry.extension}` : "";
  const end = suffix && entry.name.toLowerCase().endsWith(suffix.toLowerCase())
      ? Math.max(0, entry.name.length - suffix.length)
      : entry.name.length;
  input.setSelectionRange(0, end);
}

const resolveInputElement = (element: Element | ComponentPublicInstance | null) => {
  if (element instanceof HTMLInputElement) return element;
  if (element && "$el" in element && element.$el instanceof HTMLInputElement) return element.$el;
  return null;
}

export const useExplorerRename = ({entryByPath, ensureEntrySelected, closeContextMenu, focusViewport, submitRename}: ExplorerRenameOptions) => {
  const renamingPath = ref("");
  const renameDraft = ref("");
  const renameSubmitting = ref(false);
  const renameInputRefs = new Map<string, HTMLInputElement>();

  const setRenameInputRef = (path: string, element: Element | ComponentPublicInstance | null) => {
    const input = resolveInputElement(element);
    if (input) {
      renameInputRefs.set(path, input);
    } else {
      renameInputRefs.delete(path);
    }
  }

  const startRename = (entry: ExplorerEntry | null) => {
    if (!entry || renameSubmitting.value) return;
    ensureEntrySelected(entry);
    closeContextMenu();
    renamingPath.value = entry.path;
    renameDraft.value = entry.name;
    nextTick(() => {
      const input = renameInputRefs.get(entry.path);
      input?.focus();
      if (input) selectRenameText(input, entry);
    });
  }

  const cancelRename = () => {
    if (renameSubmitting.value) return;
    renamingPath.value = "";
    renameDraft.value = "";
    void nextTick(focusViewport);
  }

  const commitRename = async () => {
    if (!renamingPath.value || renameSubmitting.value) return;
    const entry = entryByPath(renamingPath.value);
    const nextName = renameDraft.value.trim();
    if (!entry || !nextName || nextName === entry.name) {
      cancelRename();
      return;
    }
    renameSubmitting.value = true;
    try {
      submitRename({entry, name: nextName});
      renamingPath.value = "";
      renameDraft.value = "";
    } finally {
      renameSubmitting.value = false;
    }
  }

  const isRenaming = (entry: ExplorerEntry) => renamingPath.value === entry.path;

  const resetRename = () => {
    renamingPath.value = "";
    renameDraft.value = "";
  }

  const clearRenameInputRefs = () => {
    renameInputRefs.clear();
  }

  return {
    renamingPath,
    renameDraft,
    renameSubmitting,
    setRenameInputRef,
    startRename,
    cancelRename,
    commitRename,
    isRenaming,
    resetRename,
    clearRenameInputRefs
  };
}
