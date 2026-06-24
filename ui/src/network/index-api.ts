import type {IndexStatus} from "../class";
import network from "../network";

export const getIndexStatus = async (): Promise<IndexStatus> => {
    return (await network.get("/api/index/status")).data
}

export const rebuildIndex = async (): Promise<void> => {
    await network.post("/api/index/rebuild")
}

export const cancelIndexRebuild = async (): Promise<void> => {
    await network.post("/api/index/cancel")
}
