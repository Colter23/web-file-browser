import {onBeforeUnmount, onMounted} from "vue";

type MainViewLifecycleOptions = {
  initialize: () => Promise<void>;
  stopScrollPersistence: () => void;
  stopShellNoticeTimer: () => void;
  stopTaskPolling: () => void;
  handleWindowKeyDown: (event: KeyboardEvent) => void;
  handleHistoryMouseDown: (event: MouseEvent) => void;
  handleHistoryMouseUp: (event: MouseEvent) => void;
  handleHistoryAuxClick: (event: MouseEvent) => void;
  closeTabContextMenu: () => void;
  handleSidebarResizeMove: (event: PointerEvent) => void;
  finishSidebarResize: () => void;
  handlePreviewPaneResizeMove: (event: PointerEvent) => void;
  finishPreviewPaneResize: () => void;
  handleWindowResize: () => void;
}

export const useMainViewLifecycle = ({
  initialize,
  stopScrollPersistence,
  stopShellNoticeTimer,
  stopTaskPolling,
  handleWindowKeyDown,
  handleHistoryMouseDown,
  handleHistoryMouseUp,
  handleHistoryAuxClick,
  closeTabContextMenu,
  handleSidebarResizeMove,
  finishSidebarResize,
  handlePreviewPaneResizeMove,
  finishPreviewPaneResize,
  handleWindowResize
}: MainViewLifecycleOptions) => {
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
    listen("keydown", handleWindowKeyDown);
    listen("mousedown", handleHistoryMouseDown);
    listen("mouseup", handleHistoryMouseUp);
    listen("auxclick", handleHistoryAuxClick);
    listen("click", closeTabContextMenu);
    listen("scroll", closeTabContextMenu, true);
    listen("pointermove", handleSidebarResizeMove);
    listen("pointermove", handlePreviewPaneResizeMove);
    listen("pointerup", finishSidebarResize);
    listen("pointerup", finishPreviewPaneResize);
    listen("pointercancel", finishSidebarResize);
    listen("pointercancel", finishPreviewPaneResize);
    listen("resize", handleWindowResize);
  });

  onBeforeUnmount(() => {
    disposed = true;
    for (const remove of removeListeners.splice(0)) remove();
    stopScrollPersistence();
    stopShellNoticeTimer();
    stopTaskPolling();
  });
}
