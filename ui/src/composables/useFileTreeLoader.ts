import {ref} from "vue";
import type {FileTreeData, FolderData} from "../class.ts";
import {useFileStore} from "../store";

type FileTreeLoaderOptions = {
  getFolderData: (path: string) => Promise<FolderData>;
  navigateToPath: (path: string, options?: {skipEditorLeave?: boolean; focusExplorer?: boolean}) => Promise<boolean>;
  showError: (error: unknown, fallback: string, title?: string) => void;
}

export const useFileTreeLoader = ({getFolderData, navigateToPath, showError}: FileTreeLoaderOptions) => {
  const fileStore = useFileStore();
  const treeData = ref<FileTreeData[]>([]);

  const loadRoot = async () => {
    const data = await getFolderData("/");
    treeData.value = fileStore.saveAndConvertFolderData(data);
  }

  const handleLoad = async (node: FileTreeData) => {
    if (!await fileStore.requestEditorLeave()) return false;
    try {
      const data = await getFolderData(node.path);
      node.children = fileStore.saveAndConvertFolderData(data);
      await navigateToPath(data.path, {skipEditorLeave: true});
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
