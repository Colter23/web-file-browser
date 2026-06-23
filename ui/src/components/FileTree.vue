<script setup lang="ts">
import {computed, nextTick, ref, watch} from "vue";
import FileTreeNode from "./FileTreeNode.vue";
import type {FileTreeData, LoadData} from "../class.ts";
import {normalizePathText} from "../utils/file-path.ts";

const props = defineProps<{
  data: FileTreeData[];
  loadData: LoadData;
  currentPath: string;
}>();

const expandedPaths = ref<Set<string>>(new Set(["/"]));
const loadingPaths = ref<Set<string>>(new Set());
const activePath = computed(() => normalizePathText(props.currentPath || "/"));
let syncToken = 0;

const updatePathSet = (source: typeof expandedPaths, path: string, enabled: boolean) => {
  const normalized = normalizePathText(path);
  const next = new Set(source.value);
  if (enabled) next.add(normalized);
  else next.delete(normalized);
  source.value = next;
}

const isExpanded = (path: string) => expandedPaths.value.has(normalizePathText(path));
const isLoading = (path: string) => loadingPaths.value.has(normalizePathText(path));
const setExpanded = (path: string, expanded: boolean) => updatePathSet(expandedPaths, path, expanded);
const setLoading = (path: string, loading: boolean) => updatePathSet(loadingPaths, path, loading);

const ancestorPaths = (path: string) => {
  const normalized = normalizePathText(path);
  const parts = normalized.split("/").filter(Boolean);
  if (!parts.length) return ["/"];

  const paths = ["/"];
  let current = "";
  parts.forEach(part => {
    current = `${current}/${part}`;
    paths.push(current);
  });
  return paths;
}

const findNodeByPath = (nodes: FileTreeData[], path: string): FileTreeData | null => {
  const normalized = normalizePathText(path);
  for (const node of nodes) {
    if (normalizePathText(node.path) === normalized) return node;
    const child = node.children ? findNodeByPath(node.children, normalized) : null;
    if (child) return child;
  }
  return null;
}

const expandNode = async (node: FileTreeData) => {
  if (node.isFile) return false;
  const path = normalizePathText(node.path);
  setExpanded(path, true);
  if (node.children !== undefined) return true;
  if (isLoading(path)) return false;

  setLoading(path, true);
  try {
    return await props.loadData(node, {navigate: false});
  } finally {
    setLoading(path, false);
  }
}

const toggleNode = async (node: FileTreeData) => {
  if (node.isFile || isLoading(node.path)) return;
  const path = normalizePathText(node.path);
  if (path !== "/" && isExpanded(path) && node.children !== undefined) {
    setExpanded(path, false);
    return;
  }
  await expandNode(node);
}

const navigateNode = async (node: FileTreeData) => {
  if (node.isFile || isLoading(node.path)) return;
  const path = normalizePathText(node.path);
  setLoading(path, true);
  try {
    const loaded = await props.loadData(node, {navigate: true});
    if (loaded) setExpanded(path, true);
  } finally {
    setLoading(path, false);
  }
}

const syncCurrentPath = async () => {
  const token = ++syncToken;
  await nextTick();

  for (const path of ancestorPaths(activePath.value)) {
    if (token !== syncToken) return;
    const node = findNodeByPath(props.data, path);
    if (!node) return;
    const expanded = await expandNode(node);
    if (!expanded) return;
    await nextTick();
  }
}

watch([() => props.currentPath, () => props.data], () => {
  void syncCurrentPath();
}, {immediate: true});
</script>

<template>
  <div class="file-tree" role="tree" aria-label="文件树">
    <template v-if="data.length">
      <file-tree-node
          v-for="file in data"
          :key="file.path"
          :data="file"
          :current-path="activePath"
          :expanded-paths="expandedPaths"
          :loading-paths="loadingPaths"
          :load-data="loadData"
          @toggle="toggleNode"
          @navigate="navigateNode" />
    </template>
    <div v-else class="tree-empty">暂无目录</div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.file-tree {
  @apply flex h-full w-full flex-col overflow-auto rounded-lg bg-white/35 p-1 select-none text-sm text-slate-700;
}

.tree-empty {
  @apply flex h-20 items-center justify-center rounded-lg text-xs text-slate-400;
}

.file-tree::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.file-tree::-webkit-scrollbar-thumb {
  @apply rounded-full bg-slate-300;
}

.file-tree::-webkit-scrollbar-track {
  @apply bg-transparent;
}
</style>
