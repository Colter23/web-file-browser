type PanelFocusRestoreOptions = {
  editorVisible: () => boolean;
  focusExplorer: () => Promise<void>;
}

export const usePanelFocusRestore = ({editorVisible, focusExplorer}: PanelFocusRestoreOptions) => {
  const focusExplorerAfterPanelClose = async (closed: boolean) => {
    if (!closed || editorVisible()) return;
    await focusExplorer();
  }

  const closeAndFocusExplorer = async (isVisible: () => boolean, close: () => void) => {
    const visibleBeforeClose = isVisible();
    close();
    await focusExplorerAfterPanelClose(visibleBeforeClose && !isVisible());
  }

  return {
    closeAndFocusExplorer
  };
}
