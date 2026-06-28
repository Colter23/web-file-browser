import {computed, ref} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {CreateFavoriteRequest, FavoriteItem, ReorderFavoriteItem, UpdateFavoriteRequest} from "../class.ts";
import {isApiError} from "../network";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {useI18n} from "../i18n";
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
  const {t} = useI18n();
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
      showError(error, t("favorite.loadFailed"));
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
      showNotice(t("favorite.homeUnsupported"), "warning", t("favorite.title"));
      return null;
    }

    const existing = favoriteByPath(normalized);
    if (existing) {
      showNotice(t("favorite.alreadyExists"), "info", t("favorite.title"));
      return existing;
    }

    try {
      const item = await createFavorite({
        path: normalized,
        ...(name ? {name} : {})
      });
      replaceFavorite(item);
      showNotice(t("favorite.added"), "success", t("favorite.title"));
      return item;
    } catch (error) {
      if (isApiError(error) && error.status === 409) {
        await loadFavorites();
        showNotice(t("favorite.alreadyExists"), "info", t("favorite.title"));
        return favoriteByPath(normalized);
      }
      showError(error, t("favorite.addFailed"));
      return null;
    }
  }

  const removeFavorite = async (favoriteOrPath: FavoriteItem | string) => {
    const item = typeof favoriteOrPath === "string" ? favoriteByPath(favoriteOrPath) : favoriteOrPath;
    if (!item) {
      showNotice(t("favorite.notFound"), "warning", t("favorite.title"));
      return false;
    }

    try {
      await deleteFavorite(item.id);
      favorites.value = favorites.value.filter(favorite => favorite.id !== item.id);
      showNotice(t("favorite.removed"), "success", t("favorite.title"));
      return true;
    } catch (error) {
      showError(error, t("favorite.removeFailed"));
      return false;
    }
  }

  const renameFavorite = async (favorite: FavoriteItem, name: string) => {
    const nextName = name.trim();
    if (!nextName) {
      showNotice(t("favorite.nameRequired"), "warning", t("favorite.title"));
      return false;
    }
    if (nextName === favorite.name) return true;

    try {
      const item = await updateFavorite(favorite.id, {name: nextName});
      replaceFavorite(item);
      showNotice(t("favorite.renamed"), "success", t("favorite.title"));
      return true;
    } catch (error) {
      showError(error, t("favorite.renameFailed"), t("favorite.title"));
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
      showError(error, t("favorite.reorderFailed"), t("favorite.title"));
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
