import {defineComponent, h} from "vue";
import type {Component, PropType} from "vue";
import addCircleIcon from "@iconify-icons/solar/add-circle-bold-duotone";
import addFolderIcon from "@iconify-icons/solar/add-folder-bold-duotone";
import altArrowDownIcon from "@iconify-icons/solar/alt-arrow-down-bold-duotone";
import altArrowLeftIcon from "@iconify-icons/solar/alt-arrow-left-bold-duotone";
import altArrowRightIcon from "@iconify-icons/solar/alt-arrow-right-bold-duotone";
import altArrowUpIcon from "@iconify-icons/solar/alt-arrow-up-bold-duotone";
import archiveIcon from "@iconify-icons/solar/archive-bold-duotone";
import archiveDownIcon from "@iconify-icons/solar/archive-down-bold-duotone";
import arrowLeftIcon from "@iconify-icons/solar/arrow-left-bold-duotone";
import arrowRightIcon from "@iconify-icons/solar/arrow-right-bold-duotone";
import arrowUpIcon from "@iconify-icons/solar/arrow-up-bold-duotone";
import boxIcon from "@iconify-icons/solar/box-bold-duotone";
import broomIcon from "@iconify-icons/solar/broom-bold-duotone";
import checkCircleIcon from "@iconify-icons/solar/check-circle-bold-duotone";
import closeCircleIcon from "@iconify-icons/solar/close-circle-bold-duotone";
import clipboardTextIcon from "@iconify-icons/solar/clipboard-text-bold-duotone";
import codeIcon from "@iconify-icons/solar/code-bold-duotone";
import codeFileIcon from "@iconify-icons/solar/code-file-bold-duotone";
import commandIcon from "@iconify-icons/solar/command-bold-duotone";
import copyIcon from "@iconify-icons/solar/copy-bold-duotone";
import cursorSquareIcon from "@iconify-icons/solar/cursor-square-bold-duotone";
import dangerTriangleIcon from "@iconify-icons/solar/danger-triangle-bold-duotone";
import databaseIcon from "@iconify-icons/solar/database-bold-duotone";
import disketteIcon from "@iconify-icons/solar/diskette-bold-duotone";
import documentAddIcon from "@iconify-icons/solar/document-add-bold-duotone";
import documentTextIcon from "@iconify-icons/solar/document-text-bold-duotone";
import downloadIcon from "@iconify-icons/solar/download-bold-duotone";
import eraserIcon from "@iconify-icons/solar/eraser-bold-duotone";
import eyeIcon from "@iconify-icons/solar/eye-bold-duotone";
import fileIcon from "@iconify-icons/solar/file-bold-duotone";
import fileTextIcon from "@iconify-icons/solar/file-text-bold-duotone";
import folderIcon from "@iconify-icons/solar/folder-bold-duotone";
import folderOpenIcon from "@iconify-icons/solar/folder-open-bold-duotone";
import fullScreenIcon from "@iconify-icons/solar/full-screen-bold-duotone";
import fullScreenSquareIcon from "@iconify-icons/solar/full-screen-square-bold-duotone";
import galleryIcon from "@iconify-icons/solar/gallery-bold-duotone";
import galleryWideIcon from "@iconify-icons/solar/gallery-wide-bold-duotone";
import hamburgerMenuIcon from "@iconify-icons/solar/hamburger-menu-bold-duotone";
import historyIcon from "@iconify-icons/solar/history-bold-duotone";
import homeIcon from "@iconify-icons/solar/home-bold-duotone";
import infoCircleIcon from "@iconify-icons/solar/info-circle-bold-duotone";
import keyIcon from "@iconify-icons/solar/key-bold-duotone";
import listIcon from "@iconify-icons/solar/list-bold-duotone";
import listCheckIcon from "@iconify-icons/solar/list-check-bold-duotone";
import logout2Icon from "@iconify-icons/solar/logout-2-bold-duotone";
import magnifierIcon from "@iconify-icons/solar/magnifer-bold-duotone";
import magnifierZoomInIcon from "@iconify-icons/solar/magnifer-zoom-in-bold-duotone";
import magnifierZoomOutIcon from "@iconify-icons/solar/magnifer-zoom-out-bold-duotone";
import maximizeSquareIcon from "@iconify-icons/solar/maximize-square-bold-duotone";
import menuDotsIcon from "@iconify-icons/solar/menu-dots-bold-duotone";
import minimizeSquareIcon from "@iconify-icons/solar/minimize-square-bold-duotone";
import monitorIcon from "@iconify-icons/solar/monitor-bold-duotone";
import moonIcon from "@iconify-icons/solar/moon-bold-duotone";
import musicNoteIcon from "@iconify-icons/solar/music-note-bold-duotone";
import paletteIcon from "@iconify-icons/solar/palette-bold-duotone";
import pauseIcon from "@iconify-icons/solar/pause-bold-duotone";
import pauseCircleIcon from "@iconify-icons/solar/pause-circle-bold-duotone";
import penNewSquareIcon from "@iconify-icons/solar/pen-new-square-bold-duotone";
import playIcon from "@iconify-icons/solar/play-bold-duotone";
import playlistMinimalisticIcon from "@iconify-icons/solar/playlist-minimalistic-2-bold-duotone";
import presentationGraphIcon from "@iconify-icons/solar/presentation-graph-bold-duotone";
import quitFullScreenIcon from "@iconify-icons/solar/quit-full-screen-bold-duotone";
import quitFullScreenSquareIcon from "@iconify-icons/solar/quit-full-screen-square-bold-duotone";
import refreshIcon from "@iconify-icons/solar/refresh-bold-duotone";
import reorderIcon from "@iconify-icons/solar/reorder-bold-duotone";
import repeatIcon from "@iconify-icons/solar/repeat-bold-duotone";
import repeatOneIcon from "@iconify-icons/solar/repeat-one-bold-duotone";
import scissorsIcon from "@iconify-icons/solar/scissors-bold-duotone";
import settingsIcon from "@iconify-icons/solar/settings-bold-duotone";
import shareIcon from "@iconify-icons/solar/share-bold-duotone";
import shuffleIcon from "@iconify-icons/solar/shuffle-bold-duotone";
import sidebarIcon from "@iconify-icons/solar/siderbar-bold-duotone";
import sidebarMinimalisticIcon from "@iconify-icons/solar/sidebar-minimalistic-bold-duotone";
import sledgehammerIcon from "@iconify-icons/solar/sledgehammer-bold-duotone";
import sortByAlphabetIcon from "@iconify-icons/solar/sort-by-alphabet-bold-duotone";
import sortByTimeIcon from "@iconify-icons/solar/sort-by-time-bold-duotone";
import sortFromBottomToTopIcon from "@iconify-icons/solar/sort-from-bottom-to-top-bold-duotone";
import sortFromTopToBottomIcon from "@iconify-icons/solar/sort-from-top-to-bottom-bold-duotone";
import speakerIcon from "@iconify-icons/solar/speaker-bold-duotone";
import squareAltArrowRightIcon from "@iconify-icons/solar/square-alt-arrow-right-bold-duotone";
import starIcon from "@iconify-icons/solar/star-bold-duotone";
import sunIcon from "@iconify-icons/solar/sun-bold-duotone";
import textIcon from "@iconify-icons/solar/text-bold-duotone";
import trashBinTrashIcon from "@iconify-icons/solar/trash-bin-trash-bold-duotone";
import uploadIcon from "@iconify-icons/solar/upload-bold-duotone";
import undoLeftIcon from "@iconify-icons/solar/undo-left-bold-duotone";
import undoRightIcon from "@iconify-icons/solar/undo-right-bold-duotone";
import userIcon from "@iconify-icons/solar/user-bold-duotone";
import videoFrameIcon from "@iconify-icons/solar/video-frame-bold-duotone";
import volumeCrossIcon from "@iconify-icons/solar/volume-cross-bold-duotone";
import widgetIcon from "@iconify-icons/solar/widget-bold-duotone";
import widget3Icon from "@iconify-icons/solar/widget-3-bold-duotone";
import widget4Icon from "@iconify-icons/solar/widget-4-bold-duotone";
import zipFileIcon from "@iconify-icons/solar/zip-file-bold-duotone";
import type {AppIconPack} from "./types.ts";

interface SolarIconData {
  body: string;
  width?: number;
  height?: number;
  left?: number;
  top?: number;
}

const SolarInlineSvgIcon = defineComponent({
  name: "SolarInlineSvgIcon",
  inheritAttrs: false,
  props: {
    icon: {
      type: Object as PropType<SolarIconData>,
      required: true
    }
  },
  setup(props, {attrs}) {
    return () => {
      const left = props.icon.left ?? 0;
      const top = props.icon.top ?? 0;
      const width = props.icon.width ?? 24;
      const height = props.icon.height ?? 24;
      return h("svg", {
        ...attrs,
        xmlns: "http://www.w3.org/2000/svg",
        viewBox: `${left} ${top} ${width} ${height}`,
        innerHTML: props.icon.body
      });
    };
  }
});

const solarIcons: Record<string, SolarIconData> = {
  "action.upload": uploadIcon,
  "action.new-file": documentAddIcon,
  "action.new-folder": addFolderIcon,
  "action.open": folderOpenIcon,
  "action.open-new-tab": squareAltArrowRightIcon,
  "action.cut": scissorsIcon,
  "action.copy": copyIcon,
  "action.copy-path": clipboardTextIcon,
  "action.paste": clipboardTextIcon,
  "action.download": downloadIcon,
  "action.preview": eyeIcon,
  "action.archive": archiveIcon,
  "action.extract": archiveDownIcon,
  "action.restore": refreshIcon,
  "action.rename": penNewSquareIcon,
  "action.delete": trashBinTrashIcon,
  "action.trash": trashBinTrashIcon,
  "action.edit": penNewSquareIcon,
  "action.save": disketteIcon,
  "action.close": closeCircleIcon,
  "action.add": addCircleIcon,
  "action.check": checkCircleIcon,
  "action.refresh": refreshIcon,
  "action.recent": historyIcon,
  "action.clean": broomIcon,
  "action.settings": settingsIcon,
  "action.appearance": paletteIcon,
  "action.main-menu": hamburgerMenuIcon,
  "action.more": menuDotsIcon,
  "action.logout": logout2Icon,
  "action.search": magnifierIcon,
  "action.share": shareIcon,
  "action.favorite": starIcon,
  "action.favorite-filled": starIcon,
  "action.warning": dangerTriangleIcon,
  "action.select-all": listCheckIcon,
  "action.invert-selection": cursorSquareIcon,
  "action.clear-selection": eraserIcon,
  "action.properties": infoCircleIcon,
  "action.fullscreen": fullScreenIcon,
  "action.exit-fullscreen": quitFullScreenIcon,
  "action.player-large": maximizeSquareIcon,
  "action.player-small": sidebarMinimalisticIcon,
  "action.player-mini": minimizeSquareIcon,
  "action.play": playIcon,
  "action.pause": pauseIcon,
  "action.volume": speakerIcon,
  "action.volume-muted": volumeCrossIcon,
  "action.previous": altArrowLeftIcon,
  "action.next": altArrowRightIcon,
  "action.up": altArrowUpIcon,
  "action.down": altArrowDownIcon,
  "action.drag-handle": reorderIcon,
  "action.tools": sledgehammerIcon,

  "nav.back": arrowLeftIcon,
  "nav.forward": arrowRightIcon,
  "nav.up": arrowUpIcon,
  "nav.refresh": refreshIcon,
  "nav.recent": historyIcon,

  "sort.name": sortByAlphabetIcon,
  "sort.modified": sortByTimeIcon,
  "sort.size": sortFromTopToBottomIcon,
  "sort.asc": sortFromBottomToTopIcon,
  "sort.desc": sortFromTopToBottomIcon,
  "sort.small-large": sortFromBottomToTopIcon,
  "sort.large-small": sortFromTopToBottomIcon,

  "playback.sequence": pauseCircleIcon,
  "playback.repeat-one": repeatOneIcon,
  "playback.repeat-all": repeatIcon,
  "playback.shuffle": shuffleIcon,

  "appearance.system": monitorIcon,
  "appearance.light": sunIcon,
  "appearance.dark": moonIcon,

  "view.details": listIcon,
  "view.list": listIcon,
  "view.icons": widget4Icon,
  "view.tiles": widgetIcon,
  "view.grid": widget3Icon,
  "view.preview-pane": sidebarIcon,
  "view.preview-pane-close": sidebarIcon,
  "view.image": galleryIcon,
  "view.audio": musicNoteIcon,
  "view.playlist": playlistMinimalisticIcon,
  "view.video": videoFrameIcon,
  "view.pdf": fileTextIcon,

  "viewer.page-fullscreen": maximizeSquareIcon,
  "viewer.page-fullscreen-off": minimizeSquareIcon,
  "viewer.browser-fullscreen": fullScreenSquareIcon,
  "viewer.browser-fullscreen-off": quitFullScreenSquareIcon,
  "viewer.filmstrip": galleryWideIcon,
  "viewer.filmstrip-off": galleryIcon,
  "viewer.zoom-in": magnifierZoomInIcon,
  "viewer.zoom-out": magnifierZoomOutIcon,
  "viewer.rotate-left": undoLeftIcon,
  "viewer.rotate-right": undoRightIcon,

  "file.home": homeIcon,
  "file.folder": folderIcon,
  "file.folder-open": folderOpenIcon,
  "file.file": fileIcon,
  "file.image": galleryIcon,
  "file.text": fileTextIcon,
  "file.code": codeFileIcon,
  "file.config": settingsIcon,
  "file.archive": zipFileIcon,
  "file.audio": musicNoteIcon,
  "file.video": videoFrameIcon,
  "file.pdf": fileTextIcon,
  "file.spreadsheet": documentTextIcon,
  "file.document": documentTextIcon,
  "file.presentation": presentationGraphIcon,
  "file.executable": commandIcon,
  "file.shortcut": shareIcon,
  "file.database": databaseIcon,
  "file.font": textIcon,
  "file.package": boxIcon,
  "file.markup": codeIcon,
  "file.unknown": fileIcon,
  "file.generic": fileIcon,

  "icon-upload": uploadIcon,
  "icon-file-add-fill": documentAddIcon,
  "icon-folder-add-fill": addFolderIcon,
  "icon-scissors": scissorsIcon,
  "icon-contentcut": scissorsIcon,
  "icon-copy": copyIcon,
  "icon-copy1": copyIcon,
  "icon-paste": clipboardTextIcon,
  "icon-download": downloadIcon,
  "icon-download1": downloadIcon,
  "icon-download2": archiveDownIcon,
  "icon-download2f": archiveDownIcon,
  "icon-edit-filling": penNewSquareIcon,
  "icon-rename": penNewSquareIcon,
  "icon-bx-rename": penNewSquareIcon,
  "icon-renamebox": penNewSquareIcon,
  "icon-delete": trashBinTrashIcon,
  "icon-delete1": trashBinTrashIcon,
  "icon-delete-fill": trashBinTrashIcon,
  "icon-setting": settingsIcon,
  "icon-setting-filling": settingsIcon,
  "icon-wrench": sledgehammerIcon,
  "icon-password": keyIcon,
  "icon-user-fill": userIcon,
  "icon-save-fill": disketteIcon,
  "icon-close": closeCircleIcon,
  "icon-add": addCircleIcon,
  "icon-refresh": refreshIcon,
  "icon-back_android": altArrowLeftIcon,
  "icon-unfold": altArrowDownIcon,
  "icon-fenxiang": magnifierIcon,
  "icon-viewgrid": widget4Icon,
  "icon-view-list": listIcon,
  "icon-listview": listIcon,
  "icon-homefill": homeIcon,
  "icon-home-fill": homeIcon,
  "icon-file": fileIcon,
  "icon-file-fill": fileIcon,
  "icon-file-common-filling": fileTextIcon,
  "icon-file-image-fill": galleryIcon,
  "icon-file-zip": zipFileIcon,
  "icon-file-zip-fill": zipFileIcon,
  "icon-folder": folderIcon,
  "icon-folder-fill": folderIcon,
  "icon-folder-open-fill": folderOpenIcon
};

const solarTransforms: Record<string, string> = {
  "view.preview-pane-close": "scaleX(-1)"
};

const solarComponent = SolarInlineSvgIcon as Component;

export const solarIconPack: AppIconPack = {
  resolve(icon) {
    const iconData = solarIcons[icon];
    return iconData
        ? {kind: "component", component: solarComponent, props: {icon: iconData}, className: "app-icon-solar", transform: solarTransforms[icon]}
        : undefined;
  }
};
