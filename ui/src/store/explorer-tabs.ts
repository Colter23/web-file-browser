import type {
    ClosedExplorerTab,
    DirSortKey,
    DirSortOrder,
    ExplorerIconSize,
    ExplorerTab,
    ExplorerViewMode
} from "../class.ts";
import {translate} from "../i18n";
import {readJsonStorage, readStorageItem, writeJsonStorage, writeStorageItem} from "../utils/safe-storage.ts";

export const closedTabStackLimit = 12;

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

export const normalizePath = (path: string): string => {
    let tempPath = path || "/";
    while (tempPath.indexOf("//") !== -1) {
        tempPath = tempPath.replace("//", "/");
    }
    if (tempPath.startsWith("/")) tempPath = tempPath.substring(1, tempPath.length);
    if (tempPath.endsWith("/")) tempPath = tempPath.substring(0, tempPath.length - 1);
    return "/" + tempPath;
}

export const pathTitle = (path: string): string => {
    const normalized = normalizePath(path);
    if (normalized === "/") return translate("nav.home");
    const parts = normalized.split("/").filter(Boolean);
    return parts[parts.length - 1] || normalized;
}

const normalizePathStack = (stack?: string[]): string[] => {
    if (!Array.isArray(stack)) return [];
    return stack.filter(path => typeof path === "string").map(normalizePath);
}

export const normalizeSelectedPaths = (paths?: string[]): string[] => {
    if (!Array.isArray(paths)) return [];
    return Array.from(new Set(paths.filter(path => typeof path === "string").map(normalizePath)));
}

export const normalizeFilterText = (text?: string): string => typeof text === "string" ? text.slice(0, 200) : "";

export const normalizeScrollTop = (scrollTop?: number): number => {
    if (!Number.isFinite(scrollTop)) return 0;
    return Math.max(0, Math.round(scrollTop ?? 0));
}

const createTabId = () => `${Date.now()}-${Math.random().toString(16).slice(2)}`;

export const readViewMode = (): ExplorerViewMode => {
    const value = readStorageItem(storageKeys.viewMode);
    return viewModes.includes(value as ExplorerViewMode) ? value as ExplorerViewMode : "details";
}

export const readIconSize = (): ExplorerIconSize => {
    const value = readStorageItem(storageKeys.iconSize);
    return iconSizes.includes(value as ExplorerIconSize) ? value as ExplorerIconSize : "medium";
}

export const readSortKey = (): DirSortKey => {
    const value = readStorageItem(storageKeys.sortKey);
    return sortKeys.includes(value as DirSortKey) ? value as DirSortKey : "name";
}

export const readSortOrder = (): DirSortOrder => {
    const value = readStorageItem(storageKeys.sortOrder);
    return sortOrders.includes(value as DirSortOrder) ? value as DirSortOrder : "asc";
}

export const writeViewMode = (mode: ExplorerViewMode) => writeStorageItem(storageKeys.viewMode, mode);

export const writeIconSize = (size: ExplorerIconSize) => writeStorageItem(storageKeys.iconSize, size);

export const writeSortPrefs = (key: DirSortKey, order: DirSortOrder) => {
    writeStorageItem(storageKeys.sortKey, key);
    writeStorageItem(storageKeys.sortOrder, order);
}

export const writeTabsStorage = (tabs: ExplorerTab[], activeTabId: string) => {
    writeJsonStorage(storageKeys.tabs, tabs);
    writeStorageItem(storageKeys.activeTabId, activeTabId);
}

export const createTab = (path: string): ExplorerTab => {
    const normalized = normalizePath(path);
    return {
        id: createTabId(),
        path: normalized,
        title: pathTitle(normalized),
        filterText: "",
        selectedPaths: [],
        scrollTop: 0,
        backStack: [],
        forwardStack: [],
        viewMode: readViewMode(),
        iconSize: readIconSize(),
        sortKey: "name",
        sortOrder: "asc"
    }
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
        filterText: normalizeFilterText(tab.filterText),
        selectedPaths: normalizeSelectedPaths(tab.selectedPaths),
        scrollTop: normalizeScrollTop(tab.scrollTop),
        backStack: normalizePathStack(tab.backStack),
        forwardStack: normalizePathStack(tab.forwardStack),
        viewMode,
        iconSize,
        sortKey,
        sortOrder
    };
}

export const readTabs = (): ExplorerTab[] => {
    const parsed = readJsonStorage<unknown>(storageKeys.tabs, []);
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
}

export const readActiveTabId = (tabs: ExplorerTab[]) => {
    const id = readStorageItem(storageKeys.activeTabId);
    return id && tabs.some(tab => tab.id === id) ? id : tabs[0]?.id ?? "";
}

export const cloneTab = (tab: ExplorerTab): ExplorerTab => {
    const path = normalizePath(tab.path);
    return {
        id: createTabId(),
        path,
        title: pathTitle(path),
        filterText: normalizeFilterText(tab.filterText),
        selectedPaths: normalizeSelectedPaths(tab.selectedPaths),
        scrollTop: normalizeScrollTop(tab.scrollTop),
        backStack: [...(tab.backStack ?? [])],
        forwardStack: [...(tab.forwardStack ?? [])],
        viewMode: tab.viewMode,
        iconSize: tab.iconSize,
        sortKey: tab.sortKey,
        sortOrder: tab.sortOrder
    };
}

export const cloneClosedTab = (tab: ExplorerTab): ClosedExplorerTab => ({
    ...cloneTab(tab),
    closedAt: Date.now()
});

export const reviveClosedTab = (tab: ClosedExplorerTab): ExplorerTab => {
    const revived = cloneTab(tab);
    revived.title = pathTitle(revived.path);
    return revived;
}
