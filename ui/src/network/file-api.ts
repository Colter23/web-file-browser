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

export type FolderRequestOptions = {
    forceRefresh?: boolean;
}

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

export const getFolderData = async (path: string = "", params: FolderQueryParams = {}, options: FolderRequestOptions = {}): Promise<FolderData> => {
    const requestParams = options.forceRefresh
        ? {...params, _: `${Date.now()}-${Math.random().toString(16).slice(2)}`}
        : params;
    return (await network.get(pathUrl("/api/file", path), {
        params: requestParams,
        headers: options.forceRefresh
            ? {
                "Cache-Control": "no-cache, no-store, max-age=0",
                "Pragma": "no-cache"
            }
            : undefined
    })).data
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
    return (await network.post(pathUrl("/api/file", parentPath), {type, name})).data
}

export const saveFile = async (path: string, content: string, etag: string): Promise<SaveFileResponse> => {
    const res = await network.put(pathUrl("/api/content", path), content, {
        headers: {
            "Content-Type": "text/plain;charset=utf-8",
            "If-Match": etag
        }
    })
    return {
        ...res.data,
        etag: res.headers.etag ?? etag
    }
}

export const moveEntry = async (path: string, targetPath: string): Promise<FileOperationResponse> => {
    return (await network.patch(pathUrl("/api/file", path), {targetPath})).data
}

export const deleteEntry = async (path: string): Promise<FileOperationResponse> => {
    return (await network.delete(pathUrl("/api/file", path))).data
}

export const uploadFiles = async (path: string, files: FileList | File[]): Promise<UploadResponse> => {
    const form = new FormData()
    Array.from(files).forEach(file => form.append("files", file, file.name))
    return (await network.post(pathUrl("/api/upload", path), form)).data
}

export const downloadFile = async (path: string): Promise<Blob> => {
    return (await network.get(pathUrl("/api/download", path), {responseType: "blob"})).data
}

export const downloadUrl = (path: string): string => {
    const base = config.BASE_URL.replace(/\/$/, "")
    return `${base}${pathUrl("/api/download", path)}`
}
