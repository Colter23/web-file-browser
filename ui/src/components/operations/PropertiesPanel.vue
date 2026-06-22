<script setup lang="ts">
import {computed, ref} from "vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {entryMetaRows, fileEntryIcon, formatEntrySize} from "../../utils/file-entry.ts";
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

const panelRef = ref<OperationPanelShellExpose | null>(null);

const singleEntry = computed(() => props.entries.length === 1 ? props.entries[0] : null);
const folderCount = computed(() => props.entries.filter(entry => entry.type === "folder").length);
const fileEntries = computed(() => props.entries.filter(entry => entry.type === "file"));
const knownSize = computed(() => fileEntries.value.reduce((sum, entry) => sum + (entry.size ?? 0), 0));
const missingSizeCount = computed(() => fileEntries.value.filter(entry => entry.size === undefined).length);

const title = computed(() => singleEntry.value?.name ?? `${props.entries.length} 个项目`);
const subtitle = computed(() => singleEntry.value ? "项目属性" : "选中项目属性");
const panelIcon = computed(() => {
  const entry = singleEntry.value;
  if (!entry) return "icon-file-common-filling";
  return fileEntryIcon(entry);
});
const sizeText = computed(() => {
  if (singleEntry.value) return singleEntry.value.type === "file" ? formatEntrySize(singleEntry.value.size, "0 B") : "-";
  const suffix = missingSizeCount.value ? `，${missingSizeCount.value} 个文件未加载大小` : "";
  return `${formatEntrySize(knownSize.value, "0 B")}${suffix}`;
});
const rows = computed(() => {
  const entry = singleEntry.value;
  if (entry) {
    return [
      {label: "名称", value: entry.name},
      ...entryMetaRows(entry, {
        sizeText: sizeText.value,
        includeLocation: true,
        includePath: true,
        pathBeforeStats: true,
        modifiedLabel: "修改时间"
      })
    ];
  }
  return [
    {label: "项目", value: `${props.entries.length} 项`},
    {label: "文件夹", value: `${folderCount.value} 项`},
    {label: "文件", value: `${fileEntries.value.length} 项`},
    {label: "已知文件大小", value: sizeText.value},
    {label: "当前位置", value: props.currentFolder}
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
      <button type="button" class="operation-primary" @click="emit('close')">确定</button>
    </template>
  </operation-panel-shell>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.properties-list {
  @apply flex max-h-72 flex-col overflow-auto rounded-md border border-slate-100 bg-slate-50;
}

.properties-list div {
  @apply grid min-h-9 grid-cols-[5.5rem_minmax(0,1fr)] items-center gap-3 border-b border-slate-100 px-3 py-2 text-xs last:border-b-0;
}

.properties-list span {
  @apply text-slate-500;
}

.properties-list strong {
  @apply min-w-0 truncate font-medium text-slate-800;
}

.operation-primary {
  @apply h-9 rounded-md bg-blue-600 px-4 text-sm font-medium text-white hover:bg-blue-700;
}
</style>
