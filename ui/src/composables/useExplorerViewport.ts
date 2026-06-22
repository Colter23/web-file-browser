import {nextTick, ref} from "vue";
import type {ComponentPublicInstance} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";

type ItemRefElement = Element | ComponentPublicInstance | null;

type ExplorerItemRefOptions = {
  itemRefs: Map<string, HTMLElement>;
  entryByPath: (path: string) => ExplorerEntry | undefined;
  observeEntry: (entry: ExplorerEntry, element: HTMLElement) => void;
  unobservePath: (path: string) => void;
}

const resolveElementRef = (element: ItemRefElement) => {
  if (element instanceof HTMLElement) return element;
  if (element && "$el" in element && element.$el instanceof HTMLElement) return element.$el;
  return null;
}

export const useExplorerViewport = () => {
  const viewportRef = ref<HTMLElement | null>(null);
  const itemRefs = new Map<string, HTMLElement>();

  const focusViewport = () => {
    viewportRef.value?.focus({preventScroll: true});
  }

  const getScrollTop = () => viewportRef.value?.scrollTop ?? 0;

  const setScrollTop = async (scrollTop: number) => {
    await nextTick();
    if (!viewportRef.value) return;
    viewportRef.value.scrollTop = Math.max(0, scrollTop);
  }

  const currentColumns = (entries: ExplorerEntry[]) => {
    if (!viewportRef.value) return 1;
    const first = entries[0] ? itemRefs.get(entries[0].path) : null;
    if (!first) return 1;
    const firstTop = Math.round(first.getBoundingClientRect().top);
    let columns = 0;
    for (const entry of entries) {
      const element = itemRefs.get(entry.path);
      if (!element) break;
      if (Math.abs(Math.round(element.getBoundingClientRect().top) - firstTop) > 2) break;
      columns += 1;
    }
    return Math.max(1, columns);
  }

  const currentPageStep = (entries: ExplorerEntry[], columns: number) => {
    const viewport = viewportRef.value;
    const first = entries[0] ? itemRefs.get(entries[0].path) : null;
    if (!viewport || !first) return Math.max(1, columns * 5);
    const rowHeight = Math.max(1, first.getBoundingClientRect().height);
    const visibleRows = Math.max(1, Math.floor(viewport.clientHeight / rowHeight) - 1);
    return Math.max(1, visibleRows * columns);
  }

  const entryDomId = (path: string) => `explorer-entry-${encodeURIComponent(path).replace(/[^a-zA-Z0-9_-]/g, "-")}`;

  const isViewportActive = () => Boolean(viewportRef.value?.contains(document.activeElement));

  const viewportHeight = () => viewportRef.value?.clientHeight ?? 0;

  const clearItemRefs = () => {
    itemRefs.clear();
  }

  return {
    viewportRef,
    itemRefs,
    focusViewport,
    getScrollTop,
    setScrollTop,
    currentColumns,
    currentPageStep,
    entryDomId,
    isViewportActive,
    viewportHeight,
    clearItemRefs
  };
}

export const useExplorerItemRefs = ({
  itemRefs,
  entryByPath,
  observeEntry,
  unobservePath
}: ExplorerItemRefOptions) => {
  const setItemRef = (path: string, element: ItemRefElement) => {
    const target = resolveElementRef(element);
    const current = itemRefs.get(path);
    if (current && current !== target) unobservePath(path);
    if (target) {
      itemRefs.set(path, target);
      const entry = entryByPath(path);
      if (entry) observeEntry(entry, target);
    } else {
      itemRefs.delete(path);
    }
  }

  return {setItemRef};
}
