import {ref} from "vue";
import type {IndexStatus} from "../class.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";

type SearchIndexStatusHintOptions = {
  getIndexStatus: () => Promise<IndexStatus>;
  showNotice: (message: string, kind?: ShellNoticeKind, title?: string, timeoutMs?: number) => void;
  minNoticeIntervalMs?: number;
}

const defaultNoticeIntervalMs = 15000;

const indexStateNotice = (status: IndexStatus): {
  key: string;
  title: string;
  message: string;
  kind: ShellNoticeKind;
  timeoutMs?: number;
} | null => {
  if (!status.enabled || status.state === "disabled") {
    return {
      key: "disabled",
      kind: "warning",
      title: "搜索索引未启用",
      message: "当前后端未启用搜索索引，索引搜索可能没有结果。"
    };
  }
  if (status.state === "building") {
    return {
      key: `building:${status.indexedEntries}`,
      kind: "info",
      title: "索引重建中",
      message: `搜索索引正在重建，当前已索引 ${status.indexedEntries ?? 0} 项，结果可能暂时不完整。`
    };
  }
  if (status.state === "error") {
    return {
      key: `error:${status.lastError ?? ""}`,
      kind: "error",
      title: "搜索索引异常",
      message: status.lastError ? `搜索索引异常：${status.lastError}` : "搜索索引状态异常，结果可能不可用。"
    };
  }
  if (status.indexedEntries === 0) {
    return {
      key: "empty",
      kind: "warning",
      title: "搜索索引为空",
      message: "搜索索引目前没有已索引项目，搜索可能没有结果。"
    };
  }
  return null;
}

export const useSearchIndexStatusHint = ({
  getIndexStatus,
  showNotice,
  minNoticeIntervalMs = defaultNoticeIntervalMs
}: SearchIndexStatusHintOptions) => {
  const searchIndexStatus = ref<IndexStatus | null>(null);
  let lastNoticeKey = "";
  let lastNoticeAt = 0;

  const showThrottledNotice = (
    key: string,
    message: string,
    kind: ShellNoticeKind,
    title: string,
    timeoutMs?: number
  ) => {
    const now = Date.now();
    if (key === lastNoticeKey && now - lastNoticeAt < minNoticeIntervalMs) return;
    lastNoticeKey = key;
    lastNoticeAt = now;
    showNotice(message, kind, title, timeoutMs);
  }

  const inspectSearchIndexBeforeSearch = async () => {
    try {
      const status = await getIndexStatus();
      searchIndexStatus.value = status;
      const notice = indexStateNotice(status);
      if (!notice) return;
      showThrottledNotice(notice.key, notice.message, notice.kind, notice.title, notice.timeoutMs);
    } catch {
      searchIndexStatus.value = null;
      showThrottledNotice(
        "unknown",
        "无法读取搜索索引状态，仍会尝试搜索。",
        "warning",
        "搜索状态未知"
      );
    }
  }

  return {
    searchIndexStatus,
    inspectSearchIndexBeforeSearch
  };
}
