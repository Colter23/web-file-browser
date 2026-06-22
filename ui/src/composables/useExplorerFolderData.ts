import {computed, nextTick, ref} from "vue";
import type {Ref} from "vue";
import type {DirSortKey, FolderData, FolderQueryParams} from "../class.ts";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {getFolderData} from "../network/file-api.ts";
import {useFileStore} from "../store";

type FolderLoadLifecycle = {
  resetBeforeLoad?: () => void;
  clearSelection?: () => void;
  afterRender?: () => void;
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

const mergeFolderData = (current: FolderData, next: FolderData): FolderData => ({
  ...next,
  folder: [...(current.folder ?? []), ...(next.folder ?? [])],
  file: [...(current.file ?? []), ...(next.file ?? [])]
})

export const useExplorerFolderData = ({filterText, viewportRef}: ExplorerFolderDataOptions) => {
  const fileStore = useFileStore();
  const folderData = ref<FolderData>({path: "/", folder: [], file: []});
  const loading = ref(false);
  const loadingMore = ref(false);
  const message = ref("");
  const loadedSignature = ref("");

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

  const filterKeyword = computed(() => filterText().trim());

  const entries = computed<ExplorerEntry[]>(() => {
    const keyword = filterKeyword.value.toLowerCase();
    if (!keyword) return allEntries.value;
    return allEntries.value.filter(entry => entry.name.toLowerCase().includes(keyword));
  });

  const folderRequestSignature = (path: string = fileStore.currentPath || "/") => {
    return `${path}|${fileStore.sortKey}|${fileStore.sortOrder}`;
  }

  const folderQuery = (offset = 0): FolderQueryParams => {
    const needsFullDetail = fileStore.sortKey !== "name";
    return {
      offset,
      limit: pageSize,
      detail: needsFullDetail ? "full" as const : "basic" as const,
      sort: fileStore.sortKey as DirSortKey,
      order: fileStore.sortOrder
    };
  }

  const scheduleAutoLoadMore = (lifecycle: FolderLoadLifecycle) => {
    window.requestAnimationFrame(() => maybeLoadMoreOnScroll(lifecycle));
  }

  const loadFolder = async (path: string = fileStore.currentPath || "/", lifecycle: FolderLoadLifecycle = {}) => {
    loading.value = true;
    message.value = "";
    let loaded = false;
    lifecycle.resetBeforeLoad?.();
    try {
      const data = normalizeFolderData(await getFolderData(path, folderQuery()));
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

  const loadMore = async (lifecycle: FolderLoadLifecycle = {}) => {
    if (loading.value || loadingMore.value || !folderData.value.hasMore) return false;
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
    if (!viewport || filterKeyword.value || loading.value || loadingMore.value || !folderData.value.hasMore) return;
    const distanceToBottom = viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight;
    if (distanceToBottom <= autoLoadMoreDistance) void loadMore(lifecycle);
  }

  const isLoadedFor = (path: string) => loadedSignature.value === folderRequestSignature(path);

  const markStale = () => {
    loadedSignature.value = "";
  }

  return {
    folderData,
    loading,
    loadingMore,
    message,
    allEntries,
    filterKeyword,
    entries,
    loadFolder,
    loadMore,
    maybeLoadMoreOnScroll,
    isLoadedFor,
    markStale
  };
}
