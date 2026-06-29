import type {ComputedRef, Ref} from "vue";
import {ref} from "vue";
import {fileContentUrl} from "../network/file-api.ts";

type ThumbnailEntry = {
  type: "folder" | "file";
  path: string;
  extension?: string;
}

type ThumbnailOptions<TEntry extends ThumbnailEntry> = {
  entries: ComputedRef<TEntry[]>;
  itemRefs: Map<string, HTMLElement>;
  viewportRef: Ref<HTMLElement | null>;
  active: ComputedRef<boolean>;
  isImageFile: (entry: TEntry) => boolean;
}

export const useExplorerThumbnails = <TEntry extends ThumbnailEntry>({
  entries,
  itemRefs,
  viewportRef,
  active,
  isImageFile
}: ThumbnailOptions<TEntry>) => {
  const visiblePaths = ref<Set<string>>(new Set());
  const failedPaths = ref<Set<string>>(new Set());
  let observer: IntersectionObserver | null = null;

  const addVisiblePath = (path: string) => {
    if (visiblePaths.value.has(path)) return;
    visiblePaths.value = new Set([...visiblePaths.value, path]);
  }

  const addFailedPath = (path: string) => {
    if (failedPaths.value.has(path)) return;
    failedPaths.value = new Set([...failedPaths.value, path]);
  }

  const createObserver = () => {
    if (observer || typeof IntersectionObserver === "undefined") return observer;
    observer = new IntersectionObserver(records => {
      records.forEach(record => {
        if (!record.isIntersecting) return;
        const path = (record.target as HTMLElement).dataset.thumbnailPath;
        if (!path) return;
        addVisiblePath(path);
        observer?.unobserve(record.target);
      });
    }, {
      root: viewportRef.value,
      rootMargin: "240px",
      threshold: 0.01
    });
    return observer;
  }

  const disconnectObserver = () => {
    observer?.disconnect();
    observer = null;
  }

  const clearState = () => {
    visiblePaths.value = new Set();
    failedPaths.value = new Set();
    disconnectObserver();
  }

  const unobserve = (path: string) => {
    const element = itemRefs.get(path);
    if (!element) return;
    observer?.unobserve(element);
    delete element.dataset.thumbnailPath;
  }

  const shouldLoad = (entry: TEntry) => {
    return active.value && isImageFile(entry) && visiblePaths.value.has(entry.path) && !failedPaths.value.has(entry.path);
  }

  const thumbnailUrl = (entry: TEntry) => fileContentUrl(entry.path);

  const observe = (entry: TEntry, element: HTMLElement) => {
    if (!active.value || !isImageFile(entry) || visiblePaths.value.has(entry.path) || failedPaths.value.has(entry.path)) return;
    if (typeof IntersectionObserver === "undefined") {
      addVisiblePath(entry.path);
      return;
    }
    element.dataset.thumbnailPath = entry.path;
    createObserver()?.observe(element);
  }

  const observePending = () => {
    if (!active.value) return;
    entries.value.forEach(entry => {
      const element = itemRefs.get(entry.path);
      if (element) observe(entry, element);
    });
  }

  const handleError = (entry: TEntry) => {
    addFailedPath(entry.path);
    unobserve(entry.path);
  }

  return {
    shouldLoad,
    thumbnailUrl,
    handleError,
    observe,
    observePending,
    unobserve,
    clearState,
    disconnectObserver
  };
}
