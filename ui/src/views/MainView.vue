<script setup lang="ts">
import {computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import {useRouter} from "vue-router";
import FileTree from "../components/FileTree.vue";
import {ArchiveFormat, ExplorerIconSize, ExplorerViewMode, FileInfo, FileTreeData, TaskStatus} from "../class";
import {useFileStore} from "../store";
import {
  cancelTask,
  createArchiveTask,
  createCopyTask,
  createDeleteTask,
  createEntry,
  createExtractTask,
  createMoveTask,
  downloadFile,
  getFolderData,
  listTasks,
  logout,
  moveEntry,
  uploadFiles
} from "../network/api";
import Icon from "../components/Icon.vue";
import Explorer from "../components/explorer/Explorer.vue";
import Breadcrumb from "../components/Breadcrumb.vue";
import ImageViewer from "../components/viewer/ImageViewer.vue";
import PreviewPane from "../components/viewer/PreviewPane.vue";
import TaskPanel from "../components/tasks/TaskPanel.vue";
import TabStrip from "../components/tabs/TabStrip.vue";
import OperationPanel from "../components/operations/OperationPanel.vue";
import DeleteConfirmPanel from "../components/operations/DeleteConfirmPanel.vue";
import PropertiesPanel from "../components/operations/PropertiesPanel.vue";

const EditorPanel = defineAsyncComponent(() => import("../components/editor/EditorPanel.vue"));

type ExplorerEntry = {
  type: "folder" | "file";
  name: string;
  path: string;
  modified?: string;
  size?: number;
  extension?: string;
  file?: FileInfo;
}

type CopyPathPayload = {
  paths: string[];
}

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

type BreadcrumbExpose = {
  focusInput: () => void;
}

type FocusablePanelExpose = {
  focus: () => void;
}

type FileClipboardAction = "copy" | "cut";

type OperationPanelKind = "createFile" | "createFolder" | "archive" | "extract";

type OperationPanelState = {
  visible: boolean;
  kind: OperationPanelKind | null;
  title: string;
  message: string;
  primaryText: string;
  name: string;
  format: ArchiveFormat;
  entries: ExplorerEntry[];
  sourceEntry: ExplorerEntry | null;
  submitting: boolean;
}

type DeleteConfirmState = {
  visible: boolean;
  entries: ExplorerEntry[];
  submitting: boolean;
  error: string;
}

type PropertiesPanelState = {
  visible: boolean;
  entries: ExplorerEntry[];
}

type TaskCancelConfirmState = {
  visible: boolean;
  task: TaskStatus | null;
  submitting: boolean;
  error: string;
}

type ShellNoticeKind = "info" | "success" | "warning" | "error";

type ShellNoticeState = {
  visible: boolean;
  kind: ShellNoticeKind;
  title: string;
  message: string;
}

type TabContextMenuState = {
  visible: boolean;
  x: number;
  y: number;
  tabId: string;
}

type TabDropPlacement = "before" | "after";

type RenamePayload = {
  entry: ExplorerEntry;
  name: string;
}

type DropEntriesPayload = {
  entries: ExplorerEntry[];
  target: ExplorerEntry;
  action: "copy" | "move";
}

type DropToCurrentFolderPayload = {
  entries: ExplorerEntry[];
  action: "copy" | "move";
}

type ImageViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

type NavigateToPathOptions = {
  skipEditorLeave?: boolean;
  focusExplorer?: boolean;
}

const viewModeOrder: ExplorerViewMode[] = ["details", "list", "icons", "tiles"];
const viewModeMeta: Record<ExplorerViewMode, {label: string; icon: string}> = {
  details: {label: "详细信息", icon: "icon-view-list"},
  list: {label: "列表", icon: "icon-listview"},
  icons: {label: "图标", icon: "icon-viewgrid"},
  tiles: {label: "平铺", icon: "icon-file-common-filling"}
};
const viewShortcutMap: Record<string, {mode: ExplorerViewMode; iconSize: ExplorerIconSize; label: string}> = {
  Digit1: {mode: "icons", iconSize: "large", label: "大图标"},
  Digit2: {mode: "icons", iconSize: "large", label: "大图标"},
  Digit3: {mode: "icons", iconSize: "medium", label: "中图标"},
  Digit4: {mode: "icons", iconSize: "small", label: "小图标"},
  Digit5: {mode: "list", iconSize: "small", label: "列表"},
  Digit6: {mode: "details", iconSize: "small", label: "详细信息"},
  Digit7: {mode: "tiles", iconSize: "medium", label: "平铺"}
};

const viewShortcut = (code: string) => viewShortcutMap[code] ?? viewShortcutMap[code.replace("Numpad", "Digit")];
const previewPaneStorageKey = "explorer.previewPaneWidth";
const previewPaneDefaultWidth = 352;
const previewPaneMinWidth = 280;
const previewPaneMaxWidth = 720;
const previewPaneViewportReserve = 520;

const previewPaneMaxForViewport = () => {
  if (typeof window === "undefined") return previewPaneMaxWidth;
  return Math.max(previewPaneMinWidth, Math.min(previewPaneMaxWidth, window.innerWidth - previewPaneViewportReserve));
}

const clampPreviewPaneWidth = (width: number) => {
  const safeWidth = Number.isFinite(width) ? width : previewPaneDefaultWidth;
  return Math.round(Math.min(Math.max(safeWidth, previewPaneMinWidth), previewPaneMaxForViewport()));
}

const readPreviewPaneWidth = () => {
  if (typeof localStorage === "undefined") return clampPreviewPaneWidth(previewPaneDefaultWidth);
  try {
    const raw = localStorage.getItem(previewPaneStorageKey);
    return clampPreviewPaneWidth(raw ? Number(raw) : previewPaneDefaultWidth);
  } catch {
    return clampPreviewPaneWidth(previewPaneDefaultWidth);
  }
}

const writePreviewPaneWidth = (width: number) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(previewPaneStorageKey, String(clampPreviewPaneWidth(width)));
  } catch {
    // 本地存储不可用时，只保留本次会话里的宽度。
  }
}

const router = useRouter();
const fileStore = useFileStore();
const treeData = ref<FileTreeData[]>([]);
const explorerRef = ref<ExplorerExpose | null>(null);
const breadcrumbRef = ref<BreadcrumbExpose | null>(null);
const deleteConfirmRef = ref<FocusablePanelExpose | null>(null);
const propertiesPanelRef = ref<FocusablePanelExpose | null>(null);
const uploadInput = ref<HTMLInputElement | null>(null);
const searchInput = ref<HTMLInputElement | null>(null);
const uploadDropActive = ref(false);
const uploadDropUploading = ref(false);
const taskPanelVisible = ref(false);
const tasksLoading = ref(false);
const tasks = ref<TaskStatus[]>([]);
const taskMessage = ref("");
const taskLastUpdatedAt = ref("");
const searchText = ref("");
const isFiltering = computed(() => Boolean(searchText.value.trim()));
const previewPanelVisible = ref(false);
const previewEntry = ref<ExplorerEntry | null>(null);
const previewReloadKey = ref(0);
const previewPaneWidth = ref(readPreviewPaneWidth());
const previewPaneResizing = ref(false);
const imageViewerVisible = ref(false);
const imageViewerEntry = ref<ExplorerEntry | null>(null);
const imageViewerEntries = ref<ExplorerEntry[]>([]);
const currentSelection = ref<ExplorerEntry[]>([]);
const fileClipboardAction = ref<FileClipboardAction | null>(null);
const fileClipboardEntries = ref<ExplorerEntry[]>([]);
const creatingShortcutFolder = ref(false);
const operationPanel = ref<OperationPanelState>({
  visible: false,
  kind: null,
  title: "",
  message: "",
  primaryText: "确定",
  name: "",
  format: "zip",
  entries: [],
  sourceEntry: null,
  submitting: false
});
const deleteConfirm = ref<DeleteConfirmState>({
  visible: false,
  entries: [],
  submitting: false,
  error: ""
});
const propertiesPanel = ref<PropertiesPanelState>({
  visible: false,
  entries: []
});
const taskCancelConfirm = ref<TaskCancelConfirmState>({
  visible: false,
  task: null,
  submitting: false,
  error: ""
});
const shellNotice = ref<ShellNoticeState>({
  visible: false,
  kind: "info",
  title: "提示",
  message: ""
});
const tabContextMenu = ref<TabContextMenuState>({
  visible: false,
  x: 0,
  y: 0,
  tabId: ""
});
const draggingTabId = ref("");
const tabDropTargetId = ref("");
const tabDropPlacement = ref<TabDropPlacement | "">("");
let uploadDragDepth = 0;
let taskPollTimer: number | undefined;
let shellNoticeTimer: number | undefined;
let previewPaneResizeStartX = 0;
let previewPaneResizeStartWidth = 0;
const tabContextMenuWidth = 184;
const tabContextMenuHeight = 220;
let suppressSelectionPersistence = false;
let suppressScrollPersistence = false;
let tabContextRestoreToken = 0;
let scrollPersistTimer: number | undefined;
let historyMouseButton = -1;

const activeTab = computed(() => fileStore.tabs.find(tab => tab.id === fileStore.activeTabId) ?? fileStore.tabs[0]);
const tabContextTarget = computed(() => fileStore.tabs.find(tab => tab.id === tabContextMenu.value.tabId) ?? null);
const tabContextIndex = computed(() => fileStore.tabs.findIndex(tab => tab.id === tabContextMenu.value.tabId));
const canCloseTabContext = computed(() => fileStore.tabs.length > 1);
const canCloseOtherTabsContext = computed(() => fileStore.tabs.length > 1 && Boolean(tabContextTarget.value));
const canCloseRightTabsContext = computed(() => tabContextIndex.value >= 0 && tabContextIndex.value < fileStore.tabs.length - 1);
const canNavigateBack = computed(() => Boolean(activeTab.value?.backStack?.length));
const canNavigateForward = computed(() => Boolean(activeTab.value?.forwardStack?.length));
const canNavigateUp = computed(() => currentFolder() !== "/");
const navigateBackTarget = computed(() => {
  const stack = activeTab.value?.backStack ?? [];
  return stack[stack.length - 1] ?? "";
});
const navigateForwardTarget = computed(() => activeTab.value?.forwardStack?.[0] ?? "");
const navigateUpTarget = computed(() => canNavigateUp.value ? parentPath(currentFolder()) : "");
const navigateBackTitle = computed(() => navigateBackTarget.value ? `后退到 ${navigateBackTarget.value} (Alt+← / 鼠标后退键)` : "后退 (Alt+← / 鼠标后退键)");
const navigateForwardTitle = computed(() => navigateForwardTarget.value ? `前进到 ${navigateForwardTarget.value} (Alt+→ / 鼠标前进键)` : "前进 (Alt+→ / 鼠标前进键)");
const navigateUpTitle = computed(() => navigateUpTarget.value ? `返回上级 ${navigateUpTarget.value} (Alt+↑)` : "返回上级 (Alt+↑)");
const selectedList = computed(() => currentSelection.value);
const selectedCount = computed(() => selectedList.value.length);
const hasSelection = computed(() => selectedCount.value > 0);
const singleSelection = computed(() => selectedCount.value === 1 ? selectedList.value[0] : null);
const clipboardPaths = computed(() => fileClipboardEntries.value.map(entry => entry.path));
const hasClipboard = computed(() => Boolean(fileClipboardAction.value && fileClipboardEntries.value.length));
const clipboardText = computed(() => {
  if (!hasClipboard.value) return "剪贴板为空";
  const actionText = fileClipboardAction.value === "cut" ? "剪切" : "复制";
  return `${actionText} ${fileClipboardEntries.value.length} 项`;
});
const selectionStatusText = computed(() => {
  const selectionText = hasSelection.value ? `已选择 ${selectedCount.value} 项` : "未选择项目";
  return `${selectionText} · ${clipboardText.value}`;
});
const canDownloadSelection = computed(() => singleSelection.value?.type === "file");
const canPreviewSelection = computed(() => singleSelection.value?.type === "file");
const canTogglePreviewPane = computed(() => !fileStore.showEditor);
const canRenameSelection = computed(() => Boolean(singleSelection.value));
const canArchiveSelection = computed(() => hasSelection.value);
const canDeleteSelection = computed(() => hasSelection.value);
const canExtractSelection = computed(() => isArchiveFile(singleSelection.value));
const canPasteSelection = computed(() => hasClipboard.value);
const currentViewModeMeta = computed(() => viewModeMeta[fileStore.viewMode]);
const nextViewMode = computed(() => {
  const index = viewModeOrder.indexOf(fileStore.viewMode);
  return viewModeOrder[(index + 1) % viewModeOrder.length];
});
const viewModeButtonTitle = computed(() => `当前：${currentViewModeMeta.value.label}，切换到${viewModeMeta[nextViewMode.value].label}。Ctrl+Shift+1-7 可直接切换查看模式`);
const activeTaskCount = computed(() => tasks.value.filter(task => task.state === "running" || task.state === "queued").length);
const hasActiveTasks = computed(() => activeTaskCount.value > 0);
const taskButtonText = computed(() => hasActiveTasks.value ? `任务 ${activeTaskCount.value}` : "任务");
const operationPanelNameLabel = computed(() => {
  switch (operationPanel.value.kind) {
    case "createFile":
      return "文件名";
    case "createFolder":
      return "文件夹名";
    case "archive":
      return "压缩包名称";
    case "extract":
      return "解压到文件夹";
    default:
      return "名称";
  }
});
const browserAreaStyle = computed(() => ({
  "--preview-pane-width": `${previewPaneWidth.value}px`
}));

const shellNoticeLabel = computed(() => ({
  info: "提示",
  success: "完成",
  warning: "需要注意",
  error: "操作失败"
}[shellNotice.value.kind]));

const errorMessage = (error: unknown, fallback: string) => {
  return error instanceof Error && error.message ? error.message : fallback;
}

const stopShellNoticeTimer = () => {
  if (shellNoticeTimer) {
    window.clearTimeout(shellNoticeTimer);
    shellNoticeTimer = undefined;
  }
}

const closeShellNotice = () => {
  stopShellNoticeTimer();
  shellNotice.value.visible = false;
}

const setPreviewPaneWidth = (width: number, persist = true) => {
  previewPaneWidth.value = clampPreviewPaneWidth(width);
  if (persist) writePreviewPaneWidth(previewPaneWidth.value);
}

const resetPreviewPaneWidth = () => {
  setPreviewPaneWidth(previewPaneDefaultWidth);
}

const finishPreviewPaneResize = () => {
  if (previewPaneResizing.value) writePreviewPaneWidth(previewPaneWidth.value);
  previewPaneResizing.value = false;
}

const resizePreviewPane = (clientX: number) => {
  setPreviewPaneWidth(previewPaneResizeStartWidth + previewPaneResizeStartX - clientX, false);
}

const handlePreviewPaneResizeMove = (event: PointerEvent) => {
  if (!previewPaneResizing.value) return;
  event.preventDefault();
  resizePreviewPane(event.clientX);
}

const startPreviewPaneResize = (event: PointerEvent) => {
  if (event.button !== 0) return;
  event.preventDefault();
  previewPaneResizeStartX = event.clientX;
  previewPaneResizeStartWidth = previewPaneWidth.value;
  previewPaneResizing.value = true;
}

const handleWindowResize = () => {
  setPreviewPaneWidth(previewPaneWidth.value);
}

const adjustPreviewPaneWidth = (delta: number) => {
  setPreviewPaneWidth(previewPaneWidth.value + delta);
}

const handlePreviewPaneResizeKeyDown = (event: KeyboardEvent) => {
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    adjustPreviewPaneWidth(event.shiftKey ? 64 : 24);
    return;
  }
  if (event.key === "ArrowRight") {
    event.preventDefault();
    adjustPreviewPaneWidth(event.shiftKey ? -64 : -24);
    return;
  }
  if (event.key === "Home") {
    event.preventDefault();
    setPreviewPaneWidth(previewPaneMinWidth);
    return;
  }
  if (event.key === "End") {
    event.preventDefault();
    setPreviewPaneWidth(previewPaneMaxWidth);
  }
}

const resetImageViewerState = () => {
  imageViewerVisible.value = false;
  imageViewerEntry.value = null;
  imageViewerEntries.value = [];
}

const closeImageViewer = () => {
  const nextPreviewEntry = previewPanelVisible.value && imageViewerEntry.value?.path !== previewEntry.value?.path
      ? imageViewerEntry.value
      : null;
  resetImageViewerState();
  if (nextPreviewEntry) void setPreviewEntry(nextPreviewEntry, true);
}

const setImageViewerEntry = (entry: ExplorerEntry) => {
  imageViewerEntry.value = entry;
}

const showShellNotice = (message: string, kind: ShellNoticeKind = "info", title?: string, timeoutMs?: number) => {
  stopShellNoticeTimer();
  shellNotice.value = {
    visible: true,
    kind,
    title: title ?? ({
      info: "提示",
      success: "完成",
      warning: "需要注意",
      error: "操作失败"
    }[kind]),
    message
  };
  const duration = timeoutMs ?? (kind === "error" ? 7000 : 3500);
  if (duration > 0) {
    shellNoticeTimer = window.setTimeout(closeShellNotice, duration);
  }
}

const showErrorNotice = (error: unknown, fallback: string, title = "操作失败") => {
  showShellNotice(errorMessage(error, fallback), "error", title);
}

const clearPreviewContent = () => {
  previewEntry.value = null;
}

const closePropertiesPanel = () => {
  propertiesPanel.value = {
    visible: false,
    entries: []
  };
}

const closePanels = () => {
  previewPanelVisible.value = false;
  clearPreviewContent();
  operationPanel.value.visible = false;
  resetDeleteConfirm();
  closePropertiesPanel();
  resetTaskCancelConfirm();
  closeImageViewer();
}

const clearSearch = (focus = true) => {
  searchText.value = "";
  if (focus) searchInput.value?.focus();
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

const handleSearchEscape = () => {
  if (isFiltering.value) {
    clearSearch();
    return;
  }
  searchInput.value?.blur();
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

const currentFolder = () => fileStore.currentPath || "/";

const uploadDropTitle = computed(() => uploadDropUploading.value ? "正在上传文件..." : "释放鼠标上传文件");
const uploadDropSubtitle = computed(() => uploadDropUploading.value ? `目标：${currentFolder()}` : `上传到 ${currentFolder()}`);

const canCancelTask = (task: TaskStatus) => task.state === "queued" || task.state === "running";

const shortTaskId = (id: string) => id.slice(0, 8);

const updateTaskRefreshTime = () => {
  taskLastUpdatedAt.value = new Intl.DateTimeFormat("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit"
  }).format(new Date());
}

const stopTaskPolling = () => {
  if (taskPollTimer) {
    window.clearTimeout(taskPollTimer);
    taskPollTimer = undefined;
  }
}

const scheduleTaskPolling = () => {
  stopTaskPolling();
  if (!taskPanelVisible.value || !hasActiveTasks.value) return;
  taskPollTimer = window.setTimeout(() => {
    void loadTasks(true);
  }, 1500);
}

const loadTasks = async (silent = false) => {
  if (!silent) tasksLoading.value = true;
  try {
    tasks.value = await listTasks();
    updateTaskRefreshTime();
    scheduleTaskPolling();
  } catch (error) {
    stopTaskPolling();
    showErrorNotice(error, "加载任务失败", "任务加载失败");
  } finally {
    if (!silent) tasksLoading.value = false;
  }
}

const toggleTaskPanel = async () => {
  taskPanelVisible.value = !taskPanelVisible.value;
  if (taskPanelVisible.value) {
    await loadTasks();
  } else {
    stopTaskPolling();
  }
}

const closeTaskPanel = () => {
  taskPanelVisible.value = false;
  stopTaskPolling();
}

const cancelTaskById = async (task: TaskStatus) => {
  if (!canCancelTask(task)) return;
  taskCancelConfirm.value = {
    visible: true,
    task,
    submitting: false,
    error: ""
  };
}

const submitTaskCancelConfirm = async () => {
  const task = taskCancelConfirm.value.task;
  if (!task || !canCancelTask(task) || taskCancelConfirm.value.submitting) return;
  taskCancelConfirm.value.submitting = true;
  taskCancelConfirm.value.error = "";
  try {
    await cancelTask(task.id);
    taskMessage.value = `已发送取消请求：${shortTaskId(task.id)}`;
    resetTaskCancelConfirm();
    await loadTasks();
  } catch (error) {
    taskCancelConfirm.value.error = error instanceof Error ? error.message : "取消任务失败";
  } finally {
    if (taskCancelConfirm.value.visible) taskCancelConfirm.value.submitting = false;
  }
}

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

const runOperation = async (operation: () => Promise<void>) => {
  try {
    await operation();
    await refreshCurrent();
  } catch (error) {
    showErrorNotice(error, "操作失败");
  }
}

const resetOperationPanel = () => {
  operationPanel.value = {
    visible: false,
    kind: null,
    title: "",
    message: "",
    primaryText: "确定",
    name: "",
    format: "zip",
    entries: [],
    sourceEntry: null,
    submitting: false
  };
}

const resetDeleteConfirm = () => {
  deleteConfirm.value = {
    visible: false,
    entries: [],
    submitting: false,
    error: ""
  };
}

const resetTaskCancelConfirm = () => {
  taskCancelConfirm.value = {
    visible: false,
    task: null,
    submitting: false,
    error: ""
  };
}

const closeDeleteConfirm = () => {
  if (deleteConfirm.value.submitting) return;
  resetDeleteConfirm();
}

const showProperties = async (entries = selectedEntries()) => {
  if (!entries.length) {
    showShellNotice("请选择文件或文件夹", "warning");
    return;
  }
  clearPreviewContent();
  previewPanelVisible.value = false;
  resetOperationPanel();
  resetDeleteConfirm();
  resetTaskCancelConfirm();
  closeImageViewer();
  propertiesPanel.value = {
    visible: true,
    entries
  };
  await nextTick();
  propertiesPanelRef.value?.focus();
}

const closeTaskCancelConfirm = () => {
  if (taskCancelConfirm.value.submitting) return;
  resetTaskCancelConfirm();
}

const openOperationPanel = (next: Omit<OperationPanelState, "visible" | "submitting">) => {
  closePanels();
  operationPanel.value = {
    ...next,
    visible: true,
    submitting: false
  };
}

const closeOperationPanel = () => {
  if (operationPanel.value.submitting) return;
  resetOperationPanel();
}

const openCreatePanel = (type: "file" | "folder") => {
  openOperationPanel({
    kind: type === "file" ? "createFile" : "createFolder",
    title: type === "file" ? "新建文件" : "新建文件夹",
    message: `位置：${currentFolder()}`,
    primaryText: "创建",
    name: type === "file" ? "新建文件.txt" : "新建文件夹",
    format: "zip",
    entries: [],
    sourceEntry: null
  });
}

const createFolderFromShortcut = async () => {
  if (creatingShortcutFolder.value) return;
  creatingShortcutFolder.value = true;
  closePanels();
  const folderName = "新建文件夹";
  try {
    const created = await createEntry(currentFolder(), "folder", folderName);
    taskMessage.value = `已创建：${folderName}`;
    await refreshCurrent();
    const renamed = await explorerRef.value?.selectPathForRename(created.path);
    if (!renamed) showShellNotice("新文件夹已创建，但当前页未找到它，请刷新或调整排序后重命名。", "warning");
  } catch (error) {
    showErrorNotice(error, "新建文件夹失败");
  } finally {
    creatingShortcutFolder.value = false;
  }
}

const selectedEntry = () => explorerRef.value?.getSelectedEntry() ?? null;

const selectedEntries = (fallback?: ExplorerEntry | null) => {
  const selected = explorerRef.value?.getSelectedEntries() ?? [];
  if (selected.length) return selected;
  return fallback ? [fallback] : [];
}

const handleSelectionChange = (entries: ExplorerEntry[]) => {
  currentSelection.value = entries;
  if (!fileStore.showEditor && !suppressSelectionPersistence) fileStore.setActiveTabSelectedPaths(entries.map(entry => entry.path));
  if (!previewPanelVisible.value || fileStore.showEditor) return;
  const entry = entries.length === 1 ? entries[0] : null;
  if (entry?.type === "file") {
    void setPreviewEntry(entry);
  } else {
    clearPreviewContent();
  }
}

const singleSelectedEntry = (entry = selectedEntry()) => {
  const selected = selectedEntries(entry);
  if (selected.length > 1) return null;
  return selected[0] ?? null;
}

const normalizePathText = (path: string) => {
  let normalized = path.trim() || "/";
  normalized = normalized.replace(/\/+/g, "/");
  if (!normalized.startsWith("/")) normalized = `/${normalized}`;
  if (normalized.length > 1) normalized = normalized.replace(/\/+$/, "");
  return normalized || "/";
}

const parentPath = (path: string) => {
  const parts = normalizePathText(path).split("/").filter(Boolean);
  parts.pop();
  return parts.length ? `/${parts.join("/")}` : "/";
}

const joinPath = (base: string, name: string) => {
  return base === "/" ? `/${name}` : `${base.replace(/\/$/, "")}/${name}`;
}

const isSameOrDescendantPath = (path: string, parent: string) => {
  const normalizedPath = path.replace(/\/$/, "") || "/";
  const normalizedParent = parent.replace(/\/$/, "") || "/";
  return normalizedPath === normalizedParent || normalizedPath.startsWith(`${normalizedParent}/`);
}

const archiveStem = (name: string) => {
  const lower = name.toLowerCase();
  if (lower.endsWith(".tar.gz")) return name.slice(0, -7);
  if (lower.endsWith(".tgz")) return name.slice(0, -4);
  if (lower.endsWith(".zip")) return name.slice(0, -4);
  return name;
}

const archiveExtension = (format: ArchiveFormat) => format === "tarGz" ? ".tar.gz" : ".zip";

const isArchiveFile = (entry: ExplorerEntry | null): entry is ExplorerEntry & { type: "file" } => {
  if (!entry || entry.type !== "file") return false;
  const name = entry.name.toLowerCase();
  return name.endsWith(".zip") || name.endsWith(".tar.gz") || name.endsWith(".tgz");
}

const taskStarted = async (id: string, label = "后台任务") => {
  taskMessage.value = `${label}已创建：${shortTaskId(id)}`;
  taskPanelVisible.value = true;
  await loadTasks();
}

const startRenameSelected = () => {
  if (!singleSelection.value) {
    showShellNotice("请选择一个文件或文件夹", "warning");
    return;
  }
  explorerRef.value?.startRename();
}

const renameSelected = async ({entry, name}: RenamePayload) => {
  const nextName = name.trim();
  if (!nextName || nextName === entry.name) return;
  try {
    const renamed = await moveEntry(entry.path, joinPath(parentPath(entry.path), nextName));
    taskMessage.value = `已重命名：${nextName}`;
    await refreshCurrent();
    const selected = await explorerRef.value?.selectPath(renamed.path);
    if (!selected) showShellNotice("已重命名，但当前页未找到该项目，请刷新或调整排序后查看。", "warning");
  } catch (error) {
    showErrorNotice(error, "重命名失败", "重命名失败");
  }
}

const deleteSelected = async (entry = selectedEntry()) => {
  const entries = selectedEntries(entry);
  if (!entries.length) {
    showShellNotice("请选择文件或文件夹", "warning");
    return;
  }
  closePanels();
  deleteConfirm.value = {
    visible: true,
    entries,
    submitting: false,
    error: ""
  };
  await nextTick();
  deleteConfirmRef.value?.focus();
}

const submitDeleteConfirm = async () => {
  const entries = deleteConfirm.value.entries;
  if (!entries.length || deleteConfirm.value.submitting) return;
  deleteConfirm.value.submitting = true;
  deleteConfirm.value.error = "";
  try {
    const task = await createDeleteTask(entries.map(item => item.path));
    await taskStarted(task.id, "删除任务");
    if (fileClipboardAction.value === "cut") {
      const deleted = new Set(entries.map(item => item.path));
      fileClipboardEntries.value = fileClipboardEntries.value.filter(item => !deleted.has(item.path));
      if (!fileClipboardEntries.value.length) fileClipboardAction.value = null;
    }
    resetDeleteConfirm();
    await refreshCurrent();
  } catch (error) {
    deleteConfirm.value.error = error instanceof Error ? error.message : "创建删除任务失败";
  } finally {
    if (deleteConfirm.value.visible) deleteConfirm.value.submitting = false;
  }
}

const downloadSelected = async (entry = singleSelectedEntry()) => {
  if (!entry || entry.type !== "file") {
    showShellNotice("请选择一个文件", "warning");
    return;
  }
  try {
    const blob = await downloadFile(entry.path);
    const url = window.URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = entry.name;
    anchor.click();
    window.URL.revokeObjectURL(url);
  } catch (error) {
    showErrorNotice(error, "下载失败", "下载失败");
  }
}

const setFileClipboard = (action: FileClipboardAction, entry = selectedEntry()) => {
  const entries = selectedEntries(entry);
  if (!entries.length) {
    showShellNotice("请选择文件或文件夹", "warning");
    return;
  }
  fileClipboardAction.value = action;
  fileClipboardEntries.value = entries;
  taskMessage.value = `${action === "cut" ? "已剪切" : "已复制"} ${entries.length} 项`;
  showShellNotice(taskMessage.value, "success", "剪贴板已更新");
}

const copySelected = (entry?: ExplorerEntry) => {
  setFileClipboard("copy", entry ?? selectedEntry());
}

const cutSelected = (entry?: ExplorerEntry) => {
  setFileClipboard("cut", entry ?? selectedEntry());
}

const copyEntryPaths = async ({paths}: CopyPathPayload) => {
  const normalizedPaths = paths.map(path => path.trim()).filter(Boolean);
  if (!normalizedPaths.length) return;
  try {
    await navigator.clipboard.writeText(normalizedPaths.join("\n"));
    const message = normalizedPaths.length === 1 ? "已复制路径" : `已复制 ${normalizedPaths.length} 个路径`;
    showShellNotice(message, "success", "路径已复制");
  } catch {
    showShellNotice("浏览器未允许写入剪贴板，请手动复制路径。", "error", "复制路径失败");
  }
}

const pasteSelected = async () => {
  if (!hasClipboard.value || !fileClipboardAction.value) {
    showShellNotice("剪贴板为空", "warning");
    return;
  }
  const targetPath = currentFolder();
  const entries = fileClipboardEntries.value;
  const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(targetPath, entry.path));
  if (nestedFolder) {
    showShellNotice(`不能将 ${nestedFolder.name} 粘贴到它自身或子文件夹中`, "warning");
    return;
  }
  const sameFolder = entries.some(entry => parentPath(entry.path) === targetPath);
  if (fileClipboardAction.value === "cut" && sameFolder) {
    showShellNotice("剪切项已经在当前文件夹中", "warning");
    return;
  }
  try {
    const sources = entries.map(item => item.path);
    const task = fileClipboardAction.value === "cut"
        ? await createMoveTask(sources, targetPath)
        : await createCopyTask(sources, targetPath);
    await taskStarted(task.id, fileClipboardAction.value === "cut" ? "移动任务" : "复制任务");
    if (fileClipboardAction.value === "cut") {
      fileClipboardAction.value = null;
      fileClipboardEntries.value = [];
    }
    await refreshCurrent();
  } catch (error) {
    showErrorNotice(error, "创建粘贴任务失败", "粘贴失败");
  }
}

const removeMovedEntriesFromCutClipboard = (sources: string[]) => {
  if (fileClipboardAction.value !== "cut") return;
  const moved = new Set(sources);
  fileClipboardEntries.value = fileClipboardEntries.value.filter(item => !moved.has(item.path));
  if (!fileClipboardEntries.value.length) fileClipboardAction.value = null;
}

const runDroppedEntriesTask = async (entries: ExplorerEntry[], targetPath: string, action: "copy" | "move") => {
  if (!entries.length) return;
  const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(targetPath, entry.path));
  if (nestedFolder) {
    showShellNotice(`不能将 ${nestedFolder.name} 放入它自身或子文件夹中`, "warning");
    return;
  }
  const sameFolder = entries.some(entry => parentPath(entry.path) === targetPath);
  if (action === "move" && sameFolder) {
    taskMessage.value = "拖拽目标已经是当前位置";
    return;
  }
  try {
    const sources = entries.map(item => item.path);
    const task = action === "copy"
        ? await createCopyTask(sources, targetPath)
        : await createMoveTask(sources, targetPath);
    await taskStarted(task.id, action === "copy" ? "复制任务" : "移动任务");
    if (action === "move") removeMovedEntriesFromCutClipboard(sources);
    await refreshCurrent();
  } catch (error) {
    showErrorNotice(error, "创建拖拽任务失败", "拖拽失败");
  }
}

const dropEntriesToFolder = async ({entries, target, action}: DropEntriesPayload) => {
  if (target.type !== "folder") return;
  await runDroppedEntriesTask(entries, target.path, action);
}

const dropEntriesToCurrentFolder = async ({entries, action}: DropToCurrentFolderPayload) => {
  await runDroppedEntriesTask(entries, currentFolder(), action);
}

const archiveSelected = (entry = selectedEntry()) => {
  const entries = selectedEntries(entry);
  if (!entries.length) {
    showShellNotice("请选择文件或文件夹", "warning");
    return;
  }
  const format: ArchiveFormat = "zip";
  const defaultName = entries.length === 1 ? `${entries[0].name}${archiveExtension(format)}` : `选中项目${archiveExtension(format)}`;
  openOperationPanel({
    kind: "archive",
    title: entries.length === 1 ? `压缩 ${entries[0].name}` : `压缩 ${entries.length} 项`,
    message: `输出位置：${currentFolder()}`,
    primaryText: "开始压缩",
    name: defaultName,
    format,
    entries,
    sourceEntry: null
  });
}

const extractSelected = (entry = singleSelectedEntry()) => {
  if (!isArchiveFile(entry)) {
    showShellNotice("请选择一个 zip、tar.gz 或 tgz 压缩包", "warning");
    return;
  }
  openOperationPanel({
    kind: "extract",
    title: `解压 ${entry.name}`,
    message: `输出位置：${currentFolder()}`,
    primaryText: "开始解压",
    name: archiveStem(entry.name),
    format: "zip",
    entries: [],
    sourceEntry: entry
  });
}

const submitOperationPanel = async () => {
  const panel = operationPanel.value;
  if (!panel.kind || panel.submitting) return;
  const name = panel.name.trim();
  if (!name) {
    showShellNotice(`${operationPanelNameLabel.value}不能为空`, "warning");
    return;
  }
  panel.submitting = true;
  try {
    if (panel.kind === "createFile" || panel.kind === "createFolder") {
      await createEntry(currentFolder(), panel.kind === "createFile" ? "file" : "folder", name);
      taskMessage.value = `已创建：${name}`;
      resetOperationPanel();
      await refreshCurrent();
      return;
    }
    if (panel.kind === "archive") {
      const task = await createArchiveTask(panel.entries.map(item => item.path), currentFolder(), panel.format, name);
      resetOperationPanel();
      await taskStarted(task.id, "压缩任务");
      return;
    }
    if (panel.kind === "extract" && panel.sourceEntry) {
      const task = await createExtractTask(panel.sourceEntry.path, currentFolder(), name);
      resetOperationPanel();
      await taskStarted(task.id, "解压任务");
    }
  } catch (error) {
    operationPanel.value.submitting = false;
    showErrorNotice(error, "操作失败");
  }
}

const uploadChanged = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files?.length) return;
  const files = Array.from(input.files);
  await uploadToCurrentFolder(files);
  input.value = "";
}

const uploadToCurrentFolder = async (files: FileList | File[]) => {
  const fileList = Array.from(files);
  if (!fileList.length) return;
  await runOperation(async () => {
    await uploadFiles(currentFolder(), fileList);
    taskMessage.value = `已上传 ${fileList.length} 个文件`;
  })
}

const hasDraggedFiles = (event: DragEvent) => {
  const dataTransfer = event.dataTransfer;
  if (!dataTransfer || !Array.from(dataTransfer.types ?? []).includes("Files")) return false;
  return Array.from(dataTransfer.items ?? []).some(item => item.kind === "file");
}

const resetUploadDrop = () => {
  uploadDragDepth = 0;
  uploadDropActive.value = false;
}

const canHandleUploadDrop = (event: DragEvent) => !fileStore.showEditor && hasDraggedFiles(event);

const handleUploadDragEnter = (event: DragEvent) => {
  if (!canHandleUploadDrop(event)) return;
  event.preventDefault();
  event.stopPropagation();
  uploadDragDepth += 1;
  uploadDropActive.value = true;
}

const handleUploadDragOver = (event: DragEvent) => {
  if (!canHandleUploadDrop(event)) return;
  event.preventDefault();
  event.stopPropagation();
  if (event.dataTransfer) event.dataTransfer.dropEffect = "copy";
  uploadDropActive.value = true;
}

const handleUploadDragLeave = (event: DragEvent) => {
  if (fileStore.showEditor || !uploadDropActive.value) return;
  event.preventDefault();
  event.stopPropagation();
  uploadDragDepth = Math.max(0, uploadDragDepth - 1);
  if (!uploadDragDepth) uploadDropActive.value = false;
}

const handleUploadDrop = async (event: DragEvent) => {
  if (!canHandleUploadDrop(event)) return;
  event.preventDefault();
  event.stopPropagation();
  const files = Array.from(event.dataTransfer?.files ?? []);
  resetUploadDrop();
  if (!files.length) return;
  uploadDropUploading.value = true;
  try {
    await uploadToCurrentFolder(files);
  } finally {
    uploadDropUploading.value = false;
  }
}

const shouldIgnoreNavigationShortcut = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return Boolean(target.closest("input, textarea, select, [contenteditable='true'], .ace_editor, .operation-panel, .delete-confirm-panel, .properties-panel"));
}

const shouldIgnoreActionShortcut = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return Boolean(target.closest("button, a, input, textarea, select, [contenteditable='true'], .ace_editor, .operation-panel, .delete-confirm-panel, .properties-panel, .context-menu, .task-panel"));
}

const shouldKeepEditorFindShortcut = (target: EventTarget | null) => {
  if (fileStore.showEditor) return true;
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".ace_editor, .operation-panel"));
}

const shouldIgnoreAddressShortcut = (target: EventTarget | null) => {
  if (fileStore.showEditor) return true;
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return Boolean(target.closest(".ace_editor, .operation-panel, .delete-confirm-panel, .properties-panel, .context-menu, .task-panel"));
}

const hasPageTextSelection = () => {
  const selection = window.getSelection();
  return Boolean(selection && !selection.isCollapsed && selection.toString().trim());
}

const isExplorerShortcutTarget = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".explorer-viewport"));
}

const handleClipboardShortcut = (key: string, event: KeyboardEvent) => {
  if (event.shiftKey || shouldIgnoreShellShortcut(event.target) || isExplorerShortcutTarget(event.target)) return false;
  if ((key === "c" || key === "x") && hasPageTextSelection()) return false;
  if (key === "c") {
    event.preventDefault();
    copySelected();
    return true;
  }
  if (key === "x") {
    event.preventDefault();
    cutSelected();
    return true;
  }
  if (key === "v") {
    event.preventDefault();
    void pasteSelected();
    return true;
  }
  return false;
}

const handleSelectAllShortcut = (key: string, event: KeyboardEvent) => {
  if (key !== "a" || event.shiftKey || shouldIgnoreShellShortcut(event.target) || isExplorerShortcutTarget(event.target)) return false;
  if (hasPageTextSelection()) return false;
  event.preventDefault();
  explorerRef.value?.selectAllEntries();
  return true;
}

const focusSearch = () => {
  if (fileStore.showEditor) return;
  searchInput.value?.focus();
  searchInput.value?.select();
}

const focusBreadcrumb = () => {
  if (fileStore.showEditor) return;
  breadcrumbRef.value?.focusInput();
}

const previewSelectedQuietly = async () => {
  const entry = singleSelection.value;
  if (!entry || entry.type !== "file") return false;
  await previewSelected(entry);
  return true;
}

const togglePreviewFromShortcut = async () => {
  if (previewPanelVisible.value) {
    closePreview();
    return true;
  }
  const previewed = await previewSelectedQuietly();
  if (previewed) return true;
  if (fileStore.showEditor) return false;
  clearPreviewContent();
  previewPanelVisible.value = true;
  return true;
}

const applyViewShortcut = (shortcut: {mode: ExplorerViewMode; iconSize: ExplorerIconSize; label: string}) => {
  fileStore.setViewMode(shortcut.mode);
  fileStore.setIconSize(shortcut.iconSize);
  closeTabContextMenu();
  void nextTick(() => explorerRef.value?.focus());
  showShellNotice(`已切换为${shortcut.label}`, "info", "查看模式", 1400);
}

const cycleViewMode = () => {
  fileStore.setViewMode(nextViewMode.value);
  void nextTick(() => explorerRef.value?.focus());
}

const shouldIgnoreShellShortcut = (target: EventTarget | null) => {
  return fileStore.showEditor || shouldIgnoreNavigationShortcut(target);
}

const switchRelativeTab = async (offset: number) => {
  if (fileStore.tabs.length <= 1) return false;
  const currentIndex = fileStore.tabs.findIndex(tab => tab.id === fileStore.activeTabId);
  const startIndex = currentIndex >= 0 ? currentIndex : 0;
  const nextIndex = (startIndex + offset + fileStore.tabs.length) % fileStore.tabs.length;
  const nextTab = fileStore.tabs[nextIndex];
  if (!nextTab || nextTab.id === fileStore.activeTabId) return false;
  await switchTab(nextTab.id);
  return true;
}

const closeActiveTab = async () => {
  if (fileStore.tabs.length <= 1) return false;
  if (!await fileStore.requestEditorLeave()) return false;
  persistCurrentExplorerScrollTop();
  fileStore.closeTab(fileStore.activeTabId);
  closePanels();
  await syncActiveTabContext();
  return true;
}

const handleBackspaceNavigation = () => {
  if (canNavigateBack.value) {
    void navigateBack();
    return;
  }
  void navigateUp();
}

const handleHistoryMouseButton = (event: MouseEvent) => {
  if (fileStore.showEditor || shouldIgnoreNavigationShortcut(event.target)) return false;
  if (event.button === 3 && canNavigateBack.value) {
    event.preventDefault();
    void navigateBack();
    return true;
  }
  if (event.button === 4 && canNavigateForward.value) {
    event.preventDefault();
    void navigateForward();
    return true;
  }
  return false;
}

const handleHistoryMouseDown = (event: MouseEvent) => {
  historyMouseButton = handleHistoryMouseButton(event) ? event.button : -1;
}

const handleHistoryMouseUp = (event: MouseEvent) => {
  if (historyMouseButton >= 0 && event.button === historyMouseButton) {
    event.preventDefault();
    historyMouseButton = -1;
  }
}

const handleHistoryAuxClick = (event: MouseEvent) => {
  if (event.button === 3 || event.button === 4) event.preventDefault();
}

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if (imageViewerVisible.value) return;
  const key = event.key.toLowerCase();
  const commandKey = event.ctrlKey || event.metaKey;
  if (commandKey && event.shiftKey && !event.altKey && !shouldIgnoreShellShortcut(event.target)) {
    const shortcut = viewShortcut(event.code);
    if (shortcut) {
      event.preventDefault();
      applyViewShortcut(shortcut);
      return;
    }
  }
  if ((commandKey && !event.altKey && key === "l") || (event.altKey && !event.ctrlKey && !event.metaKey && key === "d")) {
    if (shouldIgnoreAddressShortcut(event.target)) return;
    event.preventDefault();
    focusBreadcrumb();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === "f") {
    if (shouldKeepEditorFindShortcut(event.target)) return;
    event.preventDefault();
    focusSearch();
    return;
  }
  if (commandKey && !event.altKey && !shouldIgnoreShellShortcut(event.target)) {
    if (handleClipboardShortcut(key, event)) return;
    if (handleSelectAllShortcut(key, event)) return;
    if (key === "t") {
      event.preventDefault();
      void openTab();
      return;
    }
    if (key === "w") {
      event.preventDefault();
      void closeActiveTab();
      return;
    }
    if (key === "tab") {
      event.preventDefault();
      void switchRelativeTab(event.shiftKey ? -1 : 1);
      return;
    }
    if (key === "pageup" || key === "pagedown") {
      event.preventDefault();
      void switchRelativeTab(key === "pageup" ? -1 : 1);
      return;
    }
    if (event.shiftKey && key === "n") {
      event.preventDefault();
      void createFolderFromShortcut();
      return;
    }
    if (!event.shiftKey && key === "r") {
      event.preventDefault();
      void refreshCurrent(true);
      return;
    }
  }
  if (event.key === "F5" && !event.altKey && !event.ctrlKey && !event.metaKey && !shouldIgnoreShellShortcut(event.target)) {
    event.preventDefault();
    void refreshCurrent(true);
    return;
  }
  if (event.key === "Backspace" && !event.altKey && !event.ctrlKey && !event.metaKey && !shouldIgnoreNavigationShortcut(event.target)) {
    event.preventDefault();
    handleBackspaceNavigation();
    return;
  }
  if (event.altKey && !event.ctrlKey && !event.metaKey && event.key.toLowerCase() === "p" && !shouldIgnoreNavigationShortcut(event.target)) {
    event.preventDefault();
    void togglePreviewFromShortcut();
    return;
  }
  if ((event.key === " " || event.code === "Space") && !event.altKey && !event.ctrlKey && !event.metaKey && !shouldIgnoreActionShortcut(event.target)) {
    if (singleSelection.value?.type === "file") {
      event.preventDefault();
      void previewSelectedQuietly();
    }
    return;
  }
  if (!event.altKey || event.ctrlKey || event.metaKey || shouldIgnoreNavigationShortcut(event.target)) return;
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    void navigateBack();
  } else if (event.key === "ArrowRight") {
    event.preventDefault();
    void navigateForward();
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    void navigateUp();
  }
}

const navigateBack = async () => {
  if (!await fileStore.requestEditorLeave()) return;
  persistCurrentExplorerScrollTop();
  const path = fileStore.goBack();
  if (!path) return;
  await finishPathNavigation(path);
}

const navigateForward = async () => {
  if (!await fileStore.requestEditorLeave()) return;
  persistCurrentExplorerScrollTop();
  const path = fileStore.goForward();
  if (!path) return;
  await finishPathNavigation(path);
}

const finishPathNavigation = async (path: string, focusExplorer = true) => {
  closePanels();
  const loaded = await explorerRef.value?.refresh(path) ?? false;
  if (!loaded) {
    if (focusExplorer) {
      await nextTick();
      explorerRef.value?.focus();
    }
    return false;
  }
  await syncActiveTabContext();
  if (focusExplorer) {
    await nextTick();
    explorerRef.value?.focus();
  }
  return true;
}

const navigateToPath = async (path: string, options: NavigateToPathOptions = {}) => {
  const targetPath = normalizePathText(path);
  if (!options.skipEditorLeave && !await fileStore.requestEditorLeave()) return false;
  persistCurrentExplorerScrollTop();
  const loaded = await finishPathNavigation(targetPath, options.focusExplorer ?? true);
  return loaded;
}

const handleBreadcrumbNavigate = async (path: string, complete?: (navigated: boolean) => void) => {
  const navigated = await navigateToPath(path);
  complete?.(navigated);
}

const navigateUp = async () => {
  if (!canNavigateUp.value) return;
  await navigateToPath(parentPath(currentFolder()));
}

const openTab = async () => {
  if (!await fileStore.requestEditorLeave()) return;
  persistCurrentExplorerScrollTop();
  closeTabContextMenu();
  fileStore.openTab(currentFolder());
  closePanels();
  await syncActiveTabContext();
}

const openEntryInNewTab = async (entry: ExplorerEntry) => {
  if (entry.type !== "folder") return;
  if (!await fileStore.requestEditorLeave()) return;
  persistCurrentExplorerScrollTop();
  closeTabContextMenu();
  fileStore.openPathInNewTab(entry.path);
  closePanels();
  await syncActiveTabContext();
}

const closeTabContextMenu = () => {
  tabContextMenu.value.visible = false;
}

const openTabContextMenu = (event: MouseEvent, tabId: string) => {
  event.preventDefault();
  event.stopPropagation();
  const x = Math.min(Math.max(8, event.clientX), Math.max(8, window.innerWidth - tabContextMenuWidth - 8));
  const y = Math.min(Math.max(8, event.clientY), Math.max(8, window.innerHeight - tabContextMenuHeight - 8));
  tabContextMenu.value = {visible: true, x, y, tabId};
}

const duplicateTabFromMenu = async () => {
  const tabId = tabContextMenu.value.tabId;
  if (!await fileStore.requestEditorLeave()) return;
  persistCurrentExplorerScrollTop();
  closeTabContextMenu();
  fileStore.duplicateTab(tabId);
  closePanels();
  await syncActiveTabContext();
}

const closeTabById = async (tabId: string) => {
  if (fileStore.tabs.length <= 1) return false;
  const wasActive = fileStore.activeTabId === tabId;
  if (wasActive && !await fileStore.requestEditorLeave()) return false;
  if (wasActive) persistCurrentExplorerScrollTop();
  fileStore.closeTab(tabId);
  if (wasActive) {
    closePanels();
    await syncActiveTabContext();
  }
  return true;
}

const closeTabFromMenu = async () => {
  const tabId = tabContextMenu.value.tabId;
  closeTabContextMenu();
  await closeTabById(tabId);
}

const closeOtherTabsFromMenu = async () => {
  const tabId = tabContextMenu.value.tabId;
  const changesActiveTab = fileStore.activeTabId !== tabId;
  if (changesActiveTab && !await fileStore.requestEditorLeave()) return;
  if (changesActiveTab) persistCurrentExplorerScrollTop();
  closeTabContextMenu();
  fileStore.closeOtherTabs(tabId);
  if (changesActiveTab) {
    closePanels();
    await syncActiveTabContext();
  }
}

const closeRightTabsFromMenu = async () => {
  const tabId = tabContextMenu.value.tabId;
  const closesActiveTab = tabContextIndex.value >= 0 && fileStore.tabs.findIndex(tab => tab.id === fileStore.activeTabId) > tabContextIndex.value;
  if (closesActiveTab && !await fileStore.requestEditorLeave()) return;
  if (closesActiveTab) persistCurrentExplorerScrollTop();
  closeTabContextMenu();
  fileStore.closeTabsToRight(tabId);
  if (closesActiveTab) {
    closePanels();
    await syncActiveTabContext();
  }
}

const handleTabAuxClick = (event: MouseEvent, tabId: string) => {
  if (event.button !== 1) return;
  event.preventDefault();
  event.stopPropagation();
  closeTabContextMenu();
  void closeTabById(tabId);
}

const startTabDrag = (event: DragEvent, tabId: string) => {
  if (event.target instanceof HTMLElement && event.target.closest(".tab-close")) {
    event.preventDefault();
    return;
  }
  draggingTabId.value = tabId;
  tabDropTargetId.value = "";
  tabDropPlacement.value = "";
  closeTabContextMenu();
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.dropEffect = "move";
    event.dataTransfer.setData("text/plain", tabId);
  }
}

const dragOverTab = (event: DragEvent, tabId: string) => {
  if (!draggingTabId.value || draggingTabId.value === tabId) return;
  event.preventDefault();
  if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
  const target = event.currentTarget instanceof HTMLElement ? event.currentTarget : null;
  const rect = target?.getBoundingClientRect();
  tabDropTargetId.value = tabId;
  tabDropPlacement.value = rect && event.clientX > rect.left + rect.width / 2 ? "after" : "before";
}

const leaveTabDropTarget = (event: DragEvent, tabId: string) => {
  if (tabDropTargetId.value !== tabId) return;
  const related = event.relatedTarget;
  if (related instanceof Node && event.currentTarget instanceof HTMLElement && event.currentTarget.contains(related)) return;
  tabDropTargetId.value = "";
  tabDropPlacement.value = "";
}

const dropTab = (event: DragEvent, tabId: string) => {
  if (!draggingTabId.value || draggingTabId.value === tabId || !tabDropPlacement.value) return;
  event.preventDefault();
  event.stopPropagation();
  fileStore.reorderTab(draggingTabId.value, tabId, tabDropPlacement.value);
  draggingTabId.value = "";
  tabDropTargetId.value = "";
  tabDropPlacement.value = "";
}

const finishTabDrag = () => {
  draggingTabId.value = "";
  tabDropTargetId.value = "";
  tabDropPlacement.value = "";
}

const openImageViewer = async ({entry, entries}: ImageViewerPayload) => {
  if (!await fileStore.requestEditorLeave()) return;
  fileStore.closeEditor();
  imageViewerEntries.value = entries.length ? entries : [entry];
  imageViewerVisible.value = true;
  setImageViewerEntry(entry);
}

const openPreviewImageViewer = async () => {
  const entry = previewEntry.value;
  if (!entry) return;
  const entries = explorerRef.value?.getImageEntries() ?? [];
  await openImageViewer({entry, entries: entries.some(item => item.path === entry.path) ? entries : [entry]});
}

const openPreviewEntryImageViewer = async (entry: ExplorerEntry) => {
  previewEntry.value = entry;
  await openPreviewImageViewer();
}

const switchTab = async (tabId: string) => {
  closeTabContextMenu();
  if (tabId !== fileStore.activeTabId && !await fileStore.requestEditorLeave()) return;
  if (tabId !== fileStore.activeTabId) persistCurrentExplorerScrollTop();
  fileStore.switchTab(tabId);
  closePanels();
  await syncActiveTabContext();
}

const closeTab = (event: MouseEvent, tabId: string) => {
  event.stopPropagation();
  closeTabContextMenu();
  void closeTabById(tabId);
}

const previewSelected = async (entry = selectedEntry()) => {
  if (!entry || entry.type !== "file") {
    showShellNotice("请选择文件", "warning");
    return;
  }
  await setPreviewEntry(entry, true);
}

const setPreviewEntry = async (entry: ExplorerEntry, force = false) => {
  if (!force && previewEntry.value?.path === entry.path && previewPanelVisible.value) return;
  if (!await fileStore.requestEditorLeave()) return;
  fileStore.closeEditor();
  previewEntry.value = entry;
  previewReloadKey.value += 1;
  previewPanelVisible.value = true;
}

const closePreview = () => {
  closePanels();
}

const openPreviewInEditor = async () => {
  const entry = previewEntry.value;
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
  previewEntry.value = entry;
  void openPreviewInEditor();
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
              title="搜索当前文件夹 (Ctrl+F)"
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
        <div class="path-row">
          <button class="nav-button" :disabled="!canNavigateBack" :title="navigateBackTitle" @click="navigateBack">
            <icon icon="icon-back_android" size="large" />
          </button>
          <button class="nav-button" :disabled="!canNavigateForward" :title="navigateForwardTitle" @click="navigateForward">
            <icon icon="icon-back_android" size="large" class="rotate-180" />
          </button>
          <button class="nav-button" :disabled="!canNavigateUp" :title="navigateUpTitle" @click="navigateUp">
            <icon icon="icon-back_android" size="large" class="rotate-90" />
          </button>
          <button class="nav-button" title="刷新 (F5 / Ctrl+R)" @click="refreshCurrent(true)">
            <icon icon="icon-refresh" size="large" />
          </button>
          <breadcrumb ref="breadcrumbRef" @navigate="handleBreadcrumbNavigate"></breadcrumb>
          <button class="view-button" :title="viewModeButtonTitle" @click="cycleViewMode">
            <icon :icon="currentViewModeMeta.icon" />
            <span>{{ currentViewModeMeta.label }}</span>
          </button>
          <button class="view-button" :class="{active: previewPanelVisible}" :disabled="!canTogglePreviewPane" title="预览窗格 (Alt+P)" @click="togglePreviewFromShortcut">
            <icon icon="icon-file-image-fill" />
            <span>{{ previewPanelVisible ? "关闭预览" : "预览窗格" }}</span>
          </button>
        </div>

        <div class="command-bar">
          <button class="command-button" @click="openCreatePanel('file')">
            <icon icon="icon-file-add-fill" />
            <span>新建文件</span>
          </button>
          <button class="command-button" title="新建文件夹 (Ctrl+Shift+N)" @click="openCreatePanel('folder')">
            <icon icon="icon-folder-add-fill" />
            <span>新建文件夹</span>
          </button>
          <span class="command-separator"></span>
          <button class="command-button" :disabled="!hasSelection" title="剪切 (Ctrl+X)" @click="cutSelected()">
            <icon icon="icon-scissors" />
            <span>剪切</span>
          </button>
          <button class="command-button" :disabled="!hasSelection" title="复制 (Ctrl+C)" @click="copySelected()">
            <icon icon="icon-copy" />
            <span>复制</span>
          </button>
          <button class="command-button" :disabled="!canPasteSelection" title="粘贴 (Ctrl+V)" @click="pasteSelected()">
            <icon icon="icon-paste" />
            <span>粘贴</span>
          </button>
          <span class="command-separator"></span>
          <button class="command-button" :disabled="!canDownloadSelection" @click="downloadSelected()">
            <icon icon="icon-download" />
            <span>下载</span>
          </button>
          <button class="command-button" :disabled="!canPreviewSelection" title="预览 (Space / Ctrl+Enter)" @click="previewSelected()">
            <icon icon="icon-file-image-fill" />
            <span>预览</span>
          </button>
          <button class="command-button" :disabled="!canArchiveSelection" @click="archiveSelected()">
            <icon icon="icon-file-zip-fill" />
            <span>压缩</span>
          </button>
          <button class="command-button" :disabled="!canExtractSelection" @click="extractSelected()">
            <icon icon="icon-file-zip" />
            <span>解压</span>
          </button>
          <button class="command-button" :disabled="!canRenameSelection" @click="startRenameSelected">
            <icon icon="icon-rename" />
            <span>重命名</span>
          </button>
          <button class="command-button danger" :disabled="!canDeleteSelection" @click="deleteSelected()">
            <icon icon="icon-delete-fill" />
            <span>删除</span>
          </button>
          <span class="command-status" :title="`${selectionStatusText} · Ctrl+A 全选`">{{ selectionStatusText }}</span>
          <button :class="['command-button', {active: taskPanelVisible}]" @click="toggleTaskPanel">
            <icon icon="icon-file-common-filling" />
            <span>{{ taskButtonText }}</span>
          </button>
          <input ref="uploadInput" class="hidden" type="file" multiple @change="uploadChanged">
        </div>

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
            <section v-if="shellNotice.visible" :class="['shell-notice', shellNotice.kind]" role="status" aria-live="polite">
              <div class="shell-notice-mark" aria-hidden="true"></div>
              <div class="shell-notice-body">
                <strong>{{ shellNotice.title || shellNoticeLabel }}</strong>
                <span>{{ shellNotice.message }}</span>
              </div>
              <button type="button" class="shell-notice-close" title="关闭提示" @click="closeShellNotice">
                <icon icon="icon-close" />
              </button>
            </section>
            <div v-if="uploadDropActive || uploadDropUploading" class="upload-drop-layer">
              <div class="upload-drop-card">
                <div class="upload-drop-icon">
                  <icon icon="icon-upload" />
                </div>
                <strong>{{ uploadDropTitle }}</strong>
                <span>{{ uploadDropSubtitle }}</span>
              </div>
            </div>
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
.icon-tool,
.nav-button,
.view-button,
.command-button,
.task-icon-button {
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

.path-row {
  @apply flex h-14 shrink-0 items-center gap-2 border-b border-slate-200 bg-white/70 px-3;
}

.nav-button {
  @apply h-10 w-10 shrink-0;
}

.nav-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:bg-white;
}

.view-button {
  @apply h-10 shrink-0 gap-2 px-3 text-sm;
}

.view-button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.view-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:bg-white;
}

.command-bar {
  @apply flex h-11 shrink-0 items-center gap-1 overflow-x-auto border-b border-slate-200 bg-slate-50/70 px-3;
}

.command-button {
  @apply h-8 shrink-0 gap-1.5 border-transparent bg-transparent px-2.5 text-sm shadow-none;
}

.command-button:hover,
.command-button.active {
  @apply border-slate-200 bg-white;
}

.command-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:border-transparent hover:bg-transparent;
}

.command-button.danger {
  @apply text-red-600 hover:bg-red-50;
}

.command-button.danger:disabled {
  @apply text-red-200 hover:bg-transparent;
}

.command-separator {
  @apply mx-1 h-5 w-px shrink-0 bg-slate-200;
}

.command-status {
  @apply ml-auto min-w-32 truncate pl-3 text-right text-xs text-slate-500;
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

.upload-drop-layer {
  @apply pointer-events-none absolute inset-0 z-20 flex items-center justify-center border-2 border-dashed border-blue-400 bg-blue-50/55 p-6 backdrop-blur-[1px];
}

.upload-drop-card {
  @apply flex min-w-72 flex-col items-center gap-2 rounded-lg border border-blue-200 bg-white px-8 py-6 text-center text-sm text-slate-500 shadow-2xl;
}

.upload-drop-card strong {
  @apply text-base font-semibold text-slate-900;
}

.upload-drop-icon {
  @apply flex h-12 w-12 items-center justify-center rounded-lg bg-blue-600 text-2xl text-white shadow-sm;
}

.shell-notice {
  @apply absolute right-4 top-4 z-20 flex w-[min(24rem,calc(100%-2rem))] items-start gap-3 rounded-lg border bg-white/95 px-3 py-2 text-sm text-slate-700 shadow-xl backdrop-blur;
}

.shell-notice-mark {
  @apply mt-1 h-2.5 w-2.5 shrink-0 rounded-full bg-blue-500 shadow-[0_0_0_3px_rgba(59,130,246,0.15)];
}

.shell-notice-body {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.shell-notice-body strong {
  @apply truncate text-sm font-semibold text-slate-900;
}

.shell-notice-body span {
  @apply break-words text-xs leading-5 text-slate-600;
}

.shell-notice-close {
  @apply -mr-1 flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-slate-400 hover:bg-slate-100 hover:text-slate-700;
}

.shell-notice.success {
  @apply border-emerald-100;
}

.shell-notice.success .shell-notice-mark {
  @apply bg-emerald-500 shadow-[0_0_0_3px_rgba(16,185,129,0.15)];
}

.shell-notice.warning {
  @apply border-amber-100;
}

.shell-notice.warning .shell-notice-mark {
  @apply bg-amber-500 shadow-[0_0_0_3px_rgba(245,158,11,0.16)];
}

.shell-notice.error {
  @apply border-red-100;
}

.shell-notice.error .shell-notice-mark {
  @apply bg-red-500 shadow-[0_0_0_3px_rgba(239,68,68,0.16)];
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
