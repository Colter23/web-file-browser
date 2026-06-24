import {ref} from "vue";
import type {FileTreeData, FolderData, FolderQueryParams} from "../class.ts";
import {useFileStore} from "../store";

type FileTreeLoaderOptions = {
  getFolderData: (path: string, params?: FolderQueryParams, options?: {forceRefresh?: boolean}) => Promise<FolderData>;
  navigateToPath: (path: string, options?: {skipEditorLeave?: boolean; focusExplorer?: boolean}) => Promise<boolean>;
  showError: (error: unknown, fallback: string, title?: string) => void;
}

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

const rootTreeNode = (children: FileTreeData[] = []): FileTreeData => ({
  path: "/",
  name: "主页",
  isFile: false,
  children
});

export const useFileTreeLoader = ({getFolderData, navigateToPath, showError}: FileTreeLoaderOptions) => {
  const fileStore = useFileStore();
  const treeData = ref<FileTreeData[]>([]);

  const loadRoot = async (options: {forceRefresh?: boolean} = {}) => {
    const data = await getFolderData("/", treeQuery, options);
    fileStore.saveFolderData(data);
    treeData.value = [rootTreeNode(folderDataToTreeNodes(data))];
  }

  const handleLoad = async (node: FileTreeData, options: {navigate?: boolean; focusExplorer?: boolean; refresh?: boolean} = {}) => {
    if (node.isFile) return false;
    if (options.navigate !== false && !await fileStore.requestEditorLeave()) return false;
    try {
      let loadedPath = node.path;
      if (options.refresh || node.children === undefined) {
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

  return {
    treeData,
    loadRoot,
    handleLoad
  };
}
