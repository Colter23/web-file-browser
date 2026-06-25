import axios, {AxiosRequestConfig, AxiosResponse} from "axios";
import config from "../config";

export class ApiError extends Error {
    status?: number;
    code?: string;
    data?: unknown;

    constructor(message: string, status?: number, code?: string, data?: unknown) {
        super(message);
        this.name = "ApiError";
        this.status = status;
        this.code = code;
        this.data = data;
    }
}

export const isApiError = (error: unknown): error is ApiError => error instanceof ApiError;

const instance = axios.create({
    baseURL: config.BASE_URL,
    withCredentials: true
})

instance.interceptors.response.use(
    response => response,
    error => {
        const status = error.response?.status;
        const data = typeof error.response?.data === "string"
            ? parseErrorBody(error.response.data)
            : error.response?.data;
        const code = typeof data === "object" && data !== null && "code" in data
            ? String(data.code)
            : undefined;
        const message = typeof data === "object" && data !== null && "message" in data
            ? String(data.message)
            : error.message ?? "请求失败";
        if (status === 401 && window.location.pathname !== "/login") {
            const redirect = encodeURIComponent(window.location.pathname + window.location.search);
            window.location.href = `/login?redirect=${redirect}`;
        }
        return Promise.reject(new ApiError(message, status, code, data));
    }
)

const parseErrorBody = (body: string): { message?: string } | string => {
    try {
        return JSON.parse(body);
    } catch {
        return body;
    }
}

const get = async (url: string, config?: AxiosRequestConfig): Promise<AxiosResponse> => {
    return await instance.get(url, config)
}

const head = async (url: string, config?: AxiosRequestConfig): Promise<AxiosResponse> => {
    return await instance.head(url, config)
}

const post = async (url: string, data: unknown = {}, config?: AxiosRequestConfig): Promise<AxiosResponse> => {
    return await instance.post(url, data, config)
}

const put = async (url: string, data: unknown = {}, config?: AxiosRequestConfig): Promise<AxiosResponse> => {
    return await instance.put(url, data, config)
}

const patch = async (url: string, data: unknown = {}, config?: AxiosRequestConfig): Promise<AxiosResponse> => {
    return await instance.patch(url, data, config)
}

const del = async (url: string, config?: AxiosRequestConfig): Promise<AxiosResponse> => {
    return await instance.delete(url, config)
}

export default {
    instance,
    get,
    head,
    post,
    put,
    patch,
    delete: del
}
