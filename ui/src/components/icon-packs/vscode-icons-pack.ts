import {defineComponent, h} from "vue";
import type {Component, PropType} from "vue";
import defaultFileIcon from "@iconify-icons/vscode-icons/default-file";
import defaultFolderIcon from "@iconify-icons/vscode-icons/default-folder";
import defaultFolderOpenedIcon from "@iconify-icons/vscode-icons/default-folder-opened";
import defaultRootFolderIcon from "@iconify-icons/vscode-icons/default-root-folder";
import fileTypeAudioIcon from "@iconify-icons/vscode-icons/file-type-audio";
import fileTypeBatIcon from "@iconify-icons/vscode-icons/file-type-bat";
import fileTypeBinaryIcon from "@iconify-icons/vscode-icons/file-type-binary";
import fileTypeCIcon from "@iconify-icons/vscode-icons/file-type-c";
import fileTypeConfigIcon from "@iconify-icons/vscode-icons/file-type-config";
import fileTypeCppIcon from "@iconify-icons/vscode-icons/file-type-cpp";
import fileTypeCsharpIcon from "@iconify-icons/vscode-icons/file-type-csharp";
import fileTypeCssIcon from "@iconify-icons/vscode-icons/file-type-css";
import fileTypeDbIcon from "@iconify-icons/vscode-icons/file-type-db";
import fileTypeDockerIcon from "@iconify-icons/vscode-icons/file-type-docker";
import fileTypeDotenvIcon from "@iconify-icons/vscode-icons/file-type-dotenv";
import fileTypeExcelIcon from "@iconify-icons/vscode-icons/file-type-excel";
import fileTypeFontIcon from "@iconify-icons/vscode-icons/file-type-font";
import fileTypeGoIcon from "@iconify-icons/vscode-icons/file-type-go";
import fileTypeHtmlIcon from "@iconify-icons/vscode-icons/file-type-html";
import fileTypeImageIcon from "@iconify-icons/vscode-icons/file-type-image";
import fileTypeJavaIcon from "@iconify-icons/vscode-icons/file-type-java";
import fileTypeJsIcon from "@iconify-icons/vscode-icons/file-type-js";
import fileTypeJsonIcon from "@iconify-icons/vscode-icons/file-type-json";
import fileTypeLogIcon from "@iconify-icons/vscode-icons/file-type-log";
import fileTypeLuaIcon from "@iconify-icons/vscode-icons/file-type-lua";
import fileTypeMarkdownIcon from "@iconify-icons/vscode-icons/file-type-markdown";
import fileTypeNginxIcon from "@iconify-icons/vscode-icons/file-type-nginx";
import fileTypePdfIcon from "@iconify-icons/vscode-icons/file-type-pdf2";
import fileTypePowerpointIcon from "@iconify-icons/vscode-icons/file-type-powerpoint";
import fileTypePowershellIcon from "@iconify-icons/vscode-icons/file-type-powershell";
import fileTypePythonIcon from "@iconify-icons/vscode-icons/file-type-python";
import fileTypeReactIcon from "@iconify-icons/vscode-icons/file-type-reactjs";
import fileTypeRustIcon from "@iconify-icons/vscode-icons/file-type-rust";
import fileTypeSassIcon from "@iconify-icons/vscode-icons/file-type-sass";
import fileTypeScssIcon from "@iconify-icons/vscode-icons/file-type-scss";
import fileTypeShellIcon from "@iconify-icons/vscode-icons/file-type-shell";
import fileTypeSqlIcon from "@iconify-icons/vscode-icons/file-type-sql";
import fileTypeSqliteIcon from "@iconify-icons/vscode-icons/file-type-sqlite";
import fileTypeSvgIcon from "@iconify-icons/vscode-icons/file-type-svg";
import fileTypeTextIcon from "@iconify-icons/vscode-icons/file-type-text";
import fileTypeTomlIcon from "@iconify-icons/vscode-icons/file-type-toml";
import fileTypeTypescriptIcon from "@iconify-icons/vscode-icons/file-type-typescript";
import fileTypeVideoIcon from "@iconify-icons/vscode-icons/file-type-video";
import fileTypeVueIcon from "@iconify-icons/vscode-icons/file-type-vue";
import fileTypeWordIcon from "@iconify-icons/vscode-icons/file-type-word";
import fileTypeXmlIcon from "@iconify-icons/vscode-icons/file-type-xml";
import fileTypeYamlIcon from "@iconify-icons/vscode-icons/file-type-yaml";
import fileTypeZipIcon from "@iconify-icons/vscode-icons/file-type-zip";
import type {AppIconPack} from "./types.ts";

interface IconifyIconData {
  body: string;
  width?: number;
  height?: number;
  left?: number;
  top?: number;
}

const VscodeInlineSvgIcon = defineComponent({
  name: "VscodeInlineSvgIcon",
  inheritAttrs: false,
  props: {
    icon: {
      type: Object as PropType<IconifyIconData>,
      required: true
    }
  },
  setup(props, {attrs}) {
    return () => {
      const left = props.icon.left ?? 0;
      const top = props.icon.top ?? 0;
      const width = props.icon.width ?? 32;
      const height = props.icon.height ?? 32;
      return h("svg", {
        ...attrs,
        xmlns: "http://www.w3.org/2000/svg",
        viewBox: `${left} ${top} ${width} ${height}`,
        innerHTML: props.icon.body
      });
    };
  }
});

const vscodeIcons: Record<string, IconifyIconData> = {
  "file.home": defaultRootFolderIcon,
  "file.folder": defaultFolderIcon,
  "file.folder-open": defaultFolderOpenedIcon,
  "file.file": defaultFileIcon,
  "file.image": fileTypeImageIcon,
  "file.text": fileTypeTextIcon,
  "file.code": fileTypeJsIcon,
  "file.config": fileTypeConfigIcon,
  "file.archive": fileTypeZipIcon,
  "file.audio": fileTypeAudioIcon,
  "file.video": fileTypeVideoIcon,
  "file.pdf": fileTypePdfIcon,
  "file.spreadsheet": fileTypeExcelIcon,
  "file.document": fileTypeWordIcon,
  "file.presentation": fileTypePowerpointIcon,
  "file.executable": fileTypeBinaryIcon,
  "file.shortcut": defaultFileIcon,
  "file.database": fileTypeDbIcon,
  "file.font": fileTypeFontIcon,
  "file.package": fileTypeZipIcon,
  "file.markup": fileTypeHtmlIcon,
  "file.unknown": defaultFileIcon,
  "file.generic": defaultFileIcon,

  "icon-homefill": defaultRootFolderIcon,
  "icon-home-fill": defaultRootFolderIcon,
  "icon-file": defaultFileIcon,
  "icon-file-fill": defaultFileIcon,
  "icon-file-common-filling": fileTypeTextIcon,
  "icon-file-image-fill": fileTypeImageIcon,
  "icon-file-zip": fileTypeZipIcon,
  "icon-file-zip-fill": fileTypeZipIcon,
  "icon-folder": defaultFolderIcon,
  "icon-folder-fill": defaultFolderIcon,
  "icon-folder-open-fill": defaultFolderOpenedIcon
};

const extensionIcons: Record<string, IconifyIconData> = {
  "7z": fileTypeZipIcon,
  avi: fileTypeVideoIcon,
  bash: fileTypeShellIcon,
  bat: fileTypeBatIcon,
  bin: fileTypeBinaryIcon,
  bmp: fileTypeImageIcon,
  c: fileTypeCIcon,
  cc: fileTypeCppIcon,
  cmd: fileTypeBatIcon,
  conf: fileTypeConfigIcon,
  config: fileTypeConfigIcon,
  cpp: fileTypeCppIcon,
  cs: fileTypeCsharpIcon,
  css: fileTypeCssIcon,
  csv: fileTypeExcelIcon,
  cts: fileTypeTypescriptIcon,
  cxx: fileTypeCppIcon,
  db: fileTypeDbIcon,
  dll: fileTypeBinaryIcon,
  doc: fileTypeWordIcon,
  docx: fileTypeWordIcon,
  dockerfile: fileTypeDockerIcon,
  eot: fileTypeFontIcon,
  env: fileTypeDotenvIcon,
  exe: fileTypeBinaryIcon,
  fish: fileTypeShellIcon,
  flac: fileTypeAudioIcon,
  gif: fileTypeImageIcon,
  go: fileTypeGoIcon,
  gz: fileTypeZipIcon,
  h: fileTypeCIcon,
  hpp: fileTypeCppIcon,
  htm: fileTypeHtmlIcon,
  html: fileTypeHtmlIcon,
  ini: fileTypeConfigIcon,
  java: fileTypeJavaIcon,
  jpeg: fileTypeImageIcon,
  jpg: fileTypeImageIcon,
  js: fileTypeJsIcon,
  json: fileTypeJsonIcon,
  jsx: fileTypeReactIcon,
  ksh: fileTypeShellIcon,
  log: fileTypeLogIcon,
  lua: fileTypeLuaIcon,
  m4a: fileTypeAudioIcon,
  markdown: fileTypeMarkdownIcon,
  md: fileTypeMarkdownIcon,
  mkv: fileTypeVideoIcon,
  mov: fileTypeVideoIcon,
  mp3: fileTypeAudioIcon,
  mp4: fileTypeVideoIcon,
  msi: fileTypeBinaryIcon,
  mts: fileTypeTypescriptIcon,
  nginx: fileTypeNginxIcon,
  ods: fileTypeExcelIcon,
  odt: fileTypeWordIcon,
  ogg: fileTypeAudioIcon,
  otf: fileTypeFontIcon,
  pdf: fileTypePdfIcon,
  png: fileTypeImageIcon,
  ppt: fileTypePowerpointIcon,
  pptx: fileTypePowerpointIcon,
  properties: fileTypeConfigIcon,
  ps1: fileTypePowershellIcon,
  py: fileTypePythonIcon,
  rar: fileTypeZipIcon,
  rs: fileTypeRustIcon,
  rtf: fileTypeWordIcon,
  sass: fileTypeSassIcon,
  scss: fileTypeScssIcon,
  sh: fileTypeShellIcon,
  sql: fileTypeSqlIcon,
  sqlite: fileTypeSqliteIcon,
  sqlite3: fileTypeSqliteIcon,
  svg: fileTypeSvgIcon,
  tar: fileTypeZipIcon,
  tgz: fileTypeZipIcon,
  toml: fileTypeTomlIcon,
  ts: fileTypeTypescriptIcon,
  tsx: fileTypeTypescriptIcon,
  ttf: fileTypeFontIcon,
  txt: fileTypeTextIcon,
  vue: fileTypeVueIcon,
  wav: fileTypeAudioIcon,
  webm: fileTypeVideoIcon,
  webp: fileTypeImageIcon,
  woff: fileTypeFontIcon,
  woff2: fileTypeFontIcon,
  xls: fileTypeExcelIcon,
  xlsm: fileTypeExcelIcon,
  xlsx: fileTypeExcelIcon,
  xml: fileTypeXmlIcon,
  yaml: fileTypeYamlIcon,
  yml: fileTypeYamlIcon,
  zip: fileTypeZipIcon,
  zsh: fileTypeShellIcon
};

const vscodeComponent = VscodeInlineSvgIcon as Component;

const resolveSpecificFileIcon = (icon: string) => {
  const match = /^file\.([^.]+)\.([^.]+)$/.exec(icon);
  if (!match) return undefined;
  const [, kind, extension] = match;
  return extensionIcons[extension] ?? vscodeIcons[`file.${kind}`];
};

export const vscodeIconsPack: AppIconPack = {
  resolve(icon) {
    const iconData = vscodeIcons[icon] ?? resolveSpecificFileIcon(icon);
    return iconData
        ? {kind: "component", component: vscodeComponent, props: {icon: iconData}, className: "app-icon-vscode-icons"}
        : undefined;
  }
};
