import {computed, type ComputedRef} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {translate} from "../i18n";
import {isExtractableArchiveEntry} from "../utils/file-entry.ts";

type MainViewSelectionCommandsOptions = {
  singleSelection: ComputedRef<ExplorerEntry | null>;
  selectedCount: ComputedRef<number>;
  hasSelection: ComputedRef<boolean>;
  hasClipboard: ComputedRef<boolean>;
  clipboardText: ComputedRef<string>;
  editorVisible: () => boolean;
}

export const useMainViewSelectionCommands = ({
  singleSelection,
  selectedCount,
  hasSelection,
  hasClipboard,
  clipboardText,
  editorVisible
}: MainViewSelectionCommandsOptions) => {
  const canDownloadSelection = computed(() => singleSelection.value?.type === "file");
  const canPreviewSelection = computed(() => singleSelection.value?.type === "file");
  const canTogglePreviewPane = computed(() => !editorVisible());
  const canRenameSelection = computed(() => Boolean(singleSelection.value));
  const canArchiveSelection = computed(() => hasSelection.value);
  const canDeleteSelection = computed(() => hasSelection.value);
  const canExtractSelection = computed(() => isExtractableArchiveEntry(singleSelection.value));
  const canPasteSelection = computed(() => hasClipboard.value);
  const selectionStatusText = computed(() => {
    const selectionText = hasSelection.value
        ? translate("explorer.selected", {count: selectedCount.value})
        : translate("common.noSelection");
    return translate("clipboard.selectionStatus", {selection: selectionText, clipboard: clipboardText.value});
  });

  return {
    canDownloadSelection,
    canPreviewSelection,
    canTogglePreviewPane,
    canRenameSelection,
    canArchiveSelection,
    canDeleteSelection,
    canExtractSelection,
    canPasteSelection,
    selectionStatusText
  };
}
