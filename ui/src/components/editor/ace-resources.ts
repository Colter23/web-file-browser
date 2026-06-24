import ace from "ace-builds";
import cssWorkerUrl from "ace-builds/src-noconflict/worker-css?url";
import htmlWorkerUrl from "ace-builds/src-noconflict/worker-html?url";
import javascriptWorkerUrl from "ace-builds/src-noconflict/worker-javascript?url";
import luaWorkerUrl from "ace-builds/src-noconflict/worker-lua?url";
import phpWorkerUrl from "ace-builds/src-noconflict/worker-php?url";

type AceResourceLoader = () => Promise<unknown>;

const FALLBACK_MODE = "plain_text";
const FALLBACK_THEME = "github";

const modeAliases: Record<string, string> = {
  log: FALLBACK_MODE,
  text: FALLBACK_MODE
};

const modeLoaders: Record<string, AceResourceLoader> = {
  apache_conf: () => import("ace-builds/src-noconflict/mode-apache_conf"),
  asciidoc: () => import("ace-builds/src-noconflict/mode-asciidoc"),
  c_cpp: () => import("ace-builds/src-noconflict/mode-c_cpp"),
  csharp: () => import("ace-builds/src-noconflict/mode-csharp"),
  css: () => import("ace-builds/src-noconflict/mode-css"),
  dart: () => import("ace-builds/src-noconflict/mode-dart"),
  dockerfile: () => import("ace-builds/src-noconflict/mode-dockerfile"),
  golang: () => import("ace-builds/src-noconflict/mode-golang"),
  groovy: () => import("ace-builds/src-noconflict/mode-groovy"),
  html: () => import("ace-builds/src-noconflict/mode-html"),
  ini: () => import("ace-builds/src-noconflict/mode-ini"),
  java: () => import("ace-builds/src-noconflict/mode-java"),
  javascript: () => import("ace-builds/src-noconflict/mode-javascript"),
  json: () => import("ace-builds/src-noconflict/mode-json"),
  jsp: () => import("ace-builds/src-noconflict/mode-jsp"),
  kotlin: () => import("ace-builds/src-noconflict/mode-kotlin"),
  lua: () => import("ace-builds/src-noconflict/mode-lua"),
  markdown: () => import("ace-builds/src-noconflict/mode-markdown"),
  matlab: () => import("ace-builds/src-noconflict/mode-matlab"),
  mysql: () => import("ace-builds/src-noconflict/mode-mysql"),
  nginx: () => import("ace-builds/src-noconflict/mode-nginx"),
  pgsql: () => import("ace-builds/src-noconflict/mode-pgsql"),
  php: () => import("ace-builds/src-noconflict/mode-php"),
  plain_text: () => import("ace-builds/src-noconflict/mode-plain_text"),
  powershell: () => import("ace-builds/src-noconflict/mode-powershell"),
  properties: () => import("ace-builds/src-noconflict/mode-properties"),
  protobuf: () => import("ace-builds/src-noconflict/mode-protobuf"),
  python: () => import("ace-builds/src-noconflict/mode-python"),
  ruby: () => import("ace-builds/src-noconflict/mode-ruby"),
  rust: () => import("ace-builds/src-noconflict/mode-rust"),
  sql: () => import("ace-builds/src-noconflict/mode-sql"),
  swift: () => import("ace-builds/src-noconflict/mode-swift"),
  toml: () => import("ace-builds/src-noconflict/mode-toml"),
  typescript: () => import("ace-builds/src-noconflict/mode-typescript"),
  xml: () => import("ace-builds/src-noconflict/mode-xml"),
  yaml: () => import("ace-builds/src-noconflict/mode-yaml")
};

const themeLoaders: Record<string, AceResourceLoader> = {
  chaos: () => import("ace-builds/src-noconflict/theme-chaos"),
  chrome: () => import("ace-builds/src-noconflict/theme-chrome"),
  clouds: () => import("ace-builds/src-noconflict/theme-clouds"),
  cobalt: () => import("ace-builds/src-noconflict/theme-cobalt"),
  crimson_editor: () => import("ace-builds/src-noconflict/theme-crimson_editor"),
  dracula: () => import("ace-builds/src-noconflict/theme-dracula"),
  github: () => import("ace-builds/src-noconflict/theme-github"),
  github_dark: () => import("ace-builds/src-noconflict/theme-github_dark"),
  iplastic: () => import("ace-builds/src-noconflict/theme-iplastic"),
  merbivore: () => import("ace-builds/src-noconflict/theme-merbivore"),
  merbivore_soft: () => import("ace-builds/src-noconflict/theme-merbivore_soft"),
  monokai: () => import("ace-builds/src-noconflict/theme-monokai"),
  nord_dark: () => import("ace-builds/src-noconflict/theme-nord_dark"),
  one_dark: () => import("ace-builds/src-noconflict/theme-one_dark"),
  pastel_on_dark: () => import("ace-builds/src-noconflict/theme-pastel_on_dark"),
  solarized_dark: () => import("ace-builds/src-noconflict/theme-solarized_dark"),
  solarized_light: () => import("ace-builds/src-noconflict/theme-solarized_light"),
  tomorrow: () => import("ace-builds/src-noconflict/theme-tomorrow"),
  tomorrow_night: () => import("ace-builds/src-noconflict/theme-tomorrow_night"),
  tomorrow_night_blue: () => import("ace-builds/src-noconflict/theme-tomorrow_night_blue"),
  vibrant_ink: () => import("ace-builds/src-noconflict/theme-vibrant_ink"),
  xcode: () => import("ace-builds/src-noconflict/theme-xcode")
};

const snippetLoaders: Record<string, AceResourceLoader> = {
  apache_conf: () => import("ace-builds/src-noconflict/snippets/apache_conf"),
  asciidoc: () => import("ace-builds/src-noconflict/snippets/asciidoc"),
  c_cpp: () => import("ace-builds/src-noconflict/snippets/c_cpp"),
  csharp: () => import("ace-builds/src-noconflict/snippets/csharp"),
  css: () => import("ace-builds/src-noconflict/snippets/css"),
  dart: () => import("ace-builds/src-noconflict/snippets/dart"),
  dockerfile: () => import("ace-builds/src-noconflict/snippets/dockerfile"),
  golang: () => import("ace-builds/src-noconflict/snippets/golang"),
  groovy: () => import("ace-builds/src-noconflict/snippets/groovy"),
  html: () => import("ace-builds/src-noconflict/snippets/html"),
  ini: () => import("ace-builds/src-noconflict/snippets/ini"),
  java: () => import("ace-builds/src-noconflict/snippets/java"),
  javascript: () => import("ace-builds/src-noconflict/snippets/javascript"),
  json: () => import("ace-builds/src-noconflict/snippets/json"),
  jsp: () => import("ace-builds/src-noconflict/snippets/jsp"),
  kotlin: () => import("ace-builds/src-noconflict/snippets/kotlin"),
  lua: () => import("ace-builds/src-noconflict/snippets/lua"),
  markdown: () => import("ace-builds/src-noconflict/snippets/markdown"),
  matlab: () => import("ace-builds/src-noconflict/snippets/matlab"),
  mysql: () => import("ace-builds/src-noconflict/snippets/mysql"),
  nginx: () => import("ace-builds/src-noconflict/snippets/nginx"),
  pgsql: () => import("ace-builds/src-noconflict/snippets/pgsql"),
  php: () => import("ace-builds/src-noconflict/snippets/php"),
  plain_text: () => import("ace-builds/src-noconflict/snippets/plain_text"),
  powershell: () => import("ace-builds/src-noconflict/snippets/powershell"),
  properties: () => import("ace-builds/src-noconflict/snippets/properties"),
  protobuf: () => import("ace-builds/src-noconflict/snippets/protobuf"),
  python: () => import("ace-builds/src-noconflict/snippets/python"),
  ruby: () => import("ace-builds/src-noconflict/snippets/ruby"),
  rust: () => import("ace-builds/src-noconflict/snippets/rust"),
  sql: () => import("ace-builds/src-noconflict/snippets/sql"),
  swift: () => import("ace-builds/src-noconflict/snippets/swift"),
  text: () => import("ace-builds/src-noconflict/snippets/text"),
  toml: () => import("ace-builds/src-noconflict/snippets/toml"),
  typescript: () => import("ace-builds/src-noconflict/snippets/typescript"),
  xml: () => import("ace-builds/src-noconflict/snippets/xml"),
  yaml: () => import("ace-builds/src-noconflict/snippets/yaml")
};

const workerUrls: Record<string, string> = {
  "ace/mode/base_worker": "/ace/worker-base.js",
  "ace/mode/css_worker": cssWorkerUrl,
  "ace/mode/html_worker": htmlWorkerUrl,
  "ace/mode/javascript_worker": javascriptWorkerUrl,
  "ace/mode/json_worker": "/ace/worker-json.js",
  "ace/mode/lua_worker": luaWorkerUrl,
  "ace/mode/php_worker": phpWorkerUrl,
  "ace/mode/xml_worker": "/ace/worker-xml.js",
  "ace/mode/yaml_worker": "/ace/worker-yaml.js"
};

const loadedResources = new Set<string>();
let registered = false;

const normalizeMode = (mode: string) => {
  const normalized = modeAliases[mode] ?? mode;
  return modeLoaders[normalized] ? normalized : FALLBACK_MODE;
}

const normalizeTheme = (theme: string) => {
  return themeLoaders[theme] ? theme : FALLBACK_THEME;
}

const loadOnce = async (id: string, loader: AceResourceLoader) => {
  if (loadedResources.has(id)) return;
  await loader();
  loadedResources.add(id);
}

const registerModuleLoader = (moduleId: string, loader: AceResourceLoader) => {
  ace.config.setModuleLoader(moduleId, loader);
}

export const registerAceResources = () => {
  if (registered) return;
  registered = true;

  Object.entries(modeLoaders).forEach(([mode, loader]) => {
    registerModuleLoader(`ace/mode/${mode}`, loader);
  });
  Object.entries(modeAliases).forEach(([alias, target]) => {
    registerModuleLoader(`ace/mode/${alias}`, modeLoaders[target] ?? modeLoaders[FALLBACK_MODE]);
  });
  Object.entries(themeLoaders).forEach(([theme, loader]) => {
    registerModuleLoader(`ace/theme/${theme}`, loader);
  });
  Object.entries(snippetLoaders).forEach(([snippet, loader]) => {
    registerModuleLoader(`ace/snippets/${snippet}`, loader);
  });
  Object.entries(workerUrls).forEach(([worker, url]) => {
    ace.config.setModuleUrl(worker, url);
  });
}

export const aceModePath = (mode: string) => `ace/mode/${normalizeMode(mode)}`;

export const aceThemePath = (theme: string) => `ace/theme/${normalizeTheme(theme)}`;

export const loadAceMode = async (mode: string) => {
  const normalized = normalizeMode(mode);
  await loadOnce(`mode:${normalized}`, modeLoaders[normalized]);
}

export const loadAceTheme = async (theme: string) => {
  const normalized = normalizeTheme(theme);
  await loadOnce(`theme:${normalized}`, themeLoaders[normalized]);
}
