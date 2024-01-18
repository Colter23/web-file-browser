import network from "../network";
import {FolderData} from "../class";

export const getFolderData = async (path: string = ""): Promise<FolderData> => {
    return (await network.get(`/api/file${path}`)).data
}
