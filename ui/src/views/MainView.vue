<script setup lang="ts">
import {computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import {useRouter} from "vue-router";
import FileTree from "../components/FileTree.vue";
import {ArchiveFormat, ExplorerViewMode, FileInfo, FileTreeData, TaskKind, TaskState, TaskStatus} from "../class";
import {useFileStore} from "../store";
import {
  cancelTask,
  createArchiveTask,
  createCopyTask,
  createDeleteTask,
  createEntry,
  createExtractTask,
  createMoveTask,
  downloadUrl,
  downloadFile,
  getFile,
  getFolderData,
  listTasks,
  logout,
  moveEntry,
  uploadFiles
} from "../network/api";
import Icon from "../components/Icon.vue";
import Explorer from "../components/explorer/Explorer.vue";
import Breadcrumb from "../components/Breadcrumb.vue";

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

type ExplorerExpose = {
  refresh: (path?: string) => Promise<void>;
  getSelectedEntry: () => ExplorerEntry | null;
  getSelectedEntries: () => ExplorerEntry[];
  startRename: () => void;
  selectPath: (path: string) => Promise<boolean>;
  selectPaths: (paths: string[]) => Promise<boolean>;
  selectPathForRename: (path: string) => Promise<boolean>;
  selectAllEntries: () => boolean;
}

type BreadcrumbExpose = {
  focusInput: () => void;
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

type ImageViewerPayload = {
  entry: ExplorerEntry;
  entries: ExplorerEntry[];
}

const viewModeOrder: ExplorerViewMode[] = ["details", "list", "icons", "tiles"];
const viewModeMeta: Record<ExplorerViewMode, {label: string; icon: string}> = {
  details: {label: "详细信息", icon: "icon-view-list"},
  list: {label: "列表", icon: "icon-listview"},
  icons: {label: "图标", icon: "icon-viewgrid"},
  tiles: {label: "平铺", icon: "icon-file-common-filling"}
};

const router = useRouter();
const fileStore = useFileStore();
const treeData = ref<FileTreeData[]>([]);
const explorerRef = ref<ExplorerExpose | null>(null);
const breadcrumbRef = ref<BreadcrumbExpose | null>(null);
const deleteConfirmRef = ref<HTMLElement | null>(null);
const imageViewerRef = ref<HTMLElement | null>(null);
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
const previewPanelVisible = ref(false);
const previewEntry = ref<ExplorerEntry | null>(null);
const previewLoading = ref(false);
const previewText = ref("");
const previewError = ref("");
const previewImageFit = ref(true);
const previewImageZoom = ref(100);
const previewImageOffsetX = ref(0);
const previewImageOffsetY = ref(0);
const previewImageDragging = ref(false);
const previewTextWrap = ref(true);
const previewCopied = ref(false);
const imageViewerVisible = ref(false);
const imageViewerEntry = ref<ExplorerEntry | null>(null);
const imageViewerEntries = ref<ExplorerEntry[]>([]);
const imageViewerLoading = ref(false);
const imageViewerError = ref("");
const imageViewerPageFullscreen = ref(false);
const imageViewerFullscreen = ref(false);
const imageViewerShowFilmstrip = ref(true);
const imageViewerFit = ref(true);
const imageViewerZoom = ref(100);
const imageViewerOffsetX = ref(0);
const imageViewerOffsetY = ref(0);
const imageViewerDragging = ref(false);
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
let previewLoadVersion = 0;
let previewCopyTimer: number | undefined;
let uploadDragDepth = 0;
let taskPollTimer: number | undefined;
let shellNoticeTimer: number | undefined;
let previewImagePointerId: number | null = null;
let previewImageDragStartX = 0;
let previewImageDragStartY = 0;
let previewImageDragOriginX = 0;
let previewImageDragOriginY = 0;
let imageViewerPointerId: number | null = null;
let imageViewerDragStartX = 0;
let imageViewerDragStartY = 0;
let imageViewerDragOriginX = 0;
let imageViewerDragOriginY = 0;
const tabContextMenuWidth = 184;
const tabContextMenuHeight = 220;

const activeTab = computed(() => fileStore.tabs.find(tab => tab.id === fileStore.activeTabId) ?? fileStore.tabs[0]);
const tabContextTarget = computed(() => fileStore.tabs.find(tab => tab.id === tabContextMenu.value.tabId) ?? null);
const tabContextIndex = computed(() => fileStore.tabs.findIndex(tab => tab.id === tabContextMenu.value.tabId));
const canCloseTabContext = computed(() => fileStore.tabs.length > 1);
const canCloseOtherTabsContext = computed(() => fileStore.tabs.length > 1 && Boolean(tabContextTarget.value));
const canCloseRightTabsContext = computed(() => tabContextIndex.value >= 0 && tabContextIndex.value < fileStore.tabs.length - 1);
const canNavigateBack = computed(() => Boolean(activeTab.value?.backStack?.length));
const canNavigateForward = computed(() => Boolean(activeTab.value?.forwardStack?.length));
const canNavigateUp = computed(() => currentFolder() !== "/");
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
const canTogglePreviewPane = computed(() => previewPanelVisible.value || canPreviewSelection.value);
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
const viewModeButtonTitle = computed(() => `当前：${currentViewModeMeta.value.label}，切换到${viewModeMeta[nextViewMode.value].label}`);
const taskStats = computed(() => {
  const stats = {
    running: 0,
    queued: 0,
    failed: 0,
    completed: 0,
    active: 0,
    totalErrors: 0
  };
  tasks.value.forEach(task => {
    if (task.state === "running") stats.running += 1;
    if (task.state === "queued") stats.queued += 1;
    if (task.state === "failed") stats.failed += 1;
    if (task.state === "completed") stats.completed += 1;
    if (task.state === "running" || task.state === "queued") stats.active += 1;
    stats.totalErrors += task.errors.length;
  });
  return stats;
});
const hasActiveTasks = computed(() => taskStats.value.active > 0);
const taskButtonText = computed(() => hasActiveTasks.value ? `任务 ${taskStats.value.active}` : "任务");
const taskSummaryText = computed(() => {
  if (!tasks.value.length) return "暂无后台任务";
  const parts: string[] = [];
  if (taskStats.value.running) parts.push(`运行 ${taskStats.value.running}`);
  if (taskStats.value.queued) parts.push(`排队 ${taskStats.value.queued}`);
  if (taskStats.value.failed) parts.push(`失败 ${taskStats.value.failed}`);
  if (taskStats.value.totalErrors) parts.push(`错误 ${taskStats.value.totalErrors}`);
  return parts.length ? parts.join(" · ") : `已完成 ${taskStats.value.completed}/${tasks.value.length}`;
});
const taskRefreshText = computed(() => taskLastUpdatedAt.value ? `上次刷新：${taskLastUpdatedAt.value}` : "打开后自动刷新任务状态");
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
const operationPanelIcon = computed(() => {
  switch (operationPanel.value.kind) {
    case "createFile":
      return "icon-file-add-fill";
    case "createFolder":
      return "icon-folder-add-fill";
    case "archive":
    case "extract":
      return "icon-file-zip-fill";
    default:
      return "icon-file-common-filling";
  }
});
const deleteConfirmTitle = computed(() => {
  const count = deleteConfirm.value.entries.length;
  return count > 1 ? `删除 ${count} 项？` : `删除 ${deleteConfirm.value.entries[0]?.name ?? "所选项目"}？`;
});
const deleteConfirmMessage = computed(() => {
  const count = deleteConfirm.value.entries.length;
  return count > 1 ? "这些项目会被移动到回收站，之后可从回收站恢复。" : "该项目会被移动到回收站，之后可从回收站恢复。";
});
const deleteConfirmItems = computed(() => deleteConfirm.value.entries.slice(0, 5));
const deleteConfirmExtraCount = computed(() => Math.max(0, deleteConfirm.value.entries.length - deleteConfirmItems.value.length));
const taskCancelTitle = computed(() => taskCancelConfirm.value.task ? `取消${taskKindText(taskCancelConfirm.value.task.kind)}任务？` : "取消任务？");
const taskCancelMessage = computed(() => {
  const task = taskCancelConfirm.value.task;
  if (!task) return "任务取消请求会发送给后端。";
  return `#${shortTaskId(task.id)} · ${taskStateText(task.state)} · ${taskProgress(task)}`;
});
const previewKind = computed<"image" | "text" | "audio" | "video" | "unknown">(() => {
  const entry = previewEntry.value;
  if (!entry || entry.type !== "file") return "unknown";
  const extension = entry.extension?.toLowerCase() ?? "";
  if (["apng", "avif", "bmp", "gif", "ico", "jpeg", "jpg", "png", "svg", "webp"].includes(extension)) return "image";
  if (["mp3", "wav", "ogg", "flac", "m4a", "aac"].includes(extension)) return "audio";
  if (["mp4", "webm", "mov", "mkv", "avi"].includes(extension)) return "video";
  if (fileStore.extensions.includes(extension) || ["txt", "log", "md", "json", "yaml", "yml", "toml", "xml", "csv"].includes(extension)) return "text";
  return "unknown";
});

const previewTypeText = computed(() => ({
  image: "图片",
  text: "文本",
  audio: "音频",
  video: "视频",
  unknown: "文件"
}[previewKind.value]));

const previewTitleText = computed(() => previewEntry.value?.name ?? "预览窗格");

const previewSubtitleText = computed(() => previewEntry.value ? previewTypeText.value : "选择一个文件");

const previewTextStats = computed(() => {
  if (previewKind.value !== "text") return "";
  const lines = previewText.value ? previewText.value.split(/\r\n|\r|\n/).length : 0;
  return `${lines} 行，${previewText.value.length} 字符`;
});

const canEditPreview = computed(() => {
  const entry = previewEntry.value;
  if (!entry || entry.type !== "file") return false;
  return fileStore.extensions.includes(entry.extension?.toLowerCase() ?? "");
});

const previewImageStyle = computed(() => ({
  maxWidth: previewImageFit.value ? "100%" : "none",
  maxHeight: previewImageFit.value ? "100%" : "none",
  transform: previewImageFit.value ? "none" : `translate3d(${previewImageOffsetX.value}px, ${previewImageOffsetY.value}px, 0) scale(${previewImageZoom.value / 100})`,
  transformOrigin: "center center"
}));

const previewZoomText = computed(() => previewImageFit.value ? "适应" : `${previewImageZoom.value}%`);

const canPanPreviewImage = computed(() => previewKind.value === "image" && !previewImageFit.value);

const imageViewerStyle = computed(() => ({
  maxWidth: imageViewerFit.value ? "100%" : "none",
  maxHeight: imageViewerFit.value ? "100%" : "none",
  transform: imageViewerFit.value ? "none" : `translate3d(${imageViewerOffsetX.value}px, ${imageViewerOffsetY.value}px, 0) scale(${imageViewerZoom.value / 100})`,
  transformOrigin: "center center"
}));

const imageViewerZoomText = computed(() => imageViewerFit.value ? "适应" : `${imageViewerZoom.value}%`);

const canPanImageViewer = computed(() => imageViewerVisible.value && !imageViewerFit.value);

const imageViewerIndex = computed(() => {
  const entry = imageViewerEntry.value;
  if (!entry) return -1;
  return imageViewerEntries.value.findIndex(item => item.path === entry.path);
});

const imageViewerCount = computed(() => imageViewerEntries.value.length);

const canShowPreviousImage = computed(() => imageViewerIndex.value > 0);

const canShowNextImage = computed(() => imageViewerIndex.value >= 0 && imageViewerIndex.value < imageViewerEntries.value.length - 1);

const canShowImageViewerFilmstrip = computed(() => imageViewerCount.value > 1 && imageViewerShowFilmstrip.value);

const imageViewerFilmstripEntries = computed(() => {
  const entries = imageViewerEntries.value;
  if (entries.length <= 12) return entries.map((entry, index) => ({entry, index}));
  const currentIndex = Math.max(0, imageViewerIndex.value);
  const visibleCount = 11;
  const half = Math.floor(visibleCount / 2);
  let start = Math.max(0, currentIndex - half);
  let end = Math.min(entries.length, start + visibleCount);
  start = Math.max(0, end - visibleCount);
  return entries.slice(start, end).map((entry, offset) => ({entry, index: start + offset}));
});

const imageViewerSubtitle = computed(() => {
  const entry = imageViewerEntry.value;
  if (!entry) return "";
  const position = imageViewerIndex.value >= 0 && imageViewerCount.value > 1 ? `${imageViewerIndex.value + 1} / ${imageViewerCount.value} · ` : "";
  return `${position}${formatBytes(entry.size)} · ${formatDate(entry.modified)}`;
});

const previewMeta = computed(() => {
  const entry = previewEntry.value;
  if (!entry) return [];
  return [
    {label: "类型", value: previewTypeText.value},
    {label: "大小", value: formatBytes(entry.size)},
    {label: "修改", value: formatDate(entry.modified)},
    {label: "路径", value: entry.path}
  ];
});

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

const resetPreviewImagePan = () => {
  previewImageOffsetX.value = 0;
  previewImageOffsetY.value = 0;
  previewImageDragging.value = false;
  previewImagePointerId = null;
}

const resetImageViewerPan = () => {
  imageViewerOffsetX.value = 0;
  imageViewerOffsetY.value = 0;
  imageViewerDragging.value = false;
  imageViewerPointerId = null;
}

const resetImageViewerZoom = () => {
  imageViewerFit.value = true;
  imageViewerZoom.value = 100;
  resetImageViewerPan();
}

const closeImageViewer = () => {
  if (document.fullscreenElement === imageViewerRef.value) void document.exitFullscreen().catch(() => undefined);
  imageViewerVisible.value = false;
  imageViewerEntry.value = null;
  imageViewerEntries.value = [];
  imageViewerLoading.value = false;
  imageViewerError.value = "";
  imageViewerPageFullscreen.value = false;
  resetImageViewerZoom();
}

const setImageViewerEntry = (entry: ExplorerEntry) => {
  imageViewerEntry.value = entry;
  imageViewerLoading.value = true;
  imageViewerError.value = "";
  resetImageViewerZoom();
}

const handleFullscreenChange = () => {
  imageViewerFullscreen.value = document.fullscreenElement === imageViewerRef.value;
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
  previewLoadVersion += 1;
  previewEntry.value = null;
  previewLoading.value = false;
  previewText.value = "";
  previewError.value = "";
  previewImageFit.value = true;
  previewImageZoom.value = 100;
  resetPreviewImagePan();
  previewCopied.value = false;
}

const closePanels = () => {
  previewPanelVisible.value = false;
  clearPreviewContent();
  operationPanel.value.visible = false;
  resetDeleteConfirm();
  resetTaskCancelConfirm();
  closeImageViewer();
}

const clearSearch = () => {
  searchText.value = "";
}

const loadRoot = async () => {
  const data = await getFolderData("/");
  treeData.value = fileStore.saveAndConvertFolderData(data);
}

const handleLoad = (node: FileTreeData) => {
  return new Promise<void>(async (resolve) => {
    const data = await getFolderData(node.path);
    node.children = fileStore.saveAndConvertFolderData(data);
    fileStore.setCurrentPath(data.path);
    fileStore.showEditor = false;
    closePanels();
    resolve();
  });
}

onMounted(async () => {
  fileStore.ensureActiveTab();
  await loadRoot();
  window.addEventListener("keydown", handleWindowKeyDown);
  window.addEventListener("click", closeTabContextMenu);
  window.addEventListener("scroll", closeTabContextMenu, true);
  document.addEventListener("fullscreenchange", handleFullscreenChange);
})

onBeforeUnmount(() => {
  if (previewCopyTimer) window.clearTimeout(previewCopyTimer);
  stopShellNoticeTimer();
  stopTaskPolling();
  window.removeEventListener("keydown", handleWindowKeyDown);
  window.removeEventListener("click", closeTabContextMenu);
  window.removeEventListener("scroll", closeTabContextMenu, true);
  document.removeEventListener("fullscreenchange", handleFullscreenChange);
})

watch(() => fileStore.showEditor, (showEditor) => {
  if (showEditor) closePanels();
});

const currentFolder = () => fileStore.currentPath || "/";

const uploadDropTitle = computed(() => uploadDropUploading.value ? "正在上传文件..." : "释放鼠标上传文件");
const uploadDropSubtitle = computed(() => uploadDropUploading.value ? `目标：${currentFolder()}` : `上传到 ${currentFolder()}`);

const taskKindText = (kind: TaskKind) => ({
  copy: "复制",
  move: "移动",
  delete: "删除",
  archive: "压缩",
  extract: "解压"
}[kind] ?? kind);

const taskStateText = (state: TaskState) => ({
  queued: "排队中",
  running: "运行中",
  completed: "已完成",
  failed: "失败",
  cancelled: "已取消"
}[state] ?? state);

const taskStateClass = (state: TaskState) => ({
  queued: "queued",
  running: "running",
  completed: "completed",
  failed: "failed",
  cancelled: "cancelled"
}[state] ?? "queued");

const canCancelTask = (task: TaskStatus) => task.state === "queued" || task.state === "running";

const shortTaskId = (id: string) => id.slice(0, 8);

const formatBytes = (bytes?: number) => {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

const formatDate = (srcDate?: string) => {
  if (!srcDate) return "-";
  const date = new Date(srcDate);
  if (Number.isNaN(date.getTime())) return srcDate;
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit"
  }).format(date);
}

const taskProgress = (task: TaskStatus) => `${Math.round((task.progress || 0) * 100)}%`;

const taskBytesText = (task: TaskStatus) => {
  const processed = formatBytes(task.processedBytes);
  const total = task.totalBytes > 0 ? formatBytes(task.totalBytes) : "未知总量";
  return `${processed} / ${total}`;
}

const taskItemsText = (task: TaskStatus) => {
  const total = task.totalItems > 0 ? task.totalItems : "?";
  return `${task.processedItems} / ${total} 项`;
}

const taskCurrentPath = (task: TaskStatus) => task.currentPath?.trim();

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
  const selectedPaths = keepSelection ? currentSelection.value.map(entry => entry.path) : [];
  closePanels();
  if (currentFolder() === "/") {
    await loadRoot();
  }
  await explorerRef.value?.refresh(currentFolder());
  if (selectedPaths.length) await explorerRef.value?.selectPaths(selectedPaths);
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
  if (!previewPanelVisible.value || fileStore.showEditor) return;
  const entry = entries.length === 1 ? entries[0] : null;
  if (entry?.type === "file") {
    setPreviewEntry(entry);
  } else {
    clearPreviewContent();
  }
}

const singleSelectedEntry = (entry = selectedEntry()) => {
  const selected = selectedEntries(entry);
  if (selected.length > 1) return null;
  return selected[0] ?? null;
}

const parentPath = (path: string) => {
  const parts = path.split("/").filter(Boolean);
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

const dropEntriesToFolder = async ({entries, target, action}: DropEntriesPayload) => {
  if (target.type !== "folder" || !entries.length) return;
  const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(target.path, entry.path));
  if (nestedFolder) {
    showShellNotice(`不能将 ${nestedFolder.name} 放入它自身或子文件夹中`, "warning");
    return;
  }
  const sameFolder = entries.some(entry => parentPath(entry.path) === target.path);
  if (action === "move" && sameFolder) {
    taskMessage.value = "拖拽目标已经是当前位置";
    return;
  }
  try {
    const sources = entries.map(item => item.path);
    const task = action === "copy"
        ? await createCopyTask(sources, target.path)
        : await createMoveTask(sources, target.path);
    await taskStarted(task.id, action === "copy" ? "复制任务" : "移动任务");
    if (action === "move" && fileClipboardAction.value === "cut") {
      const moved = new Set(sources);
      fileClipboardEntries.value = fileClipboardEntries.value.filter(item => !moved.has(item.path));
      if (!fileClipboardEntries.value.length) fileClipboardAction.value = null;
    }
    await refreshCurrent();
  } catch (error) {
    showErrorNotice(error, "创建拖拽任务失败", "拖拽失败");
  }
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
  return Boolean(target.closest("input, textarea, select, [contenteditable='true'], .ace_editor, .operation-panel, .delete-confirm-panel"));
}

const shouldIgnoreActionShortcut = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return Boolean(target.closest("button, a, input, textarea, select, [contenteditable='true'], .ace_editor, .operation-panel, .delete-confirm-panel, .context-menu, .task-panel"));
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
  return Boolean(target.closest(".ace_editor, .operation-panel, .delete-confirm-panel, .context-menu, .task-panel"));
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

const previewSelectedQuietly = () => {
  const entry = singleSelection.value;
  if (!entry || entry.type !== "file") return false;
  previewSelected(entry);
  return true;
}

const togglePreviewFromShortcut = () => {
  if (previewPanelVisible.value) {
    closePreview();
    return true;
  }
  return previewSelectedQuietly();
}

const cycleViewMode = () => {
  fileStore.setViewMode(nextViewMode.value);
}

const shouldIgnoreShellShortcut = (target: EventTarget | null) => {
  return fileStore.showEditor || shouldIgnoreNavigationShortcut(target);
}

const switchRelativeTab = (offset: number) => {
  if (fileStore.tabs.length <= 1) return false;
  const currentIndex = fileStore.tabs.findIndex(tab => tab.id === fileStore.activeTabId);
  const startIndex = currentIndex >= 0 ? currentIndex : 0;
  const nextIndex = (startIndex + offset + fileStore.tabs.length) % fileStore.tabs.length;
  const nextTab = fileStore.tabs[nextIndex];
  if (!nextTab || nextTab.id === fileStore.activeTabId) return false;
  switchTab(nextTab.id);
  return true;
}

const closeActiveTab = () => {
  if (fileStore.tabs.length <= 1) return false;
  fileStore.closeTab(fileStore.activeTabId);
  closePanels();
  return true;
}

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if (imageViewerVisible.value) {
    const viewerKey = event.key.toLowerCase();
    if (viewerKey === "escape") {
      event.preventDefault();
      closeImageViewer();
      return;
    }
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      showAdjacentImage(-1);
      return;
    }
    if (event.key === "ArrowRight") {
      event.preventDefault();
      showAdjacentImage(1);
      return;
    }
    if (viewerKey === "+" || viewerKey === "=" || event.code === "NumpadAdd") {
      event.preventDefault();
      zoomImageViewer(25);
      return;
    }
    if (viewerKey === "-" || event.code === "NumpadSubtract") {
      event.preventDefault();
      zoomImageViewer(-25);
      return;
    }
    if (viewerKey === "0") {
      event.preventDefault();
      resetImageViewerZoom();
      return;
    }
    if (viewerKey === "f") {
      event.preventDefault();
      void toggleImageViewerPageFullscreen();
      return;
    }
    if (viewerKey === "t") {
      event.preventDefault();
      toggleImageViewerFilmstrip();
      return;
    }
    if (event.ctrlKey || event.metaKey || event.altKey) event.preventDefault();
    return;
  }
  const key = event.key.toLowerCase();
  const commandKey = event.ctrlKey || event.metaKey;
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
      openTab();
      return;
    }
    if (key === "w") {
      event.preventDefault();
      closeActiveTab();
      return;
    }
    if (key === "tab") {
      event.preventDefault();
      switchRelativeTab(event.shiftKey ? -1 : 1);
      return;
    }
    if (key === "pageup" || key === "pagedown") {
      event.preventDefault();
      switchRelativeTab(key === "pageup" ? -1 : 1);
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
    void navigateUp();
    return;
  }
  if (event.altKey && !event.ctrlKey && !event.metaKey && event.key.toLowerCase() === "p" && !shouldIgnoreNavigationShortcut(event.target)) {
    event.preventDefault();
    togglePreviewFromShortcut();
    return;
  }
  if ((event.key === " " || event.code === "Space") && !event.altKey && !event.ctrlKey && !event.metaKey && !shouldIgnoreActionShortcut(event.target)) {
    if (previewSelectedQuietly()) event.preventDefault();
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
  const path = fileStore.goBack();
  if (!path) return;
  closePanels();
  await explorerRef.value?.refresh(path);
}

const navigateForward = async () => {
  const path = fileStore.goForward();
  if (!path) return;
  closePanels();
  await explorerRef.value?.refresh(path);
}

const navigateUp = async () => {
  if (!canNavigateUp.value) return;
  fileStore.showEditor = false;
  fileStore.currentFile = null;
  closePanels();
  const path = parentPath(currentFolder());
  fileStore.setCurrentPath(path);
  await explorerRef.value?.refresh(path);
}

const openTab = () => {
  closeTabContextMenu();
  fileStore.openTab(currentFolder());
  closePanels();
}

const openEntryInNewTab = (entry: ExplorerEntry) => {
  if (entry.type !== "folder") return;
  closeTabContextMenu();
  fileStore.openPathInNewTab(entry.path);
  closePanels();
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

const duplicateTabFromMenu = () => {
  const tabId = tabContextMenu.value.tabId;
  closeTabContextMenu();
  fileStore.duplicateTab(tabId);
  closePanels();
}

const closeTabById = (tabId: string) => {
  if (fileStore.tabs.length <= 1) return false;
  const wasActive = fileStore.activeTabId === tabId;
  fileStore.closeTab(tabId);
  if (wasActive) closePanels();
  return true;
}

const closeTabFromMenu = () => {
  const tabId = tabContextMenu.value.tabId;
  closeTabContextMenu();
  closeTabById(tabId);
}

const closeOtherTabsFromMenu = () => {
  const tabId = tabContextMenu.value.tabId;
  const changesActiveTab = fileStore.activeTabId !== tabId;
  closeTabContextMenu();
  fileStore.closeOtherTabs(tabId);
  if (changesActiveTab) closePanels();
}

const closeRightTabsFromMenu = () => {
  const tabId = tabContextMenu.value.tabId;
  const closesActiveTab = tabContextIndex.value >= 0 && fileStore.tabs.findIndex(tab => tab.id === fileStore.activeTabId) > tabContextIndex.value;
  closeTabContextMenu();
  fileStore.closeTabsToRight(tabId);
  if (closesActiveTab) closePanels();
}

const handleTabAuxClick = (event: MouseEvent, tabId: string) => {
  if (event.button !== 1) return;
  event.preventDefault();
  event.stopPropagation();
  closeTabContextMenu();
  closeTabById(tabId);
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
  fileStore.showEditor = false;
  imageViewerEntries.value = entries.length ? entries : [entry];
  imageViewerVisible.value = true;
  setImageViewerEntry(entry);
  await nextTick();
  imageViewerRef.value?.focus();
}

const openPreviewImageViewer = async () => {
  const entry = previewEntry.value;
  if (!entry || previewKind.value !== "image") return;
  await openImageViewer({entry, entries: [entry]});
}

const toggleImageViewerPageFullscreen = async () => {
  imageViewerPageFullscreen.value = !imageViewerPageFullscreen.value;
  await nextTick();
  imageViewerRef.value?.focus();
}

const toggleImageViewerFullscreen = async () => {
  const target = imageViewerRef.value;
  if (!target) return;
  try {
    if (document.fullscreenElement === target) {
      await document.exitFullscreen();
    } else {
      await target.requestFullscreen();
    }
  } catch {
    showShellNotice("当前浏览器未允许进入全屏，仍可在页面内查看大图。", "warning", "无法全屏");
  }
}

const toggleImageViewerFilmstrip = () => {
  imageViewerShowFilmstrip.value = !imageViewerShowFilmstrip.value;
}

const showAdjacentImage = (direction: -1 | 1) => {
  const index = imageViewerIndex.value;
  if (index < 0) return;
  const next = imageViewerEntries.value[index + direction];
  if (!next) return;
  setImageViewerEntry(next);
}

const showImageAt = (index: number) => {
  const next = imageViewerEntries.value[index];
  if (!next || next.path === imageViewerEntry.value?.path) return;
  setImageViewerEntry(next);
}

const switchTab = (tabId: string) => {
  closeTabContextMenu();
  fileStore.switchTab(tabId);
  closePanels();
}

const closeTab = (event: MouseEvent, tabId: string) => {
  event.stopPropagation();
  closeTabContextMenu();
  closeTabById(tabId);
}

const previewSelected = (entry = selectedEntry()) => {
  if (!entry || entry.type !== "file") {
    showShellNotice("请选择文件", "warning");
    return;
  }
  setPreviewEntry(entry, true);
}

const setPreviewEntry = (entry: ExplorerEntry, force = false) => {
  if (!force && previewEntry.value?.path === entry.path && previewPanelVisible.value) return;
  fileStore.showEditor = false;
  fileStore.currentFile = null;
  previewEntry.value = entry;
  previewPanelVisible.value = true;
  previewImageFit.value = true;
  previewImageZoom.value = 100;
  resetPreviewImagePan();
  previewCopied.value = false;
  void loadPreview(entry);
}

const closePreview = () => {
  closePanels();
}

const openPreviewInEditor = () => {
  const entry = previewEntry.value;
  if (!entry || entry.type !== "file" || !canEditPreview.value) return;
  closePanels();
  fileStore.currentFile = {
    path: entry.path,
    name: entry.name,
    size: entry.size ?? 0,
    extension: entry.extension ?? "",
    modified: entry.modified ?? ""
  };
  fileStore.showEditor = true;
}

const copyPreviewText = async () => {
  if (previewKind.value !== "text" || !previewText.value) return;
  try {
    await navigator.clipboard.writeText(previewText.value);
    previewCopied.value = true;
    if (previewCopyTimer) window.clearTimeout(previewCopyTimer);
    previewCopyTimer = window.setTimeout(() => {
      previewCopied.value = false;
    }, 1500);
  } catch {
    showShellNotice("复制失败，请手动选择文本复制", "error", "复制失败");
  }
}

const zoomPreviewImage = (delta: number) => {
  previewImageFit.value = false;
  previewImageZoom.value = Math.min(300, Math.max(25, previewImageZoom.value + delta));
}

const zoomImageViewer = (delta: number) => {
  imageViewerFit.value = false;
  imageViewerZoom.value = Math.min(500, Math.max(25, imageViewerZoom.value + delta));
}

const handleImageViewerWheel = (event: WheelEvent) => {
  event.preventDefault();
  zoomImageViewer(event.deltaY < 0 ? 25 : -25);
}

const handleImageViewerLoad = () => {
  imageViewerLoading.value = false;
  imageViewerError.value = "";
}

const handleImageViewerError = () => {
  imageViewerLoading.value = false;
  imageViewerError.value = "图片加载失败，请检查文件是否仍可读取。";
}

const resetPreviewImageZoom = () => {
  previewImageFit.value = true;
  previewImageZoom.value = 100;
  resetPreviewImagePan();
}

const startPreviewImagePan = (event: PointerEvent) => {
  if (!canPanPreviewImage.value || event.button !== 0) return;
  event.preventDefault();
  const stage = event.currentTarget as HTMLElement;
  previewImagePointerId = event.pointerId;
  previewImageDragging.value = true;
  previewImageDragStartX = event.clientX;
  previewImageDragStartY = event.clientY;
  previewImageDragOriginX = previewImageOffsetX.value;
  previewImageDragOriginY = previewImageOffsetY.value;
  stage.setPointerCapture?.(event.pointerId);
}

const movePreviewImagePan = (event: PointerEvent) => {
  if (!previewImageDragging.value || previewImagePointerId !== event.pointerId) return;
  event.preventDefault();
  previewImageOffsetX.value = previewImageDragOriginX + event.clientX - previewImageDragStartX;
  previewImageOffsetY.value = previewImageDragOriginY + event.clientY - previewImageDragStartY;
}

const stopPreviewImagePan = (event: PointerEvent) => {
  if (previewImagePointerId !== event.pointerId) return;
  const stage = event.currentTarget as HTMLElement;
  stage.releasePointerCapture?.(event.pointerId);
  previewImageDragging.value = false;
  previewImagePointerId = null;
}

const startImageViewerPan = (event: PointerEvent) => {
  if (!canPanImageViewer.value || event.button !== 0) return;
  event.preventDefault();
  const stage = event.currentTarget as HTMLElement;
  imageViewerPointerId = event.pointerId;
  imageViewerDragging.value = true;
  imageViewerDragStartX = event.clientX;
  imageViewerDragStartY = event.clientY;
  imageViewerDragOriginX = imageViewerOffsetX.value;
  imageViewerDragOriginY = imageViewerOffsetY.value;
  stage.setPointerCapture?.(event.pointerId);
}

const moveImageViewerPan = (event: PointerEvent) => {
  if (!imageViewerDragging.value || imageViewerPointerId !== event.pointerId) return;
  event.preventDefault();
  imageViewerOffsetX.value = imageViewerDragOriginX + event.clientX - imageViewerDragStartX;
  imageViewerOffsetY.value = imageViewerDragOriginY + event.clientY - imageViewerDragStartY;
}

const stopImageViewerPan = (event: PointerEvent) => {
  if (imageViewerPointerId !== event.pointerId) return;
  const stage = event.currentTarget as HTMLElement;
  stage.releasePointerCapture?.(event.pointerId);
  imageViewerDragging.value = false;
  imageViewerPointerId = null;
}

const loadPreview = async (entry: ExplorerEntry) => {
  const version = ++previewLoadVersion;
  previewLoading.value = false;
  previewText.value = "";
  previewError.value = "";
  previewCopied.value = false;
  if (previewKind.value !== "text") return;
  previewLoading.value = true;
  try {
    const file = await getFile(entry.path);
    if (version !== previewLoadVersion) return;
    previewText.value = file.content;
  } catch (error) {
    if (version !== previewLoadVersion) return;
    previewError.value = error instanceof Error ? error.message : "预览失败";
  } finally {
    if (version === previewLoadVersion) previewLoading.value = false;
  }
}

const signOut = async () => {
  await logout();
  await router.replace("/login");
}
</script>

<template>
  <div class="main-shell">
    <header class="top-strip">
      <nav class="tab-strip" aria-label="目录标签">
        <button
            v-for="tab in fileStore.tabs"
            :key="tab.id"
            class="tab-button"
            :class="{active: tab.id === activeTab?.id, dragging: draggingTabId === tab.id, dropBefore: tabDropTargetId === tab.id && tabDropPlacement === 'before', dropAfter: tabDropTargetId === tab.id && tabDropPlacement === 'after'}"
            :title="`${tab.path} · Ctrl+Tab 切换 · 中键关闭`"
            draggable="true"
            @click="switchTab(tab.id)"
            @auxclick="handleTabAuxClick($event, tab.id)"
            @contextmenu="openTabContextMenu($event, tab.id)"
            @dragstart="startTabDrag($event, tab.id)"
            @dragover="dragOverTab($event, tab.id)"
            @dragleave="leaveTabDropTarget($event, tab.id)"
            @drop="dropTab($event, tab.id)"
            @dragend="finishTabDrag">
          <icon icon="icon-folder-fill" />
          <span>{{ tab.title }}</span>
          <span class="tab-close" title="关闭标签页 (Ctrl+W)" @click="closeTab($event, tab.id)">
            <icon icon="icon-close" size="small" />
          </span>
        </button>
        <button class="tab-add" title="新建标签页 (Ctrl+T)" @click="openTab">
          <icon icon="icon-add" />
        </button>
      </nav>
      <div
          v-if="tabContextMenu.visible"
          class="tab-context-menu"
          :style="{left: `${tabContextMenu.x}px`, top: `${tabContextMenu.y}px`}"
          @click.stop
          @contextmenu.prevent>
        <button @click="openTab">新建标签页</button>
        <button :disabled="!tabContextTarget" @click="duplicateTabFromMenu">复制标签页</button>
        <div class="tab-context-separator"></div>
        <button :disabled="!canCloseTabContext" @click="closeTabFromMenu">关闭标签页</button>
        <button :disabled="!canCloseOtherTabsContext" @click="closeOtherTabsFromMenu">关闭其他标签页</button>
        <button :disabled="!canCloseRightTabsContext" @click="closeRightTabsFromMenu">关闭右侧标签页</button>
      </div>
      <div class="top-actions">
        <label class="search-box">
          <input ref="searchInput" v-model="searchText" type="search" placeholder="搜索当前文件夹" @keydown.escape="clearSearch">
          <icon icon="icon-fenxiang" />
        </label>
        <button class="square-button" title="设置" @click="router.push('/setting')">
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
          <button class="nav-button" :disabled="!canNavigateBack" title="后退 (Alt+←)" @click="navigateBack">
            <icon icon="icon-back_android" size="large" />
          </button>
          <button class="nav-button" :disabled="!canNavigateForward" title="前进 (Alt+→)" @click="navigateForward">
            <icon icon="icon-back_android" size="large" class="rotate-180" />
          </button>
          <button class="nav-button" :disabled="!canNavigateUp" title="返回上级 (Alt+↑)" @click="navigateUp">
            <icon icon="icon-back_android" size="large" class="rotate-90" />
          </button>
          <button class="nav-button" title="刷新 (F5 / Ctrl+R)" @click="refreshCurrent(true)">
            <icon icon="icon-refresh" size="large" />
          </button>
          <breadcrumb ref="breadcrumbRef"></breadcrumb>
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

        <div v-if="taskPanelVisible" class="task-panel">
          <div class="task-panel-header">
            <div class="min-w-0">
              <p class="task-panel-title">后台任务 · {{ taskSummaryText }}</p>
              <p class="task-panel-message">{{ taskMessage || taskRefreshText }}</p>
            </div>
            <div class="task-panel-badges">
              <span v-if="taskStats.running" class="task-badge running">运行 {{ taskStats.running }}</span>
              <span v-if="taskStats.queued" class="task-badge queued">排队 {{ taskStats.queued }}</span>
              <span v-if="taskStats.failed" class="task-badge failed">失败 {{ taskStats.failed }}</span>
              <span v-if="taskStats.totalErrors" class="task-badge failed">错误 {{ taskStats.totalErrors }}</span>
            </div>
            <div class="task-panel-actions">
              <button class="task-icon-button" :disabled="tasksLoading" title="刷新任务" @click="loadTasks()">
                <icon icon="icon-refresh" size="normal"/>
              </button>
              <button class="task-icon-button" title="关闭任务面板" @click="closeTaskPanel">
                <icon icon="icon-close" size="normal"/>
              </button>
            </div>
          </div>

          <div v-if="tasksLoading" class="task-empty">正在加载任务...</div>
          <div v-else-if="!tasks.length" class="task-empty">暂无后台任务</div>
          <div v-else class="task-list">
            <div v-for="task in tasks" :key="task.id" class="task-row">
              <div class="task-row-main">
                <span class="task-kind">{{ taskKindText(task.kind) }}</span>
                <span :class="['task-state', taskStateClass(task.state)]">{{ taskStateText(task.state) }}</span>
                <span class="task-id">#{{ shortTaskId(task.id) }}</span>
              </div>
              <div class="task-progress">
                <div class="task-progress-track">
                  <span :style="{ width: taskProgress(task) }"></span>
                </div>
                <span class="task-progress-text">{{ taskProgress(task) }}</span>
              </div>
              <div v-if="taskCurrentPath(task)" class="task-current" :title="taskCurrentPath(task)">
                当前：{{ taskCurrentPath(task) }}
              </div>
              <div class="task-meta">
                <span>{{ taskBytesText(task) }}</span>
                <span>{{ formatBytes(task.speedBytesPerSec) }}/s</span>
                <span>{{ taskItemsText(task) }}</span>
                <span v-if="task.errors.length" class="task-errors">错误 {{ task.errors.length }}</span>
              </div>
              <div v-if="task.errors.length" class="task-error-list">
                <div v-for="error in task.errors.slice(0, 2)" :key="`${task.id}-${error.path}-${error.message}`" :title="`${error.path}：${error.message}`">
                  {{ error.path }}：{{ error.message }}
                </div>
              </div>
              <button class="task-cancel" :disabled="!canCancelTask(task)" @click="cancelTaskById(task)">取消</button>
            </div>
          </div>
          <section v-if="taskCancelConfirm.visible" class="task-cancel-confirm" @keydown.esc.prevent="closeTaskCancelConfirm">
            <div class="task-cancel-confirm-main">
              <strong>{{ taskCancelTitle }}</strong>
              <span>{{ taskCancelMessage }}</span>
              <span v-if="taskCancelConfirm.error" class="task-cancel-error">{{ taskCancelConfirm.error }}</span>
            </div>
            <div class="task-cancel-actions">
              <button type="button" class="task-cancel-secondary" :disabled="taskCancelConfirm.submitting" @click="closeTaskCancelConfirm">保留任务</button>
              <button type="button" class="task-cancel-primary" :disabled="taskCancelConfirm.submitting" @click="submitTaskCancelConfirm">
                {{ taskCancelConfirm.submitting ? "发送中..." : "确认取消" }}
              </button>
            </div>
          </section>
        </div>

        <div class="browser-area" :class="{previewing: previewPanelVisible}">
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
                @preview="previewSelected"
                @copy="copySelected"
                @cut="cutSelected"
                @paste="pasteSelected"
                @create-file="openCreatePanel('file')"
                @create-folder="openCreatePanel('folder')"
                @drop-entries="dropEntriesToFolder"
                @selection-change="handleSelectionChange"
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
            <form v-if="operationPanel.visible" class="operation-panel" @submit.prevent="submitOperationPanel">
              <div class="operation-panel-header">
                <div class="operation-panel-icon">
                  <icon :icon="operationPanelIcon" />
                </div>
                <div class="operation-panel-title">
                  <strong>{{ operationPanel.title }}</strong>
                  <span>{{ operationPanel.message }}</span>
                </div>
                <button type="button" class="operation-panel-close" title="关闭" @click="closeOperationPanel">
                  <icon icon="icon-close" />
                </button>
              </div>
              <label class="operation-field">
                <span>{{ operationPanelNameLabel }}</span>
                <input
                    v-model="operationPanel.name"
                    type="text"
                    autocomplete="off"
                    :disabled="operationPanel.submitting"
                    @keydown.esc.prevent="closeOperationPanel">
              </label>
              <div v-if="operationPanel.kind === 'archive'" class="operation-field">
                <span>压缩格式</span>
                <div class="operation-segmented">
                  <button type="button" :class="{active: operationPanel.format === 'zip'}" @click="operationPanel.format = 'zip'">ZIP</button>
                  <button type="button" :class="{active: operationPanel.format === 'tarGz'}" @click="operationPanel.format = 'tarGz'">TAR.GZ</button>
                </div>
              </div>
              <div v-if="operationPanel.kind === 'archive'" class="operation-hint">
                {{ operationPanel.entries.length }} 项将加入压缩包
              </div>
              <div v-else-if="operationPanel.kind === 'extract' && operationPanel.sourceEntry" class="operation-hint">
                源文件：{{ operationPanel.sourceEntry.name }}
              </div>
              <div class="operation-actions">
                <button type="button" class="operation-secondary" :disabled="operationPanel.submitting" @click="closeOperationPanel">取消</button>
                <button type="submit" class="operation-primary" :disabled="operationPanel.submitting || !operationPanel.name.trim()">
                  {{ operationPanel.submitting ? "处理中..." : operationPanel.primaryText }}
                </button>
              </div>
            </form>
            <section
                v-if="deleteConfirm.visible"
                ref="deleteConfirmRef"
                class="delete-confirm-panel"
                tabindex="-1"
                @keydown.esc.prevent="closeDeleteConfirm">
              <div class="delete-confirm-header">
                <div class="delete-confirm-icon">
                  <icon icon="icon-delete-fill" />
                </div>
                <div class="delete-confirm-title">
                  <strong>{{ deleteConfirmTitle }}</strong>
                  <span>{{ deleteConfirmMessage }}</span>
                </div>
                <button type="button" class="operation-panel-close" title="关闭" @click="closeDeleteConfirm">
                  <icon icon="icon-close" />
                </button>
              </div>
              <div class="delete-confirm-list">
                <div v-for="item in deleteConfirmItems" :key="item.path" :title="item.path">
                  <icon :icon="item.type === 'folder' ? 'icon-folder-fill' : 'icon-file-fill'" />
                  <span>{{ item.name }}</span>
                </div>
                <div v-if="deleteConfirmExtraCount" class="delete-confirm-more">
                  另有 {{ deleteConfirmExtraCount }} 项
                </div>
              </div>
              <p v-if="deleteConfirm.error" class="delete-confirm-error">{{ deleteConfirm.error }}</p>
              <div class="delete-confirm-actions">
                <button type="button" class="operation-secondary" :disabled="deleteConfirm.submitting" @click="closeDeleteConfirm">取消</button>
                <button type="button" class="delete-confirm-primary" :disabled="deleteConfirm.submitting" @click="submitDeleteConfirm">
                  {{ deleteConfirm.submitting ? "创建任务中..." : "移动到回收站" }}
                </button>
              </div>
            </section>
          </div>
          <aside v-if="previewPanelVisible" class="preview-pane">
            <div class="preview-header">
              <div class="preview-title-block">
                <span class="preview-title">{{ previewTitleText }}</span>
                <span class="preview-subtitle">{{ previewSubtitleText }}</span>
              </div>
              <div class="preview-actions">
                <button v-if="canEditPreview" title="编辑" @click="openPreviewInEditor">
                  <icon icon="icon-edit-filling" />
                </button>
                <button title="下载" :disabled="!previewEntry" @click="downloadSelected(previewEntry ?? undefined)">
                  <icon icon="icon-download" />
                </button>
                <button title="关闭预览" @click="closePreview">
                  <icon icon="icon-close" />
                </button>
              </div>
            </div>
            <div class="preview-meta-list">
              <div v-for="item in previewMeta" :key="item.label" :title="item.value">
                <span>{{ item.label }}</span>
                <strong>{{ item.value }}</strong>
              </div>
            </div>
            <div v-if="previewKind === 'image'" class="preview-tool-row">
              <button :class="{active: previewImageFit}" @click="resetPreviewImageZoom">适应</button>
              <button @click="zoomPreviewImage(-25)">-</button>
              <span>{{ previewZoomText }}</span>
              <button @click="zoomPreviewImage(25)">+</button>
              <button title="打开图片查看" @click="openPreviewImageViewer">
                <icon icon="icon-unfold" color="currentColor" />
                <span>打开查看</span>
              </button>
            </div>
            <div v-else-if="previewKind === 'text'" class="preview-tool-row">
              <button :class="{active: previewTextWrap}" @click="previewTextWrap = !previewTextWrap">
                {{ previewTextWrap ? "自动换行" : "不换行" }}
              </button>
              <button :disabled="!previewText" @click="copyPreviewText">{{ previewCopied ? "已复制" : "复制" }}</button>
              <span>{{ previewTextStats }}</span>
            </div>
            <div class="preview-body" :class="previewKind">
              <div v-if="!previewEntry" class="preview-placeholder muted">
                <icon icon="icon-file-fill" size="3rem" />
                <span>选择一个文件以预览</span>
              </div>
              <div v-else-if="previewLoading" class="preview-placeholder">正在加载预览...</div>
              <div v-else-if="previewError" class="preview-placeholder error">{{ previewError }}</div>
              <div
                  v-else-if="previewEntry && previewKind === 'image'"
                  class="image-stage"
                  :class="{fit: previewImageFit, panning: canPanPreviewImage, dragging: previewImageDragging}"
                  @pointerdown="startPreviewImagePan"
                  @pointermove="movePreviewImagePan"
                  @pointerup="stopPreviewImagePan"
                  @pointercancel="stopPreviewImagePan"
                  @lostpointercapture="previewImageDragging = false"
                  @dblclick="resetPreviewImageZoom">
                <img :src="downloadUrl(previewEntry.path)" :alt="previewEntry.name" :style="previewImageStyle">
              </div>
              <pre v-else-if="previewKind === 'text'" :class="{nowrap: !previewTextWrap}">{{ previewText }}</pre>
              <audio v-else-if="previewEntry && previewKind === 'audio'" :src="downloadUrl(previewEntry.path)" controls></audio>
              <video v-else-if="previewEntry && previewKind === 'video'" :src="downloadUrl(previewEntry.path)" controls></video>
              <div v-else class="preview-placeholder">
                <icon icon="icon-file-fill" size="3rem" />
                <span>暂不支持预览此类型</span>
                <button @click="downloadSelected(previewEntry ?? undefined)">下载文件</button>
              </div>
            </div>
          </aside>
        </div>

        <Teleport to="body" :disabled="!imageViewerPageFullscreen">
          <section
              v-if="imageViewerVisible && imageViewerEntry"
              ref="imageViewerRef"
              class="image-viewer"
              :class="{pageFullscreen: imageViewerPageFullscreen}"
              tabindex="-1"
              @keydown.esc.prevent="closeImageViewer">
            <div class="image-viewer-toolbar">
              <div class="image-viewer-title">
                <strong>{{ imageViewerEntry.name }}</strong>
                <span>{{ imageViewerSubtitle }}</span>
              </div>
              <div class="image-viewer-actions">
                <button title="上一张 (←)" :disabled="!canShowPreviousImage" @click="showAdjacentImage(-1)">
                  <icon icon="icon-back_android" color="currentColor" />
                </button>
                <button title="下一张 (→)" :disabled="!canShowNextImage" @click="showAdjacentImage(1)">
                  <icon icon="icon-back_android" color="currentColor" class="rotate-180" />
                </button>
                <button :class="{active: imageViewerFit}" title="适应窗口" @click="resetImageViewerZoom">适应</button>
                <button title="缩小" @click="zoomImageViewer(-25)">-</button>
                <span>{{ imageViewerZoomText }}</span>
                <button title="放大" @click="zoomImageViewer(25)">+</button>
                <button title="网页全屏 (F)" :class="{active: imageViewerPageFullscreen}" @click="toggleImageViewerPageFullscreen">
                  <icon icon="icon-renamebox" color="currentColor" />
                </button>
                <button title="浏览器全屏" :class="{active: imageViewerFullscreen}" @click="toggleImageViewerFullscreen">
                  <icon icon="icon-unfold" color="currentColor" />
                </button>
                <button title="缩略图 (T)" :class="{active: imageViewerShowFilmstrip}" :disabled="imageViewerCount <= 1" @click="toggleImageViewerFilmstrip">
                  <icon icon="icon-viewgrid" color="currentColor" />
                </button>
                <button title="下载" @click="downloadSelected(imageViewerEntry)">
                  <icon icon="icon-download" color="currentColor" />
                </button>
                <button title="关闭" @click="closeImageViewer">
                  <icon icon="icon-close" color="currentColor" />
                </button>
              </div>
            </div>
            <div
                class="image-viewer-stage"
                :class="{fit: imageViewerFit, panning: canPanImageViewer, dragging: imageViewerDragging}"
                @pointerdown="startImageViewerPan"
                @pointermove="moveImageViewerPan"
                @pointerup="stopImageViewerPan"
                @pointercancel="stopImageViewerPan"
                @lostpointercapture="imageViewerDragging = false"
                @wheel="handleImageViewerWheel"
                @dblclick="resetImageViewerZoom">
              <div v-if="imageViewerLoading" class="image-viewer-status">正在加载图片...</div>
              <div v-if="imageViewerError" class="image-viewer-status error">{{ imageViewerError }}</div>
              <img
                  :key="imageViewerEntry.path"
                  :src="downloadUrl(imageViewerEntry.path)"
                  :alt="imageViewerEntry.name"
                  :style="imageViewerStyle"
                  @load="handleImageViewerLoad"
                  @error="handleImageViewerError">
            </div>
            <div v-if="canShowImageViewerFilmstrip" class="image-viewer-filmstrip" aria-label="图片列表">
              <button
                  v-for="item in imageViewerFilmstripEntries"
                  :key="item.entry.path"
                  class="image-viewer-thumb"
                  :class="{active: item.entry.path === imageViewerEntry.path}"
                  :title="`${item.index + 1} / ${imageViewerCount} · ${item.entry.name}`"
                  @click="showImageAt(item.index)">
                <img :src="downloadUrl(item.entry.path)" :alt="item.entry.name" loading="lazy">
                <span>{{ item.index + 1 }}</span>
              </button>
            </div>
          </section>
        </Teleport>
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

.tab-strip {
  @apply flex min-w-0 grow items-center gap-2 overflow-hidden rounded-xl border border-white bg-white/70 p-1 shadow-sm backdrop-blur;
}

.tab-button {
  @apply relative inline-flex h-9 min-w-32 max-w-52 shrink items-center gap-2 rounded-lg border border-slate-200 bg-white px-3 text-sm text-slate-800 shadow-sm hover:bg-slate-50;
}

.tab-button.active {
  @apply border-blue-600 bg-blue-600 text-white;
}

.tab-button.dragging {
  @apply opacity-55;
}

.tab-button.dropBefore {
  @apply bg-blue-50 ring-2 ring-blue-200;
}

.tab-button.dropAfter {
  @apply bg-blue-50 ring-2 ring-blue-200;
}

.tab-button.active.dropBefore,
.tab-button.active.dropAfter {
  @apply bg-blue-600 ring-blue-200;
}

.tab-button.dropBefore::before,
.tab-button.dropAfter::after {
  content: "";
  @apply absolute bottom-1 top-1 w-0.5 rounded-full bg-blue-500;
}

.tab-button.dropBefore::before {
  @apply left-1;
}

.tab-button.dropAfter::after {
  @apply right-1;
}

.tab-button.active.dropBefore::before,
.tab-button.active.dropAfter::after {
  @apply bg-white;
}

.tab-button span:not(.tab-close) {
  @apply min-w-0 truncate;
}

.tab-close {
  @apply ml-auto inline-flex h-5 w-5 shrink-0 items-center justify-center rounded hover:bg-black/10;
}

.tab-add {
  @apply inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-white text-slate-700 shadow-sm hover:bg-blue-50;
}

.tab-context-menu {
  @apply fixed z-50 w-46 rounded-md border border-slate-200 bg-white py-1 text-sm shadow-xl;
}

.tab-context-menu button {
  @apply block h-8 w-full px-3 text-left text-slate-700 hover:bg-blue-50 disabled:text-slate-300 disabled:hover:bg-white;
}

.tab-context-separator {
  @apply my-1 border-t border-slate-100;
}

.top-actions {
  @apply flex h-full shrink-0 items-center gap-3;
}

.search-box {
  @apply flex h-10 w-72 items-center gap-2 rounded-xl border border-white bg-white/70 px-3 shadow-sm backdrop-blur;
}

.search-box input {
  @apply min-w-0 grow bg-transparent text-sm outline-none placeholder:text-slate-400;
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
.task-icon-button,
.preview-header button {
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
}

.browser-area.previewing {
  @apply grid-cols-[minmax(0,1fr)_22rem];
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

.operation-panel {
  @apply absolute left-1/2 top-6 z-30 flex w-[min(28rem,calc(100%-2rem))] -translate-x-1/2 flex-col gap-3 rounded-lg border border-slate-200 bg-white p-4 text-sm shadow-2xl;
}

.delete-confirm-panel {
  @apply absolute left-1/2 top-6 z-30 flex w-[min(30rem,calc(100%-2rem))] -translate-x-1/2 flex-col gap-3 rounded-lg border border-red-100 bg-white p-4 text-sm text-slate-700 shadow-2xl outline-none;
}

.operation-panel-header {
  @apply flex items-start gap-3;
}

.delete-confirm-header {
  @apply flex items-start gap-3;
}

.operation-panel-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-blue-50 text-xl text-blue-600;
}

.delete-confirm-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-red-50 text-xl text-red-600;
}

.operation-panel-title {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.delete-confirm-title {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.operation-panel-title strong {
  @apply truncate text-base font-semibold text-slate-900;
}

.delete-confirm-title strong {
  @apply truncate text-base font-semibold text-slate-900;
}

.operation-panel-title span {
  @apply truncate text-xs text-slate-500;
}

.delete-confirm-title span {
  @apply text-xs leading-5 text-slate-500;
}

.operation-panel-close {
  @apply flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-slate-500 hover:bg-slate-100;
}

.operation-field {
  @apply flex flex-col gap-1.5 text-xs font-medium text-slate-500;
}

.operation-field input {
  @apply h-9 rounded-md border border-slate-200 bg-white px-3 text-sm font-normal text-slate-900 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100 disabled:bg-slate-50 disabled:text-slate-400;
}

.operation-segmented {
  @apply inline-flex w-fit overflow-hidden rounded-md border border-slate-200 bg-slate-50;
}

.operation-segmented button {
  @apply h-8 border-r border-slate-200 px-3 text-xs font-semibold text-slate-600 last:border-r-0 hover:bg-white;
}

.operation-segmented button.active {
  @apply bg-blue-600 text-white hover:bg-blue-600;
}

.operation-hint {
  @apply rounded-md border border-blue-100 bg-blue-50 px-3 py-2 text-xs text-blue-700;
}

.operation-actions {
  @apply flex justify-end gap-2 pt-1;
}

.delete-confirm-list {
  @apply flex max-h-40 flex-col gap-1 overflow-auto rounded-md border border-slate-100 bg-slate-50 p-2;
}

.delete-confirm-list div {
  @apply flex min-h-7 min-w-0 items-center gap-2 rounded px-2 text-xs text-slate-600;
}

.delete-confirm-list span {
  @apply min-w-0 truncate;
}

.delete-confirm-more {
  @apply text-slate-400;
}

.delete-confirm-error {
  @apply rounded-md border border-red-100 bg-red-50 px-3 py-2 text-xs text-red-600;
}

.delete-confirm-actions {
  @apply flex justify-end gap-2 pt-1;
}

.operation-secondary,
.operation-primary {
  @apply h-9 rounded-md px-4 text-sm font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.operation-secondary {
  @apply border border-slate-200 bg-white text-slate-700 hover:bg-slate-50;
}

.operation-primary {
  @apply bg-blue-600 text-white hover:bg-blue-700;
}

.delete-confirm-primary {
  @apply h-9 rounded-md bg-red-600 px-4 text-sm font-medium text-white hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50;
}

.preview-pane {
  @apply flex min-h-0 flex-col border-l border-slate-200 bg-white;
}

.preview-header {
  @apply flex min-h-12 shrink-0 items-center justify-between gap-2 border-b border-slate-200 px-3 text-sm font-medium;
}

.preview-title-block {
  @apply flex min-w-0 flex-col;
}

.preview-title {
  @apply min-w-0 truncate;
}

.preview-subtitle {
  @apply text-xs font-normal text-slate-500;
}

.preview-actions {
  @apply flex shrink-0 items-center gap-1;
}

.preview-header button {
  @apply h-7 w-7 shrink-0 disabled:cursor-not-allowed disabled:opacity-40;
}

.preview-meta-list {
  @apply grid shrink-0 grid-cols-2 gap-x-3 gap-y-1 border-b border-slate-100 bg-slate-50/70 px-3 py-2 text-xs;
}

.preview-meta-list div {
  @apply min-w-0;
}

.preview-meta-list span {
  @apply mr-1 text-slate-400;
}

.preview-meta-list strong {
  @apply font-normal text-slate-700;
}

.preview-meta-list div:last-child {
  @apply col-span-2;
}

.preview-meta-list strong {
  @apply inline-block max-w-full truncate align-bottom;
}

.preview-tool-row {
  @apply flex h-9 shrink-0 items-center gap-1 border-b border-slate-100 bg-white px-3 text-xs text-slate-500;
}

.preview-tool-row button {
  @apply inline-flex h-6 items-center gap-1 rounded border border-transparent px-2 text-slate-600 hover:border-slate-200 hover:bg-blue-50 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:border-transparent disabled:hover:bg-transparent;
}

.preview-tool-row button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.preview-tool-row > span {
  @apply ml-auto tabular-nums;
}

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm text-slate-700;
}

.preview-body.image,
.preview-body.audio,
.preview-body.video {
  @apply bg-slate-50;
}

.image-stage {
  @apply flex h-full min-h-0 w-full touch-none select-none items-center justify-center overflow-hidden p-3;
}

.image-stage.fit {
  @apply overflow-hidden;
}

.image-stage.panning {
  @apply cursor-grab;
}

.image-stage.dragging {
  @apply cursor-grabbing;
}

.image-stage img {
  @apply rounded object-contain shadow-sm;
  user-select: none;
  -webkit-user-drag: none;
}

.preview-body pre {
  @apply min-h-full whitespace-pre-wrap break-words bg-white p-3 font-mono text-xs leading-5 text-slate-800;
}

.preview-body pre.nowrap {
  @apply whitespace-pre break-normal;
}

.preview-body audio,
.preview-body video {
  @apply m-auto max-h-full max-w-full;
}

.preview-placeholder {
  @apply flex h-full min-h-48 flex-col items-center justify-center gap-3 text-center text-slate-500;
}

.preview-placeholder.error {
  @apply text-red-600;
}

.preview-placeholder.muted {
  @apply text-slate-400;
}

.preview-placeholder button {
  @apply rounded-md border border-slate-200 bg-white px-3 py-1.5 text-sm text-slate-700 hover:bg-blue-50;
}

.image-viewer {
  @apply absolute inset-0 z-40 flex flex-col overflow-hidden rounded-lg bg-slate-950/72 text-white outline-none backdrop-blur-sm;
}

.image-viewer.pageFullscreen {
  @apply fixed inset-0 z-50 rounded-none;
}

.image-viewer-toolbar {
  @apply flex min-h-14 shrink-0 items-center justify-between gap-3 border-b border-white/15 bg-slate-950/75 px-4 backdrop-blur;
}

.image-viewer-title {
  @apply flex min-w-0 flex-col;
}

.image-viewer-title strong {
  @apply truncate text-sm font-semibold;
}

.image-viewer-title span {
  @apply truncate text-xs text-slate-300;
}

.image-viewer-actions {
  @apply flex shrink-0 items-center gap-1 text-xs text-slate-300;
}

.image-viewer-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-white/20 bg-white/10 px-2 text-sm font-medium text-white shadow-sm hover:border-white/30 hover:bg-white/20;
}

.image-viewer-actions button:disabled {
  @apply cursor-not-allowed border-white/10 bg-white/5 opacity-35 hover:border-white/10 hover:bg-white/5;
}

.image-viewer-actions button.active {
  @apply border-blue-200/70 bg-blue-500/35 text-white;
}

.image-viewer-actions span {
  @apply w-14 text-center tabular-nums;
}

.image-viewer-stage {
  @apply relative flex min-h-0 grow touch-none select-none items-center justify-center overflow-hidden bg-transparent p-5;
}

.image-viewer-status {
  @apply absolute rounded-md border border-white/10 bg-slate-950/60 px-3 py-2 text-sm text-slate-100 shadow-xl backdrop-blur;
}

.image-viewer-status.error {
  @apply border-red-300/30 bg-red-950/70 text-red-100;
}

.image-viewer-stage.panning {
  @apply cursor-grab;
}

.image-viewer-stage.dragging {
  @apply cursor-grabbing;
}

.image-viewer-stage img {
  @apply max-h-full max-w-full select-none rounded object-contain shadow-2xl;
  user-select: none;
  -webkit-user-drag: none;
}

.image-viewer-filmstrip {
  @apply flex h-24 shrink-0 items-center gap-2 overflow-x-auto border-t border-white/10 bg-slate-950/45 px-4 py-2 backdrop-blur;
}

.image-viewer-thumb {
  @apply relative h-16 w-20 shrink-0 overflow-hidden rounded-md border border-white/10 bg-white/5 p-0.5 text-white opacity-75 outline-none hover:border-white/35 hover:opacity-100;
}

.image-viewer-thumb.active {
  @apply border-blue-300 bg-blue-500/20 opacity-100 shadow-[0_0_0_2px_rgba(96,165,250,0.25)];
}

.image-viewer-thumb img {
  @apply h-full w-full rounded object-cover;
}

.image-viewer-thumb span {
  @apply absolute bottom-1 right-1 rounded bg-slate-950/70 px-1 text-[10px] leading-4 text-slate-100;
}

.task-panel {
  @apply flex max-h-72 shrink-0 flex-col gap-2 overflow-hidden border-b border-slate-200 bg-white px-3 py-2;
}

.task-panel-header {
  @apply flex items-center justify-between gap-3;
}

.task-panel-title {
  @apply text-sm font-semibold text-slate-900;
}

.task-panel-message {
  @apply truncate text-xs text-slate-500;
}

.task-panel-badges {
  @apply ml-auto hidden shrink-0 items-center gap-1 md:flex;
}

.task-badge {
  @apply rounded px-2 py-0.5 text-xs font-medium;
}

.task-badge.running {
  @apply bg-blue-100 text-blue-700;
}

.task-badge.queued {
  @apply bg-slate-200 text-slate-700;
}

.task-badge.failed {
  @apply bg-red-100 text-red-700;
}

.task-panel-actions {
  @apply flex shrink-0 items-center gap-1;
}

.task-icon-button {
  @apply h-8 w-8 disabled:cursor-not-allowed disabled:opacity-50;
}

.task-empty {
  @apply flex h-16 items-center justify-center rounded border border-dashed border-slate-200 text-sm text-slate-500;
}

.task-list {
  @apply flex flex-col gap-2 overflow-auto pr-1;
}

.task-cancel-confirm {
  @apply flex shrink-0 items-center justify-between gap-3 rounded border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-900;
}

.task-cancel-confirm-main {
  @apply flex min-w-0 flex-col gap-0.5;
}

.task-cancel-confirm-main strong,
.task-cancel-confirm-main span {
  @apply truncate;
}

.task-cancel-confirm-main span {
  @apply text-xs text-amber-700;
}

.task-cancel-error {
  @apply text-red-600;
}

.task-cancel-actions {
  @apply flex shrink-0 items-center gap-2;
}

.task-cancel-secondary,
.task-cancel-primary {
  @apply h-8 rounded border px-3 text-xs font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.task-cancel-secondary {
  @apply border-amber-200 bg-white text-amber-800 hover:bg-amber-100;
}

.task-cancel-primary {
  @apply border-amber-600 bg-amber-600 text-white hover:bg-amber-700;
}

.task-row {
  @apply grid min-h-16 grid-cols-[minmax(9rem,1.1fr)_minmax(10rem,1.2fr)_minmax(14rem,1.5fr)_4rem] items-center gap-x-3 gap-y-2 rounded border border-slate-100 bg-slate-50 px-3 py-2 text-sm;
}

.task-row-main {
  @apply flex min-w-0 items-center gap-2;
}

.task-kind {
  @apply shrink-0 font-medium text-slate-900;
}

.task-id {
  @apply truncate text-xs text-slate-500;
}

.task-state {
  @apply shrink-0 rounded px-2 py-0.5 text-xs;
}

.task-state.queued {
  @apply bg-slate-200 text-slate-700;
}

.task-state.running {
  @apply bg-blue-100 text-blue-700;
}

.task-state.completed {
  @apply bg-emerald-100 text-emerald-700;
}

.task-state.failed {
  @apply bg-red-100 text-red-700;
}

.task-state.cancelled {
  @apply bg-amber-100 text-amber-700;
}

.task-progress {
  @apply flex min-w-0 items-center gap-2;
}

.task-progress-track {
  @apply h-2 min-w-0 grow overflow-hidden rounded bg-slate-200;
}

.task-progress-track span {
  @apply block h-full rounded bg-blue-500;
}

.task-progress-text {
  @apply w-10 shrink-0 text-right text-xs tabular-nums text-slate-600;
}

.task-current {
  @apply truncate text-xs text-slate-500;
}

.task-meta {
  @apply flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1 text-xs text-slate-600;
}

.task-errors {
  @apply text-red-600;
}

.task-error-list {
  @apply col-span-3 flex min-w-0 flex-col gap-1 rounded border border-red-100 bg-red-50 px-2 py-1 text-xs text-red-700;
}

.task-error-list div {
  @apply truncate;
}

.task-cancel {
  @apply h-8 rounded border border-slate-200 bg-white px-2 text-sm text-slate-700 hover:bg-red-50 hover:text-red-600 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:bg-white;
}
</style>
