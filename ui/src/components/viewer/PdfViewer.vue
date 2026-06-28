<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import type {ShellNoticePayload} from "../shell/types.ts";
import {fileContentUrl} from "../../network/api.ts";
import {formatEntryDate, formatEntrySize} from "../../utils/file-entry.ts";
import FileTypeIcon from "../FileTypeIcon.vue";
import Icon from "../Icon.vue";

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
const pageFullscreenTitle = computed(() => pageFullscreen.value ? "退出网页全屏 (F)" : "网页全屏 (F)");
const browserFullscreenTitle = computed(() => browserFullscreen.value ? "退出浏览器全屏" : "浏览器全屏");
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
      title: "无法全屏",
      message: "当前浏览器未允许进入全屏，仍可在页面内查看 PDF。"
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
      <div class="pdf-viewer-toolbar">
        <div class="pdf-viewer-title">
          <span class="pdf-viewer-title-icon">
            <file-type-icon kind="pdf" :name="currentEntry.name" :extension="currentEntry.extension" size="1.15rem" />
          </span>
          <span class="pdf-viewer-title-text">
            <strong>{{ currentEntry.name }}</strong>
            <span>{{ subtitle }}</span>
          </span>
        </div>
        <div class="pdf-viewer-actions">
          <div class="pdf-viewer-action-group">
            <button title="上一份 PDF (←)" :disabled="!canShowPrevious" @click="showAdjacent(-1)">
              <icon icon="action.previous" color="currentColor" size="1.1rem" />
            </button>
            <button title="下一份 PDF (→)" :disabled="!canShowNext" @click="showAdjacent(1)">
              <icon icon="action.next" color="currentColor" size="1.1rem" />
            </button>
          </div>
          <div class="pdf-viewer-action-group">
            <button title="新窗口打开" @click="openInNewWindow">
              <icon icon="action.open-new-tab" color="currentColor" />
            </button>
          </div>
          <div class="pdf-viewer-action-group">
            <button :title="pageFullscreenTitle" :class="{active: pageFullscreen}" @click="togglePageFullscreen">
              <icon :icon="pageFullscreenIcon" color="currentColor" />
            </button>
            <button :title="browserFullscreenTitle" :class="{active: browserFullscreen}" @click="toggleBrowserFullscreen">
              <icon :icon="browserFullscreenIcon" color="currentColor" />
            </button>
          </div>
          <div class="pdf-viewer-action-group">
            <button title="下载" @click="downloadCurrent">
              <icon icon="action.download" color="currentColor" />
            </button>
            <button title="关闭" @click="close">
              <icon icon="action.close" color="currentColor" />
            </button>
          </div>
        </div>
      </div>
      <div class="pdf-viewer-stage">
        <iframe
            v-if="sourceUrl"
            class="pdf-viewer-frame"
            :src="sourceUrl"
            :title="currentEntry.name"
            @load="handleLoad">
        </iframe>
        <div v-if="loading" class="pdf-viewer-status">正在加载 PDF...</div>
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

.pdf-viewer-toolbar {
  @apply flex min-h-14 shrink-0 items-center justify-between gap-3 border-b px-3.5 backdrop-blur;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.12));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 6%, rgba(15, 23, 42, 0.74));
}

.pdf-viewer-title {
  @apply flex min-w-0 items-center gap-2.5;
}

.pdf-viewer-title-icon {
  @apply grid h-8 w-8 shrink-0 place-items-center rounded-md border border-white/10 bg-white/10 text-blue-200 shadow-sm;
}

.pdf-viewer-title-text {
  @apply flex min-w-0 flex-col;
}

.pdf-viewer-title strong {
  @apply truncate text-sm font-semibold leading-5;
}

.pdf-viewer-title-text > span {
  @apply truncate text-xs leading-4 text-slate-300;
}

.pdf-viewer-actions {
  @apply flex shrink-0 items-center gap-1.5 text-xs text-slate-100;
}

.pdf-viewer-action-group {
  @apply inline-flex h-9 items-center overflow-hidden rounded-lg border p-0.5 shadow-sm;
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 10%, rgba(255, 255, 255, 0.14));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 3%, rgba(255, 255, 255, 0.1));
}

.pdf-viewer-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-transparent bg-transparent px-2 text-sm font-medium text-white transition hover:bg-white/20;
}

.pdf-viewer-actions button:disabled {
  @apply cursor-not-allowed opacity-35 hover:bg-transparent;
}

.pdf-viewer-actions button:focus-visible {
  @apply outline-none;
  border-color: rgba(255, 255, 255, 0.78);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--app-accent, #2563eb) 45%, rgba(255, 255, 255, 0.25));
}

.pdf-viewer-actions button.active {
  @apply text-white;
  border-color: color-mix(in srgb, var(--app-accent-border, #bfdbfe) 72%, transparent);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 36%, transparent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--app-accent-border, #bfdbfe) 22%, transparent);
}

.pdf-viewer-stage {
  @apply relative min-h-0 grow overflow-hidden;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 4%, rgba(2, 6, 23, 0.18));
}

.pdf-viewer-frame {
  @apply block h-full w-full border-0 bg-white;
}

.pdf-viewer-status {
  @apply absolute left-1/2 top-1/2 rounded-lg border border-white/10 bg-slate-950/65 px-3 py-2 text-sm text-slate-100 shadow-xl backdrop-blur;
  transform: translate(-50%, -50%);
}

@media (max-width: 840px) {
  .pdf-viewer-toolbar {
    @apply items-start;
  }

  .pdf-viewer-actions {
    @apply flex-wrap justify-end;
  }
}
</style>
