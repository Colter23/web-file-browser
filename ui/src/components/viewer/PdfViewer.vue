<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import type {ShellNoticePayload} from "../shell/types.ts";
import {useI18n} from "../../i18n";
import {fileContentUrl} from "../../network/api.ts";
import {formatEntryDate, formatEntrySize} from "../../utils/file-entry.ts";
import Icon from "../Icon.vue";
import ViewerActionGroup from "./ViewerActionGroup.vue";
import ViewerStatus from "./ViewerStatus.vue";
import ViewerToolbar from "./ViewerToolbar.vue";

const props = defineProps<{
  visible: boolean;
  entry: ExplorerEntry | null;
  entries: ExplorerEntry[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "select", entry: ExplorerEntry): void;
  (e: "download", entry: ExplorerEntry): void;
  (e: "notice", payload: ShellNoticePayload): void;
}>();

const {t} = useI18n();
const viewerRef = ref<HTMLElement | null>(null);
const loading = ref(false);
const pageFullscreen = ref(false);
const browserFullscreen = ref(false);
const reloadKey = ref(0);

const currentEntry = computed(() => props.visible ? props.entry : null);
const sourceUrl = computed(() => currentEntry.value ? fileContentUrl(currentEntry.value.path, {cacheKey: reloadKey.value}) : "");

const currentIndex = computed(() => {
  const entry = props.entry;
  if (!entry) return -1;
  return props.entries.findIndex(item => item.path === entry.path);
});

const pdfCount = computed(() => props.entries.length);
const canShowPrevious = computed(() => currentIndex.value > 0);
const canShowNext = computed(() => currentIndex.value >= 0 && currentIndex.value < props.entries.length - 1);
const pageFullscreenTitle = computed(() => pageFullscreen.value ? t("viewer.exitPageFullscreen") : t("viewer.pageFullscreen"));
const browserFullscreenTitle = computed(() => browserFullscreen.value ? t("viewer.exitBrowserFullscreen") : t("viewer.browserFullscreen"));
const pageFullscreenIcon = computed(() => pageFullscreen.value ? "viewer.page-fullscreen-off" : "viewer.page-fullscreen");
const browserFullscreenIcon = computed(() => browserFullscreen.value ? "viewer.browser-fullscreen-off" : "viewer.browser-fullscreen");

const subtitle = computed(() => {
  const entry = props.entry;
  if (!entry) return "";
  const position = currentIndex.value >= 0 && pdfCount.value > 1 ? `${currentIndex.value + 1} / ${pdfCount.value} · ` : "";
  return `${position}${formatEntrySize(entry.size, "0 B")} · ${formatEntryDate(entry.modified)}`;
});

const focusViewer = async () => {
  await nextTick();
  viewerRef.value?.focus();
}

const resetRuntimeState = () => {
  const fullscreenElement = document.fullscreenElement;
  if (fullscreenElement && viewerRef.value?.contains(fullscreenElement)) {
    void document.exitFullscreen().catch(() => undefined);
  }
  loading.value = false;
  pageFullscreen.value = false;
  browserFullscreen.value = false;
}

const prepareEntry = async () => {
  if (!props.visible || !props.entry) return;
  loading.value = true;
  reloadKey.value += 1;
  await focusViewer();
}

const close = () => emit("close");

const showAdjacent = (direction: -1 | 1) => {
  const next = props.entries[currentIndex.value + direction];
  if (next) emit("select", next);
}

const togglePageFullscreen = async () => {
  pageFullscreen.value = !pageFullscreen.value;
  await focusViewer();
}

const toggleBrowserFullscreen = async () => {
  const target = viewerRef.value;
  if (!target) return;
  try {
    if (document.fullscreenElement === target) {
      await document.exitFullscreen();
    } else {
      await target.requestFullscreen();
    }
    await focusViewer();
  } catch {
    emit("notice", {
      kind: "warning",
      title: t("viewer.fullscreenFailedTitle"),
      message: t("viewer.pdfFullscreenFailed")
    });
  }
}

const openInNewWindow = () => {
  if (!sourceUrl.value) return;
  window.open(sourceUrl.value, "_blank", "noopener,noreferrer");
}

const downloadCurrent = () => {
  if (props.entry) emit("download", props.entry);
}

const handleLoad = () => {
  loading.value = false;
}

const handleFullscreenChange = () => {
  browserFullscreen.value = document.fullscreenElement === viewerRef.value;
}

const handleWindowKeyDown = (event: KeyboardEvent) => {
  if (!props.visible) return;
  const key = event.key.toLowerCase();
  if (key === "escape") {
    event.preventDefault();
    event.stopImmediatePropagation();
    close();
    return;
  }
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    event.stopImmediatePropagation();
    showAdjacent(-1);
    return;
  }
  if (event.key === "ArrowRight") {
    event.preventDefault();
    event.stopImmediatePropagation();
    showAdjacent(1);
    return;
  }
  if (key === "f" && !event.ctrlKey && !event.metaKey && !event.altKey) {
    event.preventDefault();
    event.stopImmediatePropagation();
    void togglePageFullscreen();
  }
}

watch(() => props.visible, visible => {
  if (visible) {
    void prepareEntry();
    return;
  }
  resetRuntimeState();
});

watch(() => props.entry?.path, () => {
  void prepareEntry();
});

onMounted(() => {
  window.addEventListener("keydown", handleWindowKeyDown, true);
  document.addEventListener("fullscreenchange", handleFullscreenChange);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleWindowKeyDown, true);
  document.removeEventListener("fullscreenchange", handleFullscreenChange);
  resetRuntimeState();
});
</script>

<template>
  <Teleport to="body" :disabled="!pageFullscreen">
    <section
        v-if="currentEntry"
        ref="viewerRef"
        class="pdf-viewer"
        :class="{pageFullscreen, browserFullscreen}"
        tabindex="-1"
        @keydown.esc.prevent.stop="close">
      <viewer-toolbar kind="pdf" :name="currentEntry.name" :extension="currentEntry.extension" :subtitle="subtitle" icon-tone="pdf">
        <viewer-action-group>
          <button :title="t('viewer.previousPdf')" :disabled="!canShowPrevious" @click="showAdjacent(-1)">
            <icon icon="action.previous" color="currentColor" size="1.1rem" />
          </button>
          <button :title="t('viewer.nextPdf')" :disabled="!canShowNext" @click="showAdjacent(1)">
            <icon icon="action.next" color="currentColor" size="1.1rem" />
          </button>
        </viewer-action-group>
        <viewer-action-group>
          <button :title="t('viewer.openNewWindow')" @click="openInNewWindow">
            <icon icon="action.open-new-tab" color="currentColor" />
          </button>
        </viewer-action-group>
        <viewer-action-group>
          <button :title="pageFullscreenTitle" :class="{active: pageFullscreen}" @click="togglePageFullscreen">
            <icon :icon="pageFullscreenIcon" color="currentColor" />
          </button>
          <button :title="browserFullscreenTitle" :class="{active: browserFullscreen}" @click="toggleBrowserFullscreen">
            <icon :icon="browserFullscreenIcon" color="currentColor" />
          </button>
        </viewer-action-group>
        <viewer-action-group>
          <button :title="t('common.download')" @click="downloadCurrent">
            <icon icon="action.download" color="currentColor" />
          </button>
          <button :title="t('common.close')" @click="close">
            <icon icon="action.close" color="currentColor" />
          </button>
        </viewer-action-group>
      </viewer-toolbar>
      <div class="pdf-viewer-stage">
        <iframe
            v-if="sourceUrl"
            class="pdf-viewer-frame"
            :src="sourceUrl"
            :title="currentEntry.name"
            @load="handleLoad">
        </iframe>
        <viewer-status v-if="loading">{{ t("viewer.pdfLoading") }}</viewer-status>
      </div>
    </section>
  </Teleport>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.pdf-viewer {
  @apply absolute inset-0 z-40 flex flex-col overflow-hidden rounded-lg text-white outline-none backdrop-blur-sm;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 7%, rgba(2, 6, 23, 0.78));
}

.pdf-viewer:is(.pageFullscreen, .browserFullscreen) {
  @apply fixed inset-0 z-50 rounded-none;
}

.pdf-viewer-stage {
  @apply relative min-h-0 grow overflow-hidden;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 4%, rgba(2, 6, 23, 0.18));
}

.pdf-viewer-frame {
  @apply block h-full w-full border-0 bg-white;
}
</style>
