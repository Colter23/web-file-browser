import editorConfig from "../assets/editor-config.json"

const log = (title: string, info: string, color: string = "#3b82f6") => {
  console.log(`%c[${title}]%c ${info}`, `color: #ffffff; background: ${color}; font-weight: bold; padding: 3px 5px 4px 5px; border-radius: 4px;`, "")
}

const checkFileLanguageMode = (extension: string): string => {
  let key = "";
  editorConfig.mode.forEach((mode) => {
    if (mode.extensions != undefined && mode.extensions.includes(extension)) {
      key = mode.key;
    }
  });
  return key;
}

export { log, checkFileLanguageMode };