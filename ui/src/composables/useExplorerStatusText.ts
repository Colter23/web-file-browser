import {computed} from "vue";
import type {ComputedRef, Ref} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {useI18n} from "../i18n";
import {formatEntrySize} from "../utils/file-entry.ts";
import type {ExplorerDataSourceMode} from "./useExplorerFolderData.ts";

type ExplorerStatusTextOptions = {
  allEntries: ComputedRef<ExplorerEntry[]>;
  entries: ComputedRef<ExplorerEntry[]>;
  selectedEntries: ComputedRef<ExplorerEntry[]>;
  filterKeyword: ComputedRef<string>;
  hasMore: ComputedRef<boolean>;
  sourceMode: Ref<ExplorerDataSourceMode>;
  sourceTitle: Ref<string>;
  resultTotal: Ref<number | null>;
}

export type ExplorerFolderStatus = {
  sourceText: string;
  countText: string;
  moreText: string;
  title: string;
}

export type ExplorerSelectionStatus = {
  active: boolean;
  countText: string;
  detailText: string;
  title: string;
}

const hasLoadedFileSize = (entry: ExplorerEntry): entry is ExplorerEntry & {type: "file"; size: number} => {
  return entry.type === "file" && Number.isFinite(entry.size);
}

export const useExplorerStatusText = ({
  allEntries,
  entries,
  selectedEntries,
  filterKeyword,
  hasMore,
  sourceMode,
  sourceTitle,
  resultTotal
}: ExplorerStatusTextOptions) => {
  const {locale, t} = useI18n();
  const filterActive = computed(() => Boolean(filterKeyword.value));
  const listSeparator = computed(() => locale.value === "zh-CN" ? "，" : ", ");

  const emptyText = computed(() => {
    if (sourceMode.value === "search") return t("explorer.noSearchResult", {title: sourceTitle.value});
    if (sourceMode.value === "recent") return t("explorer.noRecent");
    if (filterActive.value) return t("explorer.noMatch", {keyword: filterKeyword.value});
    return t("explorer.emptyFolder");
  });

  const emptyHintText = computed(() => {
    if (sourceMode.value === "search") return t("explorer.searchHint");
    if (sourceMode.value === "recent") return t("explorer.recentHint");
    if (!filterActive.value) return "";
    return hasMore.value ? t("explorer.filterLoadedHint") : t("explorer.filterAllHint");
  });

  const selectedFileEntries = computed(() => selectedEntries.value.filter(entry => entry.type === "file"));

  const selectedFolderCount = computed(() => selectedEntries.value.length - selectedFileEntries.value.length);

  const selectedKnownSize = computed(() => selectedFileEntries.value.reduce((total, entry) => {
    return hasLoadedFileSize(entry) ? total + entry.size : total;
  }, 0));

  const selectedMissingSizeCount = computed(() => selectedFileEntries.value.filter(entry => !hasLoadedFileSize(entry)).length);

  const folderStatus = computed<ExplorerFolderStatus>(() => {
    const source = filterActive.value ? entries.value : allEntries.value;
    const folderCount = source.filter(entry => entry.type === "folder").length;
    const fileCount = source.length - folderCount;
    const sourceText = sourceMode.value === "search"
        ? t("explorer.searchResults")
        : sourceMode.value === "recent"
          ? t("explorer.recentFiles")
          : filterActive.value ? t("explorer.filterResults") : t("explorer.currentDirectory");
    const totalText = sourceMode.value === "search" && resultTotal.value !== null
        ? `${source.length} / ${t("common.items", {count: resultTotal.value})}`
        : t("common.items", {count: source.length});
    const countParts = [];
    if (folderCount) countParts.push(t("common.folders", {count: folderCount}));
    if (fileCount) countParts.push(t("common.files", {count: fileCount}));
    const countText = countParts.length ? t("explorer.countText", {total: totalText, parts: countParts.join(listSeparator.value)}) : totalText;
    const moreText = hasMore.value
        ? filterActive.value ? t("explorer.loadedOnly") : t("explorer.moreAvailable")
        : "";
    const title = moreText
        ? t("explorer.folderStatusTitleWithMore", {source: sourceText, count: countText, more: moreText})
        : t("explorer.folderStatusTitle", {source: sourceText, count: countText});
    return {sourceText, countText, moreText, title};
  });

  const selectedSizeText = computed(() => {
    const fileCount = selectedFileEntries.value.length;
    if (!fileCount) return "";
    const missing = selectedMissingSizeCount.value;
    if (missing === fileCount) return t("explorer.fileSizesUnloaded", {count: fileCount});
    if (missing) return t("explorer.knownFileSize", {size: formatEntrySize(selectedKnownSize.value), count: missing});
    return formatEntrySize(selectedKnownSize.value);
  });

  const selectedStatus = computed<ExplorerSelectionStatus>(() => {
    const selectedCount = selectedEntries.value.length;
    if (!selectedCount) {
      return {
        active: false,
        countText: t("common.noSelection"),
        detailText: "",
        title: t("common.noSelection")
      };
    }
    const detail = [];
    if (selectedFileEntries.value.length) detail.push(t("common.files", {count: selectedFileEntries.value.length}));
    if (selectedFolderCount.value) detail.push(t("common.folders", {count: selectedFolderCount.value}));
    if (selectedSizeText.value) detail.push(selectedSizeText.value);
    const countText = t("explorer.selected", {count: selectedCount});
    const detailText = detail.join(listSeparator.value);
    return {
      active: true,
      countText,
      detailText,
      title: `${countText}${detailText ? ` · ${detailText}` : ""}`
    };
  });

  const folderStatusText = computed(() => folderStatus.value.title);
  const selectedStatusText = computed(() => selectedStatus.value.title);

  return {
    filterActive,
    emptyText,
    emptyHintText,
    folderStatus,
    selectedStatus,
    folderStatusText,
    selectedStatusText
  };
}
