<script setup lang="ts">
import {computed} from "vue";
import type {Component} from "vue";
import {useAppearanceStore} from "../store/appearance.ts";
import {
  Archive,
  ArchiveRestore,
  Box,
  Braces,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  ClipboardPaste,
  Columns3,
  Copy,
  Download,
  Eye,
  File,
  FileArchive,
  FileBox,
  FileChartColumn,
  FileCode,
  FileCog,
  FileImage,
  FileMusic,
  FilePlus,
  FileQuestionMark,
  FileSpreadsheet,
  FileText,
  FileType,
  Folder,
  FolderOpen,
  FolderPlus,
  Grid2X2,
  House,
  Image,
  LayoutGrid,
  List,
  Lock,
  Maximize2,
  Minimize2,
  PanelRight,
  Pencil,
  Plus,
  RefreshCw,
  Rows3,
  Save,
  Scissors,
  Search,
  Settings,
  Share2,
  SquarePen,
  Terminal,
  Trash2,
  Upload,
  User,
  Video,
  Wrench,
  X
} from "lucide-vue-next";

type IconSize = string | "large" | "small" | "normal";

const props = withDefaults(defineProps<{
  icon: string;
  color?: string;
  size?: IconSize;
  strokeWidth?: number;
}>(), {
  color: "currentColor",
  size: "normal",
  strokeWidth: 2
});

const appearanceStore = useAppearanceStore();

const lucideIcons: Record<string, Component> = {
  "action.upload": Upload,
  "action.new-file": FilePlus,
  "action.new-folder": FolderPlus,
  "action.cut": Scissors,
  "action.copy": Copy,
  "action.paste": ClipboardPaste,
  "action.download": Download,
  "action.preview": Eye,
  "action.archive": Archive,
  "action.extract": ArchiveRestore,
  "action.rename": Pencil,
  "action.delete": Trash2,
  "action.edit": SquarePen,
  "action.save": Save,
  "action.close": X,
  "action.add": Plus,
  "action.refresh": RefreshCw,
  "action.settings": Settings,
  "action.search": Search,
  "action.share": Share2,
  "action.fullscreen": Maximize2,
  "action.exit-fullscreen": Minimize2,
  "action.previous": ChevronLeft,
  "action.next": ChevronRight,
  "action.up": ChevronUp,
  "action.down": ChevronDown,
  "action.tools": Wrench,

  "view.details": List,
  "view.list": Rows3,
  "view.icons": LayoutGrid,
  "view.tiles": Columns3,
  "view.grid": Grid2X2,
  "view.preview-pane": PanelRight,
  "view.image": Image,

  "file.home": House,
  "file.folder": Folder,
  "file.folder-open": FolderOpen,
  "file.file": File,
  "file.image": FileImage,
  "file.text": FileText,
  "file.code": FileCode,
  "file.config": FileCog,
  "file.archive": FileArchive,
  "file.audio": FileMusic,
  "file.video": Video,
  "file.pdf": FileText,
  "file.spreadsheet": FileSpreadsheet,
  "file.document": FileText,
  "file.presentation": FileChartColumn,
  "file.executable": Terminal,
  "file.shortcut": Share2,
  "file.database": FileBox,
  "file.font": FileType,
  "file.package": Box,
  "file.markup": Braces,
  "file.unknown": FileQuestionMark,
  "file.generic": File,

  "icon-upload": Upload,
  "icon-file-add-fill": FilePlus,
  "icon-folder-add-fill": FolderPlus,
  "icon-scissors": Scissors,
  "icon-contentcut": Scissors,
  "icon-copy": Copy,
  "icon-copy1": Copy,
  "icon-paste": ClipboardPaste,
  "icon-download": Download,
  "icon-download1": Download,
  "icon-download2": Download,
  "icon-download2f": Download,
  "icon-edit-filling": SquarePen,
  "icon-rename": Pencil,
  "icon-bx-rename": Pencil,
  "icon-renamebox": SquarePen,
  "icon-delete": Trash2,
  "icon-delete1": Trash2,
  "icon-delete-fill": Trash2,
  "icon-setting": Settings,
  "icon-setting-filling": Settings,
  "icon-wrench": Wrench,
  "icon-password": Lock,
  "icon-user-fill": User,
  "icon-save-fill": Save,
  "icon-close": X,
  "icon-add": Plus,
  "icon-refresh": RefreshCw,
  "icon-back_android": ChevronLeft,
  "icon-unfold": ChevronDown,
  "icon-fenxiang": Search,
  "icon-viewgrid": LayoutGrid,
  "icon-view-list": List,
  "icon-listview": Rows3,
  "icon-homefill": House,
  "icon-home-fill": House,
  "icon-file": File,
  "icon-file-fill": File,
  "icon-file-common-filling": FileText,
  "icon-file-image-fill": FileImage,
  "icon-file-zip": FileArchive,
  "icon-file-zip-fill": FileArchive,
  "icon-folder": Folder,
  "icon-folder-fill": Folder,
  "icon-folder-open-fill": FolderOpen
};

const classicSymbolIcons: Record<string, string> = {
  "action.upload": "icon-upload",
  "action.new-file": "icon-file-add-fill",
  "action.new-folder": "icon-folder-add-fill",
  "action.cut": "icon-contentcut",
  "action.copy": "icon-copy",
  "action.paste": "icon-paste",
  "action.download": "icon-download",
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
  "action.previous": "icon-back_android",
  "action.down": "icon-unfold",
  "action.tools": "icon-wrench",

  "view.details": "icon-view-list",
  "view.list": "icon-listview",
  "view.icons": "icon-viewgrid",
  "view.tiles": "icon-viewgrid",
  "view.grid": "icon-viewgrid",

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

const classicSymbolName = computed(() => classicSymbolIcons[props.icon] ?? (props.icon.startsWith("icon-") ? props.icon : ""));
const useClassicSymbol = computed(() => appearanceStore.iconStyle === "classic" && Boolean(classicSymbolName.value));
const iconComponent = computed(() => useClassicSymbol.value ? undefined : lucideIcons[props.icon]);
const symbolHref = computed(() => `#${classicSymbolName.value || props.icon}`);

const normalizedSize = computed(() => {
  if (props.size === "large") return "1.5rem";
  if (props.size === "small") return "0.75rem";
  if (props.size === "normal") return "1rem";
  if (/^\d+$/.test(props.size)) return `${props.size}rem`;
  return props.size;
});

const iconStyle = computed(() => ({
  width: normalizedSize.value,
  height: normalizedSize.value,
  color: props.color
}));
</script>

<template>
  <component
      :is="iconComponent"
      v-if="iconComponent"
      class="app-icon app-icon-lucide"
      aria-hidden="true"
      :style="iconStyle"
      :stroke-width="strokeWidth" />
  <svg v-else class="app-icon app-icon-symbol" aria-hidden="true" :style="iconStyle">
    <use :href="symbolHref" :xlink:href="symbolHref" />
  </svg>
</template>

<style scoped>
.app-icon {
  display: inline-block;
  flex-shrink: 0;
  vertical-align: -0.125em;
}

.app-icon-lucide {
  fill: none;
  stroke: currentColor;
}

.app-icon-symbol {
  fill: currentColor;
}
</style>
