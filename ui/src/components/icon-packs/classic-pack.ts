import type {AppIconPack} from "./types.ts";
import "../../assets/iconfont.js";

const classicSymbolIcons: Record<string, string> = {
  "action.upload": "icon-upload",
  "action.new-file": "icon-file-add-fill",
  "action.new-folder": "icon-folder-add-fill",
  "action.cut": "icon-contentcut",
  "action.copy": "icon-copy",
  "action.paste": "icon-paste",
  "action.download": "icon-download",
  "action.archive": "icon-file-zip-fill",
  "action.extract": "icon-download2",
  "action.preview": "icon-fenxiang",
  "action.rename": "icon-rename",
  "action.delete": "icon-delete-fill",
  "action.edit": "icon-edit-filling",
  "action.save": "icon-save-fill",
  "action.close": "icon-close",
  "action.add": "icon-add",
  "action.refresh": "icon-refresh",
  "action.settings": "icon-setting",
  "action.search": "icon-fenxiang",
  "action.share": "icon-fenxiang",
  "action.fullscreen": "icon-viewgrid",
  "action.exit-fullscreen": "icon-viewgrid",
  "action.previous": "icon-back_android",
  "action.next": "icon-back_android",
  "action.up": "icon-back_android",
  "action.down": "icon-unfold",
  "action.tools": "icon-wrench",

  "view.details": "icon-view-list",
  "view.list": "icon-listview",
  "view.icons": "icon-viewgrid",
  "view.tiles": "icon-viewgrid",
  "view.grid": "icon-viewgrid",
  "view.preview-pane": "icon-view-list",
  "view.preview-pane-close": "icon-view-list",
  "view.image": "icon-file-image-fill",

  "file.home": "icon-home-fill",
  "file.folder": "icon-folder-fill",
  "file.folder-open": "icon-folder-open-fill",
  "file.file": "icon-file-fill",
  "file.image": "icon-file-image-fill",
  "file.text": "icon-file-common-filling",
  "file.code": "icon-file-common-filling",
  "file.config": "icon-file-common-filling",
  "file.archive": "icon-file-zip-fill",
  "file.audio": "icon-file-fill",
  "file.video": "icon-file-fill",
  "file.pdf": "icon-file-common-filling",
  "file.spreadsheet": "icon-file-common-filling",
  "file.document": "icon-file-common-filling",
  "file.presentation": "icon-file-common-filling",
  "file.executable": "icon-file-fill",
  "file.shortcut": "icon-file-fill",
  "file.database": "icon-file-fill",
  "file.font": "icon-file-fill",
  "file.package": "icon-file-zip-fill",
  "file.markup": "icon-file-common-filling",
  "file.unknown": "icon-file-fill",
  "file.generic": "icon-file-fill"
};

const classicSymbolTransforms: Record<string, string> = {
  "action.next": "rotate(180deg)",
  "action.up": "rotate(90deg)",
  "action.exit-fullscreen": "scale(0.82)",
  "view.preview-pane-close": "scaleX(-1)"
};

export const classicIconPack: AppIconPack = {
  resolve(icon) {
    const symbol = classicSymbolIcons[icon] ?? (icon.startsWith("icon-") ? icon : "");
    return symbol
        ? {kind: "symbol", symbol, className: "app-icon-symbol", transform: classicSymbolTransforms[icon]}
        : undefined;
  }
};
