import {computed, ref} from "vue";
import type {ComponentPublicInstance} from "vue";

type SearchFocusTarget = "explorer" | "search" | false;

type ExplorerSearchBoxOptions = {
  focusExplorer: () => void | Promise<void>;
}

export const useExplorerSearchBox = ({focusExplorer}: ExplorerSearchBoxOptions) => {
  const searchInput = ref<HTMLInputElement | null>(null);
  const searchText = ref("");
  const isFiltering = computed(() => Boolean(searchText.value.trim()));

  const setSearchInputRef = (element: Element | ComponentPublicInstance | null) => {
    searchInput.value = element instanceof HTMLInputElement ? element : null;
  }

  const clearSearch = (focus: SearchFocusTarget = "explorer") => {
    searchText.value = "";
    if (focus === "search") searchInput.value?.focus();
    if (focus === "explorer") void focusExplorer();
  }

  const handleSearchEscape = () => {
    if (isFiltering.value) {
      clearSearch();
      return;
    }
    searchInput.value?.blur();
    void focusExplorer();
  }

  const focusSearchInput = () => {
    searchInput.value?.focus();
    searchInput.value?.select();
  }

  return {
    setSearchInputRef,
    searchText,
    isFiltering,
    clearSearch,
    handleSearchEscape,
    focusSearchInput
  };
}
