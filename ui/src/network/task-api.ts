import network from "../network";
import type {ArchiveFormat, RuntimeSettings, TaskResponse, TaskStatus} from "../class";

type ConflictPolicy = RuntimeSettings["conflictPolicy"];

const conflictPayload = (conflictPolicy?: ConflictPolicy) => ({
    conflictPolicy: conflictPolicy || undefined
})

export const createCopyTask = async (
    sources: string[],
    targetPath: string,
    conflictPolicy?: ConflictPolicy
): Promise<TaskResponse> => {
    return (await network.post("/api/tasks/copy", {
        sources,
        targetPath,
        ...conflictPayload(conflictPolicy)
    })).data
}

export const createMoveTask = async (
    sources: string[],
    targetPath: string,
    conflictPolicy?: ConflictPolicy
): Promise<TaskResponse> => {
    return (await network.post("/api/tasks/move", {
        sources,
        targetPath,
        ...conflictPayload(conflictPolicy)
    })).data
}

export const createDeleteTask = async (paths: string[]): Promise<TaskResponse> => {
    return (await network.post("/api/tasks/delete", {paths})).data
}

export const createArchiveTask = async (
    sources: string[],
    targetPath: string,
    format: ArchiveFormat,
    outputName?: string
): Promise<TaskResponse> => {
    return (await network.post("/api/tasks/archive", {
        sources,
        targetPath,
        format,
        outputName: outputName || undefined
    })).data
}

export const createExtractTask = async (
    sourcePath: string,
    targetPath: string,
    folderName?: string
): Promise<TaskResponse> => {
    return (await network.post("/api/tasks/extract", {
        sourcePath,
        targetPath,
        folderName: folderName || undefined
    })).data
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
