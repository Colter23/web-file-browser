<script setup lang="ts">
import {onBeforeUnmount, onMounted, ref} from "vue";
import Breadcrumb from "../Breadcrumb.vue";
import Icon from "../Icon.vue";
import type {ExplorerIconSize, ExplorerViewMode} from "../../class";

type BreadcrumbExpose = {
  focusInput: () => void;
}

type NavigateComplete = (navigated: boolean) => void;

type ViewModeOption = {
  key: string;
  mode: ExplorerViewMode;
  iconSize?: ExplorerIconSize;
  label: string;
  description: string;
  icon: string;
  shortcut: string;
}

type ViewModeSelection = {
  mode: ExplorerViewMode;
  iconSize?: ExplorerIconSize;
  label: string;
}

const props = defineProps<{
  canNavigateBack: boolean;
  canNavigateForward: boolean;
  canNavigateUp: boolean;
  navigateBackTitle: string;
  navigateForwardTitle: string;
  navigateUpTitle: string;
  viewModeIcon: string;
  viewModeLabel: string;
  viewModeButtonTitle: string;
  viewMode: ExplorerViewMode;
  iconSize: ExplorerIconSize;
  previewPanelVisible: boolean;
  canTogglePreviewPane: boolean;
}>();

const emit = defineEmits<{
  (e: "navigate-back"): void;
  (e: "navigate-forward"): void;
  (e: "navigate-up"): void;
  (e: "refresh"): void;
  (e: "breadcrumb-navigate", path: string, complete?: NavigateComplete): void;
  (e: "select-view-mode", selection: ViewModeSelection): void;
  (e: "toggle-preview"): void;
}>();

const breadcrumbRef = ref<BreadcrumbExpose | null>(null);
const viewMenuRef = ref<HTMLElement | null>(null);
const viewModeMenuOpen = ref(false);

const viewModeOptions: ViewModeOption[] = [
  {
    key: "details",
    mode: "details",
    iconSize: "small",
    label: "详细信息",
    description: "显示日期、类型和大小",
    icon: "icon-view-list",
    shortcut: "Ctrl+Shift+6"
  },
  {
    key: "list",
    mode: "list",
    iconSize: "small",
    label: "列表",
    description: "紧凑排列，快速扫描",
    icon: "icon-listview",
    shortcut: "Ctrl+Shift+5"
  },
  {
    key: "tiles",
    mode: "tiles",
    iconSize: "medium",
    label: "平铺",
    description: "图标与文件信息并列",
    icon: "icon-file-common-filling",
    shortcut: "Ctrl+Shift+7"
  },
  {
    key: "icons-large",
    mode: "icons",
    iconSize: "large",
    label: "大图标",
    description: "适合浏览图片和媒体",
    icon: "icon-viewgrid",
    shortcut: "Ctrl+Shift+1/2"
  },
  {
    key: "icons-medium",
    mode: "icons",
    iconSize: "medium",
    label: "中图标",
    description: "兼顾预览和密度",
    icon: "icon-viewgrid",
    shortcut: "Ctrl+Shift+3"
  },
  {
    key: "icons-small",
    mode: "icons",
    iconSize: "small",
    label: "小图标",
    description: "更多项目同屏展示",
    icon: "icon-viewgrid",
    shortcut: "Ctrl+Shift+4"
  }
];

const closeViewModeMenu = () => {
  viewModeMenuOpen.value = false;
}

const toggleViewModeMenu = () => {
  viewModeMenuOpen.value = !viewModeMenuOpen.value;
}

const isViewModeOptionActive = (option: ViewModeOption) => {
  if (option.mode !== props.viewMode) return false;
  return option.mode !== "icons" || option.iconSize === props.iconSize;
}

const selectViewMode = (option: ViewModeOption) => {
  closeViewModeMenu();
  emit("select-view-mode", {
    mode: option.mode,
    iconSize: option.iconSize,
    label: option.label
  });
}

const handleDocumentPointerDown = (event: PointerEvent) => {
  if (!viewModeMenuOpen.value) return;
  const target = event.target;
  if (target instanceof Node && viewMenuRef.value?.contains(target)) return;
  closeViewModeMenu();
}

const handleDocumentKeyDown = (event: KeyboardEvent) => {
  if (event.key === "Escape") closeViewModeMenu();
}

onMounted(() => {
  window.addEventListener("pointerdown", handleDocumentPointerDown);
  window.addEventListener("keydown", handleDocumentKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("pointerdown", handleDocumentPointerDown);
  window.removeEventListener("keydown", handleDocumentKeyDown);
});

defineExpose({
  focusInput: () => breadcrumbRef.value?.focusInput()
});
</script>

<template>
  <div class="path-row">
    <button class="nav-button" :disabled="!canNavigateBack" :title="navigateBackTitle" @click="emit('navigate-back')">
      <icon icon="icon-back_android" size="large" />
    </button>
    <button class="nav-button" :disabled="!canNavigateForward" :title="navigateForwardTitle" @click="emit('navigate-forward')">
      <icon icon="icon-back_android" size="large" class="rotate-180" />
    </button>
    <button class="nav-button" :disabled="!canNavigateUp" :title="navigateUpTitle" @click="emit('navigate-up')">
      <icon icon="icon-back_android" size="large" class="rotate-90" />
    </button>
    <button class="nav-button" title="刷新 (F5 / Ctrl+R)" @click="emit('refresh')">
      <icon icon="icon-refresh" size="large" />
    </button>
    <breadcrumb ref="breadcrumbRef" @navigate="(path, complete) => emit('breadcrumb-navigate', path, complete)"></breadcrumb>
    <div ref="viewMenuRef" class="view-menu">
      <button
          class="view-button"
          :class="{active: viewModeMenuOpen}"
          :title="viewModeButtonTitle"
          aria-haspopup="menu"
          :aria-expanded="viewModeMenuOpen"
          @click="toggleViewModeMenu">
        <icon :icon="viewModeIcon" />
        <span>{{ viewModeLabel }}</span>
        <icon icon="icon-unfold" class="view-caret" />
      </button>
      <div v-if="viewModeMenuOpen" class="view-menu-panel" role="menu" aria-label="查看模式">
        <button
            v-for="option in viewModeOptions"
            :key="option.key"
            class="view-menu-item"
            :class="{active: isViewModeOptionActive(option)}"
            role="menuitemradio"
            :aria-checked="isViewModeOptionActive(option)"
            @click="selectViewMode(option)">
          <icon :icon="option.icon" />
          <span class="view-menu-copy">
            <strong>{{ option.label }}</strong>
            <small>{{ option.description }}</small>
          </span>
          <kbd>{{ option.shortcut }}</kbd>
        </button>
      </div>
    </div>
    <button
        class="view-button"
        :class="{active: previewPanelVisible}"
        :disabled="!canTogglePreviewPane"
        title="预览窗格 (Alt+P)"
        @click="emit('toggle-preview')">
      <icon icon="icon-file-image-fill" />
      <span>{{ previewPanelVisible ? "关闭预览" : "预览窗格" }}</span>
    </button>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.path-row {
  @apply flex h-14 shrink-0 items-center gap-2 border-b border-slate-200 bg-white/70 px-3;
}

.nav-button,
.view-button {
  @apply inline-flex items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-700 hover:bg-blue-50;
}

.nav-button {
  @apply h-10 w-10 shrink-0;
}

.nav-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:bg-white;
}

.view-button {
  @apply h-10 shrink-0 gap-2 px-3 text-sm;
}

.view-menu {
  @apply relative shrink-0;
}

.view-caret {
  @apply text-[0.65rem] text-slate-500;
}

.view-menu-panel {
  @apply absolute right-0 top-[calc(100%+0.35rem)] z-30 w-72 overflow-hidden rounded-lg border border-slate-200 bg-white py-1 shadow-xl;
}

.view-menu-item {
  @apply grid w-full grid-cols-[1.25rem_minmax(0,1fr)_auto] items-center gap-3 px-3 py-2 text-left text-sm text-slate-700 hover:bg-blue-50;
}

.view-menu-item.active {
  @apply bg-blue-50 text-blue-700;
}

.view-menu-copy {
  @apply flex min-w-0 flex-col;
}

.view-menu-copy strong {
  @apply truncate text-sm font-medium;
}

.view-menu-copy small {
  @apply truncate text-xs text-slate-500;
}

.view-menu-item.active .view-menu-copy small {
  @apply text-blue-600;
}

.view-menu-item kbd {
  @apply rounded border border-slate-200 bg-slate-50 px-1.5 py-0.5 text-[0.68rem] font-normal text-slate-500;
}

.view-button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.view-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:bg-white;
}
</style>
