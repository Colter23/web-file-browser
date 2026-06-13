import axios, {AxiosRequestConfig, AxiosResponse} from "axios";
import config from "../config";

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
        const message =
            typeof data === "object" && data !== null && "message" in data
                ? String(data.message)
                : error.message ?? "请求失败";
        if (status === 401 && window.location.pathname !== "/login") {
            const redirect = encodeURIComponent(window.location.pathname + window.location.search);
            window.location.href = `/login?redirect=${redirect}`;
        }
        return Promise.reject(new Error(message));
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
    post,
    put,
    patch,
    delete: del
}
