import network from "../network";
import {SessionResponse} from "../class";

export const login = async (password: string): Promise<SessionResponse> => {
    return (await network.post("/api/auth/login", {password})).data
}

export const logout = async (): Promise<SessionResponse> => {
    return (await network.post("/api/auth/logout")).data
}

export const changePassword = async (
    currentPassword: string,
    newPassword: string
): Promise<SessionResponse> => {
    return (await network.post("/api/auth/password", {currentPassword, newPassword})).data
}

export const getSession = async (): Promise<SessionResponse> => {
    return (await network.get("/api/auth/session")).data
}
