import type {RuntimeSettings, TrashCleanupResponse, TrashRecord, TrashRestoreResponse} from "../class";
import network from "../network";
import {parentPath} from "../utils/file-path.ts";
import {invalidateFolderDataCache} from "./file-api.ts";

type ConflictPolicy = RuntimeSettings["conflictPolicy"];

export const listTrashRecords = async (): Promise<TrashRecord[]> => {
    return (await network.get("/api/trash")).data
}

export const restoreTrashRecord = async (id: string, conflictPolicy?: ConflictPolicy): Promise<TrashRestoreResponse> => {
    const response = (await network.post(`/api/trash/${encodeURIComponent(id)}/restore`, undefined, {
        params: conflictPolicy ? {conflictPolicy} : undefined
    })).data
    invalidateFolderDataCache(parentPath(response.restoredVirtualPath), {includeAncestors: true});
    return response
}

export const deleteTrashRecord = async (id: string): Promise<void> => {
    await network.delete(`/api/trash/${encodeURIComponent(id)}`)
}

export const emptyTrash = async (): Promise<TrashCleanupResponse> => {
    return (await network.post("/api/trash/empty")).data
}

export const cleanupTrash = async (): Promise<TrashCleanupResponse> => {
    return (await network.post("/api/trash/cleanup")).data
}
