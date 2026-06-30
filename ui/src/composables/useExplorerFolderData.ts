import {computed, nextTick, ref, watch} from "vue";
import type {Ref} from "vue";
import type {DirDetail, DirEntryFilter, DirSortKey, FileInfo, FolderData, FolderQueryParams, SearchResult, SearchScope} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {useI18n} from "../i18n";
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
  type?: Exclude<DirEntryFilter, "all">;
  scope?: SearchScope;
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

const normalizePagingValue = (value: number | undefined, fallback: number) => {
  return Number.isFinite(value) ? value! : fallback;
}

const searchResultFileInfo = (item: SearchResult): FileInfo => ({
  path: item.path,
  name: item.name,
  size: item.size ?? 0,
  extension: item.extension ?? "",
  modified: item.modified ?? "",
  type: item.type
})

const normalizeSearchResultFolderData = (
    items: SearchResult[],
    path: string,
    paging: {offset?: number; limit?: number; total?: number} = {}
): FolderData => {
  const offset = normalizePagingValue(paging.offset, 0);
  const limit = normalizePagingValue(paging.limit, pageSize);
  const total = Number.isFinite(paging.total) ? paging.total! : undefined;
  return {
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
        .map(searchResultFileInfo),
    offset,
    limit,
    hasMore: total !== undefined ? offset + items.length < total : items.length >= limit
  };
}

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

const entryTypeKey = (entry: ExplorerEntry) => {
  if (entry.type === "folder") return "folder";
  return (entry.extension ?? "").toLowerCase();
}

const compareResultEntries = (left: ExplorerEntry, right: ExplorerEntry, key: DirSortKey) => {
  if (left.type !== right.type) return left.type === "folder" ? -1 : 1;
  if (key === "type") return compareEntryText(entryTypeKey(left), entryTypeKey(right)) || compareEntryText(left.name, right.name);
  if (key === "modified") return compareEntryDate(left.modified, right.modified) || compareEntryText(left.name, right.name);
  if (key === "size") return (left.size ?? 0) - (right.size ?? 0) || compareEntryText(left.name, right.name);
  return compareEntryText(left.name, right.name);
}

export const useExplorerFolderData = ({filterText, viewportRef}: ExplorerFolderDataOptions) => {
  const fileStore = useFileStore();
  const {locale, t} = useI18n();
  const folderData = ref<FolderData>({path: "/", folder: [], file: []});
  const loading = ref(false);
  const loadingMore = ref(false);
  const message = ref("");
  const loadedSignature = ref("");
  const sourceMode = ref<ExplorerDataSourceMode>("folder");
  const sourceTitle = ref(t("explorer.currentFolder"));
  const resultTotal = ref<number | null>(null);
  const searchContext = ref<{query: string; mount?: string; type?: Exclude<DirEntryFilter, "all">; scope: SearchScope} | null>(null);
  const searchKeyword = computed(() => sourceMode.value === "search" ? searchContext.value?.query ?? "" : "");

  const searchSourceTitle = () => {
    const context = searchContext.value;
    if (!context) return t("search.folder");
    const typeTitle = context.type === "file" ? t("search.files") : context.type === "folder" ? t("search.folders") : t("search.folder");
    const scopeTitle = context.scope === "all" ? t("search.scopeAll") : t("search.scopeMount");
    return t("search.title", {type: typeTitle, scope: scopeTitle, keyword: context.query});
  }

  const updateSourceTitle = () => {
    if (sourceMode.value === "search") {
      sourceTitle.value = searchSourceTitle();
      return;
    }
    if (sourceMode.value === "recent") {
      sourceTitle.value = t("explorer.recentFiles");
      return;
    }
    sourceTitle.value = t("explorer.currentFolder");
  }

  watch(locale, updateSourceTitle);

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
    const sortNeedsMetadata = fileStore.sortKey === "modified" || fileStore.sortKey === "size";
    return sortNeedsMetadata || viewNeedsMetadata ? "full" : "basic";
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
    sourceTitle.value = t("explorer.currentFolder");
    searchContext.value = null;
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
      message.value = error instanceof Error ? error.message : t("explorer.loadFailed");
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
    const scope = lifecycle.scope ?? (lifecycle.mount ? "mount" : "all");
    searchContext.value = {query: keyword, mount: lifecycle.mount, type: lifecycle.type, scope};
    sourceTitle.value = searchSourceTitle();
    resetForLoad(lifecycle);
    let loaded = false;
    try {
      const data = await searchEntries({
        q: keyword,
        mount: lifecycle.mount,
        type: lifecycle.type,
        offset: 0,
        limit: pageSize
      });
      folderData.value = normalizeSearchResultFolderData(data.items ?? [], fileStore.currentPath || "/", data);
      resultTotal.value = data.total ?? allEntries.value.length;
      loadedSignature.value = `search|${keyword}|${lifecycle.mount ?? ""}|${lifecycle.type ?? "all"}|${scope}`;
      lifecycle.clearSelection?.();
      fileStore.closeEditor();
      await nextTick();
      lifecycle.afterRender?.();
      loaded = true;
    } catch (error) {
      message.value = error instanceof Error ? error.message : t("explorer.searchFailed");
    } finally {
      loading.value = false;
    }
    return loaded;
  }

  const loadRecent = async (lifecycle: FolderLoadLifecycle = {}) => {
    loading.value = true;
    loadingMore.value = false;
    sourceMode.value = "recent";
    sourceTitle.value = t("explorer.recentFiles");
    searchContext.value = null;
    resetForLoad(lifecycle);
    let loaded = false;
    try {
      const items = await getRecentEntries(50);
      folderData.value = normalizeSearchResultFolderData(items ?? [], fileStore.currentPath || "/", {
        offset: 0,
        limit: items?.length ?? 0,
        total: items?.length ?? 0
      });
      resultTotal.value = allEntries.value.length;
      loadedSignature.value = "recent";
      lifecycle.clearSelection?.();
      fileStore.closeEditor();
      await nextTick();
      lifecycle.afterRender?.();
      loaded = true;
    } catch (error) {
      message.value = error instanceof Error ? error.message : t("explorer.loadRecentFailed");
    } finally {
      loading.value = false;
    }
    return loaded;
  }

  const loadMoreSearch = async (lifecycle: FolderLoadLifecycle = {}) => {
    if (!searchContext.value || loading.value || loadingMore.value || !folderData.value.hasMore) return false;
    loadingMore.value = true;
    message.value = "";
    let loaded = false;
    try {
      const current = folderData.value;
      const offset = (current.offset ?? 0) + (current.limit ?? allEntries.value.length);
      const data = await searchEntries({
        q: searchContext.value.query,
        mount: searchContext.value.mount,
        type: searchContext.value.type,
        offset,
        limit: pageSize
      });
      const next = normalizeSearchResultFolderData(data.items ?? [], current.path || fileStore.currentPath || "/", data);
      const merged = normalizeFolderData(mergeFolderData(current, next));
      folderData.value = merged;
      resultTotal.value = data.total ?? resultTotal.value ?? allEntries.value.length;
      loadedSignature.value = `search|${searchContext.value.query}|${searchContext.value.mount ?? ""}|${searchContext.value.type ?? "all"}|${searchContext.value.scope}|${merged.offset ?? 0}`;
      await nextTick();
      lifecycle.afterRender?.();
      loaded = true;
    } catch (error) {
      message.value = error instanceof Error ? error.message : t("explorer.loadMoreSearchFailed");
    } finally {
      loadingMore.value = false;
    }
    return loaded;
  }

  const loadMore = async (lifecycle: FolderLoadLifecycle = {}) => {
    if (sourceMode.value === "search") return loadMoreSearch(lifecycle);
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
      message.value = error instanceof Error ? error.message : t("explorer.loadMoreFailed");
    } finally {
      loadingMore.value = false;
      if (loaded) scheduleAutoLoadMore(lifecycle);
    }
    return loaded;
  }

  const maybeLoadMoreOnScroll = (lifecycle: FolderLoadLifecycle = {}) => {
    const viewport = viewportRef.value;
    if (sourceMode.value === "recent" || !viewport || filterKeyword.value || loading.value || loadingMore.value || !folderData.value.hasMore) return;
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
    searchKeyword,
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
