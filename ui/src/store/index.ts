import {defineStore} from "pinia";
import {
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

const createTab = (path: string): ExplorerTab => {
    const normalized = normalizePath(path);
    return {
        id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
        path: normalized,
        title: pathTitle(normalized)
    }
}

const storageKeys = {
    viewMode: "explorer.viewMode",
    iconSize: "explorer.iconSize",
    tabs: "explorer.tabs",
    activeTabId: "explorer.activeTabId"
}

const viewModes: ExplorerViewMode[] = ["details", "list", "icons", "tiles"];
const iconSizes: ExplorerIconSize[] = ["small", "medium", "large"];

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
            if (typeof tab.id !== "string" || typeof tab.path !== "string") return [];
            if (seen.has(tab.id)) return [];
            seen.add(tab.id);
            const path = normalizePath(tab.path);
            return [{
                id: tab.id,
                path,
                title: pathTitle(path)
            }];
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

export const useFileStore = defineStore('file', {
    state: (): FileState => {
        const tabs = readTabs();
        const activeTabId = readActiveTabId(tabs);
        const activeTab = tabs.find(tab => tab.id === activeTabId) ?? tabs[0];
        return {
            showEditor: false,
            currentPath: activeTab?.path ?? "/",
            currentFile: null,
            folderData: new Map() as Map<string, FolderData>,
            extensions: [],
            viewMode: readViewMode(),
            iconSize: readIconSize(),
            tabs,
            activeTabId
        }
    },
    actions: {
        persistTabs() {
            writeStorageItem(storageKeys.tabs, JSON.stringify(this.tabs));
            writeStorageItem(storageKeys.activeTabId, this.activeTabId);
        },

        // 设置当前路径
        setCurrentPath(path: string) {
            const normalized = normalizePath(path);
            this.currentPath = normalized;
            const activeTab = this.tabs.find(tab => tab.id === this.activeTabId) ?? this.tabs[0];
            if (activeTab) {
                activeTab.path = normalized;
                activeTab.title = pathTitle(normalized);
                this.activeTabId = activeTab.id;
            }
            this.persistTabs();
        },

        setViewMode(mode: ExplorerViewMode) {
            this.viewMode = mode;
            writeStorageItem(storageKeys.viewMode, mode);
        },

        setIconSize(size: ExplorerIconSize) {
            this.iconSize = size;
            writeStorageItem(storageKeys.iconSize, size);
        },

        ensureActiveTab() {
            if (!this.tabs.length) {
                const tab = createTab(this.currentPath || "/");
                this.tabs.push(tab);
                this.activeTabId = tab.id;
            } else if (!this.activeTabId || !this.tabs.some(tab => tab.id === this.activeTabId)) {
                this.activeTabId = this.tabs[0].id;
            }
            this.currentPath = this.tabs.find(tab => tab.id === this.activeTabId)?.path ?? this.currentPath;
            this.persistTabs();
        },

        openTab(path?: string) {
            const tab = createTab(path || this.currentPath || "/");
            this.tabs.push(tab);
            this.activeTabId = tab.id;
            this.currentPath = tab.path;
            this.showEditor = false;
            this.currentFile = null;
            this.persistTabs();
        },

        openPathInNewTab(path: string) {
            this.openTab(path);
        },

        switchTab(tabId: string) {
            const tab = this.tabs.find(item => item.id === tabId);
            if (!tab) return;
            this.activeTabId = tab.id;
            this.currentPath = tab.path;
            this.showEditor = false;
            this.currentFile = null;
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
                this.showEditor = false;
                this.currentFile = null;
            }
            this.persistTabs();
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

            this.folderData.set(data.path, data)
            return treeData;
        }

    }
})
