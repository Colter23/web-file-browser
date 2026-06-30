<script setup lang="ts">
import {computed, onBeforeUnmount, ref, watch} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {useI18n} from "../../i18n";
import {getFile} from "../../network/api.ts";
import type {ShellNoticePayload} from "../shell/types.ts";
import Icon from "../Icon.vue";
import PreviewToolRow from "./PreviewToolRow.vue";

const props = defineProps<{
  entry: ExplorerEntry;
  reloadKey: number;
  canEdit: boolean;
}>();

const emit = defineEmits<{
  (e: "edit", entry: ExplorerEntry): void;
  (e: "notice", payload: ShellNoticePayload): void;
}>();

const {t} = useI18n();
const previewLoading = ref(false);
const previewText = ref("");
const previewError = ref("");
const previewTextWrap = ref(true);
const previewCopied = ref(false);
let previewLoadVersion = 0;
let previewCopyTimer: number | undefined;

const previewTextStats = computed(() => {
  const lines = previewText.value ? previewText.value.split(/\r\n|\r|\n/).length : 0;
  return t("preview.textStats", {lines, chars: previewText.value.length});
});

const resetCopyState = () => {
  previewCopied.value = false;
  if (previewCopyTimer) {
    window.clearTimeout(previewCopyTimer);
    previewCopyTimer = undefined;
  }
}

const resetPreviewRuntime = () => {
  previewLoadVersion += 1;
  previewLoading.value = false;
  previewText.value = "";
  previewError.value = "";
  resetCopyState();
}

const loadPreview = async () => {
  const version = ++previewLoadVersion;
  previewLoading.value = true;
  previewText.value = "";
  previewError.value = "";
  resetCopyState();
  try {
    const file = await getFile(props.entry.path);
    if (version !== previewLoadVersion) return;
    previewText.value = file.content;
  } catch (error) {
    if (version !== previewLoadVersion) return;
    previewError.value = error instanceof Error ? error.message : t("preview.loadFailed");
  } finally {
    if (version === previewLoadVersion) previewLoading.value = false;
  }
}

const copyPreviewText = async () => {
  if (!previewText.value) return;
  try {
    await navigator.clipboard.writeText(previewText.value);
    previewCopied.value = true;
    if (previewCopyTimer) window.clearTimeout(previewCopyTimer);
    previewCopyTimer = window.setTimeout(() => {
      previewCopied.value = false;
    }, 1500);
  } catch {
    emit("notice", {kind: "error", title: t("preview.copyFailed"), message: t("preview.copyFailedMessage")});
  }
}

watch(() => [props.entry.path, props.reloadKey] as const, () => {
  resetPreviewRuntime();
  void loadPreview();
}, {immediate: true});

onBeforeUnmount(resetPreviewRuntime);
</script>

<template>
  <preview-tool-row>
    <button v-if="canEdit" :title="t('preview.edit')" @click="emit('edit', entry)">
      <icon icon="action.edit" color="currentColor" />
      <span>{{ t("preview.edit") }}</span>
    </button>
    <button :class="{active: previewTextWrap}" @click="previewTextWrap = !previewTextWrap">
      {{ previewTextWrap ? t("preview.wrap") : t("preview.noWrap") }}
    </button>
    <button :disabled="!previewText" @click="copyPreviewText">{{ previewCopied ? t("preview.copied") : t("preview.copy") }}</button>
    <template #status>{{ previewTextStats }}</template>
  </preview-tool-row>
  <div class="preview-body text">
    <div v-if="previewLoading" class="preview-placeholder">{{ t("preview.loading") }}</div>
    <div v-else-if="previewError" class="preview-placeholder error">{{ previewError }}</div>
    <pre v-else :class="{nowrap: !previewTextWrap}">{{ previewText }}</pre>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.preview-body {
  @apply min-h-0 grow overflow-auto text-sm;
  background: var(--app-panel-muted);
  color: var(--app-text-muted);
}

.preview-body pre {
  @apply m-2 whitespace-pre-wrap break-words rounded-lg border p-3 font-mono text-xs leading-5 shadow-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  color: var(--app-text);
  min-height: calc(100% - 1rem);
}

.preview-body pre.nowrap {
  @apply whitespace-pre break-normal;
}

.preview-placeholder {
  @apply flex h-full min-h-48 flex-col items-center justify-center gap-3 px-5 text-center;
  color: var(--app-text-subtle);
}

.preview-placeholder.error {
  color: var(--app-danger);
}
</style>
