import {computed} from "vue";
import type {Ref} from "vue";
import type {DirSortKey, DirSortOrder} from "../class.ts";
import {useFileStore} from "../store";
import type {ExplorerSelectionSnapshot} from "./useExplorerSelection.ts";
import {useExplorerViewDensity} from "./useExplorerViewDensity.ts";

type ExplorerPresentationOptions = {
  loading: Ref<boolean>;
  markStale: () => void;
  isFolderSource: () => boolean;
  loadFolder: (path?: string) => Promise<unknown>;
  captureSelectionSnapshot: () => ExplorerSelectionSnapshot;
  restoreSelectionSnapshot: (snapshot: ExplorerSelectionSnapshot) => Promise<boolean>;
  focusViewport: () => void;
  observePendingThumbnails: () => void;
  viewportHeight: () => number;
}

export const useExplorerPresentation = ({
  loading,
  markStale,
  isFolderSource,
  loadFolder,
  captureSelectionSnapshot,
  restoreSelectionSnapshot,
  focusViewport,
  observePendingThumbnails,
  viewportHeight
}: ExplorerPresentationOptions) => {
  const fileStore = useFileStore();

  const viewMode = computed(() => fileStore.viewMode);
  const viewModeClass = computed(() => `view-${fileStore.viewMode}`);
  const sortKey = computed(() => fileStore.sortKey);
  const sortOrder = computed(() => fileStore.sortOrder);
  const itemSizeClass = computed(() => ({
    "explorer-size-small": fileStore.iconSize === "small",
    "explorer-size-medium": fileStore.iconSize === "medium",
    "explorer-size-large": fileStore.iconSize === "large"
  }));

  const changeSort = async (key: DirSortKey) => {
    if (loading.value) return;
    const snapshot = captureSelectionSnapshot();
    fileStore.setSort(key);
    if (!isFolderSource()) {
      await restoreSelectionSnapshot(snapshot);
      focusViewport();
      return;
    }
    markStale();
    if (await loadFolder(fileStore.currentPath || "/")) await restoreSelectionSnapshot(snapshot);
  }

  const changeSortOrder = async (order: DirSortOrder) => {
    if (loading.value || fileStore.sortOrder === order) return;
    const snapshot = captureSelectionSnapshot();
    fileStore.setSort(fileStore.sortKey, order);
    if (!isFolderSource()) {
      await restoreSelectionSnapshot(snapshot);
      focusViewport();
      return;
    }
    markStale();
    if (await loadFolder(fileStore.currentPath || "/")) await restoreSelectionSnapshot(snapshot);
  }

  const {
    handleViewportWheel,
  } = useExplorerViewDensity({
    focusViewport,
    observePendingThumbnails,
    viewportHeight
  });

  return {
    viewMode,
    sortKey,
    sortOrder,
    viewModeClass,
    itemSizeClass,
    changeSort,
    changeSortOrder,
    handleViewportWheel,
  };
}
