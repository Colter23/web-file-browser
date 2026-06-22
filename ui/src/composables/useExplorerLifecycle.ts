import {onBeforeUnmount, onMounted} from "vue";

type ExplorerLifecycleOptions = {
  initialize: () => Promise<void>;
  handleKeyDown: (event: KeyboardEvent) => void;
  closeContextMenu: () => void;
  handleSelectionMove: (event: MouseEvent) => void;
  finishMarqueeSelection: () => void;
  resetSelectionBox: () => void;
  handleDetailsColumnResizeMove: (event: PointerEvent) => void;
  finishDetailsColumnResize: () => void;
  stopMarqueeAutoScroll: () => void;
  resetTypeahead: () => void;
  disconnectThumbnailObserver: () => void;
  clearItemRefs: () => void;
  clearRenameInputRefs: () => void;
}

export const useExplorerLifecycle = ({
  initialize,
  handleKeyDown,
  closeContextMenu,
  handleSelectionMove,
  finishMarqueeSelection,
  resetSelectionBox,
  handleDetailsColumnResizeMove,
  finishDetailsColumnResize,
  stopMarqueeAutoScroll,
  resetTypeahead,
  disconnectThumbnailObserver,
  clearItemRefs,
  clearRenameInputRefs
}: ExplorerLifecycleOptions) => {
  const removeListeners: Array<() => void> = [];
  let disposed = false;

  const listen = <K extends keyof WindowEventMap>(
    type: K,
    listener: (event: WindowEventMap[K]) => void,
    options?: boolean | AddEventListenerOptions
  ) => {
    window.addEventListener(type, listener, options);
    removeListeners.push(() => window.removeEventListener(type, listener, options));
  }

  onMounted(async () => {
    await initialize();
    if (disposed) return;
    listen("click", closeContextMenu);
    listen("keydown", handleKeyDown);
    listen("mousemove", handleSelectionMove);
    listen("mouseup", finishMarqueeSelection);
    listen("blur", resetSelectionBox);
    listen("pointermove", handleDetailsColumnResizeMove);
    listen("pointerup", finishDetailsColumnResize);
    listen("pointercancel", finishDetailsColumnResize);
  });

  onBeforeUnmount(() => {
    disposed = true;
    for (const remove of removeListeners.splice(0)) remove();
    stopMarqueeAutoScroll();
    resetTypeahead();
    disconnectThumbnailObserver();
    clearItemRefs();
    clearRenameInputRefs();
  });
}
