import {defineComponent, h} from "vue";
import type {Component} from "vue";
import addSvg from "@fluentui/svg-icons/icons/add_24_filled.svg?raw";
import addCircleSvg from "@fluentui/svg-icons/icons/add_circle_24_filled.svg?raw";
import archiveSvg from "@fluentui/svg-icons/icons/archive_24_filled.svg?raw";
import archiveRestoreSvg from "@fluentui/svg-icons/icons/archive_arrow_back_24_filled.svg?raw";
import arrowClockwiseSvg from "@fluentui/svg-icons/icons/arrow_clockwise_24_filled.svg?raw";
import arrowCounterClockwiseSvg from "@fluentui/svg-icons/icons/arrow_counterclockwise_24_filled.svg?raw";
import arrowDownloadSvg from "@fluentui/svg-icons/icons/arrow_download_24_filled.svg?raw";
import arrowExportSvg from "@fluentui/svg-icons/icons/arrow_export_24_filled.svg?raw";
import arrowLeftSvg from "@fluentui/svg-icons/icons/arrow_left_24_filled.svg?raw";
import arrowMaximizeSvg from "@fluentui/svg-icons/icons/arrow_maximize_24_filled.svg?raw";
import arrowRightSvg from "@fluentui/svg-icons/icons/arrow_right_24_filled.svg?raw";
import arrowRepeatAllSvg from "@fluentui/svg-icons/icons/arrow_repeat_all_24_filled.svg?raw";
import arrowRepeatOneSvg from "@fluentui/svg-icons/icons/arrow_repeat_1_24_filled.svg?raw";
import arrowShuffleSvg from "@fluentui/svg-icons/icons/arrow_shuffle_24_filled.svg?raw";
import arrowSortDownLinesSvg from "@fluentui/svg-icons/icons/arrow_sort_down_lines_24_filled.svg?raw";
import arrowSortUpLinesSvg from "@fluentui/svg-icons/icons/arrow_sort_up_lines_24_filled.svg?raw";
import arrowUploadSvg from "@fluentui/svg-icons/icons/arrow_upload_24_filled.svg?raw";
import arrowUpSvg from "@fluentui/svg-icons/icons/arrow_up_24_filled.svg?raw";
import boardSvg from "@fluentui/svg-icons/icons/board_24_filled.svg?raw";
import boxSvg from "@fluentui/svg-icons/icons/box_24_filled.svg?raw";
import broomSvg from "@fluentui/svg-icons/icons/broom_24_filled.svg?raw";
import calendarClockSvg from "@fluentui/svg-icons/icons/calendar_clock_24_filled.svg?raw";
import checkmarkSvg from "@fluentui/svg-icons/icons/checkmark_24_filled.svg?raw";
import chevronDownSvg from "@fluentui/svg-icons/icons/chevron_down_24_filled.svg?raw";
import chevronLeftSvg from "@fluentui/svg-icons/icons/chevron_left_24_filled.svg?raw";
import chevronRightSvg from "@fluentui/svg-icons/icons/chevron_right_24_filled.svg?raw";
import chevronUpSvg from "@fluentui/svg-icons/icons/chevron_up_24_filled.svg?raw";
import clipboardPasteSvg from "@fluentui/svg-icons/icons/clipboard_paste_24_filled.svg?raw";
import codeSvg from "@fluentui/svg-icons/icons/code_24_filled.svg?raw";
import colorSvg from "@fluentui/svg-icons/icons/color_24_filled.svg?raw";
import copySvg from "@fluentui/svg-icons/icons/copy_24_filled.svg?raw";
import cutSvg from "@fluentui/svg-icons/icons/cut_24_filled.svg?raw";
import deleteSvg from "@fluentui/svg-icons/icons/delete_24_filled.svg?raw";
import desktopSvg from "@fluentui/svg-icons/icons/desktop_24_filled.svg?raw";
import dismissSvg from "@fluentui/svg-icons/icons/dismiss_24_filled.svg?raw";
import documentAddSvg from "@fluentui/svg-icons/icons/document_add_24_filled.svg?raw";
import documentCodeSvg from "@fluentui/svg-icons/icons/document_code_16_filled.svg?raw";
import documentCopySvg from "@fluentui/svg-icons/icons/document_copy_24_filled.svg?raw";
import documentDataSvg from "@fluentui/svg-icons/icons/document_data_24_filled.svg?raw";
import documentImageSvg from "@fluentui/svg-icons/icons/document_image_20_filled.svg?raw";
import documentPdfSvg from "@fluentui/svg-icons/icons/document_pdf_24_filled.svg?raw";
import documentSettingsSvg from "@fluentui/svg-icons/icons/document_settings_20_filled.svg?raw";
import documentSvg from "@fluentui/svg-icons/icons/document_24_filled.svg?raw";
import documentTableSvg from "@fluentui/svg-icons/icons/document_table_24_filled.svg?raw";
import documentTextSvg from "@fluentui/svg-icons/icons/document_text_24_filled.svg?raw";
import editSvg from "@fluentui/svg-icons/icons/edit_24_filled.svg?raw";
import eyeSvg from "@fluentui/svg-icons/icons/eye_24_filled.svg?raw";
import filmstripImageSvg from "@fluentui/svg-icons/icons/filmstrip_image_24_filled.svg?raw";
import folderAddSvg from "@fluentui/svg-icons/icons/folder_add_24_filled.svg?raw";
import folderOpenSvg from "@fluentui/svg-icons/icons/folder_open_24_filled.svg?raw";
import folderSvg from "@fluentui/svg-icons/icons/folder_24_filled.svg?raw";
import folderZipSvg from "@fluentui/svg-icons/icons/folder_zip_24_filled.svg?raw";
import fullScreenMaximizeSvg from "@fluentui/svg-icons/icons/full_screen_maximize_24_filled.svg?raw";
import fullScreenMinimizeSvg from "@fluentui/svg-icons/icons/full_screen_minimize_24_filled.svg?raw";
import gridSvg from "@fluentui/svg-icons/icons/grid_24_filled.svg?raw";
import historySvg from "@fluentui/svg-icons/icons/history_24_filled.svg?raw";
import homeSvg from "@fluentui/svg-icons/icons/home_24_filled.svg?raw";
import imageSvg from "@fluentui/svg-icons/icons/image_24_filled.svg?raw";
import infoSvg from "@fluentui/svg-icons/icons/info_24_filled.svg?raw";
import keySvg from "@fluentui/svg-icons/icons/key_24_filled.svg?raw";
import listDetailSvg from "@fluentui/svg-icons/icons/apps_list_detail_24_filled.svg?raw";
import listSvg from "@fluentui/svg-icons/icons/list_24_filled.svg?raw";
import moreHorizontalSvg from "@fluentui/svg-icons/icons/more_horizontal_24_filled.svg?raw";
import musicNoteSvg from "@fluentui/svg-icons/icons/music_note_2_24_filled.svg?raw";
import navigationSvg from "@fluentui/svg-icons/icons/navigation_24_filled.svg?raw";
import numberSymbolSquareSvg from "@fluentui/svg-icons/icons/number_symbol_square_24_filled.svg?raw";
import openSvg from "@fluentui/svg-icons/icons/open_24_filled.svg?raw";
import panelBottomContractSvg from "@fluentui/svg-icons/icons/panel_bottom_contract_20_filled.svg?raw";
import panelRightSvg from "@fluentui/svg-icons/icons/panel_right_24_filled.svg?raw";
import pauseSvg from "@fluentui/svg-icons/icons/pause_24_filled.svg?raw";
import personSvg from "@fluentui/svg-icons/icons/person_24_filled.svg?raw";
import playSvg from "@fluentui/svg-icons/icons/play_24_filled.svg?raw";
import reOrderDotsSvg from "@fluentui/svg-icons/icons/re_order_dots_vertical_24_filled.svg?raw";
import renameSvg from "@fluentui/svg-icons/icons/rename_24_filled.svg?raw";
import resizeSmallSvg from "@fluentui/svg-icons/icons/resize_small_24_filled.svg?raw";
import rotateLeftSvg from "@fluentui/svg-icons/icons/rotate_left_24_filled.svg?raw";
import rotateRightSvg from "@fluentui/svg-icons/icons/rotate_right_24_filled.svg?raw";
import saveSvg from "@fluentui/svg-icons/icons/save_24_filled.svg?raw";
import searchSvg from "@fluentui/svg-icons/icons/search_24_filled.svg?raw";
import selectAllOffSvg from "@fluentui/svg-icons/icons/select_all_off_24_filled.svg?raw";
import selectAllOnSvg from "@fluentui/svg-icons/icons/select_all_on_24_filled.svg?raw";
import settingsSvg from "@fluentui/svg-icons/icons/settings_24_filled.svg?raw";
import shareSvg from "@fluentui/svg-icons/icons/share_24_filled.svg?raw";
import signOutSvg from "@fluentui/svg-icons/icons/sign_out_24_filled.svg?raw";
import slideTextSvg from "@fluentui/svg-icons/icons/slide_text_24_filled.svg?raw";
import speakerMuteSvg from "@fluentui/svg-icons/icons/speaker_mute_24_filled.svg?raw";
import speakerSvg from "@fluentui/svg-icons/icons/speaker_2_24_filled.svg?raw";
import starSvg from "@fluentui/svg-icons/icons/star_24_filled.svg?raw";
import subtractCircleSvg from "@fluentui/svg-icons/icons/subtract_circle_24_filled.svg?raw";
import textBulletListSvg from "@fluentui/svg-icons/icons/text_bullet_list_24_filled.svg?raw";
import textFontSvg from "@fluentui/svg-icons/icons/text_font_24_filled.svg?raw";
import textSortAscendingSvg from "@fluentui/svg-icons/icons/text_sort_ascending_24_filled.svg?raw";
import textSortDescendingSvg from "@fluentui/svg-icons/icons/text_sort_descending_24_filled.svg?raw";
import videoClipSvg from "@fluentui/svg-icons/icons/video_clip_24_filled.svg?raw";
import warningSvg from "@fluentui/svg-icons/icons/warning_24_filled.svg?raw";
import weatherMoonSvg from "@fluentui/svg-icons/icons/weather_moon_24_filled.svg?raw";
import weatherSunnySvg from "@fluentui/svg-icons/icons/weather_sunny_24_filled.svg?raw";
import windowSvg from "@fluentui/svg-icons/icons/window_24_filled.svg?raw";
import windowConsoleSvg from "@fluentui/svg-icons/icons/window_console_20_filled.svg?raw";
import wrenchSvg from "@fluentui/svg-icons/icons/wrench_24_filled.svg?raw";
import type {AppIconPack} from "./types.ts";

export const FluentInlineSvgIcon = defineComponent({
  name: "FluentInlineSvgIcon",
  inheritAttrs: false,
  props: {
    svg: {
      type: String,
      required: true
    }
  },
  setup(props, {attrs}) {
    return () => h("span", attrs, [
      h("span", {
        class: "app-icon-fluent-content",
        innerHTML: props.svg
      })
    ]);
  }
});

export const fluentIcons: Record<string, string> = {
  "action.upload": arrowUploadSvg,
  "action.new-file": documentAddSvg,
  "action.new-folder": folderAddSvg,
  "action.open": folderOpenSvg,
  "action.open-new-tab": openSvg,
  "action.cut": cutSvg,
  "action.copy": copySvg,
  "action.copy-path": documentCopySvg,
  "action.paste": clipboardPasteSvg,
  "action.download": arrowDownloadSvg,
  "action.preview": eyeSvg,
  "action.archive": archiveSvg,
  "action.extract": archiveRestoreSvg,
  "action.restore": arrowCounterClockwiseSvg,
  "action.rename": renameSvg,
  "action.delete": deleteSvg,
  "action.trash": deleteSvg,
  "action.edit": editSvg,
  "action.save": saveSvg,
  "action.close": dismissSvg,
  "action.add": addSvg,
  "action.check": checkmarkSvg,
  "action.refresh": arrowClockwiseSvg,
  "action.recent": historySvg,
  "action.clean": broomSvg,
  "action.settings": settingsSvg,
  "action.appearance": colorSvg,
  "action.main-menu": navigationSvg,
  "action.more": moreHorizontalSvg,
  "action.logout": signOutSvg,
  "action.search": searchSvg,
  "action.share": shareSvg,
  "action.favorite": starSvg,
  "action.favorite-filled": starSvg,
  "action.warning": warningSvg,
  "action.select-all": selectAllOnSvg,
  "action.invert-selection": selectAllOnSvg,
  "action.clear-selection": selectAllOffSvg,
  "action.properties": infoSvg,
  "action.fullscreen": fullScreenMaximizeSvg,
  "action.exit-fullscreen": fullScreenMinimizeSvg,
  "action.player-large": arrowMaximizeSvg,
  "action.player-small": panelBottomContractSvg,
  "action.player-mini": resizeSmallSvg,
  "action.play": playSvg,
  "action.pause": pauseSvg,
  "action.volume": speakerSvg,
  "action.volume-muted": speakerMuteSvg,
  "action.previous": chevronLeftSvg,
  "action.next": chevronRightSvg,
  "action.up": chevronUpSvg,
  "action.down": chevronDownSvg,
  "action.drag-handle": reOrderDotsSvg,
  "action.tools": wrenchSvg,

  "nav.back": arrowLeftSvg,
  "nav.forward": arrowRightSvg,
  "nav.up": arrowUpSvg,
  "nav.refresh": arrowClockwiseSvg,
  "nav.recent": historySvg,

  "sort.name": textSortAscendingSvg,
  "sort.modified": calendarClockSvg,
  "sort.size": numberSymbolSquareSvg,
  "sort.asc": arrowSortUpLinesSvg,
  "sort.desc": arrowSortDownLinesSvg,
  "sort.small-large": textSortAscendingSvg,
  "sort.large-small": textSortDescendingSvg,

  "playback.sequence": pauseSvg,
  "playback.repeat-one": arrowRepeatOneSvg,
  "playback.repeat-all": arrowRepeatAllSvg,
  "playback.shuffle": arrowShuffleSvg,

  "appearance.system": desktopSvg,
  "appearance.light": weatherSunnySvg,
  "appearance.dark": weatherMoonSvg,

  "view.details": listDetailSvg,
  "view.list": listSvg,
  "view.icons": gridSvg,
  "view.tiles": boardSvg,
  "view.grid": gridSvg,
  "view.preview-pane": panelRightSvg,
  "view.preview-pane-close": panelRightSvg,
  "view.image": imageSvg,
  "view.audio": musicNoteSvg,
  "view.playlist": textBulletListSvg,
  "view.video": videoClipSvg,
  "view.pdf": documentPdfSvg,

  "viewer.page-fullscreen": arrowMaximizeSvg,
  "viewer.page-fullscreen-off": resizeSmallSvg,
  "viewer.browser-fullscreen": fullScreenMaximizeSvg,
  "viewer.browser-fullscreen-off": fullScreenMinimizeSvg,
  "viewer.filmstrip": filmstripImageSvg,
  "viewer.filmstrip-off": windowSvg,
  "viewer.zoom-in": addCircleSvg,
  "viewer.zoom-out": subtractCircleSvg,
  "viewer.rotate-left": rotateLeftSvg,
  "viewer.rotate-right": rotateRightSvg,

  "file.home": homeSvg,
  "file.folder": folderSvg,
  "file.folder-open": folderOpenSvg,
  "file.file": documentSvg,
  "file.image": documentImageSvg,
  "file.text": documentTextSvg,
  "file.code": documentCodeSvg,
  "file.config": documentSettingsSvg,
  "file.archive": folderZipSvg,
  "file.audio": musicNoteSvg,
  "file.video": videoClipSvg,
  "file.pdf": documentPdfSvg,
  "file.spreadsheet": documentTableSvg,
  "file.document": documentTextSvg,
  "file.presentation": slideTextSvg,
  "file.executable": windowConsoleSvg,
  "file.shortcut": arrowExportSvg,
  "file.database": documentDataSvg,
  "file.font": textFontSvg,
  "file.package": boxSvg,
  "file.markup": codeSvg,
  "file.unknown": documentSvg,
  "file.generic": documentSvg,

  "icon-upload": arrowUploadSvg,
  "icon-file-add-fill": documentAddSvg,
  "icon-folder-add-fill": folderAddSvg,
  "icon-scissors": cutSvg,
  "icon-contentcut": cutSvg,
  "icon-copy": copySvg,
  "icon-copy1": documentCopySvg,
  "icon-paste": clipboardPasteSvg,
  "icon-download": arrowDownloadSvg,
  "icon-download1": arrowDownloadSvg,
  "icon-download2": archiveRestoreSvg,
  "icon-download2f": archiveRestoreSvg,
  "icon-edit-filling": editSvg,
  "icon-rename": renameSvg,
  "icon-bx-rename": renameSvg,
  "icon-renamebox": renameSvg,
  "icon-delete": deleteSvg,
  "icon-delete1": deleteSvg,
  "icon-delete-fill": deleteSvg,
  "icon-setting": settingsSvg,
  "icon-setting-filling": settingsSvg,
  "icon-wrench": wrenchSvg,
  "icon-password": keySvg,
  "icon-user-fill": personSvg,
  "icon-save-fill": saveSvg,
  "icon-close": dismissSvg,
  "icon-add": addSvg,
  "icon-refresh": arrowClockwiseSvg,
  "icon-back_android": chevronLeftSvg,
  "icon-unfold": chevronDownSvg,
  "icon-fenxiang": searchSvg,
  "icon-viewgrid": gridSvg,
  "icon-view-list": listDetailSvg,
  "icon-listview": listSvg,
  "icon-homefill": homeSvg,
  "icon-home-fill": homeSvg,
  "icon-file": documentSvg,
  "icon-file-fill": documentSvg,
  "icon-file-common-filling": documentTextSvg,
  "icon-file-image-fill": documentImageSvg,
  "icon-file-zip": folderZipSvg,
  "icon-file-zip-fill": folderZipSvg,
  "icon-folder": folderSvg,
  "icon-folder-fill": folderSvg,
  "icon-folder-open-fill": folderOpenSvg
};

export const fluentTransforms: Record<string, string> = {
  "view.preview-pane-close": "scaleX(-1)"
};

const fluentComponent = FluentInlineSvgIcon as Component;

export const createFluentSvgIconPack = (
    icons: Record<string, string>,
    className = "app-icon-fluent",
    transforms: Record<string, string> = fluentTransforms
): AppIconPack => ({
  resolve(icon) {
    const svg = icons[icon];
    return svg
        ? {kind: "component", component: fluentComponent, props: {svg}, className, transform: transforms[icon]}
        : undefined;
  }
});

export const fluentIconPack = createFluentSvgIconPack(fluentIcons);
