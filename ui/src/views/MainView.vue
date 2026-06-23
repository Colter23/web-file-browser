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
import SidebarPanel from "../components/shell/SidebarPanel.vue";
import UploadDropOverlay from "../components/shell/UploadDropOverlay.vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
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
  shouldPersistSelection,
  persistSelectedPaths,
  showNotice: showShellNotice
});

let closePanelsHandler = () => {};
let closeOperationShellPanelsHandler = () => {};
let closePreviewHandler = () => {};
let refreshCurrentHandler = async (_keepSelection = false) => {};
const closePanels = () => closePanelsHandler();
const closeOperationShellPanels = () => closeOperationShellPanelsHandler();
const closePreview = () => closePreviewHandler();
const refreshCurrent = (keepSelection = false) => refreshCurrentHandler(keepSelection);

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

const {treeData, loadRoot, handleLoad} = useFileTreeLoader({
  getFolderData,
  navigateToPath,
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
  canPasteSelection,
  selectionStatusText
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
  refreshExplorer: path => explorerRef.value?.refresh(path) ?? Promise.resolve(false),
  selectPaths: paths => explorerRef.value?.selectPaths(paths) ?? Promise.resolve(false),
  clearPersistedSelection: () => fileStore.setActiveTabSelectedPaths([]),
  closePreviewPanel,
  clearPreviewContent,
  closeImageViewer,
  hideOperationPanel: () => operationPanel.value.visible = false,
  resetOperationPanel,
  resetDeleteConfirm,
  closePropertiesPanel,
  resetTaskCancelConfirm
});

closePanelsHandler = shellActions.closePanels;
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
          @duplicate-tab="duplicateTabFromMenu"
          @close-context-tab="closeTabFromMenu"
          @reopen-closed-tab="reopenClosedTab"
          @close-other-tabs="closeOtherTabsFromMenu"
          @close-right-tabs="closeRightTabsFromMenu"
          @close-context-menu="closeTabContextMenu" />
      <div class="top-actions">
        <label class="search-box" :class="{active: isFiltering}">
          <input
              :ref="setSearchInputRef"
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

    <main class="workspace" :class="{resizingSidebar: sidebarResizing}" :style="workspaceStyle">
      <sidebar-panel
          :tree-data="treeData"
          :load-data="handleLoad"
          :current-path="fileStore.currentPath"
          @upload="uploadInput?.click()"
          @create-file="openCreatePanel('file')"
          @create-folder="openCreatePanel('folder')" />

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
            @close="closeTaskPanelAndFocus"
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
  @apply mt-3 grid min-h-0 grow gap-0;
  --sidebar-width: 17rem;
  grid-template-columns: var(--sidebar-width) 0.75rem minmax(0, 1fr);
}

.workspace.resizingSidebar {
  @apply cursor-col-resize select-none;
}

.sidebar-resizer {
  @apply relative z-20 h-full cursor-col-resize touch-none outline-none;
}

.sidebar-resizer::after {
  content: "";
  @apply absolute top-2 bottom-2 left-1/2 w-px -translate-x-1/2 rounded-full bg-transparent;
}

.sidebar-resizer:hover::after,
.sidebar-resizer:focus-visible::after,
.workspace.resizingSidebar .sidebar-resizer::after {
  @apply bg-blue-500;
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
