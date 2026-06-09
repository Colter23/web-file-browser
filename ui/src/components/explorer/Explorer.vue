<script setup lang="ts">
import SubCard from "../SubCard.vue";
import {onMounted, reactive, ref, watch} from "vue";
import {FileInfo, FolderData, FolderInfo} from "../../class.ts";
import ExplorerItem from "./ExplorerItem.vue";
import {useFileStore} from "../../store";
import {getFolderData} from "../../network/file-api.ts";
import {log} from "../../utils/common.ts";

type ExplorerEntry = {
  type: "folder" | "file";
  name: string;
  path: string;
  file?: FileInfo;
}

const emit = defineEmits<{
  (e: "rename", entry: ExplorerEntry): void;
  (e: "delete", entry: ExplorerEntry): void;
  (e: "download", entry: ExplorerEntry): void;
}>()

const fileStore = useFileStore();
const folderData = ref<FolderData>({ path: "", folder: [], file: [] });
const selectedEntry = ref<ExplorerEntry | null>(null);
const contextMenu = reactive({visible: false, x: 0, y: 0});

const normalizeFolderData = (data: FolderData): FolderData => ({
  path: data.path,
  folder: data.folder ?? [],
  file: data.file ?? []
})

const loadFolder = async (path: string = fileStore.currentPath || "/") => {
  const data = normalizeFolderData(await getFolderData(path));
  fileStore.saveAndConvertFolderData(data);
  folderData.value = data;
  selectedEntry.value = null;
  fileStore.currentPath = data.path;
  fileStore.showEditor = false;
}

watch(() => fileStore.currentPath, async (path: string) => {
  if (!path || fileStore.showEditor) return;
  const cached = fileStore.folderData.get(path);
  if (cached) {
    folderData.value = normalizeFolderData(cached);
    selectedEntry.value = null;
  } else {
    await loadFolder(path);
  }
});

onMounted(() => {
  window.addEventListener("click", () => {
    contextMenu.visible = false;
  })
})

const selectFolder = (folder: FolderInfo) => {
  selectedEntry.value = {type: "folder", name: folder.name, path: folder.path};
}

const selectFile = (file: FileInfo) => {
  selectedEntry.value = {type: "file", name: file.name, path: file.path, file};
}

const folderDoubleClickHandler = async (path: string) => {
  await loadFolder(path);
}

const fileDoubleClickHandler = (file: FileInfo) => {
  log("Open File", file.name, "#3b82f6")

  if (fileStore.extensions.includes(file.extension)) {
    fileStore.showEditor = true;
    fileStore.currentFile = file;
  }
}

const openContextMenu = (event: MouseEvent, entry: ExplorerEntry) => {
  selectedEntry.value = entry;
  contextMenu.x = event.clientX;
  contextMenu.y = event.clientY;
  contextMenu.visible = true;
}

const openSelected = async () => {
  if (selectedEntry.value?.type === "folder") {
    await loadFolder(selectedEntry.value.path);
  } else if (selectedEntry.value?.file) {
    fileDoubleClickHandler(selectedEntry.value.file);
  }
  contextMenu.visible = false;
}

const getSelectedEntry = () => selectedEntry.value;

defineExpose({
  refresh: loadFolder,
  getSelectedEntry
})
</script>

<template>
  <sub-card class="explorer-wrapper">
    <div class="explorer-header">
      <p class="w-[50%]">名称</p>
      <p class="w-[25%]">修改时间</p>
      <p>大小</p>
    </div>
    <div class="explorer-files">
      <explorer-item v-for="folder in folderData.folder"
                     :key="folder.path"
                     :modified="folder.modified"
                     :name="folder.name"
                     :selected="selectedEntry?.path === folder.path"
                     @click="selectFolder(folder)"
                     @contextmenu.prevent="openContextMenu($event, {type: 'folder', name: folder.name, path: folder.path})"
                     @dblclick="folderDoubleClickHandler(folder.path)" />
      <explorer-item v-for="file in folderData.file"
                     :key="file.path"
                     icon="icon-file"
                     :modified="file.modified"
                     :name="file.name"
                     :selected="selectedEntry?.path === file.path"
                     :size="file.size"
                     @click="selectFile(file)"
                     @contextmenu.prevent="openContextMenu($event, {type: 'file', name: file.name, path: file.path, file})"
                     @dblclick="fileDoubleClickHandler(file)"/>
    </div>

    <div v-if="contextMenu.visible" class="context-menu" :style="{left: `${contextMenu.x}px`, top: `${contextMenu.y}px`}">
      <button @click="openSelected">打开</button>
      <button :disabled="selectedEntry?.type !== 'file'" @click="selectedEntry && emit('download', selectedEntry)">下载</button>
      <button @click="selectedEntry && emit('rename', selectedEntry)">重命名</button>
      <button class="danger" @click="selectedEntry && emit('delete', selectedEntry)">删除</button>
    </div>
  </sub-card>
</template>
<style scoped lang="postcss">
.explorer-wrapper {
  @apply relative w-full h-full overflow-auto
}

.explorer-header {
  @apply border-b pb-1 text-sm flex px-4 py-2 sticky top-0 bg-white z-10
}

.explorer-files {
  @apply px-2 py-1 flex flex-col gap-1
}

.context-menu {
  @apply fixed z-50 w-32 rounded-md border border-slate-200 bg-white py-1 text-sm shadow-lg
}

.context-menu button {
  @apply block h-8 w-full px-3 text-left hover:bg-blue-50 disabled:text-slate-300 disabled:hover:bg-white
}

.context-menu .danger {
  @apply text-red-600 hover:bg-red-50
}
</style>
