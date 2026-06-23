<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import Icon from "../Icon.vue";
import CodeEditor from "./CodeEditor.vue";
import editorConfig from "../../assets/editor-config.json";
import {useFileStore} from "../../store";
import {useEditorFileSession} from "../../composables/useEditorFileSession.ts";
import {useEditorPreferences} from "../../composables/useEditorPreferences.ts";
import {useEditorSearch} from "../../composables/useEditorSearch.ts";
import {useEditorStatusText} from "../../composables/useEditorStatusText.ts";
import type {CodeEditorExpose, EditorMenuName} from "./types.ts";
import EditorGotoBar from "./EditorGotoBar.vue";
import EditorInfoBar from "./EditorInfoBar.vue";
import EditorMenuLayer from "./EditorMenuLayer.vue";
import EditorSearchBar from "./EditorSearchBar.vue";
import EditorStatusBar from "./EditorStatusBar.vue";
import EditorTitleBar from "./EditorTitleBar.vue";

const fileStore = useFileStore();
const editorRef = ref<CodeEditorExpose | null>(null);
const confirmRef = ref<HTMLElement | null>(null);
const activeMenu = ref<EditorMenuName>("");
const {
  currentTheme,
  fontSize,
  tabSize,
  wrap
} = useEditorPreferences();

let resetSearchStateHandler = () => {};
const resetSearchStateProxy = () => resetSearchStateHandler();
const focusEditor = () => editorRef.value?.focus?.();

const {
  fileInfo,
  currentMode,
  content,
  isChange,
  loading,
  saving,
  statusText,
  errorText,
  saveConflict,
  pendingAction,
  pendingBusy,
  cursorStatus,
  canSave,
  editorReadOnly,
  loadCurrentFile,
  onContentChange,
  onCursorChange,
  save,
  reload,
  close,
  cancelPendingAction,
  discardPendingAction,
  savePendingAction,
  handleBeforeUnload,
  dispose: disposeEditorSession
} = useEditorFileSession({
  closeMenus: () => closeMenus(),
  resetSearchState: resetSearchStateProxy,
  focusEditor
});

const themeClass = computed(() => `ace-${currentTheme.value.replace(/_/g, "-")}`);

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

resetSearchStateHandler = resetSearchState;

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

watch(pendingAction, async action => {
  if (!action) return;
  await nextTick();
  confirmRef.value?.focus();
});

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
  window.addEventListener("beforeunload", handleBeforeUnload);
  void loadCurrentFile();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeyDown);
  window.removeEventListener("beforeunload", handleBeforeUnload);
  disposeEditorSession();
});
</script>

<template>
  <div class="editor-shell" :class="themeClass" @click="closeMenus">
    <editor-title-bar
        :active-menu="activeMenu"
        :file-title="fileTitle"
        :file-path-text="filePathText"
        :dirty="isChange"
        :selected-mode-name="selectedModeName"
        :selected-theme-name="selectedThemeName"
        :loading="loading"
        :saving="saving"
        :can-save="canSave"
        @toggle-menu="toggleMenu"
        @reload="reload"
        @save="save"
        @close="close" />

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
        <section ref="confirmRef" class="editor-confirm" tabindex="-1" @keydown.esc.prevent.stop="cancelPendingAction">
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
  @apply grid w-full max-w-lg grid-cols-[2rem_1fr] gap-3 rounded-md border border-slate-200 bg-white p-4 text-slate-700 shadow-2xl outline-none;
}

.editor-confirm:focus-visible {
  @apply ring-2 ring-inset ring-blue-300;
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
