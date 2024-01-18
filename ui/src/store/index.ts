import {defineStore} from "pinia";
import {FileInfo, FileTreeData, FolderData, FolderInfo} from "../class.ts";


declare type FileState = {
    currentFolder: string;
    folderData: Map<string, FolderData>;
}

export const useFileStore = defineStore('file', {
    state: (): FileState => ({
        // 当前文件路径
        currentFolder: '' as string,
        // 文件夹数据  key为文件夹路径
        folderData: new Map() as Map<string, FolderData>,
    }),
    actions: {
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
