
// 接口返回数据结构
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

export type DirSortKey = "name" | "modified" | "size";
export type DirSortOrder = "asc" | "desc";
export type DirEntryFilter = "all" | "folder" | "file";
export type DirDetail = "basic" | "full";
export type SearchScope = "mount" | "all";

export interface FolderQueryParams {
    offset?: number;
    limit?: number;
    detail?: DirDetail;
    sort?: DirSortKey;
    order?: DirSortOrder;
    type?: DirEntryFilter;
    includeHidden?: boolean;
    includeTotal?: boolean;
}

export type ExplorerViewMode = "details" | "list" | "icons" | "tiles";
export type ExplorerIconSize = "small" | "medium" | "large";
export type AppIconStyle = "lucide" | "classic";
export type FileIconPalette = "category" | "accent";
export type AppAccentColor = "blue" | "teal" | "violet" | "rose" | "slate";
export type AppColorMode = "system" | "light" | "dark";

export interface ExplorerTab {
    id: string;
    path: string;
    title: string;
    filterText?: string;
    selectedPaths?: string[];
    scrollTop?: number;
    backStack?: string[];
    forwardStack?: string[];
    viewMode?: ExplorerViewMode;
    iconSize?: ExplorerIconSize;
    sortKey?: DirSortKey;
    sortOrder?: DirSortOrder;
}

export interface ClosedExplorerTab extends ExplorerTab {
    closedAt: number;
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

export type SearchResultType = "file" | "folder" | string;

export interface SearchResult {
    name: string;
    path: string;
    extension?: string;
    modified?: string;
    size?: number;
    type: SearchResultType;
    mountPath?: string;
}

export interface SearchResponse {
    items: SearchResult[];
    total: number;
    offset: number;
    limit: number;
}


export type LoadData = (node: FileTreeData, options?: {navigate?: boolean; focusExplorer?: boolean; refresh?: boolean}) => Promise<boolean>

export type MappingRootNode = MappingVirtualNode | MappingRealNode;

export interface MappingVirtualNode {
    type: "virtual";
    name: string;
    path: string;
    children: MappingRootNode[];
}

export interface MappingRealNode {
    type: "real";
    name: string;
    path: string;
    realPath: string;
}

export interface FileTreeData {
    path: string;
    name: string;
    isFile?: boolean;
    children?: FileTreeData[];
    virtual?: boolean;
    mappingId?: number;
    mappingOrder?: number;
}

export interface FavoriteItem {
    id: string;
    mountId: number | string;
    mountPath: string;
    relativePath: string;
    path: string;
    name: string;
    order: number;
    createdAt: string;
    missing?: boolean;
}

export interface CreateFavoriteRequest {
    path: string;
    name?: string;
    order?: number;
}

export interface UpdateFavoriteRequest {
    name?: string;
    order?: number;
}

export interface ReorderFavoriteItem {
    id: string;
    order: number;
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

export interface ReorderMappingItem {
    id: number;
    order: number;
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
    corsAllowedOrigins: string[];
    trustProxyHeaders: boolean;
    maxEditBytes: number;
    editableExtensions: string[];
    editableMimeTypes: string[];
    maxUploadBytes?: number;
    maxDirPageSize: number;
    maxDirConcurrency: number;
    maxTransferConcurrency: number;
    maxIpConcurrency: number;
    maxTaskConcurrency: number;
    taskHistoryLimit: number;
    taskSpeedLimitBytesPerSec?: number;
    maxExtractBytes?: number;
    maxExtractFiles?: number;
    indexEnabled: boolean;
    indexRebuildOnStartup: boolean;
    indexScanDelayMs: number;
    auditFile: string;
    auditMaxBytes?: number;
    auditRetentionFiles: number;
    trashRetentionDays?: number;
    trashMaxBytes?: number;
    conflictPolicy: "autoRename" | "reject" | "overwrite";
    authConfigured: boolean;
}

export interface FileOperationResponse {
    path: string;
}

export interface FileContentResponse {
    content: string;
    etag: string;
}

export interface SaveFileResponse extends FileOperationResponse {
    etag: string;
}

export interface UploadResponse {
    files: FileOperationResponse[];
}

export type ArchiveFormat = "tarGz" | "zip";

export type TaskKind = "copy" | "move" | "delete" | "archive" | "extract";
export type TaskState = "queued" | "running" | "completed" | "failed" | "cancelled";

export interface TaskResponse {
    id: string;
}

export interface TaskCleanupResponse {
    removed: number;
}

export type TrashRecordKind = "file" | "folder" | string;

export interface TrashRecord {
    id: string;
    originalVirtualPath: string;
    originalRealPath: string;
    trashPath: string;
    sizeBytes?: number;
    deletedAt: string;
    actor?: string;
    kind: TrashRecordKind;
}

export interface TrashRestoreResponse {
    record: TrashRecord;
    restoredVirtualPath: string;
    restoredRealPath: string;
}

export interface TrashBatchError {
    id: string;
    message: string;
}

export interface TrashBatchRestoreResponse {
    restored: TrashRestoreResponse[];
    errors: TrashBatchError[];
    success: number;
    failed: number;
}

export interface TrashBatchPurgeResponse {
    purged: string[];
    errors: TrashBatchError[];
    success: number;
    failed: number;
}

export interface TrashCleanupResponse {
    removed: number;
}

export interface TaskError {
    path: string;
    message: string;
}

export interface TaskStatus {
    id: string;
    kind: TaskKind;
    state: TaskState;
    progress: number;
    processedBytes: number;
    totalBytes: number;
    speedBytesPerSec: number;
    processedItems: number;
    totalItems: number;
    currentPath?: string;
    errors: TaskError[];
    startedAt?: string;
    finishedAt?: string;
    createdAt: string;
    cancelled: boolean;
}

export interface IndexStatus {
    enabled: boolean;
    state: "disabled" | "idle" | "building" | "error" | string;
    indexedEntries: number;
    lastStartedAt?: string | null;
    lastFinishedAt?: string | null;
    lastError?: string | null;
}

export interface TaskMetrics {
    total: number;
    queued: number;
    running: number;
    completed: number;
    failed: number;
    cancelled: number;
    errorsTotal: number;
    processedBytes: number;
    currentSpeedBytesPerSec: number;
}

export interface RequestLimitMetrics {
    dirScanLimit: number;
    activeDirScans: number;
    transferLimit: number;
    activeTransfers: number;
    ipLimit: number;
    trackedIps: number;
    activeIpRequests: number;
}

export interface MetricsResponse {
    mappings: number;
    activeSessions: number;
    trashEntries: number;
    tasks: TaskMetrics;
    limits: RequestLimitMetrics;
    index: IndexStatus;
}

export interface HealthResponse {
    status: string;
    version: string;
}

export interface ReadinessCheck {
    name: string;
    status: string;
    message: string;
}

export interface ReadinessResponse extends HealthResponse {
    checks: ReadinessCheck[];
}
