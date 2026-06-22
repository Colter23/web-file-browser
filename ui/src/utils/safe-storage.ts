export const readStorageItem = (key: string): string | null => {
  if (typeof localStorage === "undefined") return null;
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

export const writeStorageItem = (key: string, value: string) => {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(key, value);
  } catch {
    // 本地存储不可用时，保持本次会话内状态即可。
  }
}

export const readBooleanStorage = (key: string, fallback: boolean) => {
  const value = readStorageItem(key);
  if (value === "true") return true;
  if (value === "false") return false;
  return fallback;
}

export const writeBooleanStorage = (key: string, value: boolean) => {
  writeStorageItem(key, String(value));
}

export const readNumberStorage = (key: string, fallback: number) => {
  const value = readStorageItem(key);
  if (value === null || value.trim() === "") return fallback;
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : fallback;
}

export const writeNumberStorage = (key: string, value: number) => {
  writeStorageItem(key, String(value));
}

export const readJsonStorage = <T>(key: string, fallback: T): T => {
  const value = readStorageItem(key);
  if (!value) return fallback;
  try {
    return JSON.parse(value) as T;
  } catch {
    return fallback;
  }
}

export const writeJsonStorage = <T>(key: string, value: T) => {
  try {
    writeStorageItem(key, JSON.stringify(value));
  } catch {
    // 无法序列化时，保留当前会话内状态即可。
  }
}
