<script setup lang="ts">
import {computed, ref} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {entryMetaRows, fileEntryIconName, formatEntrySize} from "../../utils/file-entry.ts";
import {useI18n} from "../../i18n";
import OperationPanelShell from "./OperationPanelShell.vue";

type OperationPanelShellExpose = {
  focus: () => void;
}

const props = defineProps<{
  visible: boolean;
  entries: ExplorerEntry[];
  currentFolder: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const {locale, t} = useI18n();
const panelRef = ref<OperationPanelShellExpose | null>(null);

const singleEntry = computed(() => props.entries.length === 1 ? props.entries[0] : null);
const folderCount = computed(() => props.entries.filter(entry => entry.type === "folder").length);
const fileEntries = computed(() => props.entries.filter(entry => entry.type === "file"));
const knownSize = computed(() => fileEntries.value.reduce((sum, entry) => sum + (entry.size ?? 0), 0));
const missingSizeCount = computed(() => fileEntries.value.filter(entry => entry.size === undefined).length);

const title = computed(() => singleEntry.value?.name ?? t("properties.itemsTitle", {count: props.entries.length}));
const subtitle = computed(() => singleEntry.value ? t("properties.itemProperties") : t("properties.selectedProperties"));
const panelIcon = computed(() => {
  const entry = singleEntry.value;
  if (!entry) return "view.details";
  return fileEntryIconName(entry);
});
const sizeText = computed(() => {
  if (singleEntry.value) return singleEntry.value.type === "file" ? formatEntrySize(singleEntry.value.size, "0 B") : "-";
  const suffix = missingSizeCount.value
      ? `${locale.value === "zh-CN" ? "，" : ", "}${t("explorer.fileSizesUnloaded", {count: missingSizeCount.value})}`
      : "";
  return `${formatEntrySize(knownSize.value, "0 B")}${suffix}`;
});
const rows = computed(() => {
  const entry = singleEntry.value;
  if (entry) {
    return [
      {label: t("operation.name"), value: entry.name},
      ...entryMetaRows(entry, {
        sizeText: sizeText.value,
        includeLocation: true,
        includePath: true,
        pathBeforeStats: true,
        modifiedLabel: t("sort.modified")
      })
    ];
  }
  return [
    {label: t("properties.items"), value: t("common.items", {count: props.entries.length})},
    {label: t("common.folder"), value: t("common.items", {count: folderCount.value})},
    {label: t("common.file"), value: t("common.items", {count: fileEntries.value.length})},
    {label: t("properties.knownFileSize"), value: sizeText.value},
    {label: t("properties.currentLocation"), value: props.currentFolder}
  ];
});

defineExpose({
  focus: () => panelRef.value?.focus()
});
</script>

<template>
  <operation-panel-shell
      v-if="visible"
      ref="panelRef"
      width="properties"
      variant="neutral"
      :icon="panelIcon"
      :title="title"
      :subtitle="subtitle"
      :tabindex="-1"
      @close="emit('close')">
    <div class="properties-list">
      <div v-for="item in rows" :key="item.label" :title="item.value">
        <span>{{ item.label }}</span>
        <strong>{{ item.value }}</strong>
      </div>
    </div>
    <template #actions>
      <button type="button" class="operation-primary" @click="emit('close')">{{ t("common.close") }}</button>
    </template>
  </operation-panel-shell>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.properties-list {
  @apply flex max-h-72 flex-col overflow-auto rounded-md border;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.properties-list div {
  @apply grid min-h-9 grid-cols-[5.5rem_minmax(0,1fr)] items-center gap-3 border-b px-3 py-2 text-xs last:border-b-0;
  border-color: var(--app-border-soft);
}

.properties-list span {
  color: var(--app-text-subtle);
}

.properties-list strong {
  @apply min-w-0 truncate font-medium;
  color: var(--app-text);
}

.operation-primary {
  @apply h-9 rounded-md px-4 text-sm font-medium;
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.operation-primary:hover {
  background: var(--app-accent-strong);
}

.operation-primary:focus-visible {
  @apply outline-none;
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}
</style>
