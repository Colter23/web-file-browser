<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ComponentPublicInstance} from "vue";
import Icon from "../Icon.vue";
import CodeEditor from "./CodeEditor.vue";
import editorConfig from "../../assets/editor-config.json";
import {useFileStore} from "../../store";
import type {FileInfo} from "../../class.ts";
import {getFile, saveFile} from "../../network/file-api.ts";
import {isApiError} from "../../network";
import {checkFileLanguageMode} from "../../utils/common.ts";

type MenuName = "language" | "theme" | "settings" | "";

type CodeEditorExpose = ComponentPublicInstance & {
  focus?: () => void;
}

const storageKeys = {
  theme: "editor.theme",
  fontSize: "editor.fontSize",
  tabSize: "editor.tabSize",
  wrap: "editor.wrap"
};

const allThemeKeys = [...editorConfig.theme.light, ...editorConfig.theme.dark].map(theme => theme.key);

const readStorageItem = (key: string): string | null => {
  if (typeof localStorage === "undefined") return null;
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

const writeStorageItem = (key: string, value: string) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(key, value);
  } catch {
    // 本地存储不可用时，仍保留当前会话内的编辑设置。
  }
}

const normalizeNumberPreference = (value: unknown, fallback: number, min: number, max: number) => {
  const numeric = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(numeric)) return fallback;
  return Math.min(max, Math.max(min, Math.round(numeric)));
}

const readThemePreference = () => {
  const theme = readStorageItem(storageKeys.theme);
  return theme && allThemeKeys.includes(theme) ? theme : "github";
}

const readNumberPreference = (key: string, fallback: number, min: number, max: number) => {
  return normalizeNumberPreference(readStorageItem(key), fallback, min, max);
}

const readBooleanPreference = (key: string, fallback: boolean) => {
  const value = readStorageItem(key);
  if (value === "true") return true;
  if (value === "false") return false;
  return fallback;
}

const fileStore = useFileStore();
const fileInfo = ref<FileInfo | null>(null);
const editorRef = ref<CodeEditorExpose | null>(null);
const activeMenu = ref<MenuName>("");
const currentTheme = ref(readThemePreference());
const currentMode = ref("text");
const content = ref("");
const contentEtag = ref("");
const isChange = ref(false);
const loading = ref(false);
const saving = ref(false);
const statusText = ref("");
const errorText = ref("");
const saveConflict = ref(false);
const fontSize = ref(readNumberPreference(storageKeys.fontSize, 16, 12, 28));
const tabSize = ref(readNumberPreference(storageKeys.tabSize, 2, 2, 8));
const wrap = ref(readBooleanPreference(storageKeys.wrap, true));
let loadVersion = 0;

const themeClass = computed(() => `ace-${currentTheme.value.replace(/_/g, "-")}`);
const canSave = computed(() => Boolean(fileInfo.value && isChange.value && contentEtag.value && !loading.value && !saving.value));

const fileTitle = computed(() => fileInfo.value?.name ?? "未打开文件");

const filePathText = computed(() => fileInfo.value?.path ?? "");

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

const editorMetaText = computed(() => {
  const parts = [selectedModeName.value, formatSize(fileInfo.value?.size), wrap.value ? "自动换行" : "不换行"];
  return parts.join(" · ");
});

const dirtyText = computed(() => {
  if (saving.value) return "保存中";
  if (loading.value) return "加载中";
  if (saveConflict.value) return "需重新载入";
  if (isChange.value) return "未保存";
  return statusText.value || "已同步";
});

const editorMessageText = computed(() => errorText.value || (saveConflict.value ? "文件版本已变化，请重新载入后再保存" : ""));

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

watch(currentTheme, theme => {
  if (allThemeKeys.includes(theme)) writeStorageItem(storageKeys.theme, theme);
});

watch(fontSize, value => {
  const normalized = normalizeNumberPreference(value, 16, 12, 28);
  if (value !== normalized) {
    fontSize.value = normalized;
    return;
  }
  writeStorageItem(storageKeys.fontSize, String(normalized));
});

watch(tabSize, value => {
  const normalized = normalizeNumberPreference(value, 2, 2, 8);
  if (value !== normalized) {
    tabSize.value = normalized;
    return;
  }
  writeStorageItem(storageKeys.tabSize, String(normalized));
});

watch(wrap, value => {
  writeStorageItem(storageKeys.wrap, String(Boolean(value)));
});

const loadCurrentFile = async () => {
  if (!fileStore.showEditor || fileStore.currentFile == null) return;
  const version = ++loadVersion;
  const target = fileStore.currentFile;
  fileInfo.value = target;
  currentMode.value = checkFileLanguageMode(target.extension);
  statusText.value = "";
  errorText.value = "";
  saveConflict.value = false;
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
  if (saveConflict.value) saveConflict.value = false;
}

const save = async () => {
  if (!fileInfo.value || saving.value || loading.value) return;
  saving.value = true;
  errorText.value = "";
  saveConflict.value = false;
  try {
    if (!contentEtag.value) {
      throw new Error("文件版本信息缺失，请重新打开文件后再保存");
    }
    const saved = await saveFile(fileInfo.value.path, content.value, contentEtag.value);
    contentEtag.value = saved.etag;
    isChange.value = false;
    statusText.value = "已保存";
  } catch (error) {
    if (isApiError(error) && (error.status === 412 || error.status === 428 || error.code === "PRECONDITION_FAILED" || error.code === "PRECONDITION_REQUIRED")) {
      saveConflict.value = true;
      errorText.value = error.status === 428
          ? "缺少文件版本信息，请重新载入后再保存"
          : "文件已被外部修改，请重新载入最新版本后再保存";
    } else {
      errorText.value = error instanceof Error ? error.message : "保存失败";
    }
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
  saveConflict.value = false;
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
          <icon icon="icon-file-common-filling" color="#475569" />
          <span>语言：{{ selectedModeName }}</span>
        </button>
        <button class="menu-button" :class="{active: activeMenu === 'theme'}" @click.stop="toggleMenu('theme')">
          <icon icon="icon-setting" color="#475569" />
          <span>主题：{{ selectedThemeName }}</span>
        </button>
        <button class="icon-button" :class="{active: activeMenu === 'settings'}" title="编辑设置" @click.stop="toggleMenu('settings')">
          <icon icon="icon-setting" color="#475569" />
        </button>
        <button class="icon-button" :disabled="loading" title="重新载入" @click.stop="reload">
          <icon icon="icon-refresh" color="#475569" />
        </button>
        <button class="save-button" :disabled="!canSave" title="保存 (Ctrl+S)" @click.stop="save">
          <icon icon="icon-save-fill" :color="canSave ? '#ffffff' : '#94a3b8'" />
          <span>{{ saving ? "保存中" : "保存" }}</span>
        </button>
        <button class="icon-button close-button" title="关闭 (Esc)" @click.stop="close">
          <icon icon="icon-close" color="#475569" />
        </button>
      </div>
    </header>

    <div class="editor-infobar" @click.stop>
      <div class="editor-info-left">
        <span :class="['status-pill', {dirty: isChange, saving, conflict: saveConflict}]">{{ dirtyText }}</span>
        <span>{{ editorMetaText }}</span>
      </div>
      <div class="editor-info-right">
        <span>修改时间：{{ formatDate(fileInfo?.modified) }}</span>
        <span>UTF-8</span>
      </div>
    </div>

    <div class="menu-layer" @click.stop>
      <div v-if="activeMenu === 'language'" class="editor-menu language-menu">
        <button
            v-for="mode in editorConfig.mode"
            :key="mode.key"
            :class="{active: currentMode === mode.key}"
            @click="changeMode(mode.key)">
          <icon icon="icon-file" :color="currentMode === mode.key ? '#ffffff' : '#475569'" />
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
      <div class="editor-frame">
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
      </div>
      <div v-if="loading" class="editor-overlay">正在打开文件...</div>
      <div v-else-if="errorText" class="editor-overlay error">
        <span>{{ errorText }}</span>
        <button @click="reload">重试</button>
      </div>
    </main>

    <footer class="editor-statusbar">
      <div class="status-left">
        <span v-if="editorMessageText" :class="['editor-message', {conflict: saveConflict}]">{{ editorMessageText }}</span>
        <span v-else>{{ filePathText }}</span>
      </div>
      <div class="status-right">
        <button v-if="saveConflict" class="status-action" @click="reload">重新载入</button>
        <span>{{ selectedModeName }}</span>
        <span>{{ formatSize(fileInfo?.size) }}</span>
        <span>{{ wrap ? "自动换行" : "不换行" }}</span>
      </div>
    </footer>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-shell {
  @apply relative flex h-full min-h-0 flex-col overflow-hidden bg-[#f7fbff] text-slate-900;
}

.editor-titlebar,
.editor-infobar,
.editor-statusbar {
  @apply relative z-20 flex shrink-0 items-center justify-between border-slate-200 bg-white/90 backdrop-blur;
}

.editor-titlebar {
  @apply h-12 gap-3 border-b px-3;
}

.editor-infobar {
  @apply h-9 gap-3 border-b bg-slate-50/80 px-3 text-xs text-slate-500;
}

.editor-file-head {
  @apply flex min-w-0 items-center gap-3;
}

.file-mark {
  @apply inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-blue-600 shadow-sm;
}

.file-title-block {
  @apply flex min-w-0 flex-col;
}

.file-title-line {
  @apply flex min-w-0 items-center gap-2;
}

.file-title {
  @apply min-w-0 truncate text-sm font-semibold text-slate-900;
}

.dirty-dot {
  @apply h-2 w-2 shrink-0 rounded-full bg-amber-400;
}

.file-path {
  @apply max-w-[42rem] truncate text-xs text-slate-500;
}

.editor-actions {
  @apply flex shrink-0 items-center gap-1;
}

.menu-button,
.icon-button,
.save-button {
  @apply inline-flex h-8 items-center justify-center rounded-md border border-slate-200 bg-white text-xs text-slate-700 shadow-sm hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-45 disabled:hover:bg-white;
}

.menu-button {
  @apply max-w-40 gap-1.5 px-2;
}

.menu-button span {
  @apply min-w-0 truncate;
}

.icon-button {
  @apply w-8;
}

.menu-button.active,
.icon-button.active {
  @apply border-blue-300 bg-blue-50 text-blue-700;
}

.save-button {
  @apply gap-1.5 border-blue-600 bg-blue-600 px-3 font-medium text-white hover:bg-blue-700 disabled:border-slate-200 disabled:bg-slate-100 disabled:text-slate-400;
}

.close-button {
  @apply hover:border-red-200 hover:bg-red-50;
}

.editor-info-left,
.editor-info-right {
  @apply flex min-w-0 items-center gap-3;
}

.editor-info-left span,
.editor-info-right span {
  @apply truncate;
}

.menu-layer {
  @apply absolute right-3 top-[5.25rem] z-30;
}

.editor-menu {
  @apply mt-2 flex max-h-80 min-w-44 flex-col gap-1 overflow-auto rounded-md border border-slate-200 bg-white p-1 text-sm shadow-2xl;
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
  @apply px-2 pt-1 text-xs font-medium text-slate-400;
}

.editor-menu button {
  @apply flex h-8 items-center gap-2 rounded px-2 text-left text-slate-700 hover:bg-blue-50;
}

.editor-menu button.active {
  @apply bg-blue-600 text-white hover:bg-blue-600;
}

.editor-menu label {
  @apply flex items-center justify-between gap-3 text-sm text-slate-600;
}

.editor-menu input[type="number"] {
  @apply h-8 w-20 rounded border border-slate-200 bg-white px-2 text-right text-slate-900 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100;
}

.check-row {
  @apply justify-start;
}

.check-row input {
  @apply h-4 w-4 accent-blue-600;
}

.editor-main {
  @apply relative min-h-0 grow bg-[#f7fbff] p-2;
}

.editor-frame {
  @apply h-full min-h-0 overflow-hidden rounded-md border border-slate-200 bg-white shadow-sm;
}

.editor-overlay {
  @apply absolute inset-2 z-10 flex items-center justify-center rounded-md bg-white/85 text-sm text-slate-500 backdrop-blur-sm;
}

.editor-overlay.error {
  @apply flex-col gap-3 text-red-600;
}

.editor-overlay button {
  @apply rounded-md border border-slate-200 bg-white px-3 py-1.5 text-slate-700 hover:bg-blue-50;
}

.editor-statusbar {
  @apply h-7 gap-3 border-t px-3 text-xs text-slate-500;
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
  @apply shrink-0 rounded bg-slate-100 px-2 py-0.5 text-slate-600;
}

.status-pill.dirty {
  @apply bg-amber-100 text-amber-700;
}

.status-pill.saving {
  @apply bg-blue-100 text-blue-700;
}

.status-pill.conflict,
.editor-message.conflict {
  @apply bg-red-50 text-red-600;
}

.editor-message {
  @apply truncate rounded px-2 py-0.5 text-red-600;
}

.status-action {
  @apply h-5 shrink-0 rounded border border-red-200 bg-white px-2 text-xs text-red-600 hover:bg-red-50;
}
</style>
