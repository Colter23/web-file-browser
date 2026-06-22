import {computed, nextTick, ref} from "vue";
import type {ComponentPublicInstance, ComputedRef, Ref} from "vue";
import type {CodeEditorExpose, EditorCursorStatus, EditorSearchOptions} from "../components/editor/types.ts";

type EditorSearchOptionsConfig = {
  editorRef: Ref<CodeEditorExpose | null>;
  cursorStatus: Ref<EditorCursorStatus>;
  editorReadOnly: ComputedRef<boolean>;
  isEditorActive: () => boolean;
  closeMenus: () => void;
}

export const useEditorSearch = ({
  editorRef,
  cursorStatus,
  editorReadOnly,
  isEditorActive,
  closeMenus
}: EditorSearchOptionsConfig) => {
  const searchVisible = ref(false);
  const replaceVisible = ref(false);
  const searchText = ref("");
  const replaceText = ref("");
  const searchStatus = ref("");
  const searchCaseSensitive = ref(false);
  const searchWholeWord = ref(false);
  const searchRegex = ref(false);
  const gotoVisible = ref(false);
  const gotoLineText = ref("");
  const gotoStatus = ref("");
  const searchInputRef = ref<HTMLInputElement | null>(null);
  const replaceInputRef = ref<HTMLInputElement | null>(null);
  const gotoInputRef = ref<HTMLInputElement | null>(null);

  const resolveInputElement = (element: Element | ComponentPublicInstance | null) => {
    if (element instanceof HTMLInputElement) return element;
    if (element && "$el" in element && element.$el instanceof HTMLInputElement) return element.$el;
    return null;
  }

  const setSearchInputRef = (element: Element | ComponentPublicInstance | null) => {
    searchInputRef.value = resolveInputElement(element);
  }

  const setReplaceInputRef = (element: Element | ComponentPublicInstance | null) => {
    replaceInputRef.value = resolveInputElement(element);
  }

  const setGotoInputRef = (element: Element | ComponentPublicInstance | null) => {
    gotoInputRef.value = resolveInputElement(element);
  }

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
  const editorLineCount = computed(() => editorRef.value?.getLineCount?.() ?? cursorStatus.value.line);
  const gotoLineNumber = computed(() => Number(gotoLineText.value));
  const canGotoLine = computed(() => Number.isInteger(gotoLineNumber.value) && gotoLineNumber.value >= 1 && gotoLineNumber.value <= Math.max(1, editorLineCount.value));
  const searchStatusText = computed(() => regexErrorText.value || searchStatus.value);
  const gotoPlaceholder = computed(() => `1-${Math.max(1, editorLineCount.value)}`);

  const closeSearch = () => {
    searchVisible.value = false;
    replaceVisible.value = false;
    searchStatus.value = "";
    nextTick(() => editorRef.value?.focus?.());
  }

  const closeGoto = () => {
    gotoVisible.value = false;
    gotoStatus.value = "";
    nextTick(() => editorRef.value?.focus?.());
  }

  const resetSearchState = () => {
    searchVisible.value = false;
    replaceVisible.value = false;
    searchStatus.value = "";
    gotoVisible.value = false;
    gotoStatus.value = "";
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
    if (!isEditorActive()) return;
    closeMenus();
    gotoVisible.value = false;
    gotoStatus.value = "";
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

  const openGotoLine = async () => {
    if (!isEditorActive()) return;
    closeMenus();
    searchVisible.value = false;
    replaceVisible.value = false;
    searchStatus.value = "";
    gotoVisible.value = true;
    gotoLineText.value = String(cursorStatus.value.line);
    gotoStatus.value = "";
    await nextTick();
    gotoInputRef.value?.focus();
    gotoInputRef.value?.select();
  }

  const submitGotoLine = () => {
    if (!canGotoLine.value) {
      gotoStatus.value = "行号无效";
      gotoInputRef.value?.focus();
      return;
    }
    const moved = editorRef.value?.gotoLine?.(gotoLineNumber.value) ?? false;
    gotoStatus.value = moved ? "" : "无法跳转";
    if (moved) closeGoto();
  }

  return {
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
    regexErrorText,
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
  };
}
