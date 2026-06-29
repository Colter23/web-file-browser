import {translate, translateOptional} from "../i18n";
import type {TranslateParams} from "../i18n";

export type ApiErrorPayload = {
  code?: string;
  reason?: string;
  message?: string;
  params?: Record<string, unknown>;
}

const normalizeErrorKey = (value?: string) => value?.trim().toUpperCase().replace(/[^A-Z0-9_]+/g, "_") || "";

export const normalizeApiErrorParams = (params?: Record<string, unknown>): TranslateParams => {
  if (!params) return {};
  return Object.fromEntries(Object.entries(params).map(([key, value]) => {
    if (typeof value === "string" || typeof value === "number" || typeof value === "boolean" || value === null || value === undefined) {
      return [key, value];
    }
    return [key, String(value)];
  }));
}

export const parseApiErrorPayload = (data: unknown): ApiErrorPayload | undefined => {
  if (!data || typeof data !== "object") return undefined;
  const source = data as Record<string, unknown>;
  const params = source.params && typeof source.params === "object" && !Array.isArray(source.params)
      ? source.params as Record<string, unknown>
      : undefined;
  const payload: ApiErrorPayload = {
    code: typeof source.code === "string" ? source.code : undefined,
    reason: typeof source.reason === "string" ? source.reason : undefined,
    message: typeof source.message === "string" ? source.message : undefined,
    params
  };
  return payload.code || payload.reason || payload.message ? payload : undefined;
}

export const apiErrorMessage = (
  payload: ApiErrorPayload | undefined,
  fallback = translate("common.requestFailed")
) => {
  if (!payload) return fallback;
  const params = normalizeApiErrorParams(payload.params);
  const reason = normalizeErrorKey(payload.reason);
  if (reason) {
    const message = translateOptional(`apiError.reason.${reason}`, params);
    if (message) return message;
  }
  const code = normalizeErrorKey(payload.code);
  if (code) {
    const message = translateOptional(`apiError.code.${code}`, params);
    if (message) return message;
  }
  return payload.message?.trim() || fallback;
}

