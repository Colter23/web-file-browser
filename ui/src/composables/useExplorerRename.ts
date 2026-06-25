import {nextTick, ref} from "vue";
import type {ComponentPublicInstance} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type RenamePayload = {
  entry: ExplorerEntry;
  name: string;
}

type MaybePromise<T = unknown> = T | Promise<T>;

type ExplorerRenameOptions = {
  entryByPath: (path: string) => ExplorerEntry | undefined;
  ensureEntrySelected: (entry: ExplorerEntry) => void;
  closeContextMenu: () => void;
  focusViewport: () => void;
  submitRename: (payload: RenamePayload) => MaybePromise<boolean | void>;
}

type RenameEditElement = HTMLInputElement | HTMLTextAreaElement;

const selectRenameText = (input: RenameEditElement, entry: ExplorerEntry) => {
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
  if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) return element;
  if (element && "$el" in element && (element.$el instanceof HTMLInputElement || element.$el instanceof HTMLTextAreaElement)) {
    return element.$el;
  }
  return null;
}

export const useExplorerRename = ({entryByPath, ensureEntrySelected, closeContextMenu, focusViewport, submitRename}: ExplorerRenameOptions) => {
  const renamingPath = ref("");
  const renameDraft = ref("");
  const renameSubmitting = ref(false);
  const renameInputRefs = new Map<string, RenameEditElement>();

  const focusRenameInput = async (entry: ExplorerEntry, selectText = true) => {
    await nextTick();
    const input = renameInputRefs.get(entry.path);
    input?.focus({preventScroll: true});
    if (input && selectText) selectRenameText(input, entry);
  }

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
    void focusRenameInput(entry);
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
      const accepted = await submitRename({entry, name: nextName});
      if (accepted === false) {
        renameSubmitting.value = false;
        await focusRenameInput(entry);
        return;
      }
      renamingPath.value = "";
      renameDraft.value = "";
      void nextTick(focusViewport);
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
