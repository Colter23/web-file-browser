import type {
    CreateFavoriteRequest,
    FavoriteItem,
    ReorderFavoriteItem,
    UpdateFavoriteRequest
} from "../class.ts";
import network from "../network";

export type ListFavoritesOptions = {
    check?: boolean;
}

const favoriteUrl = (id: string) => `/api/favorites/${encodeURIComponent(id)}`;

export const listFavorites = async (options: ListFavoritesOptions = {}): Promise<FavoriteItem[]> => {
    return (await network.get("/api/favorites", {
        params: options.check ? {check: true} : undefined
    })).data;
}

export const createFavorite = async (request: CreateFavoriteRequest): Promise<FavoriteItem> => {
    return (await network.post("/api/favorites", request)).data;
}

export const updateFavorite = async (id: string, request: UpdateFavoriteRequest): Promise<FavoriteItem> => {
    return (await network.patch(favoriteUrl(id), request)).data;
}

export const reorderFavorites = async (items: ReorderFavoriteItem[]): Promise<void> => {
    await network.post("/api/favorites/reorder", {items});
}

export const deleteFavorite = async (id: string): Promise<void> => {
    await network.delete(favoriteUrl(id));
}
