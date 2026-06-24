<script setup lang="ts">
import {computed, defineAsyncComponent, nextTick, ref, watch} from "vue";
import {useRouter} from "vue-router";
import {useFileStore} from "../store";
import {
  cancelTask,
  getFolderData,
  listTasks,
  logout,
} from "../network/api";
import Explorer from "../components/explorer/Explorer.vue";
import ImageViewer from "../components/viewer/ImageViewer.vue";
import PreviewPane from "../components/viewer/PreviewPane.vue";
import TaskPanel from "../components/tasks/TaskPanel.vue";
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
import type {DirSortKey, DirSortOrder, FileTreeData} from "../class.ts";
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
import {useUploadDrop} from "../composables/useUploadDrop.ts";
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
  focus: () => void;
  getImageEntries: () => ExplorerEntry[];
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
  stopTimer: stopShellNoticeTimer
} = useShellNotice();
let refreshCurrentHandler = async (_keepSelection = false) => {};
const refreshCurrent = (keepSelection = false) => refreshCurrentHandler(keepSelection);
let refreshCurrentTreePath = async () => {};

const {
  visible: taskPanelVisible,
  loading: tasksLoading,
  tasks,
  message: taskMessage,
  lastUpdatedAt: taskLastUpdatedAt,
  cancelConfirm: taskCancelConfirm,
  buttonText: taskButtonText,
  load: loadTasks,
  toggle: toggleTaskPanel,
  close: closeTaskPanel,
  stopPolling: stopTaskPolling,
  resetCancelConfirm: resetTaskCancelConfirm,
  requestCancel: cancelTaskById,
  closeCancelConfirm: closeTaskCancelConfirm,
  submitCancelConfirm: submitTaskCancelConfirm,
  markStarted: taskStarted
} = useTaskPanel({
  listTasks,
  cancelTask,
  showError: showErrorNotice,
  onTaskSettled: async () => {
    await refreshCurrent(true);
    await refreshCurrentTreePath();
  }
});
const explorerRef = ref<ExplorerExpose | null>(null);
const contentToolbarRef = ref<ContentToolbarExpose | null>(null);
const operationPanelRef = ref<FocusablePanelExpose | null>(null);
const deleteConfirmRef = ref<FocusablePanelExpose | null>(null);
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

const updateSearchText = (value: string) => {
  searchText.value = value;
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
  closeImageViewer,
  setImageViewerEntry,
  openImageViewer,
  openPreviewEntryImageViewer,
  previewSelected,
  previewSelectedQuietly,
  showEmptyPreviewPane,
  handleSelectionChange
} = useExplorerPreview({
  getSelectedEntry: selectedEntry,
  getImageEntries: () => explorerRef.value?.getImageEntries() ?? [],
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

const {treeData, loadRoot, handleLoad, refreshPath: refreshTreePath} = useFileTreeLoader({
  getFolderData,
  navigateToPath,
  showError: showErrorNotice
});

refreshCurrentTreePath = async () => {
  await refreshTreePath(currentFolder());
}

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
  loadRoot,
  refreshExplorer: (path, options) => explorerRef.value?.refresh(path, options) ?? Promise.resolve(false),
  selectPaths: paths => explorerRef.value?.selectPaths(paths) ?? Promise.resolve(false),
  clearPersistedSelection: () => fileStore.setActiveTabSelectedPaths([]),
  closePreviewPanel,
  resetPreviewContext,
  resetImageViewer,
  closeImageViewer,
  hideOperationPanel: () => operationPanel.value.visible = false,
  resetOperationPanel,
  resetDeleteConfirm,
  closePropertiesPanel,
  resetTaskCancelConfirm
});

closePanelsHandler = shellActions.closePanels;
closeTransientPanelsHandler = shellActions.closeTransientPanels;
closeOperationShellPanelsHandler = shellActions.closeOperationShellPanels;
closePreviewHandler = shellActions.closePreview;
refreshCurrentHandler = shellActions.refreshCurrent;

watch(() => fileStore.showEditor, (showEditor) => {
  if (showEditor) closePanels();
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
  closeMenus: closeTabContextMenu,
  showNotice: showShellNotice
});

const {
  togglePreviewFromShortcut,
  handleWindowKeyDown
} = useExplorerShortcuts({
  imageViewerVisible,
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

useMainViewLifecycle({
  initialize: async () => {
    fileStore.ensureActiveTab();
    await loadRoot();
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

const editPreviewEntry = (entry: ExplorerEntry) => {
  void openPreviewInEditor(entry);
}

const {
  closeTaskPanelAndFocus,
  closeOperationPanelAndFocus,
  closeDeleteConfirmAndFocus,
  closePropertiesPanelAndFocus,
  closePreviewAndFocus,
  closeImageViewerAndFocus
} = useMainViewPanelClosers({
  editorVisible: () => fileStore.showEditor,
  focusExplorer,
  taskPanelVisible,
  operationPanel,
  deleteConfirm,
  propertiesPanel,
  previewPanelVisible,
  imageViewerVisible,
  closeTaskPanel,
  closeOperationPanel,
  closeDeleteConfirm,
  closePropertiesPanel,
  closePreview,
  closeImageViewer
});

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
            @open-settings="openSettings"
            @toggle-tasks="toggleTaskPanel"
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
          :set-search-input-ref="setSearchInputRef"
          @navigate-back="navigateBack"
          @navigate-forward="navigateForward"
          @navigate-up="navigateUp"
          @refresh="refreshCurrent(true)"
          @breadcrumb-navigate="handleBreadcrumbNavigate"
          @breadcrumb-drop="dropEntriesToPathFolder"
          @update:search-text="updateSearchText"
          @search-enter="focusExplorer"
          @search-escape="handleSearchEscape"
          @clear-search="() => clearSearch()" />

      <div class="workspace-body">
      <sidebar-panel
          :tree-data="treeData"
          :load-data="handleLoad"
          :current-path="fileStore.currentPath"
          @drop-entries="dropEntriesToPathFolder"
          @open-new-tab="openTreeFolderInNewTab"
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
                @open-new-tab="openEntryInNewTab"
                @open-image-viewer="openImageViewer">
            </explorer>
            <shell-notice
                v-if="shellNotice.visible"
                :kind="shellNotice.kind"
                :title="shellNotice.title"
                :message="shellNotice.message"
                @close="closeShellNotice" />
            <upload-drop-overlay
                v-if="uploadDropActive || uploadDropUploading"
                :title="uploadDropTitle"
                :subtitle="uploadDropSubtitle" />
            <task-panel
                v-if="taskPanelVisible"
                :tasks="tasks"
                :loading="tasksLoading"
                :message="taskMessage"
                :last-updated-at="taskLastUpdatedAt"
                :cancel-confirm="taskCancelConfirm"
                @refresh="loadTasks()"
                @close="closeTaskPanelAndFocus"
                @cancel="cancelTaskById"
                @close-cancel="closeTaskCancelConfirm"
                @confirm-cancel="submitTaskCancelConfirm">
            </task-panel>
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
                @submit="submitDeleteConfirm" />
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
                @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)">
            </preview-pane>
          </aside>
        </div>

        <image-viewer
            :visible="imageViewerVisible"
            :entry="imageViewerEntry"
            :entries="imageViewerEntries"
            @close="closeImageViewerAndFocus"
            @select="setImageViewerEntry"
            @download="downloadSelected"
            @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)">
        </image-viewer>

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
  @apply flex h-12 shrink-0 items-center gap-3;
}

.top-actions {
  @apply flex h-full shrink-0 items-center;
}

.workspace {
  @apply mt-3 flex min-h-0 grow flex-col overflow-hidden rounded-xl border shadow-sm backdrop-blur;
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

</style>
