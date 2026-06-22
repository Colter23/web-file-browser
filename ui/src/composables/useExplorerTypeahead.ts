import {ref} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type ExplorerTypeaheadOptions = {
  entries: ComputedRef<ExplorerEntry[]>;
  focusedPath: Ref<string>;
  indexOfPath: (path: string) => number;
  focusEntry: (entry: ExplorerEntry) => void;
  closeContextMenu: () => void;
}

export const useExplorerTypeahead = ({
  entries,
  focusedPath,
  indexOfPath,
  focusEntry,
  closeContextMenu
}: ExplorerTypeaheadOptions) => {
  const query = ref("");
  const resetMs = 900;
  let resetTimer = 0;

  const reset = () => {
    if (resetTimer) {
      window.clearTimeout(resetTimer);
      resetTimer = 0;
    }
    query.value = "";
  }

  const scheduleReset = () => {
    if (resetTimer) window.clearTimeout(resetTimer);
    resetTimer = window.setTimeout(() => {
      query.value = "";
      resetTimer = 0;
    }, resetMs);
  }

  const findEntry = (text: string, startIndex: number) => {
    if (!text || !entries.value.length) return null;
    const normalizedQuery = text.toLocaleLowerCase("zh-CN");
    const total = entries.value.length;
    for (let offset = 0; offset < total; offset += 1) {
      const index = (startIndex + offset + total) % total;
      const entry = entries.value[index];
      if (entry.name.toLocaleLowerCase("zh-CN").startsWith(normalizedQuery)) return entry;
    }
    return null;
  }

  const handleTypeahead = (event: KeyboardEvent) => {
    if (event.isComposing || event.ctrlKey || event.metaKey || event.altKey || event.key.length !== 1 || event.key === " ") return false;
    event.preventDefault();
    closeContextMenu();
    const key = event.key.toLocaleLowerCase("zh-CN");
    const previous = query.value;
    const repeatingSingleKey = Boolean(previous) && Array.from(previous).every(char => char === key);
    const nextQuery = repeatingSingleKey ? key : `${previous}${key}`;
    const currentIndex = focusedPath.value ? indexOfPath(focusedPath.value) : -1;
    const startIndex = previous && !repeatingSingleKey ? Math.max(0, currentIndex) : currentIndex + 1;
    let matched = findEntry(nextQuery, startIndex);
    let matchedQuery = nextQuery;
    if (!matched && nextQuery !== key) {
      matched = findEntry(key, currentIndex + 1);
      matchedQuery = key;
    }
    if (matched) {
      focusEntry(matched);
      query.value = matchedQuery;
    } else {
      query.value = key;
    }
    scheduleReset();
    return true;
  }

  return {
    reset,
    handleTypeahead
  };
}
