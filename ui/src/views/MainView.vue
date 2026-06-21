<script setup lang="ts">
import {computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch} from "vue";
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

type RenamePayload = {
  entry: ExplorerEntry;
  name: string;
}

type DropEntriesPayload = {
  entries: ExplorerEntry[];
  target: ExplorerEntry;
  action: "copy" | "move";
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
const previewTextWrap = ref(true);
const previewCopied = ref(false);
const currentSelection = ref<ExplorerEntry[]>([]);
const fileClipboardAction = ref<FileClipboardAction | null>(null);
const fileClipboardEntries = ref<ExplorerEntry[]>([]);
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
let previewLoadVersion = 0;
let previewCopyTimer: number | undefined;
let uploadDragDepth = 0;
let taskPollTimer: number | undefined;

const activeTab = computed(() => fileStore.tabs.find(tab => tab.id === fileStore.activeTabId) ?? fileStore.tabs[0]);
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
  transform: previewImageFit.value ? "none" : `scale(${previewImageZoom.value / 100})`,
  transformOrigin: "center center"
}));

const previewZoomText = computed(() => previewImageFit.value ? "适应" : `${previewImageZoom.value}%`);

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

const clearPreviewContent = () => {
  previewLoadVersion += 1;
  previewEntry.value = null;
  previewLoading.value = false;
  previewText.value = "";
  previewError.value = "";
  previewImageFit.value = true;
  previewImageZoom.value = 100;
  previewCopied.value = false;
}

const closePanels = () => {
  previewPanelVisible.value = false;
  clearPreviewContent();
  operationPanel.value.visible = false;
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
})

onBeforeUnmount(() => {
  if (previewCopyTimer) window.clearTimeout(previewCopyTimer);
  stopTaskPolling();
  window.removeEventListener("keydown", handleWindowKeyDown);
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
    window.alert(error instanceof Error ? error.message : "加载任务失败");
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
  if (!window.confirm(`取消${taskKindText(task.kind)}任务？`)) return;
  try {
    await cancelTask(task.id);
    taskMessage.value = `已发送取消请求：${shortTaskId(task.id)}`;
    await loadTasks();
  } catch (error) {
    window.alert(error instanceof Error ? error.message : "取消任务失败");
  }
}

const refreshCurrent = async () => {
  closePanels();
  if (currentFolder() === "/") {
    await loadRoot();
  }
  await explorerRef.value?.refresh(currentFolder());
}

const runOperation = async (operation: () => Promise<void>) => {
  try {
    await operation();
    await refreshCurrent();
  } catch (error) {
    window.alert(error instanceof Error ? error.message : "操作失败");
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
    window.alert("请选择一个文件或文件夹");
    return;
  }
  explorerRef.value?.startRename();
}

const renameSelected = async ({entry, name}: RenamePayload) => {
  const nextName = name.trim();
  if (!nextName || nextName === entry.name) return;
  await runOperation(async () => {
    await moveEntry(entry.path, joinPath(parentPath(entry.path), nextName));
  })
}

const deleteSelected = async (entry = selectedEntry()) => {
  const entries = selectedEntries(entry);
  if (!entries.length) {
    window.alert("请选择文件或文件夹");
    return;
  }
  const message = entries.length === 1 ? `删除 ${entries[0].name}？` : `删除选中的 ${entries.length} 项？`;
  if (!window.confirm(message)) return;
  try {
    const task = await createDeleteTask(entries.map(item => item.path));
    await taskStarted(task.id, "删除任务");
    if (fileClipboardAction.value === "cut") {
      const deleted = new Set(entries.map(item => item.path));
      fileClipboardEntries.value = fileClipboardEntries.value.filter(item => !deleted.has(item.path));
      if (!fileClipboardEntries.value.length) fileClipboardAction.value = null;
    }
    await refreshCurrent();
  } catch (error) {
    window.alert(error instanceof Error ? error.message : "创建删除任务失败");
  }
}

const downloadSelected = async (entry = singleSelectedEntry()) => {
  if (!entry || entry.type !== "file") {
    window.alert("请选择一个文件");
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
    window.alert(error instanceof Error ? error.message : "下载失败");
  }
}

const setFileClipboard = (action: FileClipboardAction, entry = selectedEntry()) => {
  const entries = selectedEntries(entry);
  if (!entries.length) {
    window.alert("请选择文件或文件夹");
    return;
  }
  fileClipboardAction.value = action;
  fileClipboardEntries.value = entries;
  taskMessage.value = `${action === "cut" ? "已剪切" : "已复制"} ${entries.length} 项`;
}

const copySelected = (entry?: ExplorerEntry) => {
  setFileClipboard("copy", entry ?? selectedEntry());
}

const cutSelected = (entry?: ExplorerEntry) => {
  setFileClipboard("cut", entry ?? selectedEntry());
}

const pasteSelected = async () => {
  if (!hasClipboard.value || !fileClipboardAction.value) {
    window.alert("剪贴板为空");
    return;
  }
  const targetPath = currentFolder();
  const entries = fileClipboardEntries.value;
  const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(targetPath, entry.path));
  if (nestedFolder) {
    window.alert(`不能将 ${nestedFolder.name} 粘贴到它自身或子文件夹中`);
    return;
  }
  const sameFolder = entries.some(entry => parentPath(entry.path) === targetPath);
  if (fileClipboardAction.value === "cut" && sameFolder) {
    window.alert("剪切项已经在当前文件夹中");
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
    window.alert(error instanceof Error ? error.message : "创建粘贴任务失败");
  }
}

const dropEntriesToFolder = async ({entries, target, action}: DropEntriesPayload) => {
  if (target.type !== "folder" || !entries.length) return;
  const nestedFolder = entries.find(entry => entry.type === "folder" && isSameOrDescendantPath(target.path, entry.path));
  if (nestedFolder) {
    window.alert(`不能将 ${nestedFolder.name} 放入它自身或子文件夹中`);
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
    window.alert(error instanceof Error ? error.message : "创建拖拽任务失败");
  }
}

const archiveSelected = (entry = selectedEntry()) => {
  const entries = selectedEntries(entry);
  if (!entries.length) {
    window.alert("请选择文件或文件夹");
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
    window.alert("请选择一个 zip、tar.gz 或 tgz 压缩包");
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
    window.alert(`${operationPanelNameLabel.value}不能为空`);
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
    window.alert(error instanceof Error ? error.message : "操作失败");
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
  return Boolean(target.closest("input, textarea, select, [contenteditable='true'], .ace_editor, .operation-panel"));
}

const shouldIgnoreActionShortcut = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  return Boolean(target.closest("button, a, input, textarea, select, [contenteditable='true'], .ace_editor, .operation-panel, .context-menu, .task-panel"));
}

const shouldKeepEditorFindShortcut = (target: EventTarget | null) => {
  if (fileStore.showEditor) return true;
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".ace_editor, .operation-panel"));
}

const focusSearch = () => {
  if (fileStore.showEditor) return;
  searchInput.value?.focus();
  searchInput.value?.select();
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

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === "f") {
    if (shouldKeepEditorFindShortcut(event.target)) return;
    event.preventDefault();
    focusSearch();
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
  fileStore.openTab(currentFolder());
  closePanels();
}

const openEntryInNewTab = (entry: ExplorerEntry) => {
  if (entry.type !== "folder") return;
  fileStore.openPathInNewTab(entry.path);
  closePanels();
}

const switchTab = (tabId: string) => {
  fileStore.switchTab(tabId);
  closePanels();
}

const closeTab = (event: MouseEvent, tabId: string) => {
  event.stopPropagation();
  fileStore.closeTab(tabId);
  closePanels();
}

const previewSelected = (entry = selectedEntry()) => {
  if (!entry || entry.type !== "file") {
    window.alert("请选择文件");
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
    window.alert("复制失败，请手动选择文本复制");
  }
}

const zoomPreviewImage = (delta: number) => {
  previewImageFit.value = false;
  previewImageZoom.value = Math.min(300, Math.max(25, previewImageZoom.value + delta));
}

const resetPreviewImageZoom = () => {
  previewImageFit.value = true;
  previewImageZoom.value = 100;
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
            :class="{active: tab.id === activeTab?.id}"
            :title="tab.path"
            @click="switchTab(tab.id)">
          <icon icon="icon-folder-fill" />
          <span>{{ tab.title }}</span>
          <span class="tab-close" @click="closeTab($event, tab.id)">
            <icon icon="icon-close" size="small" />
          </span>
        </button>
        <button class="tab-add" title="新建标签页" @click="openTab">
          <icon icon="icon-add" />
        </button>
      </nav>
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
          <button class="icon-tool" title="新建文件夹" @click="openCreatePanel('folder')">
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
          <button class="nav-button" title="刷新" @click="refreshCurrent">
            <icon icon="icon-refresh" size="large" />
          </button>
          <breadcrumb></breadcrumb>
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
          <button class="command-button" @click="openCreatePanel('folder')">
            <icon icon="icon-folder-add-fill" />
            <span>新建文件夹</span>
          </button>
          <span class="command-separator"></span>
          <button class="command-button" :disabled="!hasSelection" @click="cutSelected()">
            <icon icon="icon-scissors" />
            <span>剪切</span>
          </button>
          <button class="command-button" :disabled="!hasSelection" @click="copySelected()">
            <icon icon="icon-copy" />
            <span>复制</span>
          </button>
          <button class="command-button" :disabled="!canPasteSelection" @click="pasteSelected()">
            <icon icon="icon-paste" />
            <span>粘贴</span>
          </button>
          <span class="command-separator"></span>
          <button class="command-button" :disabled="!canDownloadSelection" @click="downloadSelected()">
            <icon icon="icon-download" />
            <span>下载</span>
          </button>
          <button class="command-button" :disabled="!canPreviewSelection" @click="previewSelected()">
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
          <span class="command-status" :title="selectionStatusText">{{ selectionStatusText }}</span>
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
                @open-new-tab="openEntryInNewTab">
            </explorer>
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
              <div v-else-if="previewEntry && previewKind === 'image'" class="image-stage" :class="{fit: previewImageFit}">
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
  @apply inline-flex h-9 min-w-32 max-w-52 shrink items-center gap-2 rounded-lg border border-slate-200 bg-white px-3 text-sm text-slate-800 shadow-sm hover:bg-slate-50;
}

.tab-button.active {
  @apply border-blue-600 bg-blue-600 text-white;
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
  @apply flex min-h-0 flex-col overflow-hidden rounded-xl border border-slate-200 bg-white/80 shadow-sm backdrop-blur;
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

.operation-panel {
  @apply absolute left-1/2 top-6 z-30 flex w-[min(28rem,calc(100%-2rem))] -translate-x-1/2 flex-col gap-3 rounded-lg border border-slate-200 bg-white p-4 text-sm shadow-2xl;
}

.operation-panel-header {
  @apply flex items-start gap-3;
}

.operation-panel-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-blue-50 text-xl text-blue-600;
}

.operation-panel-title {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.operation-panel-title strong {
  @apply truncate text-base font-semibold text-slate-900;
}

.operation-panel-title span {
  @apply truncate text-xs text-slate-500;
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
  @apply h-6 rounded border border-transparent px-2 text-slate-600 hover:border-slate-200 hover:bg-blue-50 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:border-transparent disabled:hover:bg-transparent;
}

.preview-tool-row button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.preview-tool-row span {
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
  @apply flex h-full min-h-0 w-full items-center justify-center overflow-auto p-3;
}

.image-stage.fit {
  @apply overflow-hidden;
}

.image-stage img {
  @apply rounded object-contain shadow-sm;
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
