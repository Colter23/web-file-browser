import network from "../network";
import type {ArchiveFormat, RuntimeSettings, TaskCleanupResponse, TaskResponse, TaskStatus} from "../class";
import {parentPath} from "../utils/file-path.ts";
import {invalidateFolderDataCache} from "./file-api.ts";

type ConflictPolicy = RuntimeSettings["conflictPolicy"];

const conflictPayload = (conflictPolicy?: ConflictPolicy) => ({
    conflictPolicy: conflictPolicy || undefined
})

export const createCopyTask = async (
    sources: string[],
    targetPath: string,
    conflictPolicy?: ConflictPolicy
): Promise<TaskResponse> => {
    const response = (await network.post("/api/tasks/copy", {
        sources,
        targetPath,
        ...conflictPayload(conflictPolicy)
    })).data
    invalidateFolderDataCache([targetPath, ...sources.map(parentPath)], {includeAncestors: true});
    return response
}

export const createMoveTask = async (
    sources: string[],
    targetPath: string,
    conflictPolicy?: ConflictPolicy
): Promise<TaskResponse> => {
    const response = (await network.post("/api/tasks/move", {
        sources,
        targetPath,
        ...conflictPayload(conflictPolicy)
    })).data
    invalidateFolderDataCache([targetPath, ...sources, ...sources.map(parentPath)], {includeAncestors: true});
    return response
}

export const createDeleteTask = async (paths: string[]): Promise<TaskResponse> => {
    const response = (await network.post("/api/tasks/delete", {paths})).data
    invalidateFolderDataCache([...paths, ...paths.map(parentPath)], {includeAncestors: true});
    return response
}

export const createArchiveTask = async (
    sources: string[],
    targetPath: string,
    format: ArchiveFormat,
    outputName?: string
): Promise<TaskResponse> => {
    const response = (await network.post("/api/tasks/archive", {
        sources,
        targetPath,
        format,
        outputName: outputName || undefined
    })).data
    invalidateFolderDataCache([targetPath, ...sources.map(parentPath)], {includeAncestors: true});
    return response
}

export const createExtractTask = async (
    sourcePath: string,
    targetPath: string,
    folderName?: string
): Promise<TaskResponse> => {
    const response = (await network.post("/api/tasks/extract", {
        sourcePath,
        targetPath,
        folderName: folderName || undefined
    })).data
    invalidateFolderDataCache([targetPath, parentPath(sourcePath)], {includeAncestors: true});
    return response
}

export const listTasks = async (): Promise<TaskStatus[]> => {
    return (await network.get("/api/tasks")).data
}

export const getTask = async (id: string): Promise<TaskStatus> => {
    return (await network.get(`/api/tasks/${encodeURIComponent(id)}`)).data
}

export const cancelTask = async (id: string): Promise<TaskStatus> => {
    return (await network.post(`/api/tasks/${encodeURIComponent(id)}/cancel`)).data
}

export const cleanupTasks = async (): Promise<TaskCleanupResponse> => {
    return (await network.post("/api/tasks/cleanup")).data
}
