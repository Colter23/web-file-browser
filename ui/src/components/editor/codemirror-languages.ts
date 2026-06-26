import type {Extension} from "@codemirror/state";
import {StreamLanguage} from "@codemirror/language";

type LegacyLoader = () => Promise<Extension>;

const legacyLanguage = async <T>(loader: Promise<T>, key: keyof T) => {
  const module = await loader;
  return StreamLanguage.define(module[key] as never);
}

const legacyLoaders: Record<string, LegacyLoader> = {
  apache_conf: () => legacyLanguage(import("@codemirror/legacy-modes/mode/properties"), "properties"),
  c_cpp: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "cpp"),
  csharp: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "csharp"),
  dart: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "dart"),
  dockerfile: () => legacyLanguage(import("@codemirror/legacy-modes/mode/dockerfile"), "dockerFile"),
  golang: () => legacyLanguage(import("@codemirror/legacy-modes/mode/go"), "go"),
  groovy: () => legacyLanguage(import("@codemirror/legacy-modes/mode/groovy"), "groovy"),
  ini: () => legacyLanguage(import("@codemirror/legacy-modes/mode/properties"), "properties"),
  java: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "java"),
  kotlin: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "kotlin"),
  log: () => Promise.resolve([]),
  lua: () => legacyLanguage(import("@codemirror/legacy-modes/mode/lua"), "lua"),
  matlab: () => legacyLanguage(import("@codemirror/legacy-modes/mode/octave"), "octave"),
  nginx: () => legacyLanguage(import("@codemirror/legacy-modes/mode/nginx"), "nginx"),
  powershell: () => legacyLanguage(import("@codemirror/legacy-modes/mode/powershell"), "powerShell"),
  properties: () => legacyLanguage(import("@codemirror/legacy-modes/mode/properties"), "properties"),
  protobuf: () => legacyLanguage(import("@codemirror/legacy-modes/mode/protobuf"), "protobuf"),
  ruby: () => legacyLanguage(import("@codemirror/legacy-modes/mode/ruby"), "ruby"),
  swift: () => legacyLanguage(import("@codemirror/legacy-modes/mode/swift"), "swift"),
  toml: () => legacyLanguage(import("@codemirror/legacy-modes/mode/toml"), "toml")
};

export const loadCodeMirrorLanguage = async (mode: string): Promise<Extension> => {
  switch (mode) {
    case "css":
      return import("@codemirror/lang-css").then(module => module.css());
    case "html":
    case "jsp":
      return import("@codemirror/lang-html").then(module => module.html());
    case "javascript":
      return import("@codemirror/lang-javascript").then(module => module.javascript({jsx: true, typescript: false}));
    case "typescript":
      return import("@codemirror/lang-javascript").then(module => module.javascript({jsx: true, typescript: true}));
    case "json":
      return import("@codemirror/lang-json").then(module => module.json());
    case "markdown":
    case "asciidoc":
      return import("@codemirror/lang-markdown").then(module => module.markdown());
    case "mysql":
      return import("@codemirror/lang-sql").then(module => module.sql({dialect: module.MySQL}));
    case "pgsql":
      return import("@codemirror/lang-sql").then(module => module.sql({dialect: module.PostgreSQL}));
    case "sql":
      return import("@codemirror/lang-sql").then(module => module.sql());
    case "python":
      return import("@codemirror/lang-python").then(module => module.python());
    case "rust":
      return import("@codemirror/lang-rust").then(module => module.rust());
    case "xml":
      return import("@codemirror/lang-xml").then(module => module.xml());
    case "yaml":
      return import("@codemirror/lang-yaml").then(module => module.yaml());
    case "plain_text":
    case "text":
      return [];
    default:
      return legacyLoaders[mode]?.() ?? [];
  }
}
