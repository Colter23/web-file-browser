import network from "../network";
import type {SettingsResponse, UpdateSettingsRequest} from "../class";

export const getSettings = async (): Promise<SettingsResponse> => {
    return (await network.get("/api/settings")).data
}

export const updateSettings = async (request: UpdateSettingsRequest): Promise<SettingsResponse> => {
    return (await network.patch("/api/settings", request)).data
}

export const reloadSettings = async (): Promise<SettingsResponse> => {
    return (await network.post("/api/settings/reload")).data
}
