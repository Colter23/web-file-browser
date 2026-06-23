<script setup lang="ts">
import type {FileTreeData, LoadData} from "../../class.ts";
import FileTree from "../FileTree.vue";
import Icon from "../Icon.vue";

defineProps<{
  treeData: FileTreeData[];
  loadData: LoadData;
}>();

const emit = defineEmits<{
  (e: "upload"): void;
  (e: "create-file"): void;
  (e: "create-folder"): void;
}>();
</script>

<template>
  <aside class="sidebar">
    <div class="quick-toolbar">
      <button class="primary-tool" title="上传" @click="emit('upload')">
        <icon icon="icon-upload" size="large" />
        <span>上传</span>
      </button>
      <button class="icon-tool" title="新建文件" @click="emit('create-file')">
        <icon icon="icon-file-add-fill" />
      </button>
      <button class="icon-tool" title="新建文件夹 (Ctrl+Shift+N)" @click="emit('create-folder')">
        <icon icon="icon-folder-add-fill" />
      </button>
    </div>
    <file-tree :data="treeData" :load-data="loadData" />
  </aside>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.sidebar {
  @apply flex min-h-0 flex-col overflow-hidden rounded-xl border border-slate-200 bg-white/65 p-2 shadow-sm backdrop-blur;
}

.quick-toolbar {
  @apply mb-2 grid h-11 shrink-0 grid-cols-[1fr_2.25rem_2.25rem] gap-2;
}

.primary-tool,
.icon-tool {
  @apply inline-flex items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-700 hover:bg-blue-50;
}

.primary-tool {
  @apply gap-2 bg-blue-600 px-3 text-sm font-medium text-white hover:bg-blue-700;
}

.icon-tool {
  @apply h-full w-full;
}
</style>
