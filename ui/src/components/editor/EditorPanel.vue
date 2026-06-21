<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ComponentPublicInstance} from "vue";
import Icon from "../Icon.vue";
import CodeEditor from "./CodeEditor.vue";
import editorConfig from "../../assets/editor-config.json";
import {useFileStore} from "../../store";
import type {FileInfo} from "../../class.ts";
import {getFile, saveFile} from "../../network/file-api.ts";
import {checkFileLanguageMode} from "../../utils/common.ts";

type MenuName = "language" | "theme" | "settings" | "";

type CodeEditorExpose = ComponentPublicInstance & {
  focus?: () => void;
}

const fileStore = useFileStore();
const fileInfo = ref<FileInfo | null>(null);
const editorRef = ref<CodeEditorExpose | null>(null);
const activeMenu = ref<MenuName>("");
const currentTheme = ref("dracula");
const currentMode = ref("text");
const content = ref("");
const contentEtag = ref("");
const isChange = ref(false);
const loading = ref(false);
const saving = ref(false);
const statusText = ref("");
const errorText = ref("");
const fontSize = ref(16);
const tabSize = ref(2);
const wrap = ref(true);
let loadVersion = 0;

const themeClass = computed(() => `ace-${currentTheme.value.replace(/_/g, "-")}`);
const canSave = computed(() => Boolean(fileInfo.value && isChange.value && contentEtag.value && !loading.value && !saving.value));

const fileTitle = computed(() => fileInfo.value?.name ?? "未打开文件");

const filePathText = computed(() => fileInfo.value?.path ?? "");

const dirtyText = computed(() => {
  if (saving.value) return "保存中";
  if (loading.value) return "加载中";
  if (isChange.value) return "未保存";
  return statusText.value || "已同步";
});

const selectedModeName = computed(() => editorConfig.mode.find(mode => mode.key === currentMode.value)?.name ?? currentMode.value);

const selectedThemeName = computed(() => {
  const themes = [...editorConfig.theme.light, ...editorConfig.theme.dark];
  return themes.find(theme => theme.key === currentTheme.value)?.name ?? currentTheme.value;
});

const formatSize = (size?: number) => {
  if (!size) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = size;
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

const closeMenus = () => {
  activeMenu.value = "";
}

const toggleMenu = (menu: MenuName) => {
  activeMenu.value = activeMenu.value === menu ? "" : menu;
}

const changeMode = (mode: string) => {
  currentMode.value = mode;
  closeMenus();
  nextTick(() => editorRef.value?.focus?.());
}

const changeTheme = (theme: string) => {
  currentTheme.value = theme;
  closeMenus();
  nextTick(() => editorRef.value?.focus?.());
}

const loadCurrentFile = async () => {
  if (!fileStore.showEditor || fileStore.currentFile == null) return;
  const version = ++loadVersion;
  const target = fileStore.currentFile;
  fileInfo.value = target;
  currentMode.value = checkFileLanguageMode(target.extension);
  statusText.value = "";
  errorText.value = "";
  loading.value = true;
  try {
    const file = await getFile(target.path);
    if (version !== loadVersion) return;
    content.value = file.content;
    contentEtag.value = file.etag;
    isChange.value = false;
    statusText.value = "已打开";
    await nextTick();
    editorRef.value?.focus?.();
  } catch (error) {
    if (version !== loadVersion) return;
    errorText.value = error instanceof Error ? error.message : "打开文件失败";
    content.value = "";
    contentEtag.value = "";
    isChange.value = false;
  } finally {
    if (version === loadVersion) loading.value = false;
  }
}

watch(() => [fileStore.showEditor, fileStore.currentFile?.path], loadCurrentFile);

const onContentChange = (value: string) => {
  if (loading.value) return;
  content.value = value;
  isChange.value = true;
  statusText.value = "";
  errorText.value = "";
}

const save = async () => {
  if (!fileInfo.value || saving.value || loading.value) return;
  saving.value = true;
  errorText.value = "";
  try {
    if (!contentEtag.value) {
      throw new Error("文件版本信息缺失，请重新打开文件后再保存");
    }
    const saved = await saveFile(fileInfo.value.path, content.value, contentEtag.value);
    contentEtag.value = saved.etag;
    isChange.value = false;
    statusText.value = "已保存";
  } catch (error) {
    errorText.value = error instanceof Error ? error.message : "保存失败";
  } finally {
    saving.value = false;
  }
}

const reload = async () => {
  if (isChange.value && !window.confirm("重新载入会放弃未保存的修改，继续？")) return;
  await loadCurrentFile();
}

const close = () => {
  if (isChange.value && !window.confirm("放弃未保存的修改？")) return;
  closeMenus();
  fileStore.showEditor = false;
  fileStore.currentFile = null;
  fileInfo.value = null;
  isChange.value = false;
  content.value = "";
  contentEtag.value = "";
  statusText.value = "";
  errorText.value = "";
}

const handleKeyDown = (event: KeyboardEvent) => {
  if (!fileStore.showEditor) return;
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
    event.preventDefault();
    void save();
    return;
  }
  if (event.key === "Escape") {
    if (activeMenu.value) {
      closeMenus();
      return;
    }
    close();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
  void loadCurrentFile();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeyDown);
});
</script>

<template>
  <div class="editor-shell" :class="themeClass" @click="closeMenus">
    <header class="editor-titlebar" @click.stop>
      <div class="editor-file-head">
        <div class="file-mark">
          <icon icon="icon-edit-filling" color="#ffffff" />
        </div>
        <div class="file-title-block">
          <div class="file-title-line">
            <span class="file-title">{{ fileTitle }}</span>
            <span v-if="isChange" class="dirty-dot"></span>
          </div>
          <span class="file-path" :title="filePathText">{{ filePathText }}</span>
        </div>
      </div>

      <div class="editor-actions">
        <button class="menu-button" :class="{active: activeMenu === 'language'}" @click.stop="toggleMenu('language')">
          {{ selectedModeName }}
        </button>
        <button class="menu-button" :class="{active: activeMenu === 'theme'}" @click.stop="toggleMenu('theme')">
          {{ selectedThemeName }}
        </button>
        <button class="icon-button" :class="{active: activeMenu === 'settings'}" title="编辑设置" @click.stop="toggleMenu('settings')">
          <icon icon="icon-setting" color="#ffffff" />
        </button>
        <button class="icon-button" :disabled="loading" title="重新载入" @click.stop="reload">
          <icon icon="icon-refresh" color="#ffffff" />
        </button>
        <button class="save-button" :disabled="!canSave" title="保存" @click.stop="save">
          <icon icon="icon-save-fill" color="#ffffff" />
          <span>{{ saving ? "保存中" : "保存" }}</span>
        </button>
        <button class="icon-button close-button" title="关闭" @click.stop="close">
          <icon icon="icon-close" color="#ffffff" />
        </button>
      </div>
    </header>

    <div class="menu-layer" @click.stop>
      <div v-if="activeMenu === 'language'" class="editor-menu language-menu">
        <button
            v-for="mode in editorConfig.mode"
            :key="mode.key"
            :class="{active: currentMode === mode.key}"
            @click="changeMode(mode.key)">
          <icon icon="icon-file" color="#ffffff" />
          <span>{{ mode.name }}</span>
        </button>
      </div>

      <div v-if="activeMenu === 'theme'" class="editor-menu theme-menu">
        <p>浅色主题</p>
        <button
            v-for="theme in editorConfig.theme.light"
            :key="theme.key"
            :class="{active: currentTheme === theme.key}"
            @click="changeTheme(theme.key)">
          <span>{{ theme.name }}</span>
        </button>
        <p>深色主题</p>
        <button
            v-for="theme in editorConfig.theme.dark"
            :key="theme.key"
            :class="{active: currentTheme === theme.key}"
            @click="changeTheme(theme.key)">
          <span>{{ theme.name }}</span>
        </button>
      </div>

      <div v-if="activeMenu === 'settings'" class="editor-menu settings-menu">
        <label>
          <span>字号</span>
          <input v-model.number="fontSize" type="number" min="12" max="28" step="1">
        </label>
        <label>
          <span>Tab 宽度</span>
          <input v-model.number="tabSize" type="number" min="2" max="8" step="1">
        </label>
        <label class="check-row">
          <input v-model="wrap" type="checkbox">
          <span>自动换行</span>
        </label>
      </div>
    </div>

    <main class="editor-main">
      <code-editor
          ref="editorRef"
          :mode="currentMode"
          :theme="currentTheme"
          :content="content"
          :font-size="fontSize"
          :wrap="wrap"
          :tab-size="tabSize"
          @change="onContentChange"
          @save="save">
      </code-editor>
      <div v-if="loading" class="editor-overlay">正在打开文件...</div>
      <div v-else-if="errorText" class="editor-overlay error">
        <span>{{ errorText }}</span>
        <button @click="reload">重试</button>
      </div>
    </main>

    <footer class="editor-statusbar">
      <div class="status-left">
        <span :class="['status-pill', {dirty: isChange, saving}]">{{ dirtyText }}</span>
        <span>{{ formatDate(fileInfo?.modified) }}</span>
        <span>{{ filePathText }}</span>
      </div>
      <div class="status-right">
        <span>{{ selectedModeName }}</span>
        <span>{{ formatSize(fileInfo?.size) }}</span>
        <span>UTF-8</span>
        <span>{{ wrap ? "自动换行" : "不换行" }}</span>
      </div>
    </footer>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-shell {
  @apply relative flex h-full min-h-0 flex-col overflow-hidden bg-[#171b24] text-white;
}

.editor-titlebar,
.editor-statusbar {
  @apply relative z-20 flex shrink-0 items-center justify-between border-white/10 bg-black/20 backdrop-blur;
}

.editor-titlebar {
  @apply h-11 gap-3 border-b px-3;
}

.editor-file-head {
  @apply flex min-w-0 items-center gap-3;
}

.file-mark {
  @apply inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-white/15;
}

.file-title-block {
  @apply flex min-w-0 flex-col;
}

.file-title-line {
  @apply flex min-w-0 items-center gap-2;
}

.file-title {
  @apply min-w-0 truncate text-sm font-medium;
}

.dirty-dot {
  @apply h-2 w-2 shrink-0 rounded-full bg-amber-300;
}

.file-path {
  @apply max-w-[42rem] truncate text-xs text-white/50;
}

.editor-actions {
  @apply flex shrink-0 items-center gap-1;
}

.menu-button,
.icon-button,
.save-button {
  @apply inline-flex h-8 items-center justify-center rounded-md border border-white/10 bg-white/10 text-xs text-white hover:bg-white/20 disabled:cursor-not-allowed disabled:opacity-45 disabled:hover:bg-white/10;
}

.menu-button {
  @apply max-w-32 gap-1 px-2;
}

.icon-button {
  @apply w-8;
}

.menu-button.active,
.icon-button.active {
  @apply border-blue-300/60 bg-blue-500/35;
}

.save-button {
  @apply gap-1.5 bg-blue-600 px-3 font-medium hover:bg-blue-500;
}

.close-button {
  @apply hover:bg-red-500/50;
}

.menu-layer {
  @apply absolute right-3 top-11 z-30;
}

.editor-menu {
  @apply mt-2 flex max-h-80 min-w-44 flex-col gap-1 overflow-auto rounded-md border border-white/15 bg-[#252a35] p-1 text-sm shadow-2xl;
}

.language-menu {
  @apply w-52;
}

.theme-menu {
  @apply w-56;
}

.settings-menu {
  @apply w-56 gap-3 p-3;
}

.editor-menu p {
  @apply px-2 pt-1 text-xs font-medium text-white/45;
}

.editor-menu button {
  @apply flex h-8 items-center gap-2 rounded px-2 text-left text-white/80 hover:bg-white/10;
}

.editor-menu button.active {
  @apply bg-blue-500/30 text-white;
}

.editor-menu label {
  @apply flex items-center justify-between gap-3 text-sm text-white/75;
}

.editor-menu input[type="number"] {
  @apply h-8 w-20 rounded border border-white/10 bg-black/25 px-2 text-right text-white outline-none focus:border-blue-300;
}

.check-row {
  @apply justify-start;
}

.check-row input {
  @apply h-4 w-4 accent-blue-500;
}

.editor-main {
  @apply relative min-h-0 grow;
}

.editor-overlay {
  @apply absolute inset-0 z-10 flex items-center justify-center bg-[#171b24]/85 text-sm text-white/70 backdrop-blur-sm;
}

.editor-overlay.error {
  @apply flex-col gap-3 text-red-100;
}

.editor-overlay button {
  @apply rounded-md border border-white/15 bg-white/10 px-3 py-1.5 text-white hover:bg-white/20;
}

.editor-statusbar {
  @apply h-7 gap-3 border-t px-3 text-xs text-white/60;
}

.status-left,
.status-right {
  @apply flex min-w-0 items-center gap-3;
}

.status-left span,
.status-right span {
  @apply truncate;
}

.status-pill {
  @apply shrink-0 rounded bg-white/10 px-2 py-0.5 text-white/75;
}

.status-pill.dirty {
  @apply bg-amber-400/20 text-amber-100;
}

.status-pill.saving {
  @apply bg-blue-400/20 text-blue-100;
}
</style>
