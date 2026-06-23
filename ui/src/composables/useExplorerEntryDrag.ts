import {computed, reactive, ref} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {parentPath} from "../utils/file-path.ts";

type DropAction = "copy" | "move";

type ExplorerEntryDragOptions = {
  selectedPaths: Ref<string[]>;
  selectedEntries: ComputedRef<ExplorerEntry[]>;
  itemRefs: Map<string, HTMLElement>;
  viewportRef: Ref<HTMLElement | null>;
  currentFolder: () => string;
  isSelected: (path: string) => boolean;
  isRenaming: (entry: ExplorerEntry) => boolean;
  setSelection: (paths: string[], focusPath?: string, keepAnchor?: boolean) => void;
  focusViewport: () => void;
  closeContextMenu: () => void;
  dropEntries: (entries: ExplorerEntry[], target: ExplorerEntry, action: DropAction) => void;
  dropToCurrentFolder: (entries: ExplorerEntry[], action: DropAction) => void;
}

export const useExplorerEntryDrag = ({
  selectedPaths,
  selectedEntries,
  itemRefs,
  viewportRef,
  currentFolder,
  isSelected,
  isRenaming,
  setSelection,
  focusViewport,
  closeContextMenu,
  dropEntries,
  dropToCurrentFolder
}: ExplorerEntryDragOptions) => {
  const draggingEntries = ref<ExplorerEntry[]>([]);
  const dragState = reactive({active: false, overPath: "", overCurrentFolder: false, copy: false});

  const isCopyAction = (event: DragEvent) => Boolean(event.ctrlKey || event.metaKey);

  const isDragged = (entry: ExplorerEntry) => draggingEntries.value.some(item => item.path === entry.path);

  const isDropTarget = (entry: ExplorerEntry) => dragState.active && dragState.overPath === entry.path;

  const canDropOnEntry = (entry: ExplorerEntry) => {
    if (entry.type !== "folder") return false;
    if (!draggingEntries.value.length) return false;
    return !draggingEntries.value.some(item => item.path === entry.path || entry.path.startsWith(`${item.path}/`));
  }

  const canDropOnCurrentFolder = (action: DropAction) => {
    if (!draggingEntries.value.length) return false;
    if (action === "copy") return true;
    const folder = currentFolder();
    return draggingEntries.value.some(entry => parentPath(entry.path) !== folder);
  }

  const dragHintText = computed(() => {
    if (!dragState.active || !draggingEntries.value.length || !dragState.overPath && !dragState.overCurrentFolder) return "";
    const actionText = dragState.copy ? "复制" : "移动";
    return `${actionText} ${draggingEntries.value.length} 项`;
  });

  const selectedEntriesForDrag = (entry: ExplorerEntry) => {
    if (selectedPaths.value.includes(entry.path)) return selectedEntries.value;
    return [entry];
  }

  const beginEntryDrag = (event: DragEvent, entry: ExplorerEntry) => {
    if (isRenaming(entry)) return;
    focusViewport();
    const entriesToDrag = selectedEntriesForDrag(entry);
    if (!entriesToDrag.length) return;
    if (!isSelected(entry.path)) setSelection([entry.path], entry.path);
    draggingEntries.value = entriesToDrag;
    dragState.active = true;
    dragState.overPath = "";
    dragState.copy = isCopyAction(event);
    closeContextMenu();
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "copyMove";
      event.dataTransfer.dropEffect = dragState.copy ? "copy" : "move";
      event.dataTransfer.setData("text/plain", entriesToDrag.map(item => item.path).join("\n"));
    }
  }

  const resetEntryDrag = () => {
    draggingEntries.value = [];
    dragState.active = false;
    dragState.overPath = "";
    dragState.overCurrentFolder = false;
    dragState.copy = false;
  }

  const dragOverEntry = (event: DragEvent, entry: ExplorerEntry) => {
    if (!dragState.active || !canDropOnEntry(entry)) return;
    event.preventDefault();
    event.stopPropagation();
    dragState.overPath = entry.path;
    dragState.overCurrentFolder = false;
    dragState.copy = isCopyAction(event);
    if (event.dataTransfer) event.dataTransfer.dropEffect = dragState.copy ? "copy" : "move";
  }

  const dragLeaveEntry = (event: DragEvent, entry: ExplorerEntry) => {
    if (!dragState.active || dragState.overPath !== entry.path) return;
    const related = event.relatedTarget;
    const element = itemRefs.get(entry.path);
    if (related instanceof Node && element?.contains(related)) return;
    dragState.overPath = "";
  }

  const dropOnEntry = (event: DragEvent, entry: ExplorerEntry) => {
    if (!dragState.active || !canDropOnEntry(entry)) return;
    event.preventDefault();
    event.stopPropagation();
    const entriesToDrop = draggingEntries.value;
    const action = isCopyAction(event) ? "copy" : "move";
    resetEntryDrag();
    dropEntries(entriesToDrop, entry, action);
  }

  const isInternalEntryDrag = (event: DragEvent) => {
    const types = Array.from(event.dataTransfer?.types ?? []);
    return dragState.active && types.includes("text/plain");
  }

  const isEntryDragSurface = (target: EventTarget | null) => target instanceof HTMLElement && Boolean(target.closest(".entry-item"));

  const dragOverCurrentFolder = (event: DragEvent) => {
    if (!isInternalEntryDrag(event)) return;
    if (isEntryDragSurface(event.target)) {
      dragState.overCurrentFolder = false;
      return;
    }
    const action = isCopyAction(event) ? "copy" : "move";
    if (!canDropOnCurrentFolder(action)) {
      dragState.overPath = "";
      dragState.overCurrentFolder = false;
      if (event.dataTransfer) event.dataTransfer.dropEffect = "none";
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    dragState.overPath = "";
    dragState.overCurrentFolder = true;
    dragState.copy = action === "copy";
    if (event.dataTransfer) event.dataTransfer.dropEffect = dragState.copy ? "copy" : "move";
  }

  const dragLeaveCurrentFolder = (event: DragEvent) => {
    if (!dragState.overCurrentFolder) return;
    const related = event.relatedTarget;
    if (related instanceof Node && viewportRef.value?.contains(related)) return;
    dragState.overCurrentFolder = false;
  }

  const dropOnCurrentFolder = (event: DragEvent) => {
    if (!isInternalEntryDrag(event) || !dragState.overCurrentFolder) return;
    if (isEntryDragSurface(event.target)) return;
    const action = isCopyAction(event) ? "copy" : "move";
    if (!canDropOnCurrentFolder(action)) {
      resetEntryDrag();
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    const entriesToDrop = draggingEntries.value;
    resetEntryDrag();
    dropToCurrentFolder(entriesToDrop, action);
  }

  return {
    dragState,
    dragHintText,
    isDragged,
    isDropTarget,
    beginEntryDrag,
    resetEntryDrag,
    dragOverEntry,
    dragLeaveEntry,
    dropOnEntry,
    dragOverCurrentFolder,
    dragLeaveCurrentFolder,
    dropOnCurrentFolder
  };
}
