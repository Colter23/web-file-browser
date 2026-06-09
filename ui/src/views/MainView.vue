<script setup lang="ts">
import {onMounted, ref} from "vue";
import {useRouter} from "vue-router";
import FileTree from "../components/FileTree.vue";
import {FileTreeData} from "../class";
import {useFileStore} from "../store";
import {
  createEntry,
  deleteEntry,
  downloadFile,
  getFolderData,
  logout,
  moveEntry,
  uploadFiles
} from "../network/api";
import Split from "../components/Split.vue";
import SubCard from "../components/SubCard.vue";
import Icon from "../components/Icon.vue";
import Card from "../components/Card.vue";
import Explorer from "../components/explorer/Explorer.vue";
import EditorPanel from "../components/editor/EditorPanel.vue";
import Breadcrumb from "../components/Breadcrumb.vue";

type ExplorerEntry = {
  type: "folder" | "file";
  name: string;
  path: string;
}

type ExplorerExpose = {
  refresh: (path?: string) => Promise<void>;
  getSelectedEntry: () => ExplorerEntry | null;
}

const router = useRouter();
const fileStore = useFileStore();
const treeData = ref<FileTreeData[]>([]);
const explorerRef = ref<ExplorerExpose | null>(null);
const uploadInput = ref<HTMLInputElement | null>(null);

const loadRoot = async () => {
  const data = await getFolderData("/");
  treeData.value = fileStore.saveAndConvertFolderData(data);
  fileStore.currentPath = data.path;
}

const handleLoad = (node: FileTreeData) => {
  return new Promise<void>(async (resolve) => {
    const data = await getFolderData(node.path);
    node.children = fileStore.saveAndConvertFolderData(data);
    fileStore.currentPath = data.path;
    fileStore.showEditor = false;
    resolve();
  });
}

onMounted(async () => {
  await loadRoot();
})

const currentFolder = () => fileStore.currentPath || "/";

const refreshCurrent = async () => {
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

const parentPath = (path: string) => {
  const parts = path.split("/").filter(Boolean);
  parts.pop();
  return parts.length ? `/${parts.join("/")}` : "/";
}

const joinPath = (base: string, name: string) => {
  return base === "/" ? `/${name}` : `${base.replace(/\/$/, "")}/${name}`;
}

const renameSelected = async (entry = selectedEntry()) => {
  if (!entry) {
    window.alert("请选择文件或文件夹");
    return;
  }
  const name = window.prompt("新名称", entry.name);
  if (!name || name === entry.name) return;
  await runOperation(async () => {
    await moveEntry(entry.path, joinPath(parentPath(entry.path), name));
  })
}

const deleteSelected = async (entry = selectedEntry()) => {
  if (!entry) {
    window.alert("请选择文件或文件夹");
    return;
  }
  if (!window.confirm(`删除 ${entry.name}？`)) return;
  await runOperation(async () => {
    await deleteEntry(entry.path);
  })
}

const downloadSelected = async (entry = selectedEntry()) => {
  if (!entry || entry.type !== "file") {
    window.alert("请选择文件");
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
  const path = parentPath(currentFolder());
  fileStore.currentPath = path;
  await explorerRef.value?.refresh(path);
}

const signOut = async () => {
  await logout();
  await router.replace("/login");
}
</script>

<template>
  <div class="h-screen flex flex-col p-3 gap-y-2 bg-slate-100">
    <div class="flex gap-x-3 h-12">
      <card class="grow flex items-center px-4">
        <span class="text-lg font-semibold text-slate-900">Web File Browser</span>
      </card>
      <card class="h-full w-12 icon" @click="router.push('/setting')">
        <icon icon="icon-setting" size="2"></icon>
      </card>
      <card class="h-full px-3 icon text-sm cursor-pointer" @click="signOut">
        退出
      </card>
    </div>

    <card class="flex flex-col shrink grow overflow-auto p-2 gap-y-2">
      <div class="toolbar">
        <sub-card class="tool-button" @click="newEntry('file')">
          <icon icon="icon-file" size="large"/>
          <span>新建文件</span>
        </sub-card>
        <sub-card class="tool-button" @click="newEntry('folder')">
          <icon icon="icon-folder-fill" size="large"/>
          <span>新建文件夹</span>
        </sub-card>
        <sub-card class="tool-button" @click="uploadInput?.click()">
          <icon icon="icon-add" size="large"/>
          <span>上传</span>
        </sub-card>
        <sub-card class="tool-button" @click="downloadSelected()">
          <icon icon="icon-file" size="large"/>
          <span>下载</span>
        </sub-card>
        <sub-card class="tool-button" @click="renameSelected()">
          <icon icon="icon-file" size="large"/>
          <span>重命名</span>
        </sub-card>
        <sub-card class="tool-button danger" @click="deleteSelected()">
          <icon icon="icon-close" size="large"/>
          <span>删除</span>
        </sub-card>
        <sub-card class="icon h-10 w-10" @click="goBack">
          <icon class="rotate-90" icon="icon-unfold" size="large"/>
        </sub-card>
        <sub-card class="icon h-10 w-10" @click="refreshCurrent">
          <icon icon="icon-refresh" size="large"/>
        </sub-card>
        <breadcrumb></breadcrumb>
        <input ref="uploadInput" class="hidden" type="file" multiple @change="uploadChanged">
      </div>

      <split class="shrink grow overflow-auto">
        <template #left>
          <file-tree :data="treeData" :load-data="handleLoad"></file-tree>
        </template>
        <template #right>
          <sub-card class="w-full h-full">
            <editor-panel v-show="fileStore.showEditor"></editor-panel>
            <explorer
                ref="explorerRef"
                v-show="!fileStore.showEditor"
                @rename="renameSelected"
                @delete="deleteSelected"
                @download="downloadSelected">
            </explorer>
          </sub-card>
        </template>
      </split>
    </card>
  </div>
</template>

<style scoped lang="postcss">
.icon {
  @apply shrink-0 inline-flex items-center justify-center cursor-pointer
}

.toolbar {
  @apply flex gap-x-2 items-center
}

.tool-button {
  @apply h-10 inline-flex shrink-0 items-center gap-1 px-3 text-sm cursor-pointer hover:bg-blue-50
}

.tool-button.danger {
  @apply text-red-600 hover:bg-red-50
}
</style>
