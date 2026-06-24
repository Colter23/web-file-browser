import {computed, ref} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {CreateFavoriteRequest, FavoriteItem, ReorderFavoriteItem, UpdateFavoriteRequest} from "../class.ts";
import {isApiError} from "../network";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {normalizePathText} from "../utils/file-path.ts";

type ListFavoritesOptions = {
  check?: boolean;
}

type UseFavoritesOptions = {
  listFavorites: (options?: ListFavoritesOptions) => Promise<FavoriteItem[]>;
  createFavorite: (request: CreateFavoriteRequest) => Promise<FavoriteItem>;
  updateFavorite: (id: string, request: UpdateFavoriteRequest) => Promise<FavoriteItem>;
  reorderFavorites: (items: ReorderFavoriteItem[]) => Promise<void>;
  deleteFavorite: (id: string) => Promise<void>;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
  showError: (error: unknown, fallback: string, title?: string) => void;
}

type UseFavoritesReturn = {
  favorites: Ref<FavoriteItem[]>;
  favoritesLoading: Ref<boolean>;
  favoritePaths: ComputedRef<string[]>;
  loadFavorites: (options?: ListFavoritesOptions) => Promise<void>;
  favoriteByPath: (path: string) => FavoriteItem | null;
  addFavorite: (path: string, name?: string) => Promise<FavoriteItem | null>;
  renameFavorite: (favorite: FavoriteItem, name: string) => Promise<boolean>;
  reorderFavorite: (source: FavoriteItem, target: FavoriteItem, placement: "before" | "after") => Promise<boolean>;
  removeFavorite: (favoriteOrPath: FavoriteItem | string) => Promise<boolean>;
  toggleFavoritePath: (path: string, name?: string) => Promise<void>;
}

const sortFavorites = (items: FavoriteItem[]) => {
  return [...items].sort((left, right) => {
    const orderDiff = left.order - right.order;
    if (orderDiff !== 0) return orderDiff;
    const createdDiff = left.createdAt.localeCompare(right.createdAt);
    return createdDiff !== 0 ? createdDiff : left.id.localeCompare(right.id);
  });
}

export const useFavorites = ({
  listFavorites,
  createFavorite,
  updateFavorite,
  reorderFavorites,
  deleteFavorite,
  showNotice,
  showError
}: UseFavoritesOptions): UseFavoritesReturn => {
  const favorites = ref<FavoriteItem[]>([]);
  const favoritesLoading = ref(false);

  const favoritePaths = computed(() => favorites.value.map(item => normalizePathText(item.path)));

  const favoriteByPath = (path: string) => {
    const normalized = normalizePathText(path);
    return favorites.value.find(item => normalizePathText(item.path) === normalized) ?? null;
  }

  const loadFavorites = async (options: ListFavoritesOptions = {}) => {
    favoritesLoading.value = true;
    try {
      favorites.value = sortFavorites(await listFavorites(options));
    } catch (error) {
      showError(error, "加载收藏夹失败");
    } finally {
      favoritesLoading.value = false;
    }
  }

  const replaceFavorite = (item: FavoriteItem) => {
    const next = favorites.value.filter(favorite => favorite.id !== item.id);
    next.push(item);
    favorites.value = sortFavorites(next);
  }

  const withDenseOrder = (items: FavoriteItem[]) => {
    return items.map((item, index) => ({
      ...item,
      order: (index + 1) * 10
    }));
  }

  const addFavorite = async (path: string, name?: string) => {
    const normalized = normalizePathText(path);
    if (normalized === "/") {
      showNotice("主页不能添加到收藏夹", "warning", "收藏夹");
      return null;
    }

    const existing = favoriteByPath(normalized);
    if (existing) {
      showNotice("该文件夹已经在收藏夹中", "info", "收藏夹");
      return existing;
    }

    try {
      const item = await createFavorite({
        path: normalized,
        ...(name ? {name} : {})
      });
      replaceFavorite(item);
      showNotice("已添加到收藏夹", "success", "收藏夹");
      return item;
    } catch (error) {
      if (isApiError(error) && error.status === 409) {
        await loadFavorites();
        showNotice("该文件夹已经在收藏夹中", "info", "收藏夹");
        return favoriteByPath(normalized);
      }
      showError(error, "添加收藏失败");
      return null;
    }
  }

  const removeFavorite = async (favoriteOrPath: FavoriteItem | string) => {
    const item = typeof favoriteOrPath === "string" ? favoriteByPath(favoriteOrPath) : favoriteOrPath;
    if (!item) {
      showNotice("未找到对应的收藏项", "warning", "收藏夹");
      return false;
    }

    try {
      await deleteFavorite(item.id);
      favorites.value = favorites.value.filter(favorite => favorite.id !== item.id);
      showNotice("已从收藏夹移除", "success", "收藏夹");
      return true;
    } catch (error) {
      showError(error, "移除收藏失败");
      return false;
    }
  }

  const renameFavorite = async (favorite: FavoriteItem, name: string) => {
    const nextName = name.trim();
    if (!nextName) {
      showNotice("收藏夹名称不能为空", "warning", "收藏夹");
      return false;
    }
    if (nextName === favorite.name) return true;

    try {
      const item = await updateFavorite(favorite.id, {name: nextName});
      replaceFavorite(item);
      showNotice("已重命名收藏项", "success", "收藏夹");
      return true;
    } catch (error) {
      showError(error, "重命名收藏项失败", "收藏夹");
      return false;
    }
  }

  const reorderFavorite = async (source: FavoriteItem, target: FavoriteItem, placement: "before" | "after") => {
    if (source.id === target.id) return true;
    const previous = sortFavorites(favorites.value);
    const sourceIndex = previous.findIndex(item => item.id === source.id);
    const targetIndex = previous.findIndex(item => item.id === target.id);
    if (sourceIndex < 0 || targetIndex < 0) return false;

    const next = [...previous];
    const [moved] = next.splice(sourceIndex, 1);
    const nextTargetIndex = next.findIndex(item => item.id === target.id);
    if (!moved || nextTargetIndex < 0) return false;

    next.splice(placement === "after" ? nextTargetIndex + 1 : nextTargetIndex, 0, moved);
    const ordered = withDenseOrder(next);
    favorites.value = ordered;

    try {
      await reorderFavorites(ordered.map(({id, order}) => ({id, order})));
      return true;
    } catch (error) {
      favorites.value = previous;
      showError(error, "调整收藏夹顺序失败", "收藏夹");
      return false;
    }
  }

  const toggleFavoritePath = async (path: string, name?: string) => {
    const existing = favoriteByPath(path);
    if (existing) {
      await removeFavorite(existing);
      return;
    }
    await addFavorite(path, name);
  }

  return {
    favorites,
    favoritesLoading,
    favoritePaths,
    loadFavorites,
    favoriteByPath,
    addFavorite,
    renameFavorite,
    reorderFavorite,
    removeFavorite,
    toggleFavoritePath
  };
}
