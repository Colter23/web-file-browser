import network from "../network";
import {
    FileContentResponse,
    FileOperationResponse,
    FolderData,
    FolderQueryParams,
    SaveFileResponse,
    UploadResponse
} from "../class";
import config from "../config";
import {parentPath} from "../utils/file-path.ts";

export type FolderRequestOptions = {
    forceRefresh?: boolean;
}

type FolderCacheEntry = {
    data: FolderData;
    etag?: string;
    lastModified?: string;
}

const folderCacheMaxSize = 120;
const folderResponseCache = new Map<string, FolderCacheEntry>();

const encodeVirtualPath = (path: string = ""): string => {
    return path
        .split("/")
        .filter(part => part.length > 0)
        .map(encodeURIComponent)
        .join("/")
}

const pathUrl = (base: string, path: string = ""): string => {
    const encoded = encodeVirtualPath(path)
    return encoded ? `${base}/${encoded}` : base
}

const normalizeCachePath = (path: string = ""): string => {
    const parts = path.split("/").filter(part => part.length > 0);
    return parts.length ? `/${parts.join("/")}` : "/";
}

const folderCacheKey = (path: string = "", params: FolderQueryParams = {}): string => {
    const keys: (keyof FolderQueryParams)[] = [
        "offset",
        "limit",
        "detail",
        "sort",
        "order",
        "type",
        "includeHidden",
        "includeTotal"
    ];
    const query = keys
        .filter(key => params[key] !== undefined)
        .map(key => `${key}=${String(params[key])}`)
        .join("&");
    return `${normalizeCachePath(path)}?${query}`;
}

const clearFolderCache = (path: string = "") => {
    const prefix = `${normalizeCachePath(path)}?`;
    Array.from(folderResponseCache.keys()).forEach(key => {
        if (key.startsWith(prefix)) folderResponseCache.delete(key);
    });
}

const folderCachePathAncestors = (path: string = ""): string[] => {
    const normalized = normalizeCachePath(path);
    const parts = normalized.split("/").filter(Boolean);
    const paths = ["/"];
    let current = "";
    parts.forEach(part => {
        current = `${current}/${part}`;
        paths.push(current);
    });
    return paths;
}

export const invalidateFolderDataCache = (
    paths: string | string[] = "/",
    options: {includeAncestors?: boolean} = {}
) => {
    const normalizedPaths = Array.isArray(paths) ? paths : [paths];
    const candidates = normalizedPaths.flatMap(path => {
        return options.includeAncestors ? folderCachePathAncestors(path) : [normalizeCachePath(path)];
    });
    Array.from(new Set(candidates)).forEach(clearFolderCache);
}

const cloneFolderData = (data: FolderData): FolderData => ({
    ...data,
    folder: data.folder?.map(folder => ({...folder})),
    file: data.file?.map(file => ({...file}))
})

const headerValue = (headers: Record<string, unknown>, name: string): string | undefined => {
    if ("get" in headers && typeof headers.get === "function") {
        const header = headers.get(name);
        if (header) return String(header);
    }
    const value = headers[name] ?? headers[name.toLowerCase()];
    if (Array.isArray(value)) return value[0] ? String(value[0]) : undefined;
    return value ? String(value) : undefined;
}

const saveFolderCache = (key: string, data: FolderData, headers: Record<string, unknown>) => {
    const etag = headerValue(headers, "etag");
    const lastModified = headerValue(headers, "last-modified");
    if (!etag && !lastModified) {
        folderResponseCache.delete(key);
        return;
    }
    folderResponseCache.set(key, {
        data: cloneFolderData(data),
        etag,
        lastModified
    });
    if (folderResponseCache.size <= folderCacheMaxSize) return;
    const oldestKey = folderResponseCache.keys().next().value;
    if (oldestKey) folderResponseCache.delete(oldestKey);
}

export const getFolderData = async (path: string = "", params: FolderQueryParams = {}, options: FolderRequestOptions = {}): Promise<FolderData> => {
    if (options.forceRefresh) clearFolderCache(path);
    const cacheKey = folderCacheKey(path, params);
    const cached = options.forceRefresh ? undefined : folderResponseCache.get(cacheKey);
    const conditionalHeaders = cached
        ? {
            ...(cached.etag ? {"If-None-Match": cached.etag} : {}),
            ...(!cached.etag && cached.lastModified ? {"If-Modified-Since": cached.lastModified} : {})
        }
        : {};
    const requestParams = options.forceRefresh
        ? {...params, _: `${Date.now()}-${Math.random().toString(16).slice(2)}`}
        : params;
    const response = await network.get(pathUrl("/api/file", path), {
        params: requestParams,
        headers: options.forceRefresh
            ? {
                "Cache-Control": "no-cache, no-store, max-age=0",
                "Pragma": "no-cache"
            }
            : conditionalHeaders,
        validateStatus: status => (status >= 200 && status < 300) || status === 304
    });
    if (response.status === 304) {
        if (cached) return cloneFolderData(cached.data);
        folderResponseCache.delete(cacheKey);
        return getFolderData(path, params, {forceRefresh: true});
    }
    saveFolderCache(cacheKey, response.data, response.headers as Record<string, unknown>);
    return response.data
}

export const getFile = async (path: string = ""): Promise<FileContentResponse> => {
    const res = await network.get(pathUrl("/api/content", path), {
        params: {mode: "edit"},
        transformResponse: res => res
    })
    return {
        content: res.data,
        etag: res.headers.etag ?? ""
    }
}

export const createEntry = async (
    parentPath: string,
    type: "file" | "folder",
    name: string
): Promise<FileOperationResponse> => {
    const response = (await network.post(pathUrl("/api/file", parentPath), {type, name})).data
    invalidateFolderDataCache(parentPath, {includeAncestors: true});
    return response
}

export const saveFile = async (path: string, content: string, etag: string): Promise<SaveFileResponse> => {
    const res = await network.put(pathUrl("/api/content", path), content, {
        headers: {
            "Content-Type": "text/plain;charset=utf-8",
            "If-Match": etag
        }
    })
    invalidateFolderDataCache(parentPath(path), {includeAncestors: true});
    return {
        ...res.data,
        etag: res.headers.etag ?? etag
    }
}

export const moveEntry = async (path: string, targetPath: string): Promise<FileOperationResponse> => {
    const response = (await network.patch(pathUrl("/api/file", path), {targetPath})).data
    invalidateFolderDataCache([parentPath(path), parentPath(targetPath), path, targetPath], {includeAncestors: true});
    return response
}

export const deleteEntry = async (path: string, permanent = false): Promise<FileOperationResponse> => {
    const response = (await network.delete(pathUrl("/api/file", path), {
        params: permanent ? {permanent: true} : undefined
    })).data
    invalidateFolderDataCache([parentPath(path), path], {includeAncestors: true});
    return response
}

export const uploadFiles = async (path: string, files: FileList | File[]): Promise<UploadResponse> => {
    const form = new FormData()
    Array.from(files).forEach(file => form.append("files", file, file.name))
    const response = (await network.post(pathUrl("/api/upload", path), form)).data
    invalidateFolderDataCache(path, {includeAncestors: true});
    return response
}

export const downloadFile = async (path: string): Promise<Blob> => {
    return (await network.get(pathUrl("/api/download", path), {responseType: "blob"})).data
}

export const downloadUrl = (path: string): string => {
    const base = config.BASE_URL.replace(/\/$/, "")
    return `${base}${pathUrl("/api/download", path)}`
}
