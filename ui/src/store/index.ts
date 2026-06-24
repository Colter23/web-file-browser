import {defineStore} from "pinia";
import type {
    ClosedExplorerTab,
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
import {
    cloneClosedTab,
    cloneTab,
    closedTabStackLimit,
    createTab,
    normalizeFilterText,
    normalizePath,
    normalizeScrollTop,
    normalizeSelectedPaths,
    pathTitle,
    readActiveTabId,
    readIconSize,
    readSortKey,
    readSortOrder,
    readTabs,
    readViewMode,
    reviveClosedTab,
    writeIconSize,
    writeSortPrefs,
    writeTabsStorage,
    writeViewMode
} from "./explorer-tabs.ts";


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
    // 最近关闭的目录标签页
    closedTabs: ClosedExplorerTab[];
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
            activeTabId,
            closedTabs: []
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

        resetTabBrowserState(tab: ExplorerTab) {
            tab.filterText = "";
            tab.selectedPaths = [];
            tab.scrollTop = 0;
        },

        activateTab(tab: ExplorerTab, closeEditor = true) {
            this.activeTabId = tab.id;
            this.currentPath = tab.path;
            this.viewMode = tab.viewMode ?? this.viewMode;
            this.iconSize = tab.iconSize ?? this.iconSize;
            this.sortKey = tab.sortKey ?? this.sortKey;
            this.sortOrder = tab.sortOrder ?? this.sortOrder;
            if (closeEditor) this.closeEditor();
        },

        setActiveTabFilterText(text: string) {
            const activeTab = this.activeTab();
            if (!activeTab) return;
            activeTab.filterText = normalizeFilterText(text);
            this.persistTabs();
        },

        setActiveTabSelectedPaths(paths: string[]) {
            const activeTab = this.activeTab();
            if (!activeTab) return;
            activeTab.selectedPaths = normalizeSelectedPaths(paths);
            this.persistTabs();
        },

        setActiveTabScrollTop(scrollTop: number) {
            const activeTab = this.activeTab();
            if (!activeTab) return;
            activeTab.scrollTop = normalizeScrollTop(scrollTop);
            this.persistTabs();
        },

        persistTabs() {
            this.syncActiveTabPrefs();
            writeTabsStorage(this.tabs, this.activeTabId);
        },

        rememberClosedTabs(tabs: ExplorerTab[]) {
            if (!tabs.length) return;
            this.closedTabs = [
                ...tabs.map(cloneClosedTab).reverse(),
                ...this.closedTabs
            ].slice(0, closedTabStackLimit);
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
                    this.resetTabBrowserState(activeTab);
                }
                this.applyTabPath(activeTab, normalized);
                this.syncActiveTabPrefs();
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
            this.resetTabBrowserState(activeTab);
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
            this.resetTabBrowserState(activeTab);
            this.applyTabPath(activeTab, target);
            this.closeEditor();
            this.persistTabs();
            return target;
        },

        setViewMode(mode: ExplorerViewMode) {
            this.viewMode = mode;
            writeViewMode(mode);
            this.persistTabs();
        },

        setIconSize(size: ExplorerIconSize) {
            this.iconSize = size;
            writeIconSize(size);
            this.persistTabs();
        },

        setSort(key: DirSortKey, order?: DirSortOrder) {
            const sameKey = this.sortKey === key;
            this.sortKey = key;
            this.sortOrder = order ?? (sameKey && this.sortOrder === "asc" ? "desc" : "asc");
            writeSortPrefs(this.sortKey, this.sortOrder);
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
            if (activeTab) this.activateTab(activeTab, false);
            this.persistTabs();
        },

        openTab(path = "/") {
            const tab = createTab(path);
            this.tabs.push(tab);
            this.activateTab(tab);
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
            this.activateTab(tab);
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
            this.activateTab(tab);
            this.persistTabs();
        },

        closeTab(tabId: string) {
            if (this.tabs.length <= 1) return;
            const index = this.tabs.findIndex(tab => tab.id === tabId);
            if (index < 0) return;
            const wasActive = this.activeTabId === tabId;
            const [closedTab] = this.tabs.splice(index, 1);
            this.rememberClosedTabs([closedTab]);
            if (wasActive) {
                const next = this.tabs[Math.max(0, index - 1)];
                this.activateTab(next);
            }
            this.persistTabs();
        },

        closeOtherTabs(tabId: string) {
            const tab = this.tabs.find(item => item.id === tabId);
            if (!tab || this.tabs.length <= 1) return;
            const wasActive = this.activeTabId === tab.id;
            this.rememberClosedTabs(this.tabs.filter(item => item.id !== tab.id));
            this.tabs = [tab];
            this.activateTab(tab, !wasActive);
            this.persistTabs();
        },

        closeTabsToRight(tabId: string) {
            const index = this.tabs.findIndex(tab => tab.id === tabId);
            if (index < 0 || index === this.tabs.length - 1) return;
            this.rememberClosedTabs(this.tabs.slice(index + 1));
            this.tabs = this.tabs.slice(0, index + 1);
            if (!this.tabs.some(tab => tab.id === this.activeTabId)) {
                const tab = this.tabs[index];
                this.activateTab(tab);
            }
            this.persistTabs();
        },

        reopenClosedTab() {
            const closed = this.closedTabs.shift();
            if (!closed) return null;
            const tab = reviveClosedTab(closed);
            this.tabs.push(tab);
            this.activateTab(tab);
            this.persistTabs();
            return tab;
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
