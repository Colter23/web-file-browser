import {computed} from "vue";
import type {Ref} from "vue";
import type {DirSortKey, DirSortOrder} from "../class.ts";
import {useFileStore} from "../store";
import type {ExplorerSelectionSnapshot} from "./useExplorerSelection.ts";
import {useExplorerViewDensity} from "./useExplorerViewDensity.ts";

type ExplorerPresentationOptions = {
  loading: Ref<boolean>;
  markStale: () => void;
  loadFolder: (path?: string) => Promise<unknown>;
  captureSelectionSnapshot: () => ExplorerSelectionSnapshot;
  restoreSelectionSnapshot: (snapshot: ExplorerSelectionSnapshot) => Promise<boolean>;
  focusViewport: () => void;
  observePendingThumbnails: () => void;
  viewportHeight: () => number;
}

const sortOptions: {key: DirSortKey; label: string}[] = [
  {key: "name", label: "名称"},
  {key: "modified", label: "修改"},
  {key: "size", label: "大小"}
];

const iconSizeLabel = {
  small: "小图标",
  medium: "中图标",
  large: "大图标"
};

export const useExplorerPresentation = ({
  loading,
  markStale,
  loadFolder,
  captureSelectionSnapshot,
  restoreSelectionSnapshot,
  focusViewport,
  observePendingThumbnails,
  viewportHeight
}: ExplorerPresentationOptions) => {
  const fileStore = useFileStore();

  const viewMode = computed(() => fileStore.viewMode);
  const sortKey = computed(() => fileStore.sortKey);
  const sortOrder = computed(() => fileStore.sortOrder);
  const itemSizeClass = computed(() => ({
    small: fileStore.iconSize === "small",
    medium: fileStore.iconSize === "medium",
    large: fileStore.iconSize === "large"
  }));
  const iconSizeText = computed(() => iconSizeLabel[fileStore.iconSize]);
  const sortText = computed(() => {
    const keyText = sortOptions.find(option => option.key === fileStore.sortKey)?.label ?? "名称";
    const orderText = fileStore.sortOrder === "asc" ? "升序" : "降序";
    return `${keyText} ${orderText}`;
  });
  const nextSortOrder = computed<DirSortOrder>(() => fileStore.sortOrder === "asc" ? "desc" : "asc");

  const changeSort = async (key: DirSortKey) => {
    if (loading.value) return;
    const snapshot = captureSelectionSnapshot();
    fileStore.setSort(key);
    markStale();
    if (await loadFolder(fileStore.currentPath || "/")) await restoreSelectionSnapshot(snapshot);
  }

  const changeSortOrder = async (order: DirSortOrder) => {
    if (loading.value || fileStore.sortOrder === order) return;
    const snapshot = captureSelectionSnapshot();
    fileStore.setSort(fileStore.sortKey, order);
    markStale();
    if (await loadFolder(fileStore.currentPath || "/")) await restoreSelectionSnapshot(snapshot);
  }

  const {
    setViewMode,
    handleViewportWheel,
    cycleIconSize
  } = useExplorerViewDensity({
    focusViewport,
    observePendingThumbnails,
    viewportHeight
  });

  return {
    viewMode,
    sortKey,
    sortOrder,
    sortOptions,
    itemSizeClass,
    iconSizeText,
    sortText,
    nextSortOrder,
    changeSort,
    changeSortOrder,
    setViewMode,
    handleViewportWheel,
    cycleIconSize
  };
}
