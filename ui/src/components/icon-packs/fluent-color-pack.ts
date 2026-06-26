import addCircleSvg from "@fluentui/svg-icons/icons/add_circle_24_color.svg?raw";
import appsListSvg from "@fluentui/svg-icons/icons/apps_list_24_color.svg?raw";
import appsListDetailSvg from "@fluentui/svg-icons/icons/apps_list_detail_24_color.svg?raw";
import boardSvg from "@fluentui/svg-icons/icons/board_24_color.svg?raw";
import codeBlockSvg from "@fluentui/svg-icons/icons/code_block_24_color.svg?raw";
import codeSvg from "@fluentui/svg-icons/icons/code_24_color.svg?raw";
import documentAddSvg from "@fluentui/svg-icons/icons/document_add_24_color.svg?raw";
import documentEditSvg from "@fluentui/svg-icons/icons/document_edit_24_color.svg?raw";
import documentFolderSvg from "@fluentui/svg-icons/icons/document_folder_24_color.svg?raw";
import documentSvg from "@fluentui/svg-icons/icons/document_24_color.svg?raw";
import documentTextSvg from "@fluentui/svg-icons/icons/document_text_24_color.svg?raw";
import historySvg from "@fluentui/svg-icons/icons/history_24_color.svg?raw";
import homeSvg from "@fluentui/svg-icons/icons/home_24_color.svg?raw";
import imageSvg from "@fluentui/svg-icons/icons/image_24_color.svg?raw";
import personSvg from "@fluentui/svg-icons/icons/person_24_color.svg?raw";
import searchSparkleSvg from "@fluentui/svg-icons/icons/search_sparkle_24_color.svg?raw";
import settingsSvg from "@fluentui/svg-icons/icons/settings_24_color.svg?raw";
import shareAndroidSvg from "@fluentui/svg-icons/icons/share_android_24_color.svg?raw";
import starSvg from "@fluentui/svg-icons/icons/star_24_color.svg?raw";
import textBulletListSquareSvg from "@fluentui/svg-icons/icons/text_bullet_list_square_24_color.svg?raw";
import videoSvg from "@fluentui/svg-icons/icons/video_24_color.svg?raw";
import warningSvg from "@fluentui/svg-icons/icons/warning_24_color.svg?raw";
import weatherSunnyLowSvg from "@fluentui/svg-icons/icons/weather_sunny_low_24_color.svg?raw";
import wrenchSvg from "@fluentui/svg-icons/icons/wrench_24_color.svg?raw";
import {createFluentSvgIconPack, fluentIcons, fluentTransforms} from "./fluent-pack.ts";

const fluentColorIcons: Record<string, string> = {
  ...fluentIcons,

  "action.new-file": documentAddSvg,
  "action.open": documentFolderSvg,
  "action.open-new-tab": shareAndroidSvg,
  "action.copy-path": documentSvg,
  "action.preview": imageSvg,
  "action.edit": documentEditSvg,
  "action.add": addCircleSvg,
  "action.recent": historySvg,
  "action.settings": settingsSvg,
  "action.appearance": weatherSunnyLowSvg,
  "action.search": searchSparkleSvg,
  "action.share": shareAndroidSvg,
  "action.favorite": starSvg,
  "action.favorite-filled": starSvg,
  "action.warning": warningSvg,
  "action.properties": documentSvg,
  "action.tools": wrenchSvg,

  "appearance.light": weatherSunnyLowSvg,

  "view.details": appsListDetailSvg,
  "view.list": appsListSvg,
  "view.icons": boardSvg,
  "view.tiles": boardSvg,
  "view.grid": boardSvg,
  "view.image": imageSvg,
  "view.playlist": textBulletListSquareSvg,
  "view.video": videoSvg,

  "file.home": homeSvg,
  "file.folder": documentFolderSvg,
  "file.folder-open": documentFolderSvg,
  "file.file": documentSvg,
  "file.image": imageSvg,
  "file.text": documentTextSvg,
  "file.code": codeBlockSvg,
  "file.config": settingsSvg,
  "file.video": videoSvg,
  "file.document": documentTextSvg,
  "file.executable": codeSvg,
  "file.shortcut": shareAndroidSvg,
  "file.font": textBulletListSquareSvg,
  "file.markup": codeSvg,
  "file.unknown": documentSvg,
  "file.generic": documentSvg,

  "icon-file-add-fill": documentAddSvg,
  "icon-edit-filling": documentEditSvg,
  "icon-setting": settingsSvg,
  "icon-setting-filling": settingsSvg,
  "icon-wrench": wrenchSvg,
  "icon-user-fill": personSvg,
  "icon-add": addCircleSvg,
  "icon-fenxiang": searchSparkleSvg,
  "icon-viewgrid": boardSvg,
  "icon-view-list": appsListDetailSvg,
  "icon-listview": appsListSvg,
  "icon-homefill": homeSvg,
  "icon-home-fill": homeSvg,
  "icon-file": documentSvg,
  "icon-file-fill": documentSvg,
  "icon-file-common-filling": documentTextSvg,
  "icon-file-image-fill": imageSvg,
  "icon-folder": documentFolderSvg,
  "icon-folder-fill": documentFolderSvg,
  "icon-folder-open-fill": documentFolderSvg
};

export const fluentColorIconPack = createFluentSvgIconPack(fluentColorIcons, "app-icon-fluent-color", fluentTransforms);
