import {ref} from "vue";
import type {IndexStatus} from "../class.ts";
import type {ShellNoticeKind} from "../components/shell/types.ts";
import {translate} from "../i18n";

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
      title: translate("search.indexDisabledTitle"),
      message: translate("search.indexDisabledMessage")
    };
  }
  if (status.state === "building") {
    return {
      key: `building:${status.indexedEntries}`,
      kind: "info",
      title: translate("search.indexBuildingTitle"),
      message: translate("search.indexBuildingMessage", {count: status.indexedEntries ?? 0})
    };
  }
  if (status.state === "error") {
    return {
      key: `error:${status.lastError ?? ""}`,
      kind: "error",
      title: translate("search.indexErrorTitle"),
      message: status.lastError
          ? translate("search.indexErrorMessage", {error: status.lastError})
          : translate("search.indexErrorFallback")
    };
  }
  if (status.indexedEntries === 0) {
    return {
      key: "empty",
      kind: "warning",
      title: translate("search.indexEmptyTitle"),
      message: translate("search.indexEmptyMessage")
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
        translate("search.indexUnknownMessage"),
        "warning",
        translate("search.indexUnknownTitle")
      );
    }
  }

  return {
    searchIndexStatus,
    inspectSearchIndexBeforeSearch
  };
}
