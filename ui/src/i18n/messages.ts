import {enUS} from "./locales/en-US.ts";
import {zhCN} from "./locales/zh-CN.ts";

export const messages = {
  "zh-CN": zhCN,
  "en-US": enUS
} as const;

export type LocaleCode = keyof typeof messages;
export type MessageKey = keyof typeof messages["zh-CN"];
