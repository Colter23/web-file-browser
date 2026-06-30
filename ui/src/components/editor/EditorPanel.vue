<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import CodeEditor from "./CodeEditor.vue";
import editorConfig from "../../assets/editor-config.json";
import {useI18n} from "../../i18n";
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
import type {ShellNoticeKind} from "../shell/types.ts";

const fileStore = useFileStore();
const {t} = useI18n();
const emit = defineEmits<{
  (e: "notice", payload: {message: string; kind?: ShellNoticeKind; title?: string}): void;
}>();
const editorRef = ref<CodeEditorExpose | null>(null);
const activeMenu = ref<EditorMenuName>("");
const menuAnchor = ref<EditorMenuAnchor | null>(null);
const {
  currentTheme,
  currentHighlight,
  fontSize,
  tabSize,
  wrap,
  showWhitespace,
  autoSave,
  defaultEditMode
} = useEditorPreferences();

const editMode = ref(defaultEditMode.value);
const autoSaveDelayMs = 1200;
let resetSearchStateHandler = () => {};
let autoSaveTimer: ReturnType<typeof window.setTimeout> | undefined;
const resetSearchStateProxy = () => resetSearchStateHandler();
const focusEditor = () => editorRef.value?.focus?.();
const showEditorNotice = (message: string, kind?: ShellNoticeKind, title?: string) => {
  emit("notice", {message, kind, title});
}

const {
  fileInfo,
  currentMode,
  content,
  isChange,
  loading,
  saving,
  openFailed,
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
  focusEditor,
  showNotice: showEditorNotice
});

const editorLocked = computed(() => editorReadOnly.value || !editMode.value);
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
  editorReadOnly: editorLocked,
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

const clampFontSize = (value: number) => {
  if (!Number.isFinite(value)) return 18;
  return Math.min(28, Math.max(12, Math.round(value)));
}

const adjustFontSize = (step: number) => {
  fontSize.value = clampFontSize(fontSize.value + step);
}

const saveManually = () => {
  void save({notifySuccess: true});
}

const clearAutoSaveTimer = () => {
  if (autoSaveTimer === undefined) return;
  window.clearTimeout(autoSaveTimer);
  autoSaveTimer = undefined;
}

const scheduleAutoSave = () => {
  clearAutoSaveTimer();
  if (!autoSave.value || !canSave.value || pendingAction.value) return;
  autoSaveTimer = window.setTimeout(() => {
    autoSaveTimer = undefined;
    if (autoSave.value && canSave.value && !pendingAction.value) {
      void save({notifySuccess: false});
    }
  }, autoSaveDelayMs);
}

const showReplace = () => {
  if (editorLocked.value) return;
  replaceVisible.value = true;
  focusReplaceInput();
}

const clearSearchStatus = () => {
  searchStatus.value = "";
}

const clearGotoStatus = () => {
  gotoStatus.value = "";
}

const openReplacePanel = async () => {
  if (editorLocked.value) return;
  await openReplace();
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
    void openReplacePanel();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "g") {
    event.preventDefault();
    void openGotoLine();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
    event.preventDefault();
    saveManually();
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

watch(() => [fileStore.showEditor, fileStore.currentFile?.path] as const, ([visible, path]) => {
  if (!visible || !path) return;
  clearAutoSaveTimer();
  editMode.value = defaultEditMode.value;
}, {immediate: true});

watch(() => [autoSave.value, content.value, canSave.value, pendingAction.value] as const, scheduleAutoSave);

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
  window.addEventListener("pointerdown", handleGlobalPointerDown, true);
  window.addEventListener("beforeunload", handleBeforeUnload);
  void loadCurrentFile();
});

onBeforeUnmount(() => {
  clearAutoSaveTimer();
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
        v-model:edit-mode="editMode"
        :loading="loading"
        :saving="saving"
        :can-save="canSave"
        @toggle-menu="toggleMenu"
        @reload="reload"
        @save="saveManually"
        @close="close" />

    <editor-menu-layer
        v-model:font-size="fontSize"
        v-model:tab-size="tabSize"
        v-model:wrap="wrap"
        v-model:show-whitespace="showWhitespace"
        v-model:auto-save="autoSave"
        v-model:default-edit-mode="defaultEditMode"
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
          :read-only="editorLocked"
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
            :show-whitespace="showWhitespace"
            :read-only="editorLocked"
            @change="onContentChange"
            @cursor-change="onCursorChange"
            @zoom-font="adjustFontSize"
            @find="openSearch(false)"
            @goto-line="openGotoLine"
            @replace="openReplacePanel"
            @save="saveManually">
        </code-editor>
      </div>
      <div v-if="loading" class="editor-overlay">{{ t("editor.loadingFile") }}</div>
      <div v-else-if="openFailed && errorText" class="editor-overlay error">
        <span>{{ errorText }}</span>
        <button @click="reload">{{ t("editor.retry") }}</button>
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
        :dirty="isChange"
        :saving="saving"
        :edit-mode="editMode"
        :auto-save="autoSave"
        :font-size="fontSize"
        :tab-size="tabSize"
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
