import type {SearchResponse, SearchResult} from "../class";
import network from "../network";

export type SearchQueryParams = {
    q: string;
    mount?: string;
    type?: "file" | "folder";
    offset?: number;
    limit?: number;
}

export const searchEntries = async (params: SearchQueryParams): Promise<SearchResponse> => {
    return (await network.get("/api/search", {params})).data
}

export const getRecentEntries = async (limit = 50): Promise<SearchResult[]> => {
    return (await network.get("/api/recent", {params: {limit}})).data
}
