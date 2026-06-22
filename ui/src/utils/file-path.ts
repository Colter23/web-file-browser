export const normalizePathText = (path: string) => {
  let normalized = (path.trim() || "/").replace(/\\/g, "/");
  normalized = normalized.replace(/\/+$/g, "");
  while (normalized.includes("//")) normalized = normalized.replace(/\/+/g, "/");
  if (!normalized.startsWith("/")) normalized = `/${normalized}`;
  return normalized || "/";
}

export const parentPath = (path: string) => {
  const parts = normalizePathText(path).split("/").filter(Boolean);
  parts.pop();
  return parts.length ? `/${parts.join("/")}` : "/";
}

export const joinPath = (base: string, name: string) => {
  const normalizedBase = normalizePathText(base);
  return normalizedBase === "/" ? `/${name}` : `${normalizedBase}/${name}`;
}

export const isSameOrDescendantPath = (path: string, parent: string) => {
  const normalizedPath = normalizePathText(path);
  const normalizedParent = normalizePathText(parent);
  if (normalizedParent === "/") return true;
  return normalizedPath === normalizedParent || normalizedPath.startsWith(`${normalizedParent}/`);
}
