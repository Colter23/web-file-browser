<script setup lang="ts">
import {ref} from "vue";
import Breadcrumb from "../Breadcrumb.vue";
import Icon from "../Icon.vue";
import type {ExplorerIconSize, ExplorerViewMode} from "../../class";
import type {ExplorerViewModeSelection} from "../../composables/useExplorerViewMode.ts";
import ViewModeMenu from "./ViewModeMenu.vue";

type BreadcrumbExpose = {
  focusInput: () => void;
}

type NavigateComplete = (navigated: boolean) => void;

defineProps<{
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
  (e: "select-view-mode", selection: ExplorerViewModeSelection): void;
  (e: "toggle-preview"): void;
}>();

const breadcrumbRef = ref<BreadcrumbExpose | null>(null);

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
    <view-mode-menu
        :icon="viewModeIcon"
        :label="viewModeLabel"
        :title="viewModeButtonTitle"
        :view-mode="viewMode"
        :icon-size="iconSize"
        @select="selection => emit('select-view-mode', selection)" />
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

.view-button.active {
  @apply border-blue-200 bg-blue-50 text-blue-700;
}

.view-button:disabled {
  @apply cursor-not-allowed text-slate-300 hover:bg-white;
}
</style>
