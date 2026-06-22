<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import Icon from "../Icon.vue";
import CodeEditor from "./CodeEditor.vue";
import editorConfig from "../../assets/editor-config.json";
import {useFileStore} from "../../store";
import type {FileInfo} from "../../class.ts";
import {getFile, saveFile} from "../../network/file-api.ts";
import {isApiError} from "../../network";
import {useEditorPreferences} from "../../composables/useEditorPreferences.ts";
import {useEditorSearch} from "../../composables/useEditorSearch.ts";
import {useEditorStatusText} from "../../composables/useEditorStatusText.ts";
import {checkFileLanguageMode} from "../../utils/common.ts";
import type {CodeEditorExpose, EditorCursorStatus, EditorMenuName, PendingEditorAction} from "./types.ts";
import EditorGotoBar from "./EditorGotoBar.vue";
import EditorInfoBar from "./EditorInfoBar.vue";
import EditorMenuLayer from "./EditorMenuLayer.vue";
import EditorSearchBar from "./EditorSearchBar.vue";
import EditorStatusBar from "./EditorStatusBar.vue";

const fileStore = useFileStore();
const defaultCursorStatus = (): EditorCursorStatus => ({line: 1, column: 1, selectedRows: 0, selectedCharacters: 0});
const fileInfo = ref<FileInfo | null>(null);
const editorRef = ref<CodeEditorExpose | null>(null);
const activeMenu = ref<EditorMenuName>("");
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
const {
  currentTheme,
  fontSize,
  tabSize,
  wrap
} = useEditorPreferences();
let loadVersion = 0;

const themeClass = computed(() => `ace-${currentTheme.value.replace(/_/g, "-")}`);
const canSave = computed(() => Boolean(fileInfo.value && isChange.value && contentEtag.value && !saveConflict.value && !loading.value && !saving.value));
const editorReadOnly = computed(() => loading.value || saving.value || Boolean(pendingAction.value));

const closeMenus = () => {
  activeMenu.value = "";
}

const {
  searchVisible,
  replaceVisible,
  searchText,
  replaceText,
  searchStatus,
  searchCaseSensitive,
  searchWholeWord,
  searchRegex,
  gotoVisible,
  gotoLineText,
  gotoStatus,
  setSearchInputRef,
  setReplaceInputRef,
  setGotoInputRef,
  canFind,
  canReplace,
  editorLineCount,
  canGotoLine,
  searchStatusText,
  gotoPlaceholder,
  closeSearch,
  closeGoto,
  resetSearchState,
  runSearch,
  openSearch,
  openReplace,
  toggleSearchOption,
  findFromInput,
  replaceCurrentMatch,
  replaceAllMatches,
  focusReplaceInput,
  openGotoLine,
  submitGotoLine
} = useEditorSearch({
  editorRef,
  cursorStatus,
  editorReadOnly,
  isEditorActive: () => fileStore.showEditor,
  closeMenus
});

const {
  fileTitle,
  filePathText,
  selectedModeName,
  selectedThemeName,
  editorMetaText,
  modifiedText,
  fileSizeText,
  wrapText,
  cursorStatusText,
  selectionStatusText,
  dirtyText,
  editorMessageText,
  confirmTitle,
  confirmDescription,
  confirmSaveText,
  confirmDiscardText
} = useEditorStatusText({
  fileInfo,
  currentMode,
  currentTheme,
  modes: editorConfig.mode,
  themes: editorConfig.theme,
  wrap,
  cursorStatus,
  saving,
  loading,
  saveConflict,
  isChange,
  statusText,
  errorText,
  pendingAction,
  pendingBusy
});

const toggleMenu = (menu: EditorMenuName) => {
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

const showReplace = () => {
  replaceVisible.value = true;
  focusReplaceInput();
}

const clearSearchStatus = () => {
  searchStatus.value = "";
}

const clearGotoStatus = () => {
  gotoStatus.value = "";
}

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
  resetSearchState();
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
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "g") {
    event.preventDefault();
    void openGotoLine();
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
    if (gotoVisible.value) {
      closeGoto();
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

    <editor-info-bar
        :dirty-text="dirtyText"
        :meta-text="editorMetaText"
        :modified-text="modifiedText"
        :dirty="isChange"
        :saving="saving"
        :conflict="saveConflict" />

    <editor-menu-layer
        v-model:font-size="fontSize"
        v-model:tab-size="tabSize"
        v-model:wrap="wrap"
        :active-menu="activeMenu"
        :modes="editorConfig.mode"
        :themes="editorConfig.theme"
        :current-mode="currentMode"
        :current-theme="currentTheme"
        @change-mode="changeMode"
        @change-theme="changeTheme" />

    <main class="editor-main">
      <editor-search-bar
          v-model:search-text="searchText"
          v-model:replace-text="replaceText"
          :visible="searchVisible"
          :replace-visible="replaceVisible"
          :search-status-text="searchStatusText"
          :case-sensitive="searchCaseSensitive"
          :whole-word="searchWholeWord"
          :regex="searchRegex"
          :can-find="canFind"
          :can-replace="canReplace"
          :set-search-input-ref="setSearchInputRef"
          :set-replace-input-ref="setReplaceInputRef"
          @show-replace="showReplace"
          @search="runSearch"
          @search-input="findFromInput"
          @replace-current="replaceCurrentMatch"
          @replace-all="replaceAllMatches"
          @toggle-option="toggleSearchOption"
          @clear-status="clearSearchStatus"
          @close="closeSearch" />
      <editor-goto-bar
          v-model:line-text="gotoLineText"
          :visible="gotoVisible"
          :status="gotoStatus"
          :line-count="editorLineCount"
          :placeholder="gotoPlaceholder"
          :can-goto-line="canGotoLine"
          :set-goto-input-ref="setGotoInputRef"
          @clear-status="clearGotoStatus"
          @submit="submitGotoLine"
          @close="closeGoto" />
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
            @goto-line="openGotoLine"
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

    <editor-status-bar
        :message-text="editorMessageText"
        :file-path-text="filePathText"
        :conflict="saveConflict"
        :cursor-text="cursorStatusText"
        :selection-text="selectionStatusText"
        :mode-text="selectedModeName"
        :size-text="fileSizeText"
        :wrap-text="wrapText"
        @reload="reload" />
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.editor-shell {
  @apply relative flex h-full min-h-0 flex-col overflow-hidden bg-[#f7fbff] text-slate-900;
}

.editor-titlebar {
  @apply relative z-20 flex shrink-0 items-center justify-between border-slate-200 bg-white/90 backdrop-blur;
}

.editor-titlebar {
  @apply h-12 gap-3 border-b px-3;
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

.editor-main {
  @apply relative flex min-h-0 grow flex-col gap-2 bg-[#f7fbff] p-2;
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

</style>
