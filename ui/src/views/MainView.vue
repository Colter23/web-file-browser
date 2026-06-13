<script setup lang="ts">
import {computed, defineAsyncComponent, onMounted, ref, watch} from "vue";
import {useRouter} from "vue-router";
import FileTree from "../components/FileTree.vue";
import {ArchiveFormat, FileTreeData, TaskKind, TaskState, TaskStatus} from "../class";
import {useFileStore} from "../store";
import {
  cancelTask,
  createArchiveTask,
  createEntry,
  createExtractTask,
  deleteEntry,
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
}

type ExplorerExpose = {
  refresh: (path?: string) => Promise<void>;
  getSelectedEntry: () => ExplorerEntry | null;
  getSelectedEntries: () => ExplorerEntry[];
}

const router = useRouter();
const fileStore = useFileStore();
const treeData = ref<FileTreeData[]>([]);
const explorerRef = ref<ExplorerExpose | null>(null);
const uploadInput = ref<HTMLInputElement | null>(null);
const taskPanelVisible = ref(false);
const tasksLoading = ref(false);
const tasks = ref<TaskStatus[]>([]);
const taskMessage = ref("");
const searchText = ref("");
const previewPanelVisible = ref(false);
const previewEntry = ref<ExplorerEntry | null>(null);
const previewLoading = ref(false);
const previewText = ref("");
const previewError = ref("");

const activeTab = computed(() => fileStore.tabs.find(tab => tab.id === fileStore.activeTabId) ?? fileStore.tabs[0]);
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

const closePanels = () => {
  previewPanelVisible.value = false;
  previewEntry.value = null;
  previewText.value = "";
  previewError.value = "";
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
})

watch(() => fileStore.showEditor, (showEditor) => {
  if (showEditor) closePanels();
});

const currentFolder = () => fileStore.currentPath || "/";

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

const formatBytes = (bytes: number) => {
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

const loadTasks = async () => {
  tasksLoading.value = true;
  try {
    tasks.value = await listTasks();
  } catch (error) {
    window.alert(error instanceof Error ? error.message : "加载任务失败");
  } finally {
    tasksLoading.value = false;
  }
}

const toggleTaskPanel = async () => {
  taskPanelVisible.value = !taskPanelVisible.value;
  if (taskPanelVisible.value) {
    await loadTasks();
  }
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

const newEntry = async (type: "file" | "folder") => {
  const name = window.prompt(type === "file" ? "文件名" : "文件夹名");
  if (!name) return;
  await runOperation(async () => {
    await createEntry(currentFolder(), type, name);
  })
}

const selectedEntry = () => explorerRef.value?.getSelectedEntry() ?? null;

const selectedEntries = (fallback?: ExplorerEntry | null) => {
  const selected = explorerRef.value?.getSelectedEntries() ?? [];
  if (selected.length) return selected;
  return fallback ? [fallback] : [];
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

const taskStarted = async (id: string) => {
  taskMessage.value = `后台任务已创建：${id}`;
  taskPanelVisible.value = true;
  await loadTasks();
}

const renameSelected = async (entry = singleSelectedEntry()) => {
  if (!entry) {
    window.alert("请选择一个文件或文件夹");
    return;
  }
  const name = window.prompt("新名称", entry.name);
  if (!name || name === entry.name) return;
  await runOperation(async () => {
    await moveEntry(entry.path, joinPath(parentPath(entry.path), name));
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
  await runOperation(async () => {
    for (const item of entries) {
      await deleteEntry(item.path);
    }
  })
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

const archiveSelected = async (entry = selectedEntry()) => {
  const entries = selectedEntries(entry);
  if (!entries.length) {
    window.alert("请选择文件或文件夹");
    return;
  }
  const rawFormat = window.prompt("压缩格式：zip 或 tarGz", "zip");
  if (!rawFormat) return;
  const format = rawFormat.trim() === "tarGz" ? "tarGz" : rawFormat.trim() === "zip" ? "zip" : null;
  if (!format) {
    window.alert("压缩格式只支持 zip 或 tarGz");
    return;
  }
  const defaultName = entries.length === 1 ? `${entries[0].name}${archiveExtension(format)}` : `选中项目${archiveExtension(format)}`;
  const outputName = window.prompt("压缩包名称", defaultName);
  if (!outputName) return;
  try {
    const task = await createArchiveTask(entries.map(item => item.path), currentFolder(), format, outputName);
    await taskStarted(task.id);
  } catch (error) {
    window.alert(error instanceof Error ? error.message : "创建压缩任务失败");
  }
}

const extractSelected = async (entry = singleSelectedEntry()) => {
  if (!isArchiveFile(entry)) {
    window.alert("请选择一个 zip、tar.gz 或 tgz 压缩包");
    return;
  }
  const folderName = window.prompt("解压到文件夹", archiveStem(entry.name));
  if (!folderName) return;
  try {
    const task = await createExtractTask(entry.path, currentFolder(), folderName);
    await taskStarted(task.id);
  } catch (error) {
    window.alert(error instanceof Error ? error.message : "创建解压任务失败");
  }
}

const uploadChanged = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files?.length) return;
  await runOperation(async () => {
    await uploadFiles(currentFolder(), input.files as FileList);
  })
  input.value = "";
}

const goBack = async () => {
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
  fileStore.showEditor = false;
  fileStore.currentFile = null;
  previewEntry.value = entry;
  previewPanelVisible.value = true;
  void loadPreview(entry);
}

const closePreview = () => {
  closePanels();
}

const loadPreview = async (entry: ExplorerEntry) => {
  previewText.value = "";
  previewError.value = "";
  if (previewKind.value !== "text") return;
  previewLoading.value = true;
  try {
    const file = await getFile(entry.path);
    previewText.value = file.content;
  } catch (error) {
    previewError.value = error instanceof Error ? error.message : "预览失败";
  } finally {
    previewLoading.value = false;
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
          <input v-model="searchText" type="search" placeholder="搜索当前文件夹" @keydown.escape="clearSearch">
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
          <button class="icon-tool" title="新建文件" @click="newEntry('file')">
            <icon icon="icon-file-add-fill" />
          </button>
          <button class="icon-tool" title="新建文件夹" @click="newEntry('folder')">
            <icon icon="icon-folder-add-fill" />
          </button>
        </div>
        <file-tree :data="treeData" :load-data="handleLoad"></file-tree>
      </aside>

      <section class="content-pane">
        <div class="path-row">
          <button class="nav-button" title="返回上级" @click="goBack">
            <icon icon="icon-back_android" size="large" />
          </button>
          <button class="nav-button" title="刷新" @click="refreshCurrent">
            <icon icon="icon-refresh" size="large" />
          </button>
          <breadcrumb></breadcrumb>
          <button class="view-button" @click="fileStore.setViewMode(fileStore.viewMode === 'details' ? 'icons' : 'details')">
            <icon :icon="fileStore.viewMode === 'details' ? 'icon-viewgrid' : 'icon-view-list'" />
            <span>{{ fileStore.viewMode === 'details' ? '图标模式' : '详细信息' }}</span>
          </button>
        </div>

        <div class="command-bar">
          <button class="command-button" @click="newEntry('file')">
            <icon icon="icon-file-add-fill" />
            <span>新建文件</span>
          </button>
          <button class="command-button" @click="newEntry('folder')">
            <icon icon="icon-folder-add-fill" />
            <span>新建文件夹</span>
          </button>
          <button class="command-button" @click="downloadSelected()">
            <icon icon="icon-download" />
            <span>下载</span>
          </button>
          <button class="command-button" @click="previewSelected()">
            <icon icon="icon-file-image-fill" />
            <span>预览</span>
          </button>
          <button class="command-button" @click="archiveSelected()">
            <icon icon="icon-file-zip-fill" />
            <span>压缩</span>
          </button>
          <button class="command-button" @click="extractSelected()">
            <icon icon="icon-file-zip" />
            <span>解压</span>
          </button>
          <button class="command-button" @click="renameSelected()">
            <icon icon="icon-rename" />
            <span>重命名</span>
          </button>
          <button class="command-button danger" @click="deleteSelected()">
            <icon icon="icon-delete-fill" />
            <span>删除</span>
          </button>
          <button :class="['command-button', {active: taskPanelVisible}]" @click="toggleTaskPanel">
            <icon icon="icon-file-common-filling" />
            <span>任务</span>
          </button>
          <input ref="uploadInput" class="hidden" type="file" multiple @change="uploadChanged">
        </div>

        <div v-if="taskPanelVisible" class="task-panel">
          <div class="task-panel-header">
            <div class="min-w-0">
              <p class="task-panel-title">后台任务</p>
              <p class="task-panel-message">{{ taskMessage || "手动刷新查看最新任务状态" }}</p>
            </div>
            <div class="task-panel-actions">
              <button class="task-icon-button" :disabled="tasksLoading" title="刷新任务" @click="loadTasks">
                <icon icon="icon-refresh" size="normal"/>
              </button>
              <button class="task-icon-button" title="关闭任务面板" @click="taskPanelVisible = false">
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
              <button class="task-cancel" :disabled="!canCancelTask(task)" @click="cancelTaskById(task)">取消</button>
            </div>
          </div>
        </div>

        <div class="browser-area" :class="{previewing: previewPanelVisible}">
          <div class="browser-main">
            <editor-panel v-show="fileStore.showEditor"></editor-panel>
            <explorer
                ref="explorerRef"
                v-show="!fileStore.showEditor"
                :filter-text="searchText"
                @rename="renameSelected"
                @delete="deleteSelected"
                @download="downloadSelected"
                @archive="archiveSelected"
                @extract="extractSelected"
                @preview="previewSelected"
                @open-new-tab="openEntryInNewTab">
            </explorer>
          </div>
          <aside v-if="previewPanelVisible" class="preview-pane">
            <div class="preview-header">
              <span>{{ previewEntry?.name }}</span>
              <button title="关闭预览" @click="closePreview">
                <icon icon="icon-close" />
              </button>
            </div>
            <div class="preview-body" :class="previewKind">
              <div v-if="previewLoading" class="preview-placeholder">正在加载预览...</div>
              <div v-else-if="previewError" class="preview-placeholder error">{{ previewError }}</div>
              <img v-else-if="previewEntry && previewKind === 'image'" :src="downloadUrl(previewEntry.path)" :alt="previewEntry.name">
              <pre v-else-if="previewKind === 'text'">{{ previewText }}</pre>
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

.view-button {
  @apply h-10 shrink-0 gap-2 px-3 text-sm;
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

.command-button.danger {
  @apply text-red-600 hover:bg-red-50;
}

.browser-area {
  @apply grid min-h-0 grow grid-cols-[minmax(0,1fr)] overflow-hidden;
}

.browser-area.previewing {
  @apply grid-cols-[minmax(0,1fr)_20rem];
}

.browser-main {
  @apply min-h-0 overflow-hidden;
}

.preview-pane {
  @apply flex min-h-0 flex-col border-l border-slate-200 bg-white;
}

.preview-header {
  @apply flex h-10 shrink-0 items-center justify-between gap-2 border-b border-slate-200 px-3 text-sm font-medium;
}

.preview-header span {
  @apply min-w-0 truncate;
}

.preview-header button {
  @apply h-7 w-7 shrink-0;
}

.preview-body {
  @apply min-h-0 grow overflow-auto p-3 text-sm text-slate-700;
}

.preview-body.image,
.preview-body.audio,
.preview-body.video {
  @apply flex items-center justify-center bg-slate-50;
}

.preview-body img {
  @apply max-h-full max-w-full rounded object-contain;
}

.preview-body pre {
  @apply whitespace-pre-wrap break-words font-mono text-xs leading-5;
}

.preview-body audio,
.preview-body video {
  @apply max-h-full max-w-full;
}

.preview-placeholder {
  @apply flex h-full min-h-48 flex-col items-center justify-center gap-3 text-center text-slate-500;
}

.preview-placeholder.error {
  @apply text-red-600;
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
  @apply grid min-h-16 grid-cols-[minmax(9rem,1.1fr)_minmax(10rem,1.2fr)_minmax(14rem,1.5fr)_4rem] items-center gap-3 rounded border border-slate-100 bg-slate-50 px-3 py-2 text-sm;
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

.task-cancel {
  @apply h-8 rounded border border-slate-200 bg-white px-2 text-sm text-slate-700 hover:bg-red-50 hover:text-red-600 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:bg-white;
}
</style>
