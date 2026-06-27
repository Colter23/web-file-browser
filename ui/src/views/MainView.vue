<script setup lang="ts">
import {computed, defineAsyncComponent, nextTick, ref, watch} from "vue";
import {useRouter} from "vue-router";
import {useFileStore} from "../store";
import {
  cancelTask,
  cleanupTasks,
  cleanupTrash,
  createFavorite,
  deleteFavorite,
  deleteTrashRecords,
  deleteTrashRecord,
  emptyTrash,
  getIndexStatus,
  getFolderData,
  getMappingRoot,
  getMappings,
  getSettings,
  listFavorites,
  listTasks,
  listTrashRecords,
  logout,
  reorderFavorites,
  reorderMappings,
  restoreTrashRecords,
  restoreTrashRecord,
  updateFavorite,
} from "../network/api";
import Explorer from "../components/explorer/Explorer.vue";
import ImageViewer from "../components/viewer/ImageViewer.vue";
import AudioPlayer from "../components/viewer/AudioPlayer.vue";
import VideoViewer from "../components/viewer/VideoViewer.vue";
import PdfViewer from "../components/viewer/PdfViewer.vue";
import PreviewPane from "../components/viewer/PreviewPane.vue";
import TaskPanel from "../components/tasks/TaskPanel.vue";
import TaskStatusPill from "../components/tasks/TaskStatusPill.vue";
import TrashPanel from "../components/trash/TrashPanel.vue";
import TrashConfirmPanel from "../components/trash/TrashConfirmPanel.vue";
import TabStrip from "../components/tabs/TabStrip.vue";
import OperationPanel from "../components/operations/OperationPanel.vue";
import DeleteConfirmPanel from "../components/operations/DeleteConfirmPanel.vue";
import PropertiesPanel from "../components/operations/PropertiesPanel.vue";
import ContentToolbar from "../components/shell/ContentToolbar.vue";
import CommandBar from "../components/shell/CommandBar.vue";
import ShellNotice from "../components/shell/ShellNotice.vue";
import SidebarPanel from "../components/shell/SidebarPanel.vue";
import ShellMoreMenu from "../components/shell/ShellMoreMenu.vue";
import UploadDropOverlay from "../components/shell/UploadDropOverlay.vue";
import type {DirEntryFilter, DirSortKey, DirSortOrder, FavoriteItem, FileTreeData, SearchScope} from "../class.ts";
import type {ExplorerEntry, ExplorerEntryPathDropPayload} from "../components/explorer/types.ts";
import {usePreviewPaneResize} from "../composables/usePreviewPaneResize.ts";
import {useSidebarResize} from "../composables/useSidebarResize.ts";
import {useShellNotice} from "../composables/useShellNotice.ts";
import {useFileOperations} from "../composables/useFileOperations.ts";
import {useExplorerTabs} from "../composables/useExplorerTabs.ts";
import {useExplorerTabContext} from "../composables/useExplorerTabContext.ts";
import {useExplorerPreview} from "../composables/useExplorerPreview.ts";
import {useExplorerNavigation} from "../composables/useExplorerNavigation.ts";
import {useExplorerSearchBox} from "../composables/useExplorerSearchBox.ts";
import {useExplorerViewMode} from "../composables/useExplorerViewMode.ts";
import {shouldIgnoreNavigationShortcut, useExplorerShortcuts} from "../composables/useExplorerShortcuts.ts";
import {useFileTreeLoader} from "../composables/useFileTreeLoader.ts";
import {useMainViewShellActions} from "../composables/useMainViewShellActions.ts";
import {useMainViewLifecycle} from "../composables/useMainViewLifecycle.ts";
import {useMainViewPanelClosers} from "../composables/useMainViewPanelClosers.ts";
import {useMainViewSelectionCommands} from "../composables/useMainViewSelectionCommands.ts";
import {useTaskPanel} from "../composables/useTaskPanel.ts";
import {useTrashPanel} from "../composables/useTrashPanel.ts";
import {useUploadDrop} from "../composables/useUploadDrop.ts";
import {useSearchIndexStatusHint} from "../composables/useSearchIndexStatusHint.ts";
import {useFavorites} from "../composables/useFavorites.ts";
import {entryFileInfo} from "../utils/file-entry.ts";

const EditorPanel = defineAsyncComponent(() => import("../components/editor/EditorPanel.vue"));

type ExplorerExpose = {
  refresh: (path?: string, options?: {forceRefresh?: boolean}) => Promise<boolean>;
  getSelectedEntry: () => ExplorerEntry | null;
  getSelectedEntries: () => ExplorerEntry[];
  startRename: () => void;
  selectPath: (path: string) => Promise<boolean>;
  selectPaths: (paths: string[], scrollToSelection?: boolean) => Promise<boolean>;
  selectPathForRename: (path: string) => Promise<boolean>;
  selectAllEntries: () => boolean;
  setSortKey: (key: DirSortKey) => Promise<void>;
  setSortOrder: (order: DirSortOrder) => Promise<void>;
  search: (query: string, type?: DirEntryFilter, scope?: SearchScope) => Promise<boolean>;
  showRecent: () => Promise<boolean>;
  clearResults: () => Promise<boolean>;
  isResultActive: () => boolean;
  focus: () => void;
  getImageEntries: () => ExplorerEntry[];
  getAudioEntries: () => ExplorerEntry[];
  getVideoEntries: () => ExplorerEntry[];
  getPdfEntries: () => ExplorerEntry[];
  getScrollTop: () => number;
  setScrollTop: (scrollTop: number) => Promise<void>;
}

type ContentToolbarExpose = {
  focusInput: () => void;
}

type FocusablePanelExpose = {
  focus: () => void;
}

const router = useRouter();
const fileStore = useFileStore();
const {
  width: sidebarWidth,
  resizing: sidebarResizing,
  minWidth: sidebarMinWidth,
  maxWidth: sidebarMaxWidth,
  workspaceStyle,
  startResize: startSidebarResize,
  handleResizeMove: handleSidebarResizeMove,
  finishResize: finishSidebarResize,
  resetWidth: resetSidebarWidth,
  handleWindowResize: handleSidebarWindowResize,
  handleResizeKeyDown: handleSidebarResizeKeyDown
} = useSidebarResize();
const {
  width: previewPaneWidth,
  resizing: previewPaneResizing,
  minWidth: previewPaneMinWidth,
  maxWidth: previewPaneMaxWidth,
  areaStyle: browserAreaStyle,
  startResize: startPreviewPaneResize,
  handleResizeMove: handlePreviewPaneResizeMove,
  finishResize: finishPreviewPaneResize,
  resetWidth: resetPreviewPaneWidth,
  handleWindowResize: handlePreviewPaneWindowResize,
  handleResizeKeyDown: handlePreviewPaneResizeKeyDown
} = usePreviewPaneResize();
const {
  notice: shellNotice,
  show: showShellNotice,
  showError: showErrorNotice,
  close: closeShellNotice,
  stopTimer: stopShellNoticeTimer,
  resumeTimer: resumeShellNoticeTimer
} = useShellNotice();
let refreshCurrentHandler = async (_keepSelection = false) => {};
const refreshCurrent = (keepSelection = false) => refreshCurrentHandler(keepSelection);
let refreshFolderInExplorerAndTree = async (_path: string) => {};

const {
  visible: taskPanelVisible,
  summaryVisible: taskSummaryVisible,
  loading: tasksLoading,
  cleanupLoading: tasksCleanupLoading,
  tasks,
  message: taskMessage,
  lastUpdatedAt: taskLastUpdatedAt,
  cancelConfirm: taskCancelConfirm,
  buttonText: taskButtonText,
  cleanupTaskCount,
  activeTaskCount,
  summaryTask,
  summaryFailed,
  load: loadTasks,
  toggle: toggleTaskPanel,
  open: openTaskPanel,
  close: closeTaskPanel,
  dismissSummary: dismissTaskSummary,
  stopPolling: stopTaskPolling,
  resetCancelConfirm: resetTaskCancelConfirm,
  requestCancel: cancelTaskById,
  closeCancelConfirm: closeTaskCancelConfirm,
  submitCancelConfirm: submitTaskCancelConfirm,
  cleanupFinishedTasks,
  markStarted: taskStarted
} = useTaskPanel({
  listTasks,
  cancelTask,
  cleanupTasks,
  showNotice: showShellNotice,
  showError: showErrorNotice,
  onTaskSettled: async () => {
    await refreshCurrent(true);
  }
});
const {
  visible: trashPanelVisible,
  loading: trashLoading,
  actionLoading: trashActionLoading,
  records: trashRecords,
  message: trashMessage,
  confirm: trashConfirm,
  selectedId: trashSelectedId,
  selectedIds: trashSelectedIds,
  selectedRecord: trashSelectedRecord,
  load: loadTrash,
  toggle: toggleTrashPanelBase,
  close: closeTrashPanel,
  selectRecord: selectTrashRecord,
  moveSelection: moveTrashSelection,
  selectAllRecords: selectAllTrashRecords,
  toggleFocusedRecord: toggleFocusedTrashRecord,
  closeConfirm: closeTrashConfirm,
  submitConfirm: submitTrashConfirm,
  restoreSelected: restoreTrashSelected,
  deleteSelected: deleteTrashSelected,
  empty: emptyTrashPanel,
  cleanup: cleanupTrashPanel
} = useTrashPanel({
  listTrashRecords,
  restoreTrashRecord,
  restoreTrashRecords,
  deleteTrashRecord,
  deleteTrashRecords,
  emptyTrash,
  cleanupTrash,
  showError: showErrorNotice,
  onRestored: async () => {
    await refreshFolderInExplorerAndTree(currentFolder());
  }
});
const explorerRef = ref<ExplorerExpose | null>(null);
const contentToolbarRef = ref<ContentToolbarExpose | null>(null);
const operationPanelRef = ref<FocusablePanelExpose | null>(null);
const deleteConfirmRef = ref<FocusablePanelExpose | null>(null);
const trashPanelRef = ref<FocusablePanelExpose | null>(null);
const trashConfirmRef = ref<FocusablePanelExpose | null>(null);
const propertiesPanelRef = ref<FocusablePanelExpose | null>(null);
const uploadInput = ref<HTMLInputElement | null>(null);

const activeTab = computed(() => fileStore.tabs.find(tab => tab.id === fileStore.activeTabId) ?? fileStore.tabs[0]);

const selectedEntry = () => explorerRef.value?.getSelectedEntry() ?? null;

const focusExplorer = async () => {
  if (fileStore.showEditor) return;
  await nextTick();
  explorerRef.value?.focus();
}

const {
  setSearchInputRef,
  searchText,
  isFiltering,
  clearSearch,
  handleSearchEscape,
  focusSearchInput
} = useExplorerSearchBox({focusExplorer});
const searchType = ref<DirEntryFilter>("all");
const searchScope = ref<SearchScope>("mount");
const {
  inspectSearchIndexBeforeSearch
} = useSearchIndexStatusHint({
  getIndexStatus,
  showNotice: showShellNotice
});

const updateSearchText = (value: string) => {
  searchText.value = value;
}

const runIndexedSearch = async () => {
  const query = searchText.value.trim();
  if (!query) {
    await focusExplorer();
    return;
  }
  if (!await fileStore.requestEditorLeave()) return;
  await inspectSearchIndexBeforeSearch();
  closeTransientPanels();
  const loaded = await explorerRef.value?.search(query, searchType.value, searchScope.value);
  if (loaded) await focusExplorer();
}

const rerunSearchIfResultActive = async () => {
  const query = searchText.value.trim();
  if (!query || !(explorerRef.value?.isResultActive() ?? false)) return;
  await runIndexedSearch();
}

const updateSearchType = async (value: DirEntryFilter) => {
  searchType.value = value;
  await rerunSearchIfResultActive();
}

const updateSearchScope = async (value: SearchScope) => {
  searchScope.value = value;
  await rerunSearchIfResultActive();
}

const showRecentEntries = async () => {
  if (!await fileStore.requestEditorLeave()) return;
  clearSearch(false);
  closeTransientPanels();
  const loaded = await explorerRef.value?.showRecent();
  if (loaded) await focusExplorer();
}

const clearSearchOrResults = async () => {
  const resultActive = explorerRef.value?.isResultActive() ?? false;
  clearSearch(false);
  if (resultActive) {
    await explorerRef.value?.clearResults();
  }
  await focusExplorer();
}

const clearExplorerResults = async () => {
  clearSearch(false);
  await explorerRef.value?.clearResults();
  await focusExplorer();
}

const {
  shouldPersistSelection,
  persistSelectedPaths,
  persistActiveTabScrollTop,
  persistCurrentExplorerScrollTop,
  syncActiveTabContext,
  stopScrollPersistence
} = useExplorerTabContext({
  activeTab,
  searchText,
  selectPaths: (paths, scrollToSelection) => explorerRef.value?.selectPaths(paths, scrollToSelection) ?? Promise.resolve(false),
  getScrollTop: () => explorerRef.value?.getScrollTop() ?? 0,
  setScrollTop: scrollTop => explorerRef.value?.setScrollTop(scrollTop) ?? Promise.resolve()
});

const {
  previewPanelVisible,
  previewEntry,
  previewReloadKey,
  imageViewerVisible,
  imageViewerEntry,
  imageViewerEntries,
  audioPlayerVisible,
  audioPlayerEntry,
  audioPlayerEntries,
  audioPlayerReloadKey,
  videoViewerVisible,
  videoViewerEntry,
  videoViewerEntries,
  pdfViewerVisible,
  pdfViewerEntry,
  pdfViewerEntries,
  currentSelection,
  selectedCount,
  hasSelection,
  singleSelection,
  previewEmptyTitle,
  previewEmptySubtitle,
  previewEmptyIcon,
  resetPreviewContext,
  closePreviewPanel,
  resetImageViewer,
  resetVideoViewer,
  resetPdfViewer,
  closeImageViewer,
  closeAudioPlayer,
  closeVideoViewer,
  closePdfViewer,
  setImageViewerEntry,
  setAudioPlayerEntry,
  setVideoViewerEntry,
  setPdfViewerEntry,
  openImageViewer,
  openAudioPlayer,
  openVideoViewer,
  openPdfViewer,
  openPreviewEntryImageViewer,
  openPreviewEntryAudioPlayer,
  openPreviewEntryVideoViewer,
  openPreviewEntryPdfViewer,
  previewSelected,
  previewSelectedQuietly,
  showEmptyPreviewPane,
  handleSelectionChange
} = useExplorerPreview({
  getSelectedEntry: selectedEntry,
  getImageEntries: () => explorerRef.value?.getImageEntries() ?? [],
  getAudioEntries: () => explorerRef.value?.getAudioEntries() ?? [],
  getVideoEntries: () => explorerRef.value?.getVideoEntries() ?? [],
  getPdfEntries: () => explorerRef.value?.getPdfEntries() ?? [],
  shouldPersistSelection,
  persistSelectedPaths,
  showNotice: showShellNotice
});

let closePanelsHandler = () => {};
let closeTransientPanelsHandler = () => {};
let closeOperationShellPanelsHandler = () => {};
let closePreviewHandler = () => {};
let tabHoverSwitchTimer: number | undefined;
let tabHoverSwitchTargetId = "";
const closePanels = () => closePanelsHandler();
const closeTransientPanels = () => closeTransientPanelsHandler();
const closeOperationShellPanels = () => closeOperationShellPanelsHandler();
const closePreview = () => closePreviewHandler();

const stopTabHoverSwitch = (tabId?: string) => {
  if (tabId && tabHoverSwitchTargetId !== tabId) return;
  if (tabHoverSwitchTimer) {
    window.clearTimeout(tabHoverSwitchTimer);
    tabHoverSwitchTimer = undefined;
  }
  if (!tabId || tabHoverSwitchTargetId === tabId) tabHoverSwitchTargetId = "";
}

const {
  currentFolder,
  canNavigateBack,
  canNavigateForward,
  canNavigateUp,
  navigateBackTitle,
  navigateForwardTitle,
  navigateUpTitle,
  navigateToPath,
  navigateBack,
  navigateForward,
  navigateUp,
  handleBreadcrumbNavigate,
  handleBackspaceNavigation,
  handleHistoryMouseDown,
  handleHistoryMouseUp,
  handleHistoryAuxClick
} = useExplorerNavigation({
  activeTab,
  refreshExplorer: async path => await explorerRef.value?.refresh(path) ?? false,
  focusExplorer,
  closeTransientPanels,
  syncActiveTabContext,
  persistCurrentExplorerScrollTop,
  shouldIgnoreNavigationShortcut: target => shouldIgnoreNavigationShortcut(target)
});

const {
  tabContextMenu,
  tabContextTarget,
  canCloseTabContext,
  canCloseOtherTabsContext,
  canCloseRightTabsContext,
  canReopenClosedTab,
  draggingTabId,
  tabDropTargetId,
  tabDropPlacement,
  closeTabContextMenu,
  openTabContextMenuAt,
  openTabContextMenu,
  openTab,
  openEntryInNewTab,
  switchTab,
  switchRelativeTab,
  tabShortcutTargetId,
  closeActiveTab,
  closeTab,
  handleTabAuxClick,
  duplicateTabFromMenu,
  closeTabFromMenu,
  reopenClosedTab,
  closeOtherTabsFromMenu,
  closeRightTabsFromMenu,
  startTabDrag,
  dragOverTab,
  leaveTabDropTarget,
  dropTab,
  finishTabDrag
} = useExplorerTabs({
  closeTransientPanels,
  syncActiveTabContext,
  persistCurrentExplorerScrollTop,
  showNotice: showShellNotice
});

const scheduleTabHoverSwitch = (tabId: string) => {
  if (!tabId || tabId === fileStore.activeTabId) {
    stopTabHoverSwitch();
    return;
  }
  if (tabHoverSwitchTargetId === tabId && tabHoverSwitchTimer) return;
  stopTabHoverSwitch();
  tabHoverSwitchTargetId = tabId;
  tabHoverSwitchTimer = window.setTimeout(() => {
    const targetTabId = tabHoverSwitchTargetId;
    tabHoverSwitchTimer = undefined;
    tabHoverSwitchTargetId = "";
    if (!targetTabId || targetTabId === fileStore.activeTabId) return;
    void switchTab(targetTabId);
  }, 650);
}

const {treeData, loadRoot, handleLoad, reorderMount, refreshPath: refreshTreePath} = useFileTreeLoader({
  getFolderData,
  getMappings,
  getMappingRoot,
  reorderMappings,
  navigateToPath,
  showError: showErrorNotice
});

refreshFolderInExplorerAndTree = async (path: string) => {
  await Promise.all([
    explorerRef.value?.refresh(path, {forceRefresh: true}) ?? Promise.resolve(false),
    refreshTreePath(path)
  ]);
}

const {
  favorites,
  favoritesLoading,
  favoritePaths,
  loadFavorites,
  addFavorite,
  renameFavorite,
  reorderFavorite,
  removeFavorite
} = useFavorites({
  listFavorites,
  createFavorite,
  updateFavorite,
  reorderFavorites,
  deleteFavorite,
  showNotice: showShellNotice,
  showError: showErrorNotice
});

const {
  fileClipboardAction,
  operationPanel,
  deleteConfirm,
  propertiesPanel,
  clipboardPaths,
  hasClipboard,
  clipboardText,
  resetOperationPanel,
  resetDeleteConfirm,
  closePropertiesPanel,
  closeDeleteConfirm,
  showProperties,
  closeOperationPanel,
  openCreatePanel,
  createFolderFromShortcut,
  startRenameSelected,
  renameSelected,
  deleteSelected,
  submitDeleteConfirm,
  downloadSelected,
  copySelected,
  cutSelected,
  copyEntryPaths,
  pasteSelected,
  dropEntriesToFolder,
  dropEntriesToCurrentFolder,
  archiveSelected,
  extractSelected,
  submitOperationPanel,
  uploadChanged,
  uploadToCurrentFolder
} = useFileOperations({
  currentFolder,
  refreshCurrent: () => refreshCurrent(),
  closeShellPanels: closeOperationShellPanels,
  getSelectedEntry: selectedEntry,
  getSelectedEntries: () => explorerRef.value?.getSelectedEntries() ?? [],
  startExplorerRename: () => explorerRef.value?.startRename(),
  selectPath: async path => Boolean(await explorerRef.value?.selectPath(path)),
  selectPathForRename: async path => Boolean(await explorerRef.value?.selectPathForRename(path)),
  showNotice: showShellNotice,
  showError: showErrorNotice,
  taskStarted,
  setTaskMessage: message => taskMessage.value = message,
  focusOperationPanel: () => operationPanelRef.value?.focus(),
  focusDeleteConfirm: () => deleteConfirmRef.value?.focus(),
  focusPropertiesPanel: () => propertiesPanelRef.value?.focus()
});

const {
  canDownloadSelection,
  canPreviewSelection,
  canTogglePreviewPane,
  canRenameSelection,
  canArchiveSelection,
  canDeleteSelection,
  canExtractSelection,
  canPasteSelection
} = useMainViewSelectionCommands({
  singleSelection,
  selectedCount,
  hasSelection,
  hasClipboard,
  clipboardText,
  editorVisible: () => fileStore.showEditor
});

const shellActions = useMainViewShellActions({
  previewPanelVisible,
  currentSelection,
  editorVisible: () => fileStore.showEditor,
  currentFolder,
  refreshExplorer: (path, options) => explorerRef.value?.refresh(path, options) ?? Promise.resolve(false),
  refreshTreePath,
  selectPaths: paths => explorerRef.value?.selectPaths(paths) ?? Promise.resolve(false),
  clearPersistedSelection: () => fileStore.setActiveTabSelectedPaths([]),
  closePreviewPanel,
  resetPreviewContext,
  resetImageViewer,
  resetVideoViewer,
  resetPdfViewer,
  closeImageViewer,
  closeVideoViewer,
  closePdfViewer,
  hideOperationPanel: () => operationPanel.value.visible = false,
  resetOperationPanel,
  resetDeleteConfirm,
  closePropertiesPanel,
  resetTaskCancelConfirm,
  closeTrashPanel
});

closePanelsHandler = shellActions.closePanels;
closeTransientPanelsHandler = shellActions.closeTransientPanels;
closeOperationShellPanelsHandler = shellActions.closeOperationShellPanels;
closePreviewHandler = shellActions.closePreview;
refreshCurrentHandler = shellActions.refreshCurrent;

watch(() => fileStore.showEditor, (showEditor) => {
  if (showEditor) closePanels();
});

watch(() => trashConfirm.value.visible, async (visible, wasVisible) => {
  await nextTick();
  if (visible) {
    trashConfirmRef.value?.focus();
  } else if (wasVisible && trashPanelVisible.value) {
    trashPanelRef.value?.focus();
  }
});

const {
  active: uploadDropActive,
  uploading: uploadDropUploading,
  title: uploadDropTitle,
  subtitle: uploadDropSubtitle,
  handleDragEnter: handleUploadDragEnter,
  handleDragOver: handleUploadDragOver,
  handleDragLeave: handleUploadDragLeave,
  handleDrop: handleUploadDrop
} = useUploadDrop({
  canAccept: () => !fileStore.showEditor,
  currentFolder,
  upload: uploadToCurrentFolder
});

const {
  currentViewModeMeta,
  currentViewModeLabel,
  viewModeButtonTitle,
  selectViewMode,
  applyViewShortcut
} = useExplorerViewMode({
  focusExplorer: () => explorerRef.value?.focus(),
  closeMenus: closeTabContextMenu
});

const {
  togglePreviewFromShortcut,
  handleWindowKeyDown
} = useExplorerShortcuts({
  imageViewerVisible,
  videoViewerVisible,
  pdfViewerVisible,
  previewPanelVisible,
  hasPreviewableSelection: () => singleSelection.value?.type === "file",
  focusSearchInput,
  focusBreadcrumbInput: () => contentToolbarRef.value?.focusInput(),
  selectAllEntries: () => explorerRef.value?.selectAllEntries(),
  applyViewShortcut,
  tabShortcutTargetId,
  switchTab,
  openTab,
  reopenClosedTab,
  closeActiveTab,
  switchRelativeTab,
  createFolderFromShortcut,
  refreshCurrent: keepSelection => refreshCurrent(keepSelection),
  copySelected: () => copySelected(),
  cutSelected: () => cutSelected(),
  pasteSelected: () => pasteSelected(),
  closePreview: () => closePreview(),
  previewSelectedQuietly,
  showEmptyPreviewPane,
  handleBackspaceNavigation,
  navigateBack,
  navigateForward,
  navigateUp
});

const loadEditableExtensions = async () => {
  try {
    const settings = await getSettings();
    fileStore.setExtensions(settings.runtime.editableExtensions);
  } catch (error) {
    console.warn("同步可编辑扩展名失败", error);
  }
}

useMainViewLifecycle({
  initialize: async () => {
    fileStore.ensureActiveTab();
    await Promise.all([loadEditableExtensions(), loadRoot(), loadFavorites()]);
    await syncActiveTabContext();
  },
  stopScrollPersistence,
  stopTabHoverSwitch,
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
  handleWindowResize: () => {
    handleSidebarWindowResize();
    handlePreviewPaneWindowResize();
  }
});

const openPreviewInEditor = async (entry = previewEntry.value) => {
  if (!entry || entry.type !== "file") return;
  if (!await fileStore.requestEditorLeave()) return;
  closePanels();
  fileStore.openEditor(entryFileInfo(entry));
}

const treeNodeToFolderEntry = (node: Pick<FileTreeData, "path" | "name">): ExplorerEntry => ({
  type: "folder",
  path: node.path,
  name: node.name,
  modified: ""
});

const favoriteToFolderEntry = (favorite: FavoriteItem): ExplorerEntry => ({
  type: "folder",
  path: favorite.path,
  name: favorite.name,
  modified: ""
});

const dropEntriesToPathFolder = ({entries, target, action}: ExplorerEntryPathDropPayload) => {
  stopTabHoverSwitch();
  void dropEntriesToFolder({
    entries,
    action,
    target: treeNodeToFolderEntry(target)
  });
}

const openTreeFolderInNewTab = (node: FileTreeData) => {
  void openEntryInNewTab(treeNodeToFolderEntry(node));
}

const openFavorite = async (favorite: FavoriteItem) => {
  await navigateToPath(favorite.path);
}

const openFavoriteInNewTab = (favorite: FavoriteItem) => {
  void openEntryInNewTab(favoriteToFolderEntry(favorite));
}

const addTreeNodeToFavorites = (node: FileTreeData) => {
  void addFavorite(node.path, node.name);
}

const addExplorerEntryToFavorites = (entry: ExplorerEntry) => {
  if (entry.type !== "folder") return;
  void addFavorite(entry.path, entry.name);
}

const editPreviewEntry = (entry: ExplorerEntry) => {
  void openPreviewInEditor(entry);
}

const {
  closeTaskPanelAndFocus,
  closeTrashPanelAndFocus,
  closeOperationPanelAndFocus,
  closeDeleteConfirmAndFocus,
  closePropertiesPanelAndFocus,
  closePreviewAndFocus,
  closeImageViewerAndFocus,
  closeVideoViewerAndFocus,
  closePdfViewerAndFocus
} = useMainViewPanelClosers({
  editorVisible: () => fileStore.showEditor,
  focusExplorer,
  taskPanelVisible,
  trashPanelVisible,
  operationPanel,
  deleteConfirm,
  propertiesPanel,
  previewPanelVisible,
  imageViewerVisible,
  videoViewerVisible,
  pdfViewerVisible,
  closeTaskPanel,
  closeTrashPanel,
  closeOperationPanel,
  closeDeleteConfirm,
  closePropertiesPanel,
  closePreview,
  closeImageViewer,
  closeVideoViewer,
  closePdfViewer
});

const toggleTasksFromMenu = async () => {
  if (trashPanelVisible.value) closeTrashPanel();
  await toggleTaskPanel();
}

const openTasksFromSummary = async () => {
  if (trashPanelVisible.value) closeTrashPanel();
  await openTaskPanel();
}

const toggleTrashFromMenu = async () => {
  if (taskPanelVisible.value) closeTaskPanel();
  await toggleTrashPanelBase();
}

const openSettings = async () => {
  if (!await fileStore.requestEditorLeave()) return;
  await router.push("/setting");
}

const signOut = async () => {
  if (!await fileStore.requestEditorLeave()) return;
  await logout();
  await router.replace("/login");
}
</script>

<template>
  <div class="main-shell">
    <header class="top-strip">
      <tab-strip
          :tabs="fileStore.tabs"
          :active-tab-id="activeTab?.id ?? ''"
          :dragging-tab-id="draggingTabId"
          :drop-target-id="tabDropTargetId"
          :drop-placement="tabDropPlacement"
          :context-menu="tabContextMenu"
          :context-target="tabContextTarget"
          :can-close-tab="canCloseTabContext"
          :can-close-other-tabs="canCloseOtherTabsContext"
          :can-close-right-tabs="canCloseRightTabsContext"
          :can-reopen-closed-tab="canReopenClosedTab"
          @new-tab="openTab"
          @activate-tab="switchTab"
          @close-tab="closeTab"
          @tab-aux-click="handleTabAuxClick"
          @tab-context-menu="openTabContextMenu"
          @tab-keyboard-context-menu="payload => openTabContextMenuAt(payload.x, payload.y, payload.tabId)"
          @tab-drag-start="startTabDrag"
          @tab-drag-over="dragOverTab"
          @tab-drag-leave="leaveTabDropTarget"
          @tab-drop="dropTab"
          @tab-drag-end="finishTabDrag"
          @drop-entries="dropEntriesToPathFolder"
          @entry-drag-hover-tab="scheduleTabHoverSwitch"
          @entry-drag-leave-tab="stopTabHoverSwitch"
          @open-entry-new-tab="openEntryInNewTab"
          @duplicate-tab="duplicateTabFromMenu"
          @close-context-tab="closeTabFromMenu"
          @reopen-closed-tab="reopenClosedTab"
          @close-other-tabs="closeOtherTabsFromMenu"
          @close-right-tabs="closeRightTabsFromMenu"
          @close-context-menu="closeTabContextMenu" />
      <div class="top-actions">
        <shell-more-menu
            :task-button-text="taskButtonText"
            :task-active="taskPanelVisible"
            :trash-active="trashPanelVisible"
            @open-settings="openSettings"
            @toggle-tasks="toggleTasksFromMenu"
            @toggle-trash="toggleTrashFromMenu"
            @sign-out="signOut" />
      </div>
    </header>

    <main class="workspace" :class="{resizingSidebar: sidebarResizing}" :style="workspaceStyle">
      <content-toolbar
          ref="contentToolbarRef"
          :can-navigate-back="canNavigateBack"
          :can-navigate-forward="canNavigateForward"
          :can-navigate-up="canNavigateUp"
          :navigate-back-title="navigateBackTitle"
          :navigate-forward-title="navigateForwardTitle"
          :navigate-up-title="navigateUpTitle"
          :search-text="searchText"
          :is-filtering="isFiltering"
          :search-type="searchType"
          :search-scope="searchScope"
          :set-search-input-ref="setSearchInputRef"
          @navigate-back="navigateBack"
          @navigate-forward="navigateForward"
          @navigate-up="navigateUp"
          @refresh="refreshCurrent(true)"
          @show-recent="showRecentEntries"
          @breadcrumb-navigate="handleBreadcrumbNavigate"
          @breadcrumb-drop="dropEntriesToPathFolder"
          @update:search-text="updateSearchText"
          @update:search-type="updateSearchType"
          @update:search-scope="updateSearchScope"
          @search-enter="runIndexedSearch"
          @search-escape="handleSearchEscape"
          @clear-search="clearSearchOrResults" />

      <div class="workspace-body">
      <sidebar-panel
          :tree-data="treeData"
          :load-data="handleLoad"
          :current-path="fileStore.currentPath"
          :favorites="favorites"
          :favorites-loading="favoritesLoading"
          :favorite-paths="favoritePaths"
          @drop-entries="dropEntriesToPathFolder"
          @reorder-mount="payload => reorderMount(payload.source, payload.target, payload.placement)"
          @open-new-tab="openTreeFolderInNewTab"
          @open-favorite="openFavorite"
          @open-favorite-new-tab="openFavoriteInNewTab"
          @rename-favorite="payload => renameFavorite(payload.favorite, payload.name)"
          @reorder-favorite="payload => reorderFavorite(payload.source, payload.target, payload.placement)"
          @remove-favorite="favorite => removeFavorite(favorite)"
          @refresh-favorites="loadFavorites({check: true})"
          @add-favorite="addTreeNodeToFavorites"
          @remove-favorite-path="path => removeFavorite(path)"
          @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)" />

      <div
          class="sidebar-resizer"
          role="separator"
          aria-orientation="vertical"
          :aria-valuemin="sidebarMinWidth"
          :aria-valuemax="sidebarMaxWidth"
          :aria-valuenow="sidebarWidth"
          tabindex="0"
          title="拖动调整文件树宽度，双击恢复默认"
          @pointerdown="startSidebarResize"
          @keydown="handleSidebarResizeKeyDown"
          @dblclick="resetSidebarWidth">
      </div>

      <section class="content-pane">
        <command-bar
            :has-selection="hasSelection"
            :can-paste-selection="canPasteSelection"
            :can-download-selection="canDownloadSelection"
            :can-preview-selection="canPreviewSelection"
            :can-archive-selection="canArchiveSelection"
            :can-extract-selection="canExtractSelection"
            :can-rename-selection="canRenameSelection"
            :can-delete-selection="canDeleteSelection"
            :view-mode-icon="currentViewModeMeta.icon"
            :view-mode-label="currentViewModeLabel"
            :view-mode-button-title="viewModeButtonTitle"
            :view-mode="fileStore.viewMode"
            :icon-size="fileStore.iconSize"
            :sort-key="fileStore.sortKey"
            :sort-order="fileStore.sortOrder"
            :preview-panel-visible="previewPanelVisible"
            :can-toggle-preview-pane="canTogglePreviewPane"
            @upload="uploadInput?.click()"
            @create-file="openCreatePanel('file')"
            @create-folder="openCreatePanel('folder')"
            @cut="cutSelected()"
            @copy="copySelected()"
            @paste="pasteSelected()"
            @download="downloadSelected()"
            @preview="previewSelected()"
            @archive="archiveSelected()"
            @extract="extractSelected()"
            @rename="startRenameSelected"
            @delete="deleteSelected()"
            @select-view-mode="selectViewMode"
            @set-sort-key="key => explorerRef?.setSortKey(key)"
            @set-sort-order="order => explorerRef?.setSortOrder(order)"
            @toggle-preview="togglePreviewFromShortcut" />
        <input ref="uploadInput" class="hidden" type="file" multiple @change="uploadChanged">

        <div class="browser-area" :class="{previewing: previewPanelVisible, resizingPreview: previewPaneResizing}" :style="browserAreaStyle">
          <div
              class="browser-main"
              :class="{dropActive: uploadDropActive || uploadDropUploading}"
              @dragenter="handleUploadDragEnter"
              @dragover="handleUploadDragOver"
              @dragleave="handleUploadDragLeave"
              @drop="handleUploadDrop">
            <explorer
                ref="explorerRef"
                v-show="!fileStore.showEditor"
                :filter-text="searchText"
                :dimmed-paths="fileClipboardAction === 'cut' ? clipboardPaths : []"
                :can-paste="canPasteSelection"
                :favorite-paths="favoritePaths"
                :apply-view-shortcut="applyViewShortcut"
                @rename="renameSelected"
                @delete="deleteSelected"
                @download="downloadSelected"
                @archive="archiveSelected"
                @extract="extractSelected"
                @properties="showProperties"
                @preview="previewSelected"
                @copy="copySelected"
                @cut="cutSelected"
                @copy-path="copyEntryPaths"
                @paste="pasteSelected"
                @create-file="openCreatePanel('file')"
                @create-folder="openCreatePanel('folder')"
                @drop-entries="dropEntriesToFolder"
                @drop-to-current-folder="dropEntriesToCurrentFolder"
                @selection-change="handleSelectionChange"
                @scroll-change="persistActiveTabScrollTop"
                @clear-filter="() => clearSearch()"
                @clear-result="clearExplorerResults"
                @open-new-tab="openEntryInNewTab"
                @add-favorite="addExplorerEntryToFavorites"
                @remove-favorite="path => removeFavorite(path)"
                @open-image-viewer="openImageViewer"
                @open-audio-player="openAudioPlayer"
                @open-video-viewer="openVideoViewer"
                @open-pdf-viewer="openPdfViewer">
            </explorer>
            <upload-drop-overlay
                v-if="uploadDropActive || uploadDropUploading"
                :title="uploadDropTitle"
                :subtitle="uploadDropSubtitle" />
            <task-panel
                v-if="taskPanelVisible"
                :tasks="tasks"
                :loading="tasksLoading"
                :cleanup-loading="tasksCleanupLoading"
                :cleanup-task-count="cleanupTaskCount"
                :message="taskMessage"
                :last-updated-at="taskLastUpdatedAt"
                :cancel-confirm="taskCancelConfirm"
                @refresh="loadTasks()"
                @cleanup-finished="cleanupFinishedTasks"
                @close="closeTaskPanelAndFocus"
                @cancel="cancelTaskById"
                @close-cancel="closeTaskCancelConfirm"
                @confirm-cancel="submitTaskCancelConfirm">
            </task-panel>
            <div class="task-status-layer">
              <task-status-pill
                  :visible="taskSummaryVisible"
                  :task="summaryTask"
                  :active-count="activeTaskCount"
                  :failed="summaryFailed"
                  :panel-open="taskPanelVisible"
                  @open="openTasksFromSummary"
                  @dismiss="dismissTaskSummary" />
            </div>
            <trash-panel
                v-if="trashPanelVisible"
                ref="trashPanelRef"
                :records="trashRecords"
                :selected-id="trashSelectedId"
                :selected-ids="trashSelectedIds"
                :selected-record="trashSelectedRecord"
                :loading="trashLoading"
                :action-loading="trashActionLoading"
                :message="trashMessage"
                @select="selectTrashRecord"
                @move-selection="moveTrashSelection"
                @select-all="selectAllTrashRecords"
                @toggle-focused="toggleFocusedTrashRecord"
                @refresh="loadTrash()"
                @restore="restoreTrashSelected"
                @delete="deleteTrashSelected"
                @empty="emptyTrashPanel"
                @cleanup="cleanupTrashPanel"
                @close="closeTrashPanelAndFocus">
            </trash-panel>
            <operation-panel
                ref="operationPanelRef"
                :state="operationPanel"
                @update:name="value => operationPanel.name = value"
                @update:format="value => operationPanel.format = value"
                @close="closeOperationPanelAndFocus"
                @submit="submitOperationPanel" />
            <delete-confirm-panel
                ref="deleteConfirmRef"
                :state="deleteConfirm"
                @close="closeDeleteConfirmAndFocus"
                @update:permanent="value => deleteConfirm.permanent = value"
                @submit="submitDeleteConfirm" />
            <trash-confirm-panel
                ref="trashConfirmRef"
                :state="trashConfirm"
                :total-count="trashRecords.length"
                @close="closeTrashConfirm"
                @submit="submitTrashConfirm" />
            <properties-panel
                ref="propertiesPanelRef"
                :visible="propertiesPanel.visible"
                :entries="propertiesPanel.entries"
                :current-folder="currentFolder()"
                @close="closePropertiesPanelAndFocus" />
          </div>
          <aside v-if="previewPanelVisible" class="preview-pane">
            <div
                class="preview-resizer"
                role="separator"
                aria-orientation="vertical"
                :aria-valuemin="previewPaneMinWidth"
                :aria-valuemax="previewPaneMaxWidth"
                :aria-valuenow="previewPaneWidth"
                tabindex="0"
                title="拖动调整预览窗格宽度，双击恢复默认"
                @pointerdown="startPreviewPaneResize"
                @keydown="handlePreviewPaneResizeKeyDown"
                @dblclick="resetPreviewPaneWidth">
            </div>
            <preview-pane
                :entry="previewEntry"
                :editable-extensions="fileStore.extensions"
                :reload-key="previewReloadKey"
                :empty-title="previewEmptyTitle"
                :empty-subtitle="previewEmptySubtitle"
                :empty-icon="previewEmptyIcon"
                @close="closePreviewAndFocus"
                @edit="editPreviewEntry"
                @download="downloadSelected"
                @open-image="openPreviewEntryImageViewer"
                @open-audio="openPreviewEntryAudioPlayer"
                @open-video="openPreviewEntryVideoViewer"
                @open-pdf="openPreviewEntryPdfViewer"
                @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)">
            </preview-pane>
          </aside>
        </div>

        <Transition name="shell-notice-pop" mode="out-in">
          <div
              v-if="shellNotice.visible"
              :key="shellNotice.id"
              class="shell-notice-layer">
            <shell-notice
                :kind="shellNotice.kind"
                :title="shellNotice.title"
                :message="shellNotice.message"
                @close="closeShellNotice"
                @pause="stopShellNoticeTimer"
                @resume="resumeShellNoticeTimer" />
          </div>
        </Transition>

        <image-viewer
            :visible="imageViewerVisible"
            :entry="imageViewerEntry"
            :entries="imageViewerEntries"
            @close="closeImageViewerAndFocus"
            @select="setImageViewerEntry"
            @download="downloadSelected"
            @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)">
        </image-viewer>

        <audio-player
            :visible="audioPlayerVisible"
            :entry="audioPlayerEntry"
            :entries="audioPlayerEntries"
            :reload-key="audioPlayerReloadKey"
            @close="closeAudioPlayer"
            @select="setAudioPlayerEntry"
            @download="downloadSelected"
            @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)">
        </audio-player>

        <video-viewer
            :visible="videoViewerVisible"
            :entry="videoViewerEntry"
            :entries="videoViewerEntries"
            @close="closeVideoViewerAndFocus"
            @select="setVideoViewerEntry"
            @download="downloadSelected"
            @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)">
        </video-viewer>

        <pdf-viewer
            :visible="pdfViewerVisible"
            :entry="pdfViewerEntry"
            :entries="pdfViewerEntries"
            @close="closePdfViewerAndFocus"
            @select="setPdfViewerEntry"
            @download="downloadSelected"
            @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)">
        </pdf-viewer>

        <div v-show="fileStore.showEditor" class="editor-overlay-panel">
          <editor-panel></editor-panel>
        </div>
      </section>
      </div>
    </main>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.main-shell {
  @apply flex h-screen flex-col overflow-hidden p-3;
  background: var(--app-bg);
  color: var(--app-text);
}

.top-strip {
  @apply flex h-12 shrink-0 items-center gap-2;
}

.top-actions {
  @apply flex h-full shrink-0 items-center;
}

.workspace {
  @apply mt-2 flex min-h-0 grow flex-col overflow-hidden rounded-xl border shadow-sm backdrop-blur;
  --sidebar-width: 17rem;
  border-color: var(--app-border);
  background: var(--app-panel);
}

.workspace-body {
  @apply grid min-h-0 grow gap-0;
  grid-template-columns: var(--sidebar-width) 0.375rem minmax(0, 1fr);
}

.workspace.resizingSidebar {
  @apply cursor-col-resize select-none;
}

.sidebar-resizer {
  @apply relative z-20 h-full cursor-col-resize touch-none outline-none;
  background: var(--app-bg-muted);
}

.sidebar-resizer::after {
  content: "";
  @apply absolute top-2 bottom-2 left-1/2 w-px -translate-x-1/2 rounded-full;
  background: var(--app-divider);
}

.sidebar-resizer:hover::after,
.sidebar-resizer:focus-visible::after,
.workspace.resizingSidebar .sidebar-resizer::after {
  background: var(--app-accent, #2563eb);
}

.content-pane {
  @apply relative flex min-h-0 flex-col overflow-hidden;
  background: var(--app-panel-solid);
}

.browser-area {
  @apply grid min-h-0 grow grid-cols-[minmax(0,1fr)] overflow-hidden;
  --preview-pane-width: 22rem;
}

.browser-area.previewing {
  grid-template-columns: minmax(0, 1fr) minmax(17.5rem, var(--preview-pane-width));
}

.browser-area.resizingPreview {
  @apply cursor-col-resize select-none;
}

.browser-main {
  @apply relative min-h-0 overflow-hidden;
}

.browser-main.dropActive {
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 42%, transparent);
}

.editor-overlay-panel {
  @apply absolute inset-0 z-30 min-h-0 overflow-hidden;
}

.preview-pane {
  @apply relative flex min-h-0 flex-col border-l;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

.preview-resizer {
  @apply absolute -left-1 top-0 z-10 h-full w-2 cursor-col-resize touch-none;
}

.preview-resizer::after {
  content: "";
  @apply absolute left-1 top-0 h-full w-px bg-transparent;
}

.preview-resizer:hover::after,
.preview-resizer:focus-visible::after,
.browser-area.resizingPreview .preview-resizer::after {
  background: var(--app-accent, #2563eb);
}

.preview-resizer:focus-visible {
  @apply outline-none;
}

.shell-notice-layer {
  @apply pointer-events-none absolute inset-x-0 bottom-12 z-[70] flex justify-center px-4;
}

.shell-notice-layer :deep(.shell-notice) {
  @apply pointer-events-auto;
}

.task-status-layer {
  @apply pointer-events-none absolute inset-x-0 bottom-3 z-[58] flex justify-center px-4;
}

.shell-notice-pop-enter-active,
.shell-notice-pop-leave-active {
  transition:
      opacity 0.14s ease,
      transform 0.16s cubic-bezier(0.2, 0, 0, 1);
}

.shell-notice-pop-enter-from,
.shell-notice-pop-leave-to {
  opacity: 0;
  transform: translateY(0.5rem) scale(0.98);
}

@media (prefers-reduced-motion: reduce) {
  .shell-notice-pop-enter-active,
  .shell-notice-pop-leave-active {
    transition: none;
  }
}

</style>
