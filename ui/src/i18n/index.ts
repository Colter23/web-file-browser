import {computed, ref} from "vue";
import {readStorageItem, writeStorageItem} from "../utils/safe-storage.ts";
import {messages} from "./messages.ts";
import type {LocaleCode, MessageKey} from "./messages.ts";

type TranslateParams = Record<string, string | number | boolean | null | undefined>;

const storageKey = "app.locale";
const fallbackLocale: LocaleCode = "zh-CN";
const localeCodes = Object.keys(messages) as LocaleCode[];

const isLocaleCode = (value: string | null): value is LocaleCode => {
  return Boolean(value && localeCodes.includes(value as LocaleCode));
}

const readLocale = (): LocaleCode => {
  const stored = readStorageItem(storageKey);
  return isLocaleCode(stored) ? stored : fallbackLocale;
}

const currentLocale = ref<LocaleCode>(readLocale());

export const localeOptions: {value: LocaleCode; labelKey: MessageKey}[] = [
  {value: "zh-CN", labelKey: "locale.zhCN"},
  {value: "en-US", labelKey: "locale.enUS"}
];

export const setLocale = (locale: LocaleCode) => {
  if (!localeCodes.includes(locale)) return;
  currentLocale.value = locale;
  writeStorageItem(storageKey, locale);
}

export const getLocale = () => currentLocale.value;

export const translate = (key: MessageKey, params: TranslateParams = {}) => {
  const template = messages[currentLocale.value][key] ?? messages[fallbackLocale][key] ?? key;
  return template.replace(/\{(\w+)}/g, (_, name: string) => {
    const value = params[name];
    return value === null || value === undefined ? "" : String(value);
  });
}

export const useI18n = () => {
  const locale = computed(() => currentLocale.value);

  return {
    locale,
    localeOptions,
    setLocale,
    t: translate
  };
}

export type {LocaleCode, MessageKey};
