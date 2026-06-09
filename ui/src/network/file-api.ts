import network from "../network";
import {FileOperationResponse, FolderData, UploadResponse} from "../class";

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

export const getFolderData = async (path: string = ""): Promise<FolderData> => {
    return (await network.get(pathUrl("/api/file", path))).data
}

export const getFile = async (path: string = ""): Promise<string> => {
    const res = await network.get(pathUrl("/api/content", path), {
        transformResponse: res => res
    })
    return res.data
}

export const createEntry = async (
    parentPath: string,
    type: "file" | "folder",
    name: string
): Promise<FileOperationResponse> => {
    return (await network.post(pathUrl("/api/file", parentPath), {type, name})).data
}

export const saveFile = async (path: string, content: string): Promise<FileOperationResponse> => {
    return (await network.put(pathUrl("/api/content", path), content, {
        headers: {"Content-Type": "text/plain;charset=utf-8"}
    })).data
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
