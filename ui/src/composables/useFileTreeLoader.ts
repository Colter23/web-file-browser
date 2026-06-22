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

  const handleLoad = (node: FileTreeData) => {
    return new Promise<void>(async resolve => {
      if (!await fileStore.requestEditorLeave()) {
        resolve();
        return;
      }
      try {
        const data = await getFolderData(node.path);
        node.children = fileStore.saveAndConvertFolderData(data);
        await navigateToPath(data.path, {skipEditorLeave: true});
      } catch (error) {
        showError(error, "加载目录失败");
      }
      resolve();
    });
  }

  return {
    treeData,
    loadRoot,
    handleLoad
  };
}
