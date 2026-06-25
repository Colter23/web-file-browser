import {ref} from "vue";
import type {
  FileTreeData,
  FolderData,
  FolderQueryParams,
  MappingRootNode,
  PathMapping,
  ReorderMappingItem
} from "../class.ts";
import {useFileStore} from "../store";
import {normalizePathText, parentPath} from "../utils/file-path.ts";

type FileTreeLoaderOptions = {
  getFolderData: (path: string, params?: FolderQueryParams, options?: {forceRefresh?: boolean}) => Promise<FolderData>;
  getMappings: () => Promise<PathMapping[]>;
  getMappingRoot: () => Promise<MappingRootNode | null>;
  reorderMappings: (items: ReorderMappingItem[]) => Promise<void>;
  navigateToPath: (path: string, options?: {skipEditorLeave?: boolean; focusExplorer?: boolean}) => Promise<boolean>;
  showError: (error: unknown, fallback: string, title?: string) => void;
}

type MountDropPlacement = "before" | "after";
type OrderedMapping = PathMapping & {id: number};

const treeQuery: FolderQueryParams = {
  detail: "basic",
  sort: "name",
  order: "asc",
  type: "folder"
};

const folderDataToTreeNodes = (data: FolderData): FileTreeData[] => {
  return (data.folder ?? []).map(folder => ({
    path: folder.path,
    name: folder.name,
    isFile: false
  }));
}

const rootTreeNode = (children?: FileTreeData[]): FileTreeData => {
  const node: FileTreeData = {
    path: "/",
    name: "主页",
    isFile: false
  };
  if (children !== undefined) node.children = children;
  return node;
}

const sortMappings = (items: PathMapping[]): OrderedMapping[] => {
  return [...items]
      .filter((mapping): mapping is OrderedMapping => typeof mapping.id === "number")
      .sort((left, right) => {
        const orderDiff = (left.order ?? 0) - (right.order ?? 0);
        return orderDiff !== 0 ? orderDiff : left.id - right.id;
      });
}

const withDenseMappingOrder = (items: OrderedMapping[]): OrderedMapping[] => {
  return items.map((mapping, index) => ({
    ...mapping,
    order: (index + 1) * 10
  }));
}

const mappingRootToTreeChildren = (root: MappingRootNode | null, mappings: PathMapping[]): FileTreeData[] | undefined => {
  if (!root) return [];

  const mappingsByPath = new Map(
      sortMappings(mappings).map(mapping => [normalizePathText(mapping.mountPath), mapping])
  );

  const convertNode = (node: MappingRootNode): FileTreeData => {
    const normalized = normalizePathText(node.path);
    const mapping = mappingsByPath.get(normalized);
    const treeNode: FileTreeData = {
      path: normalized,
      name: node.name,
      isFile: false
    };
    if (mapping) {
      treeNode.mappingId = mapping.id;
      treeNode.mappingOrder = mapping.order ?? 0;
    }
    if (node.type === "virtual") {
      treeNode.virtual = true;
      treeNode.children = node.children.map(convertNode);
    }
    return treeNode;
  }

  if (normalizePathText(root.path) === "/") {
    if (root.type === "virtual") return root.children.map(convertNode);
    return undefined;
  }

  return [convertNode(root)];
}

const findNodeByPath = (nodes: FileTreeData[], path: string): FileTreeData | null => {
  const normalized = normalizePathText(path);
  for (const node of nodes) {
    if (normalizePathText(node.path) === normalized) return node;
    const child = node.children ? findNodeByPath(node.children, normalized) : null;
    if (child) return child;
  }
  return null;
}

const reorderTreeNodes = (
    nodes: FileTreeData[],
    sourcePath: string,
    targetPath: string,
    placement: MountDropPlacement
): FileTreeData[] => {
  const normalizedSource = normalizePathText(sourcePath);
  const normalizedTarget = normalizePathText(targetPath);
  let changed = false;

  const visit = (items: FileTreeData[]): FileTreeData[] => {
    const sourceIndex = items.findIndex(item => normalizePathText(item.path) === normalizedSource);
    const targetIndex = items.findIndex(item => normalizePathText(item.path) === normalizedTarget);
    if (sourceIndex >= 0 && targetIndex >= 0) {
      const next = [...items];
      const [moved] = next.splice(sourceIndex, 1);
      const nextTargetIndex = next.findIndex(item => normalizePathText(item.path) === normalizedTarget);
      if (!moved || nextTargetIndex < 0) return items;
      next.splice(placement === "after" ? nextTargetIndex + 1 : nextTargetIndex, 0, moved);
      changed = true;
      return next;
    }

    return items.map(item => {
      if (!item.children?.length) return item;
      const children = visit(item.children);
      return children === item.children ? item : {...item, children};
    });
  }

  const next = visit(nodes);
  return changed ? next : nodes;
}

export const useFileTreeLoader = ({
  getFolderData,
  getMappings,
  getMappingRoot,
  reorderMappings,
  navigateToPath,
  showError
}: FileTreeLoaderOptions) => {
  const fileStore = useFileStore();
  const treeData = ref<FileTreeData[]>([]);
  const mountMappings = ref<OrderedMapping[]>([]);

  const loadRoot = async (_options: {forceRefresh?: boolean} = {}) => {
    const [root, mappings] = await Promise.all([getMappingRoot(), getMappings()]);
    mountMappings.value = sortMappings(mappings);
    treeData.value = [rootTreeNode(mappingRootToTreeChildren(root, mappings))];
  }

  const handleLoad = async (node: FileTreeData, options: {navigate?: boolean; focusExplorer?: boolean; refresh?: boolean} = {}) => {
    if (node.isFile) return false;
    if (options.navigate !== false && !await fileStore.requestEditorLeave()) return false;
    try {
      let loadedPath = node.path;
      if (options.refresh && node.virtual) {
        await loadRoot({forceRefresh: true});
      } else if (options.refresh || node.children === undefined) {
        const data = await getFolderData(node.path, treeQuery, {forceRefresh: options.refresh});
        fileStore.saveFolderData(data);
        node.children = folderDataToTreeNodes(data);
        loadedPath = data.path || node.path;
      }
      if (options.navigate !== false) {
        await navigateToPath(loadedPath, {skipEditorLeave: true, focusExplorer: options.focusExplorer ?? true});
      }
      return true;
    } catch (error) {
      showError(error, "加载目录失败");
      return false;
    }
  }

  const reorderMount = async (source: FileTreeData, target: FileTreeData, placement: MountDropPlacement) => {
    const sourceId = source.mappingId;
    const targetId = target.mappingId;
    if (typeof sourceId !== "number" || typeof targetId !== "number") return false;
    if (sourceId === targetId) return true;
    if (parentPath(source.path) !== parentPath(target.path)) return false;

    const previousMappings = mountMappings.value;
    const sourceIndex = previousMappings.findIndex(mapping => mapping.id === sourceId);
    const targetIndex = previousMappings.findIndex(mapping => mapping.id === targetId);
    if (sourceIndex < 0 || targetIndex < 0) return false;

    const next = [...previousMappings];
    const [moved] = next.splice(sourceIndex, 1);
    const nextTargetIndex = next.findIndex(mapping => mapping.id === targetId);
    if (!moved || nextTargetIndex < 0) return false;

    next.splice(placement === "after" ? nextTargetIndex + 1 : nextTargetIndex, 0, moved);
    const ordered = withDenseMappingOrder(next);
    const previousTree = treeData.value;
    mountMappings.value = ordered;
    treeData.value = reorderTreeNodes(previousTree, source.path, target.path, placement);

    try {
      await reorderMappings(ordered.map(({id, order}) => ({id, order: order ?? 0})));
      await loadRoot({forceRefresh: true});
      return true;
    } catch (error) {
      mountMappings.value = previousMappings;
      treeData.value = previousTree;
      showError(error, "调整目录树顺序失败", "目录树");
      return false;
    }
  }

  const refreshPath = async (path: string) => {
    const normalized = normalizePathText(path);
    try {
      if (normalized === "/") {
        await loadRoot({forceRefresh: true});
        return true;
      }
      const node = findNodeByPath(treeData.value, normalized);
      if (!node || node.isFile) return false;
      if (node.virtual) {
        await loadRoot({forceRefresh: true});
        return true;
      }
      const data = await getFolderData(node.path, treeQuery, {forceRefresh: true});
      fileStore.saveFolderData(data);
      node.children = folderDataToTreeNodes(data);
      return true;
    } catch (error) {
      showError(error, "刷新文件树失败");
      return false;
    }
  }

  return {
    treeData,
    loadRoot,
    handleLoad,
    reorderMount,
    refreshPath
  };
}
