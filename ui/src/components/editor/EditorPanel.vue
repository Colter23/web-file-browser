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
type PendingEditorAction = "close" | "reload" | "external" | "";

type EditorCursorStatus = {
  line: number;
  column: number;
  selectedRows: number;
  selectedCharacters: number;
}

type CodeEditorExpose = ComponentPublicInstance & {
  focus?: () => void;
  getSelectedText?: () => string;
  find?: (options: EditorSearchOptions) => boolean;
  replaceCurrent?: (replacement: string) => boolean;
  replaceAll?: (replacement: string) => boolean;
}

type EditorSearchOptions = {
  needle: string;
  backwards?: boolean;
  caseSensitive?: boolean;
  wholeWord?: boolean;
  regex?: boolean;
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
const defaultCursorStatus = (): EditorCursorStatus => ({line: 1, column: 1, selectedRows: 0, selectedCharacters: 0});
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
const pendingAction = ref<PendingEditorAction>("");
const pendingBusy = ref(false);
const cursorStatus = ref<EditorCursorStatus>(defaultCursorStatus());
const fontSize = ref(readNumberPreference(storageKeys.fontSize, 16, 12, 28));
const tabSize = ref(readNumberPreference(storageKeys.tabSize, 2, 2, 8));
const wrap = ref(readBooleanPreference(storageKeys.wrap, true));
const searchVisible = ref(false);
const replaceVisible = ref(false);
const searchText = ref("");
const replaceText = ref("");
const searchStatus = ref("");
const searchCaseSensitive = ref(false);
const searchWholeWord = ref(false);
const searchRegex = ref(false);
const searchInputRef = ref<HTMLInputElement | null>(null);
const replaceInputRef = ref<HTMLInputElement | null>(null);
let loadVersion = 0;

const themeClass = computed(() => `ace-${currentTheme.value.replace(/_/g, "-")}`);
const canSave = computed(() => Boolean(fileInfo.value && isChange.value && contentEtag.value && !saveConflict.value && !loading.value && !saving.value));
const editorReadOnly = computed(() => loading.value || saving.value || Boolean(pendingAction.value));
const regexErrorText = computed(() => {
  if (!searchRegex.value || !searchText.value) return "";
  try {
    new RegExp(searchText.value);
    return "";
  } catch {
    return "正则表达式无效";
  }
});
const canFind = computed(() => Boolean(searchText.value) && !regexErrorText.value);
const canReplace = computed(() => canFind.value && !editorReadOnly.value);

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

const cursorStatusText = computed(() => `第 ${cursorStatus.value.line} 行，第 ${cursorStatus.value.column} 列`);

const selectionStatusText = computed(() => {
  if (!cursorStatus.value.selectedCharacters) return "";
  const rows = cursorStatus.value.selectedRows > 1 ? `${cursorStatus.value.selectedRows} 行，` : "";
  return `已选中 ${rows}${cursorStatus.value.selectedCharacters} 字符`;
});

const searchStatusText = computed(() => regexErrorText.value || searchStatus.value);

const dirtyText = computed(() => {
  if (saving.value) return "保存中";
  if (loading.value) return "加载中";
  if (saveConflict.value) return "需重新载入";
  if (isChange.value) return "未保存";
  return statusText.value || "已同步";
});

const editorMessageText = computed(() => errorText.value || (saveConflict.value ? "文件版本已变化，请重新载入后再保存" : ""));

const confirmTitle = computed(() => {
  if (pendingAction.value === "reload") return "重新载入文件？";
  if (pendingAction.value === "external") return "离开编辑器？";
  return "关闭编辑器？";
});

const confirmDescription = computed(() => {
  if (pendingAction.value === "reload") return "当前修改还没有保存，重新载入会用磁盘上的最新内容覆盖编辑区。";
  if (pendingAction.value === "external") return "当前修改还没有保存，继续操作会离开编辑器并丢弃未保存内容。";
  return "当前修改还没有保存，关闭后未保存的内容会被丢弃。";
});

const confirmSaveText = computed(() => {
  if (pendingBusy.value) return "处理中";
  if (pendingAction.value === "external") return "保存并离开";
  return pendingAction.value === "reload" ? "保存并重新载入" : "保存并关闭";
});

const confirmDiscardText = computed(() => {
  if (pendingAction.value === "reload") return "放弃并重新载入";
  if (pendingAction.value === "external") return "放弃并离开";
  return "放弃并关闭";
});

const closeMenus = () => {
  activeMenu.value = "";
}

const closeSearch = () => {
  searchVisible.value = false;
  replaceVisible.value = false;
  searchStatus.value = "";
  nextTick(() => editorRef.value?.focus?.());
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

const searchOptions = (backwards = false): EditorSearchOptions => ({
  needle: searchText.value,
  backwards,
  caseSensitive: searchCaseSensitive.value,
  wholeWord: searchWholeWord.value,
  regex: searchRegex.value
});

const runSearch = (backwards = false, keepSearchFocus = false) => {
  if (!searchText.value) {
    searchStatus.value = "";
    searchInputRef.value?.focus();
    return false;
  }
  if (regexErrorText.value) {
    searchStatus.value = regexErrorText.value;
    searchInputRef.value?.focus();
    return false;
  }
  const found = editorRef.value?.find?.(searchOptions(backwards)) ?? false;
  searchStatus.value = found ? "" : "未找到";
  if (keepSearchFocus) {
    nextTick(() => searchInputRef.value?.focus());
  }
  return found;
}

const openSearch = async (replace = false) => {
  if (!fileStore.showEditor) return;
  closeMenus();
  searchVisible.value = true;
  replaceVisible.value = replace;
  const selected = editorRef.value?.getSelectedText?.().trim() ?? "";
  if (selected && !selected.includes("\n")) searchText.value = selected.slice(0, 200);
  searchStatus.value = "";
  await nextTick();
  searchInputRef.value?.focus();
  searchInputRef.value?.select();
  if (searchText.value) runSearch(false, true);
}

const openReplace = async () => {
  await openSearch(true);
}

const toggleSearchOption = (option: "case" | "word" | "regex") => {
  if (option === "case") searchCaseSensitive.value = !searchCaseSensitive.value;
  if (option === "word") searchWholeWord.value = !searchWholeWord.value;
  if (option === "regex") searchRegex.value = !searchRegex.value;
  searchStatus.value = "";
  if (searchText.value) nextTick(() => runSearch(false, true));
}

const findFromInput = (event: KeyboardEvent) => {
  runSearch(event.shiftKey, true);
}

const replaceCurrentMatch = async () => {
  if (!canReplace.value) return;
  let replaced = editorRef.value?.replaceCurrent?.(replaceText.value) ?? false;
  if (!replaced && runSearch(false)) {
    replaced = editorRef.value?.replaceCurrent?.(replaceText.value) ?? false;
  }
  searchStatus.value = replaced ? "已替换" : regexErrorText.value || "未找到";
  if (replaced) await nextTick(() => runSearch(false));
}

const replaceAllMatches = () => {
  if (!canReplace.value) return;
  if (!runSearch(false)) return;
  const replaced = editorRef.value?.replaceAll?.(replaceText.value) ?? false;
  searchStatus.value = replaced ? "已全部替换" : "未找到";
}

const focusReplaceInput = () => {
  if (!replaceVisible.value) return;
  nextTick(() => {
    replaceInputRef.value?.focus();
    replaceInputRef.value?.select();
  });
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
  cursorStatus.value = defaultCursorStatus();
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

const resetEditorState = () => {
  closeMenus();
  searchVisible.value = false;
  replaceVisible.value = false;
  searchStatus.value = "";
  pendingAction.value = "";
  pendingBusy.value = false;
  fileStore.closeEditor();
  fileInfo.value = null;
  isChange.value = false;
  content.value = "";
  contentEtag.value = "";
  statusText.value = "";
  errorText.value = "";
  saveConflict.value = false;
  cursorStatus.value = defaultCursorStatus();
}

watch(() => [fileStore.showEditor, fileStore.currentFile?.path], loadCurrentFile);

watch(isChange, value => {
  fileStore.setEditorDirty(value);
});

watch(() => fileStore.editorLeaveRequestId, requestId => {
  if (!requestId || !fileStore.showEditor) return;
  if (!isChange.value) {
    fileStore.resolveEditorLeave(true);
    return;
  }
  pendingAction.value = "external";
  closeMenus();
});

const onContentChange = (value: string) => {
  if (loading.value) return;
  pendingAction.value = "";
  content.value = value;
  isChange.value = true;
  statusText.value = "";
  errorText.value = "";
  if (saveConflict.value) saveConflict.value = false;
}

const onCursorChange = (status: EditorCursorStatus) => {
  cursorStatus.value = status;
}

const save = async () => {
  if (!fileInfo.value || saving.value || loading.value || saveConflict.value) return;
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
  if (isChange.value) {
    pendingAction.value = "reload";
    closeMenus();
    return;
  }
  await loadCurrentFile();
}

const close = () => {
  if (isChange.value) {
    pendingAction.value = "close";
    closeMenus();
    return;
  }
  resetEditorState();
}

const cancelPendingAction = () => {
  if (pendingBusy.value) return;
  if (pendingAction.value === "external") fileStore.resolveEditorLeave(false);
  pendingAction.value = "";
  nextTick(() => editorRef.value?.focus?.());
}

const finishPendingAction = async () => {
  const action = pendingAction.value;
  pendingAction.value = "";
  if (action === "reload") {
    await loadCurrentFile();
    return;
  }
  if (action === "external") {
    fileStore.resolveEditorLeave(true);
    return;
  }
  if (action === "close") resetEditorState();
}

const discardPendingAction = async () => {
  if (pendingBusy.value) return;
  pendingBusy.value = true;
  isChange.value = false;
  await finishPendingAction();
  pendingBusy.value = false;
}

const savePendingAction = async () => {
  if (pendingBusy.value || !canSave.value) return;
  pendingBusy.value = true;
  await save();
  pendingBusy.value = false;
  if (!isChange.value && !saveConflict.value && !errorText.value) {
    await finishPendingAction();
  }
}

const handleKeyDown = (event: KeyboardEvent) => {
  if (!fileStore.showEditor) return;
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f") {
    event.preventDefault();
    void openSearch(false);
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "h") {
    event.preventDefault();
    void openReplace();
    return;
  }
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
    if (pendingAction.value) {
      cancelPendingAction();
      return;
    }
    if (searchVisible.value) {
      closeSearch();
      return;
    }
    close();
  }
}

const handleBeforeUnload = (event: BeforeUnloadEvent) => {
  if (!fileStore.showEditor || !isChange.value) return;
  event.preventDefault();
  event.returnValue = "";
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
  window.addEventListener("beforeunload", handleBeforeUnload);
  void loadCurrentFile();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeyDown);
  window.removeEventListener("beforeunload", handleBeforeUnload);
  fileStore.resolveEditorLeave(false);
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
      <div v-if="searchVisible" class="search-bar" @click.stop>
        <div class="search-fields">
          <input
              ref="searchInputRef"
              v-model="searchText"
              class="search-input"
              type="text"
              placeholder="查找"
              @keydown.enter.prevent="findFromInput"
              @input="searchStatus = ''">
          <input
              v-if="replaceVisible"
              ref="replaceInputRef"
              v-model="replaceText"
              class="search-input replace-input"
              type="text"
              placeholder="替换为"
              @keydown.enter.prevent="replaceCurrentMatch">
        </div>
        <div class="search-actions">
          <span v-if="searchStatusText" class="search-status">{{ searchStatusText }}</span>
          <button title="上一个 (Shift+Enter)" :disabled="!canFind" @click="runSearch(true)">
            <icon icon="icon-back_android" class="rotate-90" />
          </button>
          <button title="下一个 (Enter)" :disabled="!canFind" @click="runSearch(false)">
            <icon icon="icon-back_android" class="-rotate-90" />
          </button>
          <button v-if="!replaceVisible" title="显示替换 (Ctrl+H)" @click="replaceVisible = true; focusReplaceInput()">
            <icon icon="icon-renamebox" />
          </button>
          <button v-if="replaceVisible" class="text-tool" title="替换当前" :disabled="!canReplace" @click="replaceCurrentMatch">替换</button>
          <button v-if="replaceVisible" class="text-tool" title="全部替换" :disabled="!canReplace" @click="replaceAllMatches">全部</button>
          <button class="text-tool" :class="{active: searchCaseSensitive}" title="区分大小写" @click="toggleSearchOption('case')">Aa</button>
          <button class="text-tool" :class="{active: searchWholeWord}" title="全词匹配" @click="toggleSearchOption('word')">W</button>
          <button class="text-tool" :class="{active: searchRegex}" title="正则表达式" @click="toggleSearchOption('regex')">.*</button>
          <button title="关闭查找" @click="closeSearch">
            <icon icon="icon-close" />
          </button>
        </div>
      </div>
      <div class="editor-frame">
        <code-editor
            ref="editorRef"
            :mode="currentMode"
            :theme="currentTheme"
            :content="content"
            :font-size="fontSize"
            :wrap="wrap"
            :tab-size="tabSize"
            :read-only="editorReadOnly"
            @change="onContentChange"
            @cursor-change="onCursorChange"
            @find="openSearch(false)"
            @replace="openReplace"
            @save="save">
        </code-editor>
      </div>
      <div v-if="loading" class="editor-overlay">正在打开文件...</div>
      <div v-else-if="errorText" class="editor-overlay error">
        <span>{{ errorText }}</span>
        <button @click="reload">重试</button>
      </div>
      <div v-if="pendingAction" class="editor-confirm-mask" @click.stop>
        <section class="editor-confirm">
          <div class="confirm-icon">
            <icon icon="icon-edit-filling" color="#2563eb" />
          </div>
          <div class="confirm-content">
            <h3>{{ confirmTitle }}</h3>
            <p>{{ confirmDescription }}</p>
          </div>
          <div class="confirm-actions">
            <button class="confirm-secondary" :disabled="pendingBusy" @click="cancelPendingAction">取消</button>
            <button class="confirm-danger" :disabled="pendingBusy" @click="discardPendingAction">{{ confirmDiscardText }}</button>
            <button class="confirm-primary" :disabled="!canSave || pendingBusy" @click="savePendingAction">{{ confirmSaveText }}</button>
          </div>
        </section>
      </div>
    </main>

    <footer class="editor-statusbar">
      <div class="status-left">
        <span v-if="editorMessageText" :class="['editor-message', {conflict: saveConflict}]">{{ editorMessageText }}</span>
        <span v-else>{{ filePathText }}</span>
      </div>
      <div class="status-right">
        <button v-if="saveConflict" class="status-action" @click="reload">重新载入</button>
        <span>{{ cursorStatusText }}</span>
        <span v-if="selectionStatusText">{{ selectionStatusText }}</span>
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
  @apply relative flex min-h-0 grow flex-col gap-2 bg-[#f7fbff] p-2;
}

.search-bar {
  @apply relative z-20 flex shrink-0 items-center justify-between gap-2 rounded-md border border-slate-200 bg-white px-2 py-1.5 text-xs shadow-sm;
}

.search-fields {
  @apply flex min-w-0 grow items-center gap-2;
}

.search-input {
  @apply h-8 min-w-0 flex-1 rounded-md border border-slate-200 bg-white px-2 text-sm text-slate-900 outline-none placeholder:text-slate-400 focus:border-blue-500 focus:ring-2 focus:ring-blue-100;
}

.replace-input {
  @apply border-slate-300;
}

.search-actions {
  @apply flex shrink-0 items-center gap-1 text-slate-600;
}

.search-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-slate-200 bg-white px-2 text-xs font-medium text-slate-600 hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-white;
}

.search-actions button.active {
  @apply border-blue-300 bg-blue-50 text-blue-700;
}

.search-actions .text-tool {
  @apply min-w-9;
}

.search-status {
  @apply max-w-28 truncate px-1 text-amber-600;
}

.editor-frame {
  @apply min-h-0 grow overflow-hidden rounded-md border border-slate-200 bg-white shadow-sm;
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

.editor-confirm-mask {
  @apply absolute inset-2 z-20 flex items-center justify-center rounded-md bg-slate-900/15 px-4 backdrop-blur-sm;
}

.editor-confirm {
  @apply grid w-full max-w-lg grid-cols-[2rem_1fr] gap-3 rounded-md border border-slate-200 bg-white p-4 text-slate-700 shadow-2xl;
}

.confirm-icon {
  @apply flex h-8 w-8 items-center justify-center rounded-md bg-blue-50;
}

.confirm-content {
  @apply min-w-0;
}

.confirm-content h3 {
  @apply text-sm font-semibold text-slate-900;
}

.confirm-content p {
  @apply mt-1 text-xs leading-5 text-slate-500;
}

.confirm-actions {
  @apply col-span-2 mt-1 flex justify-end gap-2;
}

.confirm-primary,
.confirm-secondary,
.confirm-danger {
  @apply h-8 rounded-md border px-3 text-xs font-medium disabled:cursor-not-allowed disabled:opacity-50;
}

.confirm-primary {
  @apply border-blue-600 bg-blue-600 text-white hover:bg-blue-700;
}

.confirm-secondary {
  @apply border-slate-200 bg-white text-slate-700 hover:bg-slate-50;
}

.confirm-danger {
  @apply border-red-200 bg-white text-red-600 hover:bg-red-50;
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
