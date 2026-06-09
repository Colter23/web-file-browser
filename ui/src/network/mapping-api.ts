import network from "../network";
import {PathMapping} from "../class";

export const getMappings = async (): Promise<PathMapping[]> => {
    return (await network.get("/api/mapping")).data
}

export const createMapping = async (mapping: PathMapping): Promise<number> => {
    return (await network.post("/api/mapping", mapping)).data
}

export const updateMapping = async (id: number, mapping: PathMapping): Promise<void> => {
    await network.put(`/api/mapping/${id}`, mapping)
}

export const deleteMapping = async (id: number): Promise<void> => {
    await network.delete(`/api/mapping/${id}`)
}
