import {computed, nextTick, ref, watch} from "vue";
import type {ComputedRef} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

export type ExplorerSelectionSnapshot = {
  paths: string[];
  focusedPath: string;
  anchorPath: string;
}

export type ExplorerFocusMoveMode = "replaceSelection" | "extendSelection" | "moveFocusOnly";

type ExplorerSelectionOptions = {
  entries: ComputedRef<ExplorerEntry[]>;
  itemRefs: Map<string, HTMLElement>;
  focusViewport: () => void;
  closeContextMenu: () => void;
  currentColumns: () => number;
  currentPageStep: (columns: number) => number;
}

export const useExplorerSelection = ({
  entries,
  itemRefs,
  focusViewport,
  closeContextMenu,
  currentColumns,
  currentPageStep
}: ExplorerSelectionOptions) => {
  const selectedPaths = ref<string[]>([]);
  const focusedPath = ref("");
  const anchorPath = ref("");

  const selectedEntries = computed(() => {
    const selected = new Set(selectedPaths.value);
    return entries.value.filter(entry => selected.has(entry.path));
  });

  const selectedSet = () => new Set(selectedPaths.value);

  const entryByPath = (path: string) => entries.value.find(entry => entry.path === path);

  const existingAnchorOrFocus = (fallbackPath: string) => {
    if (anchorPath.value && entryByPath(anchorPath.value)) return anchorPath.value;
    if (focusedPath.value && entryByPath(focusedPath.value)) return focusedPath.value;
    return fallbackPath;
  }

  const firstSelectedEntry = () => {
    if (!selectedPaths.value.length) return null;
    return entryByPath(selectedPaths.value[0]) ?? null;
  }

  const focusedOrSelectedEntry = () => entryByPath(focusedPath.value) ?? firstSelectedEntry();

  const isSelected = (path: string) => selectedPaths.value.includes(path);

  const indexOfPath = (path: string) => entries.value.findIndex(entry => entry.path === path);

  const applySelectionState = (paths: string[], focusPath: string, nextAnchorPath: string) => {
    selectedPaths.value = Array.from(new Set(paths));
    focusedPath.value = focusPath;
    anchorPath.value = nextAnchorPath;
  }

  const setSelection = (paths: string[], focusPath = paths[paths.length - 1] ?? "", keepAnchor = false) => {
    const fallbackAnchor = focusPath || (paths[0] ?? "");
    applySelectionState(paths, focusPath, keepAnchor ? existingAnchorOrFocus(fallbackAnchor) : focusPath || anchorPath.value);
  }

  const clearSelection = () => {
    applySelectionState([], "", "");
  }

  const clearSelectionKeepingFocus = () => {
    const retainedFocus = focusedPath.value && entryByPath(focusedPath.value)
        ? focusedPath.value
        : selectedPaths.value[selectedPaths.value.length - 1] ?? "";
    applySelectionState([], retainedFocus, retainedFocus);
  }

  const commitSelectionAnchor = () => {
    if (focusedPath.value) anchorPath.value = focusedPath.value;
  }

  const scrollEntryIntoView = async (path: string) => {
    await nextTick();
    itemRefs.get(path)?.scrollIntoView({block: "nearest", inline: "nearest"});
  }

  const syncSelectionWithEntries = () => {
    const visiblePaths = new Set(entries.value.map(entry => entry.path));
    if (!visiblePaths.size) {
      if (selectedPaths.value.length || focusedPath.value || anchorPath.value) clearSelection();
      return;
    }
    const nextSelected = selectedPaths.value.filter(path => visiblePaths.has(path));
    const nextFocus = focusedPath.value && visiblePaths.has(focusedPath.value)
        ? focusedPath.value
        : nextSelected[nextSelected.length - 1] ?? "";
    const nextAnchor = anchorPath.value && visiblePaths.has(anchorPath.value) ? anchorPath.value : nextFocus;
    const selectionChanged = nextSelected.length !== selectedPaths.value.length
        || nextSelected.some((path, index) => path !== selectedPaths.value[index]);
    if (selectionChanged || focusedPath.value !== nextFocus || anchorPath.value !== nextAnchor) {
      applySelectionState(nextSelected, nextFocus, nextAnchor);
    }
  }

  watch(entries, syncSelectionWithEntries);

  const ensureFocusAnchor = () => {
    if (!entries.value.length || focusedPath.value && entryByPath(focusedPath.value)) return;
    const anchor = selectedEntries.value[selectedEntries.value.length - 1] ?? entries.value[0];
    focusedPath.value = anchor?.path ?? "";
    if (!anchorPath.value) anchorPath.value = focusedPath.value;
  }

  const selectRange = (targetPath: string, additive: boolean) => {
    const targetIndex = indexOfPath(targetPath);
    if (targetIndex < 0) return;
    const anchorCandidate = anchorPath.value || focusedPath.value || targetPath;
    const anchorIndex = indexOfPath(anchorCandidate);
    const start = Math.min(anchorIndex < 0 ? targetIndex : anchorIndex, targetIndex);
    const end = Math.max(anchorIndex < 0 ? targetIndex : anchorIndex, targetIndex);
    const range = entries.value.slice(start, end + 1).map(entry => entry.path);
    if (additive) {
      setSelection([...selectedPaths.value, ...range], targetPath, true);
    } else {
      setSelection(range, targetPath, true);
    }
  }

  const selectEntry = (entry: ExplorerEntry, event?: MouseEvent) => {
    closeContextMenu();
    focusViewport();
    const ctrl = Boolean(event?.ctrlKey || event?.metaKey);
    const shift = Boolean(event?.shiftKey);
    if (shift) {
      selectRange(entry.path, ctrl);
      return;
    }
    if (ctrl) {
      const selected = selectedSet();
      if (selected.has(entry.path)) {
        selected.delete(entry.path);
        setSelection(Array.from(selected), entry.path);
      } else {
        setSelection([...selectedPaths.value, entry.path], entry.path);
      }
      return;
    }
    setSelection([entry.path], entry.path);
  }

  const toggleFocusedSelection = () => {
    const entry = focusedOrSelectedEntry();
    if (!entry) return false;
    const selected = selectedSet();
    if (selected.has(entry.path)) {
      selected.delete(entry.path);
    } else {
      selected.add(entry.path);
    }
    setSelection(Array.from(selected), entry.path);
    return true;
  }

  const ensureEntrySelected = (entry: ExplorerEntry) => {
    if (isSelected(entry.path)) {
      applySelectionState(selectedPaths.value, entry.path, existingAnchorOrFocus(entry.path));
      return;
    }
    setSelection([entry.path], entry.path);
  }

  const focusEntryOnly = (entry: ExplorerEntry) => {
    applySelectionState(selectedPaths.value, entry.path, existingAnchorOrFocus(entry.path));
  }

  const selectAllEntries = () => {
    if (!entries.value.length) return false;
    setSelection(entries.value.map(entry => entry.path), focusedPath.value || entries.value[0]?.path || "");
    return true;
  }

  const clearCurrentSelection = () => {
    if (!selectedPaths.value.length) return false;
    clearSelectionKeepingFocus();
    return true;
  }

  const invertCurrentSelection = () => {
    if (!entries.value.length) return false;
    const selected = selectedSet();
    const inverted = entries.value.filter(entry => !selected.has(entry.path)).map(entry => entry.path);
    setSelection(inverted, inverted[inverted.length - 1] ?? focusedPath.value);
    return true;
  }

  const moveFocus = (key: string, mode: ExplorerFocusMoveMode = "replaceSelection") => {
    if (!entries.value.length) return;
    const current = focusedPath.value ? indexOfPath(focusedPath.value) : -1;
    const columns = currentColumns();
    const pageStep = currentPageStep(columns);
    let nextIndex = current < 0 ? 0 : current;
    if (current >= 0) {
      if (key === "ArrowDown") nextIndex = Math.min(entries.value.length - 1, current + columns);
      if (key === "ArrowUp") nextIndex = Math.max(0, current - columns);
      if (key === "ArrowRight") nextIndex = Math.min(entries.value.length - 1, current + 1);
      if (key === "ArrowLeft") nextIndex = Math.max(0, current - 1);
      if (key === "PageDown") nextIndex = Math.min(entries.value.length - 1, current + pageStep);
      if (key === "PageUp") nextIndex = Math.max(0, current - pageStep);
    }
    if (key === "Home") nextIndex = 0;
    if (key === "End") nextIndex = entries.value.length - 1;
    const entry = entries.value[nextIndex];
    if (!entry) return;
    if (mode === "moveFocusOnly") {
      focusEntryOnly(entry);
    } else if (mode === "extendSelection") {
      selectRange(entry.path, false);
    } else {
      setSelection([entry.path], entry.path);
    }
    void scrollEntryIntoView(entry.path);
  }

  const focusEntryByTypeahead = (entry: ExplorerEntry) => {
    setSelection([entry.path], entry.path);
    void scrollEntryIntoView(entry.path);
  }

  const selectPath = async (path: string, additive = false) => {
    const entry = entryByPath(path);
    if (!entry) return false;
    setSelection(additive ? [...selectedPaths.value, entry.path] : [entry.path], entry.path);
    await scrollEntryIntoView(entry.path);
    return true;
  }

  const selectPaths = async (paths: string[], scrollToSelection = true) => {
    const existingPaths = paths.filter(path => Boolean(entryByPath(path)));
    if (!existingPaths.length) return false;
    setSelection(existingPaths, existingPaths[existingPaths.length - 1]);
    if (scrollToSelection) await scrollEntryIntoView(existingPaths[existingPaths.length - 1]);
    return true;
  }

  const captureSelectionSnapshot = (): ExplorerSelectionSnapshot => ({
    paths: [...selectedPaths.value],
    focusedPath: focusedPath.value,
    anchorPath: anchorPath.value
  })

  const restoreSelectionSnapshot = async (snapshot: ExplorerSelectionSnapshot) => {
    const existingPaths = snapshot.paths.filter(path => Boolean(entryByPath(path)));
    const nextFocus = snapshot.focusedPath && entryByPath(snapshot.focusedPath)
        ? snapshot.focusedPath
        : existingPaths[existingPaths.length - 1] ?? "";
    const nextAnchor = snapshot.anchorPath && entryByPath(snapshot.anchorPath) ? snapshot.anchorPath : nextFocus;
    if (!existingPaths.length && !nextFocus) return false;
    applySelectionState(existingPaths, nextFocus, nextAnchor);
    if (nextFocus) await scrollEntryIntoView(nextFocus);
    return true;
  }

  return {
    selectedPaths,
    focusedPath,
    anchorPath,
    selectedEntries,
    entryByPath,
    firstSelectedEntry,
    focusedOrSelectedEntry,
    isSelected,
    indexOfPath,
    setSelection,
    clearSelection,
    clearSelectionKeepingFocus,
    commitSelectionAnchor,
    scrollEntryIntoView,
    ensureFocusAnchor,
    selectRange,
    selectEntry,
    toggleFocusedSelection,
    ensureEntrySelected,
    focusEntryOnly,
    selectAllEntries,
    clearCurrentSelection,
    invertCurrentSelection,
    moveFocus,
    focusEntryByTypeahead,
    selectPath,
    selectPaths,
    captureSelectionSnapshot,
    restoreSelectionSnapshot
  };
}
