import network from "../network";
import {ArchiveFormat, TaskResponse, TaskStatus} from "../class";

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
