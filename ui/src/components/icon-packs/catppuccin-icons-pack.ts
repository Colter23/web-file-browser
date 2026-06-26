import {defineComponent, h} from "vue";
import type {Component, PropType} from "vue";
import catppuccinCollection from "@iconify-json/catppuccin/icons.json";
import type {AppIconPack} from "./types.ts";

interface CatppuccinIconData {
  body: string;
  width?: number;
  height?: number;
  left?: number;
  top?: number;
}

interface CatppuccinIconCollection {
  icons: Record<string, CatppuccinIconData>;
  aliases?: Record<string, {parent: string} & Partial<CatppuccinIconData>>;
  width?: number;
  height?: number;
}

const collection = catppuccinCollection as CatppuccinIconCollection;
const icons = collection.icons;
const folderNames = new Set(
    Object.keys(icons)
        .filter(name => name.startsWith("folder-") && !name.endsWith("-open"))
        .map(name => name.slice("folder-".length))
);

const CatppuccinInlineSvgIcon = defineComponent({
  name: "CatppuccinInlineSvgIcon",
  inheritAttrs: false,
  props: {
    icon: {
      type: Object as PropType<CatppuccinIconData>,
      required: true
    }
  },
  setup(props, {attrs}) {
    return () => {
      const svgAttrs = {...attrs};
      delete svgAttrs.strokeWidth;
      delete svgAttrs["stroke-width"];
      const left = props.icon.left ?? 0;
      const top = props.icon.top ?? 0;
      const width = props.icon.width ?? collection.width ?? 16;
      const height = props.icon.height ?? collection.height ?? 16;
      return h("svg", {
        ...svgAttrs,
        xmlns: "http://www.w3.org/2000/svg",
        viewBox: `${left} ${top} ${width} ${height}`,
        innerHTML: props.icon.body
      });
    };
  }
});

const kindIcons: Record<string, string> = {
  "file.home": "root",
  "file.folder": "folder",
  "file.folder-open": "folder-open",
  "file.file": "file",
  "file.image": "image",
  "file.text": "text",
  "file.code": "javascript",
  "file.config": "config",
  "file.archive": "zip",
  "file.audio": "audio",
  "file.video": "video",
  "file.pdf": "pdf",
  "file.spreadsheet": "ms-excel",
  "file.document": "ms-word",
  "file.presentation": "ms-powerpoint",
  "file.executable": "exe",
  "file.shortcut": "file",
  "file.database": "database",
  "file.font": "font",
  "file.package": "package-json",
  "file.markup": "html",
  "file.unknown": "file",
  "file.generic": "file",

  "icon-homefill": "root",
  "icon-home-fill": "root",
  "icon-file": "file",
  "icon-file-fill": "file",
  "icon-file-common-filling": "text",
  "icon-file-image-fill": "image",
  "icon-file-zip": "zip",
  "icon-file-zip-fill": "zip",
  "icon-folder": "folder",
  "icon-folder-fill": "folder",
  "icon-folder-open-fill": "folder-open"
};

const extensionIcons: Record<string, string> = {
  "7z": "zip",
  avi: "video",
  bash: "bash",
  bat: "batch",
  bin: "binary",
  bmp: "image",
  c: "c",
  cc: "cpp",
  cmd: "batch",
  conf: "config",
  config: "config",
  cpp: "cpp",
  cs: "csharp",
  css: "css",
  csv: "csv",
  cts: "typescript",
  cxx: "cpp",
  db: "database",
  dll: "binary",
  doc: "ms-word",
  docx: "ms-word",
  dockerfile: "docker",
  eot: "font",
  env: "env",
  exe: "exe",
  fish: "bash",
  flac: "audio",
  gif: "image",
  go: "go",
  gz: "zip",
  h: "c-header",
  hpp: "cpp-header",
  htm: "html",
  html: "html",
  ini: "config",
  java: "java",
  jpeg: "image",
  jpg: "image",
  js: "javascript",
  json: "json",
  jsx: "javascript-react",
  ksh: "bash",
  log: "log",
  lua: "lua",
  m4a: "audio",
  markdown: "markdown",
  md: "markdown",
  mkv: "video",
  mov: "video",
  mp3: "audio",
  mp4: "video",
  msi: "binary",
  mts: "typescript",
  nginx: "nginx",
  ods: "ms-excel",
  odt: "ms-word",
  ogg: "audio",
  otf: "font",
  pdf: "pdf",
  png: "image",
  ppt: "ms-powerpoint",
  pptx: "ms-powerpoint",
  properties: "properties",
  ps1: "powershell",
  py: "python",
  rar: "zip",
  rs: "rust",
  rtf: "text",
  sass: "sass",
  scss: "sass",
  sh: "bash",
  sql: "database",
  sqlite: "database",
  sqlite3: "database",
  svg: "svg",
  tar: "zip",
  tgz: "zip",
  toml: "toml",
  ts: "typescript",
  tsx: "typescript-react",
  ttf: "font",
  txt: "text",
  vue: "vue",
  wav: "audio",
  webm: "video",
  webp: "image",
  woff: "font",
  woff2: "font",
  xls: "ms-excel",
  xlsm: "ms-excel",
  xlsx: "ms-excel",
  xml: "xml",
  yaml: "yaml",
  yml: "yaml",
  zip: "zip",
  zsh: "bash"
};

const folderAliases: Record<string, string> = {
  ".github": "github",
  ".gitlab": "gitlab",
  ".vscode": "vscode",
  conf: "config",
  configs: "config",
  doc: "docs",
  document: "docs",
  documents: "docs",
  helper: "utils",
  helpers: "utils",
  img: "images",
  image: "images",
  pictures: "images",
  script: "scripts",
  source: "src",
  test: "tests",
  tmp: "temp",
  uploads: "upload",
  util: "utils",
  video: "video",
  videos: "video"
};

const catppuccinComponent = CatppuccinInlineSvgIcon as Component;

const resolveIconData = (name: string) => {
  const iconData = icons[name];
  if (iconData) return iconData;
  const alias = collection.aliases?.[name];
  return alias ? {...icons[alias.parent], ...alias} : undefined;
};

const resolveFolderIconName = (kind: string, key: string) => {
  const normalizedKey = folderAliases[key] ?? key;
  if (!folderNames.has(normalizedKey)) return undefined;
  return kind === "folder-open" ? `folder-${normalizedKey}-open` : `folder-${normalizedKey}`;
};

const resolveSpecificFileIconName = (icon: string) => {
  const match = /^file\.([^.]+)\.([^.]+)$/.exec(icon);
  if (!match) return undefined;
  const [, kind, key] = match;
  if (kind === "folder" || kind === "folder-open") {
    return resolveFolderIconName(kind, key) ?? kindIcons[`file.${kind}`];
  }
  return extensionIcons[key] ?? kindIcons[`file.${kind}`];
};

export const catppuccinIconsPack: AppIconPack = {
  resolve(icon) {
    const iconName = kindIcons[icon] ?? resolveSpecificFileIconName(icon);
    const iconData = iconName ? resolveIconData(iconName) : undefined;
    return iconData
        ? {kind: "component", component: catppuccinComponent, props: {icon: iconData}, className: "app-icon-catppuccin"}
        : undefined;
  }
};
