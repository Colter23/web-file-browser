import addCircleSvg from "@fluentui/svg-icons/icons/add_circle_24_color.svg?raw";
import addStarburstSvg from "@fluentui/svg-icons/icons/add_starburst_24_color.svg?raw";
import appsSvg from "@fluentui/svg-icons/icons/apps_24_color.svg?raw";
import appsListSvg from "@fluentui/svg-icons/icons/apps_list_24_color.svg?raw";
import appsListDetailSvg from "@fluentui/svg-icons/icons/apps_list_detail_24_color.svg?raw";
import arrowClockwiseDashesSvg from "@fluentui/svg-icons/icons/arrow_clockwise_dashes_24_color.svg?raw";
import arrowSquareDownSvg from "@fluentui/svg-icons/icons/arrow_square_down_24_color.svg?raw";
import arrowSyncSvg from "@fluentui/svg-icons/icons/arrow_sync_24_color.svg?raw";
import boardSvg from "@fluentui/svg-icons/icons/board_24_color.svg?raw";
import calendarClockSvg from "@fluentui/svg-icons/icons/calendar_clock_24_color.svg?raw";
import checkmarkCircleSvg from "@fluentui/svg-icons/icons/checkmark_circle_24_color.svg?raw";
import checkboxSvg from "@fluentui/svg-icons/icons/checkbox_24_color.svg?raw";
import clipboardTaskSvg from "@fluentui/svg-icons/icons/clipboard_task_24_color.svg?raw";
import clipboardTextEditSvg from "@fluentui/svg-icons/icons/clipboard_text_edit_24_color.svg?raw";
import codeBlockSvg from "@fluentui/svg-icons/icons/code_block_24_color.svg?raw";
import codeSvg from "@fluentui/svg-icons/icons/code_24_color.svg?raw";
import contentViewSvg from "@fluentui/svg-icons/icons/content_view_24_color.svg?raw";
import databaseSvg from "@fluentui/svg-icons/icons/database_24_color.svg?raw";
import dismissCircleSvg from "@fluentui/svg-icons/icons/dismiss_circle_24_color.svg?raw";
import documentAddSvg from "@fluentui/svg-icons/icons/document_add_24_color.svg?raw";
import documentFolderSvg from "@fluentui/svg-icons/icons/document_folder_24_color.svg?raw";
import documentSvg from "@fluentui/svg-icons/icons/document_24_color.svg?raw";
import documentTextSvg from "@fluentui/svg-icons/icons/document_text_24_color.svg?raw";
import editSvg from "@fluentui/svg-icons/icons/edit_24_color.svg?raw";
import globeSvg from "@fluentui/svg-icons/icons/globe_24_color.svg?raw";
import headphonesSvg from "@fluentui/svg-icons/icons/headphones_24_color.svg?raw";
import historySvg from "@fluentui/svg-icons/icons/history_24_color.svg?raw";
import homeSvg from "@fluentui/svg-icons/icons/home_24_color.svg?raw";
import imageSvg from "@fluentui/svg-icons/icons/image_24_color.svg?raw";
import laptopSvg from "@fluentui/svg-icons/icons/laptop_24_color.svg?raw";
import linkSvg from "@fluentui/svg-icons/icons/link_24_color.svg?raw";
import linkMultipleSvg from "@fluentui/svg-icons/icons/link_multiple_24_color.svg?raw";
import lockClosedSvg from "@fluentui/svg-icons/icons/lock_closed_24_color.svg?raw";
import numberSymbolSquareSvg from "@fluentui/svg-icons/icons/number_symbol_square_24_color.svg?raw";
import optionsSvg from "@fluentui/svg-icons/icons/options_24_color.svg?raw";
import paintBrushSvg from "@fluentui/svg-icons/icons/paint_brush_24_color.svg?raw";
import personSvg from "@fluentui/svg-icons/icons/person_24_color.svg?raw";
import searchSparkleSvg from "@fluentui/svg-icons/icons/search_sparkle_24_color.svg?raw";
import settingsSvg from "@fluentui/svg-icons/icons/settings_24_color.svg?raw";
import shareAndroidSvg from "@fluentui/svg-icons/icons/share_android_24_color.svg?raw";
import slideTextSparkleSvg from "@fluentui/svg-icons/icons/slide_text_sparkle_24_color.svg?raw";
import starSvg from "@fluentui/svg-icons/icons/star_24_color.svg?raw";
import tableSvg from "@fluentui/svg-icons/icons/table_24_color.svg?raw";
import textBulletListSquareSvg from "@fluentui/svg-icons/icons/text_bullet_list_square_24_color.svg?raw";
import textEditStyleSvg from "@fluentui/svg-icons/icons/text_edit_style_24_color.svg?raw";
import toolboxSvg from "@fluentui/svg-icons/icons/toolbox_24_color.svg?raw";
import vaultSvg from "@fluentui/svg-icons/icons/vault_24_color.svg?raw";
import videoSvg from "@fluentui/svg-icons/icons/video_24_color.svg?raw";
import warningSvg from "@fluentui/svg-icons/icons/warning_24_color.svg?raw";
import weatherSunnyLowSvg from "@fluentui/svg-icons/icons/weather_sunny_low_24_color.svg?raw";
import wrenchSvg from "@fluentui/svg-icons/icons/wrench_24_color.svg?raw";
import wrenchScrewdriverSvg from "@fluentui/svg-icons/icons/wrench_screwdriver_24_color.svg?raw";
import {createFluentSvgIconPack, fluentIcons, fluentTransforms} from "./fluent-pack.ts";

const fluentColorIcons: Record<string, string> = {
  ...fluentIcons,

  "action.new-file": documentAddSvg,
  "action.new-folder": documentFolderSvg,
  "action.open": documentFolderSvg,
  "action.copy-path": linkMultipleSvg,
  "action.download": arrowSquareDownSvg,
  "action.preview": imageSvg,
  "action.archive": vaultSvg,
  "action.extract": arrowSquareDownSvg,
  "action.restore": arrowSyncSvg,
  "action.rename": clipboardTextEditSvg,
  "action.edit": editSvg,
  "action.close": dismissCircleSvg,
  "action.add": addCircleSvg,
  "action.check": checkmarkCircleSvg,
  "action.refresh": arrowClockwiseDashesSvg,
  "action.recent": historySvg,
  "action.clean": addStarburstSvg,
  "action.settings": settingsSvg,
  "action.appearance": paintBrushSvg,
  "action.language": globeSvg,
  "action.main-menu": appsSvg,
  "action.more": optionsSvg,
  "action.search": searchSparkleSvg,
  "action.share": shareAndroidSvg,
  "action.favorite": starSvg,
  "action.favorite-filled": starSvg,
  "action.select-all": checkboxSvg,
  "action.invert-selection": clipboardTaskSvg,
  "action.clear-selection": dismissCircleSvg,
  "action.warning": warningSvg,
  "action.properties": contentViewSvg,
  "action.tools": wrenchScrewdriverSvg,

  "nav.refresh": arrowClockwiseDashesSvg,
  "nav.recent": historySvg,

  "sort.type": documentTextSvg,
  "sort.modified": calendarClockSvg,
  "sort.size": numberSymbolSquareSvg,

  "appearance.system": laptopSvg,
  "appearance.light": weatherSunnyLowSvg,

  "view.details": appsListDetailSvg,
  "view.list": appsListSvg,
  "view.icons": boardSvg,
  "view.tiles": boardSvg,
  "view.grid": boardSvg,
  "view.preview-pane": contentViewSvg,
  "view.preview-pane-close": contentViewSvg,
  "view.image": imageSvg,
  "view.audio": headphonesSvg,
  "view.playlist": textBulletListSquareSvg,
  "view.video": videoSvg,
  "view.pdf": documentTextSvg,

  "file.home": homeSvg,
  "file.folder": documentFolderSvg,
  "file.folder-open": documentFolderSvg,
  "file.file": documentSvg,
  "file.image": imageSvg,
  "file.text": documentTextSvg,
  "file.code": codeBlockSvg,
  "file.config": settingsSvg,
  "file.archive": vaultSvg,
  "file.audio": headphonesSvg,
  "file.video": videoSvg,
  "file.pdf": documentTextSvg,
  "file.spreadsheet": tableSvg,
  "file.document": documentTextSvg,
  "file.presentation": slideTextSparkleSvg,
  "file.executable": toolboxSvg,
  "file.shortcut": linkSvg,
  "file.database": databaseSvg,
  "file.font": textEditStyleSvg,
  "file.package": toolboxSvg,
  "file.markup": codeSvg,
  "file.unknown": documentSvg,
  "file.generic": documentSvg,

  "icon-file-add-fill": documentAddSvg,
  "icon-edit-filling": editSvg,
  "icon-rename": clipboardTextEditSvg,
  "icon-bx-rename": clipboardTextEditSvg,
  "icon-renamebox": clipboardTextEditSvg,
  "icon-setting": settingsSvg,
  "icon-setting-filling": settingsSvg,
  "icon-wrench": wrenchSvg,
  "icon-password": lockClosedSvg,
  "icon-user-fill": personSvg,
  "icon-close": dismissCircleSvg,
  "icon-add": addCircleSvg,
  "icon-refresh": arrowClockwiseDashesSvg,
  "icon-fenxiang": shareAndroidSvg,
  "icon-viewgrid": boardSvg,
  "icon-view-list": appsListDetailSvg,
  "icon-listview": appsListSvg,
  "icon-homefill": homeSvg,
  "icon-home-fill": homeSvg,
  "icon-file": documentSvg,
  "icon-file-fill": documentSvg,
  "icon-file-common-filling": documentTextSvg,
  "icon-file-image-fill": imageSvg,
  "icon-file-zip": vaultSvg,
  "icon-file-zip-fill": vaultSvg,
  "icon-folder": documentFolderSvg,
  "icon-folder-fill": documentFolderSvg,
  "icon-folder-open-fill": documentFolderSvg
};

export const fluentColorIconPack = createFluentSvgIconPack(fluentColorIcons, "app-icon-fluent-color", fluentTransforms);
