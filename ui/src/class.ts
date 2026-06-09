
// JSON Decode Data
export type FolderInfos = FolderInfo[]
export type FileInfos = FileInfo[]
export interface FolderData {
    path: string;
    folder?: FolderInfos;
    file?: FileInfos;
    folderTotal?: number;
    fileTotal?: number;
    offset?: number;
    limit?: number;
    hasMore?: boolean;
}

export interface FolderInfo {
    path: string;
    name: string;
    modified: string;
    type?: string;
}

export interface FileInfo {
    path: string;
    name: string;
    size: number;
    extension: string;
    modified: string;
    type?: string;
}


export type LoadData = (node: FileTreeData) => Promise<unknown>
export interface FileTreeData {
    path: string;
    name: string;
    isFile?: boolean;
    children?: FileTreeData[]
}

export enum FileType {
     
    IMAGE,
    AUDIO,
    VIDEO,
    DOC,
    ARCHIVE,

    NONE,
    OTHER,
    UNKNOWN
}

export interface PathMapping {
    id?: number;
    mountPath: string;
    folderPath: string;
    remark?: string;
    order?: number;
    writable: boolean;
}

export interface SessionResponse {
    authenticated: boolean;
    authConfigured: boolean;
}

export interface RuntimeSettings {
    bindAddress: string;
    port: number;
    mappingFile: string;
    configFile: string;
    trashDir: string;
    staticDir: string;
    maxUploadBytes?: number;
    maxDirPageSize: number;
    maxDirConcurrency: number;
    maxTransferConcurrency: number;
    maxIpConcurrency: number;
    maxTaskConcurrency: number;
    taskSpeedLimitBytesPerSec?: number;
    indexEnabled: boolean;
    indexScanDelayMs: number;
    auditFile: string;
    trashRetentionDays?: number;
    trashMaxBytes?: number;
    authConfigured: boolean;
}

export interface FileOperationResponse {
    path: string;
}

export interface UploadResponse {
    files: FileOperationResponse[];
}
