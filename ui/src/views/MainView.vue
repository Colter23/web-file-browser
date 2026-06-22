<script setup lang="ts">
import {computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import {useRouter} from "vue-router";
import FileTree from "../components/FileTree.vue";
import {FileTreeData} from "../class";
import {useFileStore} from "../store";
import {
  cancelTask,
  getFolderData,
  listTasks,
  logout,
} from "../network/api";
import Icon from "../components/Icon.vue";
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
import UploadDropOverlay from "../components/shell/UploadDropOverlay.vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {usePreviewPaneResize} from "../composables/usePreviewPaneResize.ts";
import {useShellNotice} from "../composables/useShellNotice.ts";
import {useFileOperations} from "../composables/useFileOperations.ts";
import {useExplorerTabs} from "../composables/useExplorerTabs.ts";
import {useExplorerPreview} from "../composables/useExplorerPreview.ts";
import {useExplorerNavigation} from "../composables/useExplorerNavigation.ts";
import {useExplorerViewMode} from "../composables/useExplorerViewMode.ts";
import {shouldIgnoreNavigationShortcut, useExplorerShortcuts} from "../composables/useExplorerShortcuts.ts";
import {useTaskPanel} from "../composables/useTaskPanel.ts";
import {useUploadDrop} from "../composables/useUploadDrop.ts";
import {isExtractableArchiveEntry} from "../utils/file-entry.ts";

const EditorPanel = defineAsyncComponent(() => import("../components/editor/EditorPanel.vue"));

type ExplorerExpose = {
  refresh: (path?: string) => Promise<boolean>;
  getSelectedEntry: () => ExplorerEntry | null;
  getSelectedEntries: () => ExplorerEntry[];
  startRename: () => void;
  selectPath: (path: string) => Promise<boolean>;
  selectPaths: (paths: string[], scrollToSelection?: boolean) => Promise<boolean>;
  selectPathForRename: (path: string) => Promise<boolean>;
  selectAllEntries: () => boolean;
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
  width: previewPaneWidth,
  resizing: previewPaneResizing,
  minWidth: previewPaneMinWidth,
  maxWidth: previewPaneMaxWidth,
  areaStyle: browserAreaStyle,
  startResize: startPreviewPaneResize,
  handleResizeMove: handlePreviewPaneResizeMove,
  finishResize: finishPreviewPaneResize,
  resetWidth: resetPreviewPaneWidth,
  handleWindowResize,
  handleResizeKeyDown: handlePreviewPaneResizeKeyDown
} = usePreviewPaneResize();
const {
  notice: shellNotice,
  show: showShellNotice,
  showError: showErrorNotice,
  close: closeShellNotice,
  stopTimer: stopShellNoticeTimer
} = useShellNotice();
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
  showError: showErrorNotice
});
const treeData = ref<FileTreeData[]>([]);
const explorerRef = ref<ExplorerExpose | null>(null);
const contentToolbarRef = ref<ContentToolbarExpose | null>(null);
const deleteConfirmRef = ref<FocusablePanelExpose | null>(null);
const propertiesPanelRef = ref<FocusablePanelExpose | null>(null);
const uploadInput = ref<HTMLInputElement | null>(null);
const searchInput = ref<HTMLInputElement | null>(null);
const searchText = ref("");
const isFiltering = computed(() => Boolean(searchText.value.trim()));
let suppressSelectionPersistence = false;
let suppressScrollPersistence = false;
let tabContextRestoreToken = 0;
let scrollPersistTimer: number | undefined;

const activeTab = computed(() => fileStore.tabs.find(tab => tab.id === fileStore.activeTabId) ?? fileStore.tabs[0]);

const selectedEntry = () => explorerRef.value?.getSelectedEntry() ?? null;

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
  clearPreviewContent,
  closePreviewPanel,
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
  shouldPersistSelection: () => !fileStore.showEditor && !suppressSelectionPersistence,
  persistSelectedPaths: paths => fileStore.setActiveTabSelectedPaths(paths),
  showNotice: showShellNotice
});

const canDownloadSelection = computed(() => singleSelection.value?.type === "file");
const canPreviewSelection = computed(() => singleSelection.value?.type === "file");
const canTogglePreviewPane = computed(() => !fileStore.showEditor);
const canRenameSelection = computed(() => Boolean(singleSelection.value));
const canArchiveSelection = computed(() => hasSelection.value);
const canDeleteSelection = computed(() => hasSelection.value);
const canExtractSelection = computed(() => isExtractableArchiveEntry(singleSelection.value));
const canPasteSelection = computed(() => hasClipboard.value);
const selectionStatusText = computed(() => {
  const selectionText = hasSelection.value ? `已选择 ${selectedCount.value} 项` : "未选择项目";
  return `${selectionText} · ${clipboardText.value}`;
});

const closePanels = () => {
  closePreviewPanel();
  operationPanel.value.visible = false;
  resetDeleteConfirm();
  closePropertiesPanel();
  resetTaskCancelConfirm();
  closeImageViewer();
}

const focusExplorer = async () => {
  if (fileStore.showEditor) return;
  await nextTick();
  explorerRef.value?.focus();
}

const clearSearch = (focus: "explorer" | "search" | false = "explorer") => {
  searchText.value = "";
  if (focus === "search") searchInput.value?.focus();
  if (focus === "explorer") void focusExplorer();
}

const finishTabContextRestore = (token: number) => {
  if (token !== tabContextRestoreToken) return;
  suppressSelectionPersistence = false;
}

const restoreActiveTabSelection = async (paths: string[], token: number, attempt = 0) => {
  if (!paths.length || token !== tabContextRestoreToken) {
    finishTabContextRestore(token);
    return;
  }
  const restored = await explorerRef.value?.selectPaths(paths, false);
  if (restored || token !== tabContextRestoreToken || attempt >= 6) {
    finishTabContextRestore(token);
    return;
  }
  window.setTimeout(() => {
    void restoreActiveTabSelection(paths, token, attempt + 1);
  }, 80);
}

const finishTabScrollRestore = (token: number) => {
  if (token !== tabContextRestoreToken) return;
  suppressScrollPersistence = false;
}

const persistActiveTabScrollTop = (scrollTop: number, tabId = fileStore.activeTabId, path = fileStore.currentPath) => {
  if (suppressScrollPersistence || fileStore.showEditor) return;
  if (scrollPersistTimer) window.clearTimeout(scrollPersistTimer);
  scrollPersistTimer = window.setTimeout(() => {
    scrollPersistTimer = undefined;
    if (suppressScrollPersistence || fileStore.showEditor || tabId !== fileStore.activeTabId || path !== fileStore.currentPath) return;
    fileStore.setActiveTabScrollTop(scrollTop);
  }, 120);
}

const persistCurrentExplorerScrollTop = () => {
  if (suppressScrollPersistence || fileStore.showEditor) return;
  if (scrollPersistTimer) {
    window.clearTimeout(scrollPersistTimer);
    scrollPersistTimer = undefined;
  }
  fileStore.setActiveTabScrollTop(explorerRef.value?.getScrollTop() ?? 0);
}

const syncActiveTabContext = async () => {
  const tab = activeTab.value;
  const selectedPaths = [...(tab?.selectedPaths ?? [])];
  const scrollTop = tab?.scrollTop ?? 0;
  const token = ++tabContextRestoreToken;
  suppressSelectionPersistence = true;
  suppressScrollPersistence = true;
  searchText.value = tab?.filterText ?? "";
  await nextTick();
  await restoreActiveTabSelection(selectedPaths, token);
  await explorerRef.value?.setScrollTop(scrollTop);
  finishTabScrollRestore(token);
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
  closePanels,
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
  currentFolder,
  closePanels,
  syncActiveTabContext,
  persistCurrentExplorerScrollTop,
  showNotice: showShellNotice
});

const handleSearchEscape = () => {
  if (isFiltering.value) {
    clearSearch();
    return;
  }
  searchInput.value?.blur();
  void focusExplorer();
}

const loadRoot = async () => {
  const data = await getFolderData("/");
  treeData.value = fileStore.saveAndConvertFolderData(data);
}

const handleLoad = (node: FileTreeData) => {
  return new Promise<void>(async (resolve) => {
    if (!await fileStore.requestEditorLeave()) {
      resolve();
      return;
    }
    try {
      const data = await getFolderData(node.path);
      node.children = fileStore.saveAndConvertFolderData(data);
      await navigateToPath(data.path, {skipEditorLeave: true});
    } catch (error) {
      showErrorNotice(error, "加载目录失败");
    }
    resolve();
  });
}

onMounted(async () => {
  fileStore.ensureActiveTab();
  await loadRoot();
  await syncActiveTabContext();
  window.addEventListener("keydown", handleWindowKeyDown);
  window.addEventListener("mousedown", handleHistoryMouseDown);
  window.addEventListener("mouseup", handleHistoryMouseUp);
  window.addEventListener("auxclick", handleHistoryAuxClick);
  window.addEventListener("click", closeTabContextMenu);
  window.addEventListener("scroll", closeTabContextMenu, true);
  window.addEventListener("pointermove", handlePreviewPaneResizeMove);
  window.addEventListener("pointerup", finishPreviewPaneResize);
  window.addEventListener("pointercancel", finishPreviewPaneResize);
  window.addEventListener("resize", handleWindowResize);
})

onBeforeUnmount(() => {
  if (scrollPersistTimer) window.clearTimeout(scrollPersistTimer);
  stopShellNoticeTimer();
  stopTaskPolling();
  window.removeEventListener("keydown", handleWindowKeyDown);
  window.removeEventListener("mousedown", handleHistoryMouseDown);
  window.removeEventListener("mouseup", handleHistoryMouseUp);
  window.removeEventListener("auxclick", handleHistoryAuxClick);
  window.removeEventListener("click", closeTabContextMenu);
  window.removeEventListener("scroll", closeTabContextMenu, true);
  window.removeEventListener("pointermove", handlePreviewPaneResizeMove);
  window.removeEventListener("pointerup", finishPreviewPaneResize);
  window.removeEventListener("pointercancel", finishPreviewPaneResize);
  window.removeEventListener("resize", handleWindowResize);
})

watch(() => fileStore.showEditor, (showEditor) => {
  if (showEditor) closePanels();
});

watch(searchText, text => {
  if (fileStore.showEditor) return;
  fileStore.setActiveTabFilterText(text);
});

const refreshCurrent = async (keepSelection = false) => {
  const keepPreview = keepSelection && previewPanelVisible.value && !fileStore.showEditor;
  const selectedPaths = keepSelection ? currentSelection.value.map(entry => entry.path) : [];
  if (keepPreview) {
    clearPreviewContent();
    resetOperationPanel();
    resetDeleteConfirm();
    resetTaskCancelConfirm();
    closeImageViewer();
  } else {
    closePanels();
  }
  if (currentFolder() === "/") {
    await loadRoot();
  }
  await explorerRef.value?.refresh(currentFolder());
  if (selectedPaths.length) {
    const restored = await explorerRef.value?.selectPaths(selectedPaths);
    if (!restored) fileStore.setActiveTabSelectedPaths([]);
  }
}

const closeOperationShellPanels = () => {
  closePreviewPanel();
  resetTaskCancelConfirm();
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
  focusDeleteConfirm: () => deleteConfirmRef.value?.focus(),
  focusPropertiesPanel: () => propertiesPanelRef.value?.focus()
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
  focusSearchInput: () => {
    searchInput.value?.focus();
    searchInput.value?.select();
  },
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

const closePreview = () => {
  closePanels();
}

const openPreviewInEditor = async (entry = previewEntry.value) => {
  if (!entry || entry.type !== "file") return;
  if (!await fileStore.requestEditorLeave()) return;
  closePanels();
  fileStore.openEditor({
    path: entry.path,
    name: entry.name,
    size: entry.size ?? 0,
    extension: entry.extension ?? "",
    modified: entry.modified ?? ""
  });
}

const editPreviewEntry = (entry: ExplorerEntry) => {
  void openPreviewInEditor(entry);
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
          @tab-drag-start="startTabDrag"
          @tab-drag-over="dragOverTab"
          @tab-drag-leave="leaveTabDropTarget"
          @tab-drop="dropTab"
          @tab-drag-end="finishTabDrag"
          @duplicate-tab="duplicateTabFromMenu"
          @close-context-tab="closeTabFromMenu"
          @reopen-closed-tab="reopenClosedTab"
          @close-other-tabs="closeOtherTabsFromMenu"
          @close-right-tabs="closeRightTabsFromMenu" />
      <div class="top-actions">
        <label class="search-box" :class="{active: isFiltering}">
          <input
              ref="searchInput"
              v-model="searchText"
              type="search"
              placeholder="搜索当前文件夹"
              aria-label="搜索当前文件夹"
              title="搜索当前文件夹 (Ctrl+F / Ctrl+E)"
              @keydown.enter.prevent="focusExplorer"
              @keydown.escape.prevent="handleSearchEscape">
          <button v-if="isFiltering" type="button" title="清除筛选" @click.prevent="() => clearSearch()">
            <icon icon="icon-close" />
          </button>
          <icon v-else icon="icon-fenxiang" />
        </label>
        <button class="square-button" title="设置" @click="openSettings">
          <icon icon="icon-setting" size="large" />
        </button>
        <button class="plain-button" @click="signOut">退出</button>
      </div>
    </header>

    <main class="workspace">
      <aside class="sidebar">
        <div class="quick-toolbar">
          <button class="primary-tool" title="上传" @click="uploadInput?.click()">
            <icon icon="icon-upload" size="large" />
            <span>上传</span>
          </button>
          <button class="icon-tool" title="新建文件" @click="openCreatePanel('file')">
            <icon icon="icon-file-add-fill" />
          </button>
          <button class="icon-tool" title="新建文件夹 (Ctrl+Shift+N)" @click="openCreatePanel('folder')">
            <icon icon="icon-folder-add-fill" />
          </button>
        </div>
        <file-tree :data="treeData" :load-data="handleLoad"></file-tree>
      </aside>

      <section class="content-pane">
        <content-toolbar
            ref="contentToolbarRef"
            :can-navigate-back="canNavigateBack"
            :can-navigate-forward="canNavigateForward"
            :can-navigate-up="canNavigateUp"
            :navigate-back-title="navigateBackTitle"
            :navigate-forward-title="navigateForwardTitle"
            :navigate-up-title="navigateUpTitle"
            :view-mode-icon="currentViewModeMeta.icon"
            :view-mode-label="currentViewModeLabel"
            :view-mode-button-title="viewModeButtonTitle"
            :view-mode="fileStore.viewMode"
            :icon-size="fileStore.iconSize"
            :preview-panel-visible="previewPanelVisible"
            :can-toggle-preview-pane="canTogglePreviewPane"
            @navigate-back="navigateBack"
            @navigate-forward="navigateForward"
            @navigate-up="navigateUp"
            @refresh="refreshCurrent(true)"
            @breadcrumb-navigate="handleBreadcrumbNavigate"
            @select-view-mode="selectViewMode"
            @toggle-preview="togglePreviewFromShortcut" />

        <command-bar
            :has-selection="hasSelection"
            :can-paste-selection="canPasteSelection"
            :can-download-selection="canDownloadSelection"
            :can-preview-selection="canPreviewSelection"
            :can-archive-selection="canArchiveSelection"
            :can-extract-selection="canExtractSelection"
            :can-rename-selection="canRenameSelection"
            :can-delete-selection="canDeleteSelection"
            :selection-status-text="selectionStatusText"
            :task-panel-visible="taskPanelVisible"
            :task-button-text="taskButtonText"
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
            @toggle-tasks="toggleTaskPanel" />
        <input ref="uploadInput" class="hidden" type="file" multiple @change="uploadChanged">

        <task-panel
            v-if="taskPanelVisible"
            :tasks="tasks"
            :loading="tasksLoading"
            :message="taskMessage"
            :last-updated-at="taskLastUpdatedAt"
            :cancel-confirm="taskCancelConfirm"
            @refresh="loadTasks()"
            @close="closeTaskPanel"
            @cancel="cancelTaskById"
            @close-cancel="closeTaskCancelConfirm"
            @confirm-cancel="submitTaskCancelConfirm">
        </task-panel>

        <div class="browser-area" :class="{previewing: previewPanelVisible, resizingPreview: previewPaneResizing}" :style="browserAreaStyle">
          <div
              class="browser-main"
              :class="{dropActive: uploadDropActive || uploadDropUploading}"
              @dragenter="handleUploadDragEnter"
              @dragover="handleUploadDragOver"
              @dragleave="handleUploadDragLeave"
              @drop="handleUploadDrop">
            <editor-panel v-show="fileStore.showEditor"></editor-panel>
            <explorer
                ref="explorerRef"
                v-show="!fileStore.showEditor"
                :filter-text="searchText"
                :dimmed-paths="fileClipboardAction === 'cut' ? clipboardPaths : []"
                :can-paste="canPasteSelection"
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
            <operation-panel
                :state="operationPanel"
                @update:name="value => operationPanel.name = value"
                @update:format="value => operationPanel.format = value"
                @close="closeOperationPanel"
                @submit="submitOperationPanel" />
            <delete-confirm-panel
                ref="deleteConfirmRef"
                :state="deleteConfirm"
                @close="closeDeleteConfirm"
                @submit="submitDeleteConfirm" />
            <properties-panel
                ref="propertiesPanelRef"
                :visible="propertiesPanel.visible"
                :entries="propertiesPanel.entries"
                :current-folder="currentFolder()"
                @close="closePropertiesPanel" />
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
                @close="closePreview"
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
            @close="closeImageViewer"
            @select="setImageViewerEntry"
            @download="downloadSelected"
            @notice="payload => showShellNotice(payload.message, payload.kind, payload.title)">
        </image-viewer>
      </section>
    </main>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.main-shell {
  @apply flex h-screen flex-col overflow-hidden bg-[#eef5fb] p-3 text-slate-900;
}

.top-strip {
  @apply flex h-12 shrink-0 items-center gap-3;
}

.top-actions {
  @apply flex h-full shrink-0 items-center gap-3;
}

.search-box {
  @apply flex h-10 w-72 items-center gap-2 rounded-xl border border-white bg-white/70 px-3 shadow-sm backdrop-blur;
}

.search-box.active {
  @apply border-blue-200 bg-white text-blue-700 ring-2 ring-blue-100;
}

.search-box input {
  @apply min-w-0 grow bg-transparent text-sm outline-none placeholder:text-slate-400;
}

.search-box button {
  @apply -mr-1 inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-slate-400 hover:bg-slate-100 hover:text-slate-700;
}

.square-button {
  @apply inline-flex h-11 w-11 items-center justify-center rounded-xl border border-white bg-white/70 text-blue-600 shadow-sm backdrop-blur hover:bg-white;
}

.plain-button {
  @apply h-10 rounded-lg border border-white bg-white/70 px-3 text-sm shadow-sm hover:bg-white;
}

.workspace {
  @apply mt-3 grid min-h-0 grow grid-cols-[17rem_minmax(0,1fr)] gap-3;
}

.sidebar {
  @apply flex min-h-0 flex-col overflow-hidden rounded-xl border border-slate-200 bg-white/65 p-2 shadow-sm backdrop-blur;
}

.quick-toolbar {
  @apply mb-2 grid h-11 shrink-0 grid-cols-[1fr_2.25rem_2.25rem] gap-2;
}

.primary-tool,
.icon-tool {
  @apply inline-flex items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-700 hover:bg-blue-50;
}

.primary-tool {
  @apply gap-2 bg-blue-600 px-3 text-sm font-medium text-white hover:bg-blue-700;
}

.icon-tool {
  @apply h-full w-full;
}

.content-pane {
  @apply relative flex min-h-0 flex-col overflow-hidden rounded-xl border border-slate-200 bg-white/80 shadow-sm backdrop-blur;
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
  @apply bg-blue-50/40;
}

.preview-pane {
  @apply relative flex min-h-0 flex-col border-l border-slate-200 bg-white;
}

.preview-resizer {
  @apply absolute -left-1 top-0 z-10 h-full w-2 cursor-col-resize touch-none;
}

.preview-resizer::after {
  content: "";
  @apply absolute left-1 top-0 h-full w-px bg-transparent;
}

.preview-resizer:hover::after,
.browser-area.resizingPreview .preview-resizer::after {
  @apply bg-blue-500;
}

</style>
