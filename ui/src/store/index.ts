import {defineStore} from "pinia";
import type {
    DirSortKey,
    DirSortOrder,
    ExplorerIconSize,
    ExplorerTab,
    ExplorerViewMode,
    FileInfo,
    FileTreeData,
    FolderData,
    FolderInfo
} from "../class.ts";


declare type FileState = {
    // 是否展示文件编辑器
    showEditor: boolean;
    // 编辑器是否存在未保存修改
    editorDirty: boolean;
    // 请求编辑器确认离开的版本号
    editorLeaveRequestId: number;
    // 当前文件路径
    currentPath: string;
    // 当前文件信息
    currentFile: FileInfo | null;
    // 文件夹数据  key为文件夹路径
    folderData: Map<string, FolderData>;
    // 扩展名合集
    extensions: string[];
    // 浏览器查看模式
    viewMode: ExplorerViewMode;
    // 图标视图尺寸
    iconSize: ExplorerIconSize;
    // 当前目录排序字段
    sortKey: DirSortKey;
    // 当前目录排序方向
    sortOrder: DirSortOrder;
    // 目录标签页
    tabs: ExplorerTab[];
    activeTabId: string;
}

const normalizePath = (path: string): string => {
    let tempPath = path || "/";
    while (tempPath.indexOf("//") !== -1) {
        tempPath = tempPath.replace("//", "/");
    }
    if (tempPath.startsWith("/")) tempPath = tempPath.substring(1, tempPath.length);
    if (tempPath.endsWith("/")) tempPath = tempPath.substring(0, tempPath.length - 1);
    return "/" + tempPath;
}

const pathTitle = (path: string): string => {
    const normalized = normalizePath(path);
    if (normalized === "/") return "主页";
    const parts = normalized.split("/").filter(Boolean);
    return parts[parts.length - 1] || normalized;
}

const normalizePathStack = (stack?: string[]): string[] => {
    if (!Array.isArray(stack)) return [];
    return stack.filter(path => typeof path === "string").map(normalizePath);
}

const createTabId = () => `${Date.now()}-${Math.random().toString(16).slice(2)}`;

const createTab = (path: string): ExplorerTab => {
    const normalized = normalizePath(path);
    return {
        id: createTabId(),
        path: normalized,
        title: pathTitle(normalized),
        backStack: [],
        forwardStack: [],
        viewMode: readViewMode(),
        iconSize: readIconSize(),
        sortKey: "name",
        sortOrder: "asc"
    }
}

const storageKeys = {
    viewMode: "explorer.viewMode",
    iconSize: "explorer.iconSize",
    sortKey: "explorer.sortKey",
    sortOrder: "explorer.sortOrder",
    tabs: "explorer.tabs",
    activeTabId: "explorer.activeTabId"
}

const viewModes: ExplorerViewMode[] = ["details", "list", "icons", "tiles"];
const iconSizes: ExplorerIconSize[] = ["small", "medium", "large"];
const sortKeys: DirSortKey[] = ["name", "modified", "size"];
const sortOrders: DirSortOrder[] = ["asc", "desc"];

const readStorageItem = (key: string): string | null => {
    if (typeof localStorage === "undefined") return null;
    try {
        return localStorage.getItem(key);
    } catch {
        return null;
    }
}

const writeStorageItem = (key: string, value: string) => {
    if (typeof localStorage === "undefined") return;
    try {
        localStorage.setItem(key, value);
    } catch {
        // 浏览器禁用本地存储时，不影响本次会话内使用。
    }
}

const readViewMode = (): ExplorerViewMode => {
    const value = readStorageItem(storageKeys.viewMode);
    return viewModes.includes(value as ExplorerViewMode) ? value as ExplorerViewMode : "details";
}

const readIconSize = (): ExplorerIconSize => {
    const value = readStorageItem(storageKeys.iconSize);
    return iconSizes.includes(value as ExplorerIconSize) ? value as ExplorerIconSize : "medium";
}

const readSortKey = (): DirSortKey => {
    const value = readStorageItem(storageKeys.sortKey);
    return sortKeys.includes(value as DirSortKey) ? value as DirSortKey : "name";
}

const readSortOrder = (): DirSortOrder => {
    const value = readStorageItem(storageKeys.sortOrder);
    return sortOrders.includes(value as DirSortOrder) ? value as DirSortOrder : "asc";
}

const normalizeTab = (tab: Partial<ExplorerTab>): ExplorerTab | null => {
    if (typeof tab.id !== "string" || typeof tab.path !== "string") return null;
    const path = normalizePath(tab.path);
    const viewMode = viewModes.includes(tab.viewMode as ExplorerViewMode) ? tab.viewMode as ExplorerViewMode : readViewMode();
    const iconSize = iconSizes.includes(tab.iconSize as ExplorerIconSize) ? tab.iconSize as ExplorerIconSize : readIconSize();
    const sortKey = sortKeys.includes(tab.sortKey as DirSortKey) ? tab.sortKey as DirSortKey : readSortKey();
    const sortOrder = sortOrders.includes(tab.sortOrder as DirSortOrder) ? tab.sortOrder as DirSortOrder : readSortOrder();
    return {
        id: tab.id,
        path,
        title: pathTitle(path),
        backStack: normalizePathStack(tab.backStack),
        forwardStack: normalizePathStack(tab.forwardStack),
        viewMode,
        iconSize,
        sortKey,
        sortOrder
    };
}

const readTabs = (): ExplorerTab[] => {
    const raw = readStorageItem(storageKeys.tabs);
    if (!raw) return [createTab("/")];
    try {
        const parsed = JSON.parse(raw) as unknown;
        if (!Array.isArray(parsed)) return [createTab("/")];
        const seen = new Set<string>();
        const tabs = parsed.flatMap((item): ExplorerTab[] => {
            if (!item || typeof item !== "object") return [];
            const tab = item as Partial<ExplorerTab>;
            const normalized = normalizeTab(tab);
            if (!normalized) return [];
            if (seen.has(normalized.id)) return [];
            seen.add(normalized.id);
            return [normalized];
        });
        return tabs.length ? tabs : [createTab("/")];
    } catch {
        return [createTab("/")];
    }
}

const readActiveTabId = (tabs: ExplorerTab[]) => {
    const id = readStorageItem(storageKeys.activeTabId);
    return id && tabs.some(tab => tab.id === id) ? id : tabs[0]?.id ?? "";
}

const cloneTab = (tab: ExplorerTab): ExplorerTab => {
    const path = normalizePath(tab.path);
    return {
        id: createTabId(),
        path,
        title: pathTitle(path),
        backStack: [...(tab.backStack ?? [])],
        forwardStack: [...(tab.forwardStack ?? [])],
        viewMode: tab.viewMode,
        iconSize: tab.iconSize,
        sortKey: tab.sortKey,
        sortOrder: tab.sortOrder
    };
}

let pendingEditorLeaveResolver: ((confirmed: boolean) => void) | null = null;

export const useFileStore = defineStore('file', {
    state: (): FileState => {
        const tabs = readTabs();
        const activeTabId = readActiveTabId(tabs);
        const activeTab = tabs.find(tab => tab.id === activeTabId) ?? tabs[0];
        return {
            showEditor: false,
            editorDirty: false,
            editorLeaveRequestId: 0,
            currentPath: activeTab?.path ?? "/",
            currentFile: null,
            folderData: new Map() as Map<string, FolderData>,
            extensions: [],
            viewMode: activeTab?.viewMode ?? readViewMode(),
            iconSize: activeTab?.iconSize ?? readIconSize(),
            sortKey: activeTab?.sortKey ?? readSortKey(),
            sortOrder: activeTab?.sortOrder ?? readSortOrder(),
            tabs,
            activeTabId
        }
    },
    actions: {
        activeTab() {
            return this.tabs.find(tab => tab.id === this.activeTabId) ?? this.tabs[0] ?? null;
        },

        openEditor(file: FileInfo) {
            this.currentFile = file;
            this.showEditor = true;
            this.editorDirty = false;
        },

        closeEditor() {
            this.showEditor = false;
            this.currentFile = null;
            this.editorDirty = false;
        },

        setEditorDirty(dirty: boolean) {
            this.editorDirty = dirty;
        },

        requestEditorLeave() {
            if (!this.showEditor || !this.editorDirty) return Promise.resolve(true);
            pendingEditorLeaveResolver?.(false);
            this.editorLeaveRequestId += 1;
            return new Promise<boolean>(resolve => {
                pendingEditorLeaveResolver = resolve;
            });
        },

        resolveEditorLeave(confirmed: boolean) {
            pendingEditorLeaveResolver?.(confirmed);
            pendingEditorLeaveResolver = null;
        },

        syncActiveTabPrefs() {
            const activeTab = this.activeTab();
            if (!activeTab) return;
            activeTab.viewMode = this.viewMode;
            activeTab.iconSize = this.iconSize;
            activeTab.sortKey = this.sortKey;
            activeTab.sortOrder = this.sortOrder;
        },

        applyTabPath(tab: ExplorerTab, path: string) {
            const normalized = normalizePath(path);
            tab.path = normalized;
            tab.title = pathTitle(normalized);
            this.currentPath = normalized;
            this.activeTabId = tab.id;
        },

        persistTabs() {
            this.syncActiveTabPrefs();
            writeStorageItem(storageKeys.tabs, JSON.stringify(this.tabs));
            writeStorageItem(storageKeys.activeTabId, this.activeTabId);
        },

        // 设置当前路径
        setCurrentPath(path: string) {
            const normalized = normalizePath(path);
            const activeTab = this.tabs.find(tab => tab.id === this.activeTabId) ?? this.tabs[0];
            if (activeTab) {
                const current = normalizePath(activeTab.path || this.currentPath || "/");
                if (current !== normalized) {
                    activeTab.backStack = [...(activeTab.backStack ?? []), current].slice(-50);
                    activeTab.forwardStack = [];
                }
                this.applyTabPath(activeTab, normalized);
                activeTab.viewMode = this.viewMode;
                activeTab.iconSize = this.iconSize;
                activeTab.sortKey = this.sortKey;
                activeTab.sortOrder = this.sortOrder;
            } else {
                this.currentPath = normalized;
            }
            this.persistTabs();
        },

        canGoBack() {
            return Boolean(this.activeTab()?.backStack?.length);
        },

        canGoForward() {
            return Boolean(this.activeTab()?.forwardStack?.length);
        },

        goBack() {
            const activeTab = this.activeTab();
            if (!activeTab?.backStack?.length) return null;
            const current = normalizePath(activeTab.path || this.currentPath || "/");
            const target = activeTab.backStack[activeTab.backStack.length - 1];
            activeTab.backStack = activeTab.backStack.slice(0, -1);
            activeTab.forwardStack = [current, ...(activeTab.forwardStack ?? [])].slice(0, 50);
            this.applyTabPath(activeTab, target);
            this.closeEditor();
            this.persistTabs();
            return target;
        },

        goForward() {
            const activeTab = this.activeTab();
            if (!activeTab?.forwardStack?.length) return null;
            const current = normalizePath(activeTab.path || this.currentPath || "/");
            const target = activeTab.forwardStack[0];
            activeTab.forwardStack = activeTab.forwardStack.slice(1);
            activeTab.backStack = [...(activeTab.backStack ?? []), current].slice(-50);
            this.applyTabPath(activeTab, target);
            this.closeEditor();
            this.persistTabs();
            return target;
        },

        setViewMode(mode: ExplorerViewMode) {
            this.viewMode = mode;
            writeStorageItem(storageKeys.viewMode, mode);
            this.persistTabs();
        },

        setIconSize(size: ExplorerIconSize) {
            this.iconSize = size;
            writeStorageItem(storageKeys.iconSize, size);
            this.persistTabs();
        },

        setSort(key: DirSortKey, order?: DirSortOrder) {
            const sameKey = this.sortKey === key;
            this.sortKey = key;
            this.sortOrder = order ?? (sameKey && this.sortOrder === "asc" ? "desc" : "asc");
            writeStorageItem(storageKeys.sortKey, this.sortKey);
            writeStorageItem(storageKeys.sortOrder, this.sortOrder);
            this.persistTabs();
        },

        ensureActiveTab() {
            if (!this.tabs.length) {
                const tab = createTab(this.currentPath || "/");
                this.tabs.push(tab);
                this.activeTabId = tab.id;
            } else if (!this.activeTabId || !this.tabs.some(tab => tab.id === this.activeTabId)) {
                this.activeTabId = this.tabs[0].id;
            }
            const activeTab = this.activeTab();
            this.currentPath = activeTab?.path ?? this.currentPath;
            this.viewMode = activeTab?.viewMode ?? this.viewMode;
            this.iconSize = activeTab?.iconSize ?? this.iconSize;
            this.sortKey = activeTab?.sortKey ?? this.sortKey;
            this.sortOrder = activeTab?.sortOrder ?? this.sortOrder;
            this.persistTabs();
        },

        openTab(path?: string) {
            const tab = createTab(path || this.currentPath || "/");
            this.tabs.push(tab);
            this.activeTabId = tab.id;
            this.currentPath = tab.path;
            this.viewMode = tab.viewMode ?? this.viewMode;
            this.iconSize = tab.iconSize ?? this.iconSize;
            this.sortKey = tab.sortKey ?? this.sortKey;
            this.sortOrder = tab.sortOrder ?? this.sortOrder;
            this.closeEditor();
            this.persistTabs();
        },

        openPathInNewTab(path: string) {
            this.openTab(path);
        },

        duplicateTab(tabId: string) {
            const index = this.tabs.findIndex(tab => tab.id === tabId);
            if (index < 0) return;
            const tab = cloneTab(this.tabs[index]);
            this.tabs.splice(index + 1, 0, tab);
            this.activeTabId = tab.id;
            this.currentPath = tab.path;
            this.viewMode = tab.viewMode ?? this.viewMode;
            this.iconSize = tab.iconSize ?? this.iconSize;
            this.sortKey = tab.sortKey ?? this.sortKey;
            this.sortOrder = tab.sortOrder ?? this.sortOrder;
            this.closeEditor();
            this.persistTabs();
        },

        reorderTab(sourceTabId: string, targetTabId: string, placement: "before" | "after") {
            if (sourceTabId === targetTabId) return;
            const sourceIndex = this.tabs.findIndex(tab => tab.id === sourceTabId);
            const targetIndex = this.tabs.findIndex(tab => tab.id === targetTabId);
            if (sourceIndex < 0 || targetIndex < 0) return;
            const [tab] = this.tabs.splice(sourceIndex, 1);
            const nextTargetIndex = this.tabs.findIndex(item => item.id === targetTabId);
            const insertIndex = placement === "after" ? nextTargetIndex + 1 : nextTargetIndex;
            this.tabs.splice(insertIndex, 0, tab);
            this.persistTabs();
        },

        switchTab(tabId: string) {
            const tab = this.tabs.find(item => item.id === tabId);
            if (!tab) return;
            this.activeTabId = tab.id;
            this.currentPath = tab.path;
            this.viewMode = tab.viewMode ?? this.viewMode;
            this.iconSize = tab.iconSize ?? this.iconSize;
            this.sortKey = tab.sortKey ?? this.sortKey;
            this.sortOrder = tab.sortOrder ?? this.sortOrder;
            this.closeEditor();
            this.persistTabs();
        },

        closeTab(tabId: string) {
            if (this.tabs.length <= 1) return;
            const index = this.tabs.findIndex(tab => tab.id === tabId);
            if (index < 0) return;
            const wasActive = this.activeTabId === tabId;
            this.tabs.splice(index, 1);
            if (wasActive) {
                const next = this.tabs[Math.max(0, index - 1)];
                this.activeTabId = next.id;
                this.currentPath = next.path;
                this.viewMode = next.viewMode ?? this.viewMode;
                this.iconSize = next.iconSize ?? this.iconSize;
                this.sortKey = next.sortKey ?? this.sortKey;
                this.sortOrder = next.sortOrder ?? this.sortOrder;
                this.closeEditor();
            }
            this.persistTabs();
        },

        closeOtherTabs(tabId: string) {
            const tab = this.tabs.find(item => item.id === tabId);
            if (!tab || this.tabs.length <= 1) return;
            const wasActive = this.activeTabId === tab.id;
            this.tabs = [tab];
            this.activeTabId = tab.id;
            this.currentPath = tab.path;
            this.viewMode = tab.viewMode ?? this.viewMode;
            this.iconSize = tab.iconSize ?? this.iconSize;
            this.sortKey = tab.sortKey ?? this.sortKey;
            this.sortOrder = tab.sortOrder ?? this.sortOrder;
            if (!wasActive) this.closeEditor();
            this.persistTabs();
        },

        closeTabsToRight(tabId: string) {
            const index = this.tabs.findIndex(tab => tab.id === tabId);
            if (index < 0 || index === this.tabs.length - 1) return;
            this.tabs = this.tabs.slice(0, index + 1);
            if (!this.tabs.some(tab => tab.id === this.activeTabId)) {
                const tab = this.tabs[index];
                this.activeTabId = tab.id;
                this.currentPath = tab.path;
                this.viewMode = tab.viewMode ?? this.viewMode;
                this.iconSize = tab.iconSize ?? this.iconSize;
                this.sortKey = tab.sortKey ?? this.sortKey;
                this.sortOrder = tab.sortOrder ?? this.sortOrder;
                this.closeEditor();
            }
            this.persistTabs();
        },

        saveFolderData(data: FolderData) {
            this.folderData.set(data.path, data);
        },

        // 保存转换文件夹数据到文件树
        saveAndConvertFolderData(data: FolderData): FileTreeData[] {
            const treeData: FileTreeData[] = [];

            data.folder?.forEach((folder: FolderInfo) => {
                treeData.push({
                    path: folder.path,
                    name: folder.name,
                    isFile: false,
                });
            })

            data.file?.forEach((file: FileInfo) => {
                treeData.push({
                    path: file.path,
                    name: file.name,
                    isFile: true,
                });
            })

            this.saveFolderData(data)
            return treeData;
        }

    }
})
