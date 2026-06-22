import {nextTick} from "vue";
import type {ExplorerIconSize, ExplorerViewMode} from "../class.ts";
import {useFileStore} from "../store";

type ViewDensityStep = {
  mode: ExplorerViewMode;
  iconSize: ExplorerIconSize;
}

type ExplorerViewDensityOptions = {
  focusViewport: () => void;
  observePendingThumbnails: () => void;
  viewportHeight: () => number;
}

const viewWheelStepThreshold = 80;

const viewDensitySteps: ViewDensityStep[] = [
  {mode: "details", iconSize: "small"},
  {mode: "list", iconSize: "small"},
  {mode: "tiles", iconSize: "medium"},
  {mode: "icons", iconSize: "small"},
  {mode: "icons", iconSize: "medium"},
  {mode: "icons", iconSize: "large"}
];

export const useExplorerViewDensity = ({focusViewport, observePendingThumbnails, viewportHeight}: ExplorerViewDensityOptions) => {
  const fileStore = useFileStore();
  let viewWheelDelta = 0;

  const focusAfterViewChange = async () => {
    await nextTick();
    focusViewport();
  }

  const setViewMode = (mode: ExplorerViewMode) => {
    fileStore.setViewMode(mode);
    void focusAfterViewChange();
  }

  const currentViewDensityIndex = () => {
    if (fileStore.viewMode === "icons") {
      const index = viewDensitySteps.findIndex(step => step.mode === "icons" && step.iconSize === fileStore.iconSize);
      return index >= 0 ? index : viewDensitySteps.findIndex(step => step.mode === "icons" && step.iconSize === "medium");
    }
    const index = viewDensitySteps.findIndex(step => step.mode === fileStore.viewMode);
    return index >= 0 ? index : 0;
  }

  const setViewDensityStep = async (index: number) => {
    const nextIndex = Math.min(viewDensitySteps.length - 1, Math.max(0, index));
    const step = viewDensitySteps[nextIndex];
    if (!step) return;
    if (fileStore.viewMode === step.mode && fileStore.iconSize === step.iconSize) return;
    fileStore.setViewMode(step.mode);
    fileStore.setIconSize(step.iconSize);
    await focusAfterViewChange();
    observePendingThumbnails();
  }

  const wheelDeltaPixels = (event: WheelEvent) => {
    if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) return event.deltaY * 32;
    if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) return event.deltaY * (viewportHeight() || 800);
    return event.deltaY;
  }

  const handleViewportWheel = (event: WheelEvent) => {
    if (!event.ctrlKey && !event.metaKey) {
      viewWheelDelta = 0;
      return;
    }
    event.preventDefault();
    const delta = wheelDeltaPixels(event);
    if (!delta) return;
    viewWheelDelta += delta;
    if (Math.abs(viewWheelDelta) < viewWheelStepThreshold) return;
    const direction = viewWheelDelta < 0 ? 1 : -1;
    viewWheelDelta = 0;
    void setViewDensityStep(currentViewDensityIndex() + direction);
  }

  const cycleIconSize = () => {
    const next = fileStore.iconSize === "small" ? "medium" : fileStore.iconSize === "medium" ? "large" : "small";
    fileStore.setIconSize(next);
    void focusAfterViewChange();
  }

  return {
    setViewMode,
    handleViewportWheel,
    cycleIconSize
  };
}
