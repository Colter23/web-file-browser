import network from "../network";
import type {MappingRootNode, PathMapping, ReorderMappingItem} from "../class";

export const getMappings = async (): Promise<PathMapping[]> => {
    return (await network.get("/api/mapping")).data
}

export const getMappingRoot = async (): Promise<MappingRootNode | null> => {
    return (await network.get("/api/mapping/root")).data
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

export const reorderMappings = async (items: ReorderMappingItem[]): Promise<void> => {
    await network.post("/api/mapping/reorder", {items})
}
