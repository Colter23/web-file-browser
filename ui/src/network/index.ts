import axios, {AxiosRequestConfig, AxiosResponse} from "axios";
import config from "../config";
import {translate} from "../i18n";
import {apiErrorMessage, parseApiErrorPayload} from "../utils/api-error-message.ts";

export class ApiError extends Error {
    status?: number;
    code?: string;
    reason?: string;
    params?: Record<string, unknown>;
    backendMessage?: string;
    data?: unknown;

    constructor(
        message: string,
        status?: number,
        code?: string,
        data?: unknown,
        reason?: string,
        params?: Record<string, unknown>,
        backendMessage?: string
    ) {
        super(message);
        this.name = "ApiError";
        this.status = status;
        this.code = code;
        this.reason = reason;
        this.params = params;
        this.backendMessage = backendMessage;
        this.data = data;
    }
}

export const isApiError = (error: unknown): error is ApiError => error instanceof ApiError;

const instance = axios.create({
    baseURL: config.BASE_URL,
    withCredentials: true
})

const errorCodeFromStatus = (status?: number) => ({
    400: "BAD_REQUEST",
    401: "UNAUTHORIZED",
    403: "FORBIDDEN",
    404: "NOT_FOUND",
    405: "METHOD_NOT_ALLOWED",
    409: "CONFLICT",
    412: "PRECONDITION_FAILED",
    413: "PAYLOAD_TOO_LARGE",
    415: "UNSUPPORTED_MEDIA_TYPE",
    416: "RANGE_NOT_SATISFIABLE",
    428: "PRECONDITION_REQUIRED",
    429: "TOO_MANY_REQUESTS",
    500: "INTERNAL_ERROR"
}[status ?? 0]);

instance.interceptors.response.use(
    response => response,
    error => {
        const status = error.response?.status;
        const data = typeof error.response?.data === "string"
            ? parseErrorBody(error.response.data)
            : error.response?.data;
        const payload = parseApiErrorPayload(data);
        const code = payload?.code ?? errorCodeFromStatus(status);
        const normalizedPayload = payload
            ? {...payload, code}
            : code ? {code} : undefined;
        const message = apiErrorMessage(normalizedPayload, error.message ?? translate("common.requestFailed"));
        if (status === 401 && window.location.pathname !== "/login") {
            const redirect = encodeURIComponent(window.location.pathname + window.location.search);
            window.location.href = `/login?redirect=${redirect}`;
        }
        return Promise.reject(new ApiError(message, status, code, data, payload?.reason, payload?.params, payload?.message));
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
