import {computed, nextTick, ref} from "vue";
import type {Ref} from "vue";
import type {DirDetail, DirSortKey, FileInfo, FolderData, FolderQueryParams, SearchResult} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {getFolderData} from "../network/file-api.ts";
import {getRecentEntries, searchEntries} from "../network/search-api.ts";
import {useFileStore} from "../store";

export type ExplorerDataSourceMode = "folder" | "search" | "recent";

type FolderLoadLifecycle = {
  resetBeforeLoad?: () => void;
  clearSelection?: () => void;
  afterRender?: () => void;
}

type FolderLoadOptions = FolderLoadLifecycle & {
  forceRefresh?: boolean;
}

type SearchLoadOptions = FolderLoadLifecycle & {
  mount?: string;
}

type ExplorerFolderDataOptions = {
  filterText: () => string;
  viewportRef: Ref<HTMLElement | null>;
}

const pageSize = 200;
const autoLoadMoreDistance = 360;

const normalizeFolderData = (data: FolderData): FolderData => ({
  path: data.path || "/",
  folder: data.folder ?? [],
  file: data.file ?? [],
  folderTotal: data.folderTotal,
  fileTotal: data.fileTotal,
  offset: data.offset,
  limit: data.limit,
  hasMore: data.hasMore
})

const searchResultFileInfo = (item: SearchResult): FileInfo => ({
  path: item.path,
  name: item.name,
  size: item.size ?? 0,
  extension: item.extension ?? "",
  modified: item.modified ?? "",
  type: item.type
})

const normalizeSearchResultFolderData = (items: SearchResult[], path: string): FolderData => ({
  path,
  folder: items
      .filter(item => item.type === "folder")
      .map(item => ({
        path: item.path,
        name: item.name,
        modified: item.modified ?? "",
        type: "folder"
      })),
  file: items
      .filter(item => item.type !== "folder")
      .map(searchResultFileInfo)
})

const mergeFolderData = (current: FolderData, next: FolderData): FolderData => ({
  ...next,
  folder: [...(current.folder ?? []), ...(next.folder ?? [])],
  file: [...(current.file ?? []), ...(next.file ?? [])]
})

const compareEntryText = (left?: string, right?: string) => {
  return (left ?? "").localeCompare(right ?? "", "zh-CN", {numeric: true, sensitivity: "base"});
}

const compareEntryDate = (left?: string, right?: string) => {
  const parseDate = (value?: string) => {
    if (!value) return 0;
    const trimmed = value.trim();
    if (/^\d+$/.test(trimmed)) {
      const numeric = Number(trimmed);
      if (Number.isFinite(numeric)) return numeric < 10_000_000_000 ? numeric * 1000 : numeric;
    }
    const time = Date.parse(value);
    return Number.isNaN(time) ? 0 : time;
  }
  return parseDate(left) - parseDate(right);
}

const compareResultEntries = (left: ExplorerEntry, right: ExplorerEntry, key: DirSortKey) => {
  if (left.type !== right.type) return left.type === "folder" ? -1 : 1;
  if (key === "modified") return compareEntryDate(left.modified, right.modified) || compareEntryText(left.name, right.name);
  if (key === "size") return (left.size ?? 0) - (right.size ?? 0) || compareEntryText(left.name, right.name);
  return compareEntryText(left.name, right.name);
}

export const useExplorerFolderData = ({filterText, viewportRef}: ExplorerFolderDataOptions) => {
  const fileStore = useFileStore();
  const folderData = ref<FolderData>({path: "/", folder: [], file: []});
  const loading = ref(false);
  const loadingMore = ref(false);
  const message = ref("");
  const loadedSignature = ref("");
  const sourceMode = ref<ExplorerDataSourceMode>("folder");
  const sourceTitle = ref("当前文件夹");
  const resultTotal = ref<number | null>(null);

  const allEntries = computed<ExplorerEntry[]>(() => [
    ...(folderData.value.folder ?? []).map(folder => ({
      type: "folder" as const,
      name: folder.name,
      path: folder.path,
      modified: folder.modified
    })),
    ...(folderData.value.file ?? []).map(file => ({
      type: "file" as const,
      name: file.name,
      path: file.path,
      modified: file.modified,
      size: file.size,
      extension: file.extension,
      file
    }))
  ]);

  const filterKeyword = computed(() => sourceMode.value === "folder" ? filterText().trim() : "");
  const currentDetail = computed<DirDetail>(() => {
    const viewNeedsMetadata = fileStore.viewMode === "details" || fileStore.viewMode === "tiles";
    return fileStore.sortKey !== "name" || viewNeedsMetadata ? "full" : "basic";
  });

  const entries = computed<ExplorerEntry[]>(() => {
    const keyword = filterKeyword.value.toLowerCase();
    const visibleEntries = keyword
        ? allEntries.value.filter(entry => entry.name.toLowerCase().includes(keyword))
        : allEntries.value;
    if (sourceMode.value === "folder") return visibleEntries;
    const direction = fileStore.sortOrder === "desc" ? -1 : 1;
    return [...visibleEntries].sort((left, right) => compareResultEntries(left, right, fileStore.sortKey) * direction);
  });

  const folderRequestSignature = (path: string = fileStore.currentPath || "/", detail: DirDetail = currentDetail.value) => {
    return `${path}|${fileStore.sortKey}|${fileStore.sortOrder}|${detail}`;
  }

  const folderQuery = (offset = 0): FolderQueryParams => {
    return {
      offset,
      limit: pageSize,
      detail: currentDetail.value,
      sort: fileStore.sortKey as DirSortKey,
      order: fileStore.sortOrder
    };
  }

  const scheduleAutoLoadMore = (lifecycle: FolderLoadLifecycle) => {
    window.requestAnimationFrame(() => maybeLoadMoreOnScroll(lifecycle));
  }

  const resetForLoad = (lifecycle: FolderLoadLifecycle) => {
    message.value = "";
    resultTotal.value = null;
    lifecycle.resetBeforeLoad?.();
  }

  const loadFolder = async (path: string = fileStore.currentPath || "/", lifecycle: FolderLoadOptions = {}) => {
    loading.value = true;
    let loaded = false;
    sourceMode.value = "folder";
    sourceTitle.value = "当前文件夹";
    resetForLoad(lifecycle);
    try {
      const data = normalizeFolderData(await getFolderData(path, folderQuery(), {forceRefresh: lifecycle.forceRefresh}));
      fileStore.saveFolderData(data);
      folderData.value = data;
      loadedSignature.value = folderRequestSignature(data.path || path);
      lifecycle.clearSelection?.();
      fileStore.setCurrentPath(data.path);
      fileStore.closeEditor();
      await nextTick();
      lifecycle.afterRender?.();
      loaded = true;
    } catch (error) {
      message.value = error instanceof Error ? error.message : "加载目录失败";
    } finally {
      loading.value = false;
      if (loaded) scheduleAutoLoadMore(lifecycle);
    }
    return loaded;
  }

  const loadSearch = async (query: string, lifecycle: SearchLoadOptions = {}) => {
    const keyword = query.trim();
    if (!keyword) return false;
    loading.value = true;
    loadingMore.value = false;
    sourceMode.value = "search";
    sourceTitle.value = `搜索：“${keyword}”`;
    resetForLoad(lifecycle);
    let loaded = false;
    try {
      const data = await searchEntries({
        q: keyword,
        mount: lifecycle.mount,
        offset: 0,
        limit: pageSize
      });
      folderData.value = normalizeSearchResultFolderData(data.items ?? [], fileStore.currentPath || "/");
      resultTotal.value = data.total ?? allEntries.value.length;
      loadedSignature.value = `search|${keyword}|${lifecycle.mount ?? ""}`;
      lifecycle.clearSelection?.();
      fileStore.closeEditor();
      await nextTick();
      lifecycle.afterRender?.();
      loaded = true;
    } catch (error) {
      message.value = error instanceof Error ? error.message : "搜索失败";
    } finally {
      loading.value = false;
    }
    return loaded;
  }

  const loadRecent = async (lifecycle: FolderLoadLifecycle = {}) => {
    loading.value = true;
    loadingMore.value = false;
    sourceMode.value = "recent";
    sourceTitle.value = "最近文件";
    resetForLoad(lifecycle);
    let loaded = false;
    try {
      const items = await getRecentEntries(50);
      folderData.value = normalizeSearchResultFolderData(items ?? [], fileStore.currentPath || "/");
      resultTotal.value = allEntries.value.length;
      loadedSignature.value = "recent";
      lifecycle.clearSelection?.();
      fileStore.closeEditor();
      await nextTick();
      lifecycle.afterRender?.();
      loaded = true;
    } catch (error) {
      message.value = error instanceof Error ? error.message : "加载最近文件失败";
    } finally {
      loading.value = false;
    }
    return loaded;
  }

  const loadMore = async (lifecycle: FolderLoadLifecycle = {}) => {
    if (sourceMode.value !== "folder" || loading.value || loadingMore.value || !folderData.value.hasMore) return false;
    loadingMore.value = true;
    message.value = "";
    let loaded = false;
    try {
      const current = folderData.value;
      const offset = (current.offset ?? 0) + (current.limit ?? allEntries.value.length);
      const data = normalizeFolderData(await getFolderData(current.path || fileStore.currentPath || "/", folderQuery(offset)));
      const merged = normalizeFolderData(mergeFolderData(current, data));
      fileStore.saveFolderData(merged);
      folderData.value = merged;
      loadedSignature.value = folderRequestSignature(merged.path || current.path);
      await nextTick();
      lifecycle.afterRender?.();
      loaded = true;
    } catch (error) {
      message.value = error instanceof Error ? error.message : "加载更多失败";
    } finally {
      loadingMore.value = false;
      if (loaded) scheduleAutoLoadMore(lifecycle);
    }
    return loaded;
  }

  const maybeLoadMoreOnScroll = (lifecycle: FolderLoadLifecycle = {}) => {
    const viewport = viewportRef.value;
    if (sourceMode.value !== "folder" || !viewport || filterKeyword.value || loading.value || loadingMore.value || !folderData.value.hasMore) return;
    const distanceToBottom = viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight;
    if (distanceToBottom <= autoLoadMoreDistance) void loadMore(lifecycle);
  }

  const isLoadedFor = (path: string) => {
    if (sourceMode.value !== "folder") return false;
    if (loadedSignature.value === folderRequestSignature(path)) return true;
    return currentDetail.value === "basic" && loadedSignature.value === folderRequestSignature(path, "full");
  }

  const markStale = () => {
    loadedSignature.value = "";
  }

  return {
    folderData,
    loading,
    loadingMore,
    message,
    sourceMode,
    sourceTitle,
    resultTotal,
    allEntries,
    filterKeyword,
    entries,
    loadFolder,
    loadSearch,
    loadRecent,
    loadMore,
    maybeLoadMoreOnScroll,
    isLoadedFor,
    markStale
  };
}
