<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import CodeEditor from "./CodeEditor.vue";
import editorConfig from "../../assets/editor-config.json";
import {useFileStore} from "../../store";
import {useEditorFileSession} from "../../composables/useEditorFileSession.ts";
import {useEditorPreferences} from "../../composables/useEditorPreferences.ts";
import {useEditorSearch} from "../../composables/useEditorSearch.ts";
import {useEditorStatusText} from "../../composables/useEditorStatusText.ts";
import type {CodeEditorExpose, EditorMenuAnchor, EditorMenuName} from "./types.ts";
import EditorGotoBar from "./EditorGotoBar.vue";
import EditorConfirmDialog from "./EditorConfirmDialog.vue";
import EditorMenuLayer from "./EditorMenuLayer.vue";
import EditorSearchBar from "./EditorSearchBar.vue";
import EditorStatusBar from "./EditorStatusBar.vue";
import EditorTitleBar from "./EditorTitleBar.vue";

const fileStore = useFileStore();
const editorRef = ref<CodeEditorExpose | null>(null);
const activeMenu = ref<EditorMenuName>("");
const menuAnchor = ref<EditorMenuAnchor | null>(null);
const {
  currentTheme,
  currentHighlight,
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

const themeClass = computed(() => `editor-theme-${currentTheme.value.replace(/_/g, "-")}`);

const closeMenus = () => {
  activeMenu.value = "";
  menuAnchor.value = null;
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
  selectedHighlightName,
  fileSizeText,
  wrapText,
  cursorStatusText,
  selectionStatusText,
  editorMessageText,
  confirmTitle,
  confirmDescription,
  confirmSaveText,
  confirmDiscardText
} = useEditorStatusText({
  fileInfo,
  currentMode,
  currentTheme,
  currentHighlight,
  modes: editorConfig.mode,
  themes: editorConfig.theme,
  highlights: editorConfig.highlight,
  wrap,
  cursorStatus,
  saveConflict,
  errorText,
  pendingAction,
  pendingBusy
});

const toggleMenu = (menu: EditorMenuName, anchor: EditorMenuAnchor) => {
  if (activeMenu.value === menu) {
    closeMenus();
    return;
  }
  activeMenu.value = menu;
  menuAnchor.value = anchor;
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

const changeHighlight = (highlight: string) => {
  currentHighlight.value = highlight;
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

const handleGlobalPointerDown = (event: PointerEvent) => {
  if (!activeMenu.value) return;
  const target = event.target as Element | null;
  if (!target) return;
  if (target.closest("[data-editor-menu-layer]") || target.closest("[data-editor-menu-button]")) return;
  closeMenus();
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
  window.addEventListener("pointerdown", handleGlobalPointerDown, true);
  window.addEventListener("beforeunload", handleBeforeUnload);
  void loadCurrentFile();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeyDown);
  window.removeEventListener("pointerdown", handleGlobalPointerDown, true);
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
        :selected-highlight-name="selectedHighlightName"
        :loading="loading"
        :saving="saving"
        :can-save="canSave"
        @toggle-menu="toggleMenu"
        @reload="reload"
        @save="save"
        @close="close" />

    <editor-menu-layer
        v-model:font-size="fontSize"
        v-model:tab-size="tabSize"
        v-model:wrap="wrap"
        :active-menu="activeMenu"
        :anchor="menuAnchor"
        :modes="editorConfig.mode"
        :themes="editorConfig.theme"
        :highlights="editorConfig.highlight"
        :current-mode="currentMode"
        :current-theme="currentTheme"
        :current-highlight="currentHighlight"
        @change-mode="changeMode"
        @change-theme="changeTheme"
        @change-highlight="changeHighlight" />

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
            :highlight="currentHighlight"
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
      <editor-confirm-dialog
          :visible="Boolean(pendingAction)"
          :title="confirmTitle"
          :description="confirmDescription"
          :save-text="confirmSaveText"
          :discard-text="confirmDiscardText"
          :can-save="canSave"
          :busy="pendingBusy"
          @cancel="cancelPendingAction"
          @discard="discardPendingAction"
          @save="savePendingAction" />
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
  @apply relative flex h-full min-h-0 flex-col overflow-hidden;
  background: var(--app-panel-muted);
  color: var(--app-text);
}

.editor-main {
  @apply relative flex min-h-0 grow flex-col gap-2 p-2;
  background: var(--app-panel-muted);
}

.editor-frame {
  @apply min-h-0 grow overflow-hidden rounded-md border shadow-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

.editor-overlay {
  @apply absolute inset-2 z-10 flex items-center justify-center rounded-md text-sm backdrop-blur-sm;
  background: color-mix(in srgb, var(--app-panel-solid) 86%, transparent);
  color: var(--app-text-subtle);
}

.editor-overlay.error {
  @apply flex-col gap-3;
  color: var(--app-danger);
}

.editor-overlay button {
  @apply rounded-md border px-3 py-1.5;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.editor-overlay button:hover {
  background: var(--app-accent-hover, #eff6ff);
}

</style>
