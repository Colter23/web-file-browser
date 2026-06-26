import type {Extension} from "@codemirror/state";
import {StreamLanguage} from "@codemirror/language";
import type {StreamParser, StringStream} from "@codemirror/language";

type LegacyLoader = () => Promise<Extension>;

const logLevelErrorPattern = /^(?:FATAL|CRITICAL|SEVERE|ERROR|ERR)\b/i;
const logLevelWarnPattern = /^(?:WARN|WARNING)\b/i;
const logLevelInfoPattern = /^(?:INFO|NOTICE|SUCCESS|OK)\b/i;
const logLevelTracePattern = /^(?:DEBUG|TRACE|VERBOSE)\b/i;
const batchKeywordPattern = /^(?:assoc|break|call|cd|chcp|choice|cls|cmd|color|copy|date|del|dir|do|echo|else|endlocal|erase|exit|for|goto|if|in|md|mkdir|move|path|pause|popd|pushd|rd|rem|ren|rename|rmdir|set|setlocal|shift|start|time|title|type|ver|verify|xcopy|robocopy)\b/i;

// Logs can be very large and loosely structured, so keep this tokenizer line-local and cheap.
const logParser: StreamParser<unknown> = {
  name: "log",
  token(stream: StringStream) {
    if (stream.eatSpace()) return null;
    if (stream.match(/^\d{4}-\d\d-\d\d[T\s]\d\d:\d\d:\d\d(?:[.,]\d{1,6})?(?:Z|[+-]\d\d:?\d\d)?/)) return "number";
    if (stream.match(/^\d\d?:\d\d:\d\d(?:[.,]\d{1,6})?/)) return "number";
    if (stream.match(/^\[(?:FATAL|CRITICAL|SEVERE|ERROR|ERR)\]/i)) return "invalid";
    if (stream.match(/^\[(?:WARN|WARNING)\]/i)) return "string";
    if (stream.match(/^\[(?:INFO|NOTICE|SUCCESS|OK)\]/i)) return "keyword";
    if (stream.match(/^\[(?:DEBUG|TRACE|VERBOSE)\]/i)) return "comment";
    if (stream.match(logLevelErrorPattern)) return "invalid";
    if (stream.match(logLevelWarnPattern)) return "string";
    if (stream.match(logLevelInfoPattern)) return "keyword";
    if (stream.match(logLevelTracePattern)) return "comment";
    if (stream.match(/^\[[^\]\r\n]{1,120}\]/) || stream.match(/^\([^\)\r\n]{1,120}\)/)) return "variableName";
    if (stream.match(/^https?:\/\/[^\s)\]}]+/i)) return "link";
    if (stream.match(/^[A-Za-z]:[\\/][^\s)\]}]+/)) return "string";
    if (stream.match(/^(?:\.{0,2}[\\/])?[\w.-]+(?:[\\/][\w .-]+)+/)) return "string";
    if (stream.match(/^[A-Za-z_][\w.-]*(?=\s*[=:])/)) return "propertyName";
    if (stream.match(/^[+-]?\d+(?:\.\d+)?(?:ms|s|m|h|B|KB|MB|GB|%)?\b/i)) return "number";
    if (stream.match(/^[\w$-]+(?:\.[\w$-]+){2,}/)) return "typeName";
    if (stream.match(/^["'`][^"'`]*["'`]?/)) return "string";
    if (stream.match(/^[{}()[\],;:<>|]/)) return "operator";
    if (stream.match(/^[^\s{}()[\],;:<>"'`|]+/)) return null;
    stream.next();
    return null;
  }
};

const batchParser: StreamParser<unknown> = {
  name: "batch",
  token(stream: StringStream) {
    if (stream.eatSpace()) return null;
    if (stream.sol() && (stream.match(/^@?rem\b.*$/i) || stream.match(/^::.*$/))) return "comment";
    if (stream.match(/^:[A-Za-z_][\w.-]*/)) return "typeName";
    if (stream.match(/^%[A-Za-z0-9_*#?@$!~-]+%|^![A-Za-z0-9_*#?@$!~-]+!|^%[0-9*]/)) return "variableName";
    if (stream.match(batchKeywordPattern)) return "keyword";
    if (stream.match(/^\/[A-Za-z?]+/)) return "atom";
    if (stream.match(/^[+-]?\d+(?:\.\d+)?\b/)) return "number";
    if (stream.match(/^"(?:[^"]|"")*"?/)) return "string";
    if (stream.match(/^[&|<>^=()+,;]/)) return "operator";
    if (stream.match(/^[^\s&|<>^=()+,;"%!]+/)) return null;
    stream.next();
    return null;
  }
};

const legacyLanguage = async <T>(loader: Promise<T>, key: keyof T) => {
  const module = await loader;
  return StreamLanguage.define(module[key] as never);
}

const legacyLoaders: Record<string, LegacyLoader> = {
  apache_conf: () => legacyLanguage(import("@codemirror/legacy-modes/mode/properties"), "properties"),
  batch: () => Promise.resolve(StreamLanguage.define(batchParser)),
  c_cpp: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "cpp"),
  csharp: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "csharp"),
  dart: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "dart"),
  dockerfile: () => legacyLanguage(import("@codemirror/legacy-modes/mode/dockerfile"), "dockerFile"),
  golang: () => legacyLanguage(import("@codemirror/legacy-modes/mode/go"), "go"),
  groovy: () => legacyLanguage(import("@codemirror/legacy-modes/mode/groovy"), "groovy"),
  ini: () => legacyLanguage(import("@codemirror/legacy-modes/mode/properties"), "properties"),
  java: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "java"),
  kotlin: () => legacyLanguage(import("@codemirror/legacy-modes/mode/clike"), "kotlin"),
  log: () => Promise.resolve(StreamLanguage.define(logParser)),
  lua: () => legacyLanguage(import("@codemirror/legacy-modes/mode/lua"), "lua"),
  matlab: () => legacyLanguage(import("@codemirror/legacy-modes/mode/octave"), "octave"),
  nginx: () => legacyLanguage(import("@codemirror/legacy-modes/mode/nginx"), "nginx"),
  powershell: () => legacyLanguage(import("@codemirror/legacy-modes/mode/powershell"), "powerShell"),
  properties: () => legacyLanguage(import("@codemirror/legacy-modes/mode/properties"), "properties"),
  protobuf: () => legacyLanguage(import("@codemirror/legacy-modes/mode/protobuf"), "protobuf"),
  ruby: () => legacyLanguage(import("@codemirror/legacy-modes/mode/ruby"), "ruby"),
  shell: () => legacyLanguage(import("@codemirror/legacy-modes/mode/shell"), "shell"),
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
