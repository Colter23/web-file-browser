import network from "../network";
import {RuntimeSettings} from "../class";

export const getSettings = async (): Promise<RuntimeSettings> => {
    return (await network.get("/api/settings")).data
}
