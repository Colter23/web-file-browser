import {computed} from "vue";
import type {ComputedRef} from "vue";
import type {ExplorerEntry} from "../components/explorer/types.ts";
import {formatEntrySize} from "../utils/file-entry.ts";

type ExplorerStatusTextOptions = {
  allEntries: ComputedRef<ExplorerEntry[]>;
  entries: ComputedRef<ExplorerEntry[]>;
  selectedEntries: ComputedRef<ExplorerEntry[]>;
  filterKeyword: ComputedRef<string>;
  hasMore: ComputedRef<boolean>;
}

const hasLoadedFileSize = (entry: ExplorerEntry): entry is ExplorerEntry & {type: "file"; size: number} => {
  return entry.type === "file" && Number.isFinite(entry.size);
}

export const useExplorerStatusText = ({
  allEntries,
  entries,
  selectedEntries,
  filterKeyword,
  hasMore
}: ExplorerStatusTextOptions) => {
  const filterActive = computed(() => Boolean(filterKeyword.value));

  const emptyText = computed(() => {
    if (filterActive.value) return `没有匹配“${filterKeyword.value}”的项目`;
    return "此文件夹为空";
  });

  const emptyHintText = computed(() => {
    if (!filterActive.value) return "";
    return hasMore.value ? "当前只筛选已加载项目，清除筛选后可继续加载更多。" : "清除筛选可查看全部已加载项目。";
  });

  const selectedFileEntries = computed(() => selectedEntries.value.filter(entry => entry.type === "file"));

  const selectedFolderCount = computed(() => selectedEntries.value.length - selectedFileEntries.value.length);

  const selectedKnownSize = computed(() => selectedFileEntries.value.reduce((total, entry) => {
    return hasLoadedFileSize(entry) ? total + entry.size : total;
  }, 0));

  const selectedMissingSizeCount = computed(() => selectedFileEntries.value.filter(entry => !hasLoadedFileSize(entry)).length);

  const selectedCountText = computed(() => {
    const count = selectedEntries.value.length;
    if (!count) return "未选择项目";
    return `已选择 ${count} 项`;
  });

  const totalCountText = computed(() => {
    const loadedCount = allEntries.value.length;
    const hasMoreText = hasMore.value ? "，还有更多" : "";
    return filterActive.value ? `已加载 ${loadedCount} 项，筛选 ${entries.value.length} 项${hasMoreText}` : `已加载 ${loadedCount} 项${hasMoreText}`;
  });

  const folderStatusText = computed(() => {
    const source = filterActive.value ? entries.value : allEntries.value;
    const folderCount = source.filter(entry => entry.type === "folder").length;
    const fileCount = source.length - folderCount;
    const prefix = filterActive.value ? "筛选结果" : "当前已加载";
    const suffix = hasMore.value && !filterActive.value ? "，还有更多" : "";
    return `${prefix}：${folderCount} 个文件夹，${fileCount} 个文件${suffix}`;
  });

  const selectedSizeText = computed(() => {
    const fileCount = selectedFileEntries.value.length;
    if (!fileCount) return "";
    const missing = selectedMissingSizeCount.value;
    if (missing === fileCount) return `${fileCount} 个文件大小未加载`;
    if (missing) return `${formatEntrySize(selectedKnownSize.value)} 已知，${missing} 个文件未加载大小`;
    return formatEntrySize(selectedKnownSize.value);
  });

  const selectedStatusText = computed(() => {
    const selectedCount = selectedEntries.value.length;
    if (!selectedCount) return "未选择项目";
    const detail = [];
    if (selectedFileEntries.value.length) detail.push(`${selectedFileEntries.value.length} 个文件`);
    if (selectedFolderCount.value) detail.push(`${selectedFolderCount.value} 个文件夹`);
    if (selectedSizeText.value) detail.push(selectedSizeText.value);
    return `已选择 ${selectedCount} 项${detail.length ? ` · ${detail.join("，")}` : ""}`;
  });

  return {
    filterActive,
    emptyText,
    emptyHintText,
    selectedCountText,
    totalCountText,
    folderStatusText,
    selectedStatusText
  };
}
