<script setup lang="ts">
import {computed, ref} from "vue";
import Icon from "../Icon.vue";
import type {ExplorerEntry} from "../explorer/types.ts";
import {entryTypeText, fileEntryIcon, formatEntryDate, formatEntrySize} from "../../utils/file-entry.ts";
import {parentPath} from "../../utils/file-path.ts";

const props = defineProps<{
  visible: boolean;
  entries: ExplorerEntry[];
  currentFolder: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const panelRef = ref<HTMLElement | null>(null);

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
      {label: "类型", value: entryTypeText(entry)},
      {label: "位置", value: parentPath(entry.path)},
      {label: "路径", value: entry.path},
      {label: "大小", value: sizeText.value},
      {label: "修改时间", value: formatEntryDate(entry.modified)}
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
  <section
      v-if="visible"
      ref="panelRef"
      class="properties-panel"
      tabindex="-1"
      @keydown.esc.prevent.stop="emit('close')">
    <div class="properties-header">
      <div class="properties-icon">
        <icon :icon="panelIcon" />
      </div>
      <div class="properties-title">
        <strong>{{ title }}</strong>
        <span>{{ subtitle }}</span>
      </div>
      <button type="button" class="operation-panel-close" title="关闭" @click="emit('close')">
        <icon icon="icon-close" />
      </button>
    </div>
    <div class="properties-list">
      <div v-for="item in rows" :key="item.label" :title="item.value">
        <span>{{ item.label }}</span>
        <strong>{{ item.value }}</strong>
      </div>
    </div>
    <div class="properties-actions">
      <button type="button" class="operation-primary" @click="emit('close')">确定</button>
    </div>
  </section>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.properties-panel {
  @apply absolute left-1/2 top-6 z-30 flex w-[min(32rem,calc(100%-2rem))] -translate-x-1/2 flex-col gap-3 rounded-lg border border-slate-200 bg-white p-4 text-sm text-slate-700 shadow-2xl outline-none;
}

.properties-header {
  @apply flex items-start gap-3;
}

.properties-icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-slate-100 text-xl text-slate-600;
}

.properties-title {
  @apply flex min-w-0 grow flex-col gap-0.5;
}

.properties-title strong {
  @apply truncate text-base font-semibold text-slate-900;
}

.properties-title span {
  @apply text-xs leading-5 text-slate-500;
}

.operation-panel-close {
  @apply flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-slate-500 hover:bg-slate-100;
}

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

.properties-actions {
  @apply flex justify-end gap-2 pt-1;
}

.operation-primary {
  @apply h-9 rounded-md bg-blue-600 px-4 text-sm font-medium text-white hover:bg-blue-700;
}
</style>
