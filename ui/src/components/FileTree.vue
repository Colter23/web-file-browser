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
const focusedPath = ref("/");
const activePath = computed(() => normalizePathText(props.currentPath || "/"));
let syncToken = 0;

type VisibleTreeNode = {
  node: FileTreeData;
  path: string;
  parentPath: string | null;
}

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

const visibleNodes = computed<VisibleTreeNode[]>(() => {
  const nodes: VisibleTreeNode[] = [];
  const walk = (items: FileTreeData[], parentPath: string | null) => {
    items.forEach(item => {
      const path = normalizePathText(item.path);
      nodes.push({node: item, path, parentPath});
      if (isExpanded(path) && item.children?.length) walk(item.children, path);
    });
  }
  walk(props.data, null);
  return nodes;
});

const currentFocusedPath = computed(() => {
  const normalizedFocus = normalizePathText(focusedPath.value || activePath.value || "/");
  if (visibleNodes.value.some(item => item.path === normalizedFocus)) return normalizedFocus;
  if (visibleNodes.value.some(item => item.path === activePath.value)) return activePath.value;
  return visibleNodes.value[0]?.path ?? "/";
});

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

const focusPath = async (path: string) => {
  const normalized = normalizePathText(path);
  focusedPath.value = normalized;
  await nextTick();
  const row = Array.from(document.querySelectorAll<HTMLElement>(".file-tree .tree-node"))
      .find(item => item.dataset.treePath === normalized);
  row?.focus();
}

const focusByOffset = async (path: string, offset: number) => {
  const nodes = visibleNodes.value;
  const index = nodes.findIndex(item => item.path === normalizePathText(path));
  if (index < 0) return;
  const next = nodes[Math.max(0, Math.min(nodes.length - 1, index + offset))];
  if (next) await focusPath(next.path);
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
    const loaded = await props.loadData(node, {navigate: true, focusExplorer: false});
    if (loaded) setExpanded(path, true);
  } finally {
    setLoading(path, false);
  }
}

const handleNodeFocus = (node: FileTreeData) => {
  focusedPath.value = normalizePathText(node.path);
}

const handleNodeKeyDown = async (node: FileTreeData, event: KeyboardEvent) => {
  const path = normalizePathText(node.path);
  const item = visibleNodes.value.find(visibleNode => visibleNode.path === path);
  if (!item) return;

  if (event.key === "ArrowDown") {
    event.preventDefault();
    await focusByOffset(path, 1);
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    await focusByOffset(path, -1);
    return;
  }

  if (event.key === "Home") {
    event.preventDefault();
    const first = visibleNodes.value[0];
    if (first) await focusPath(first.path);
    return;
  }

  if (event.key === "End") {
    event.preventDefault();
    const last = visibleNodes.value[visibleNodes.value.length - 1];
    if (last) await focusPath(last.path);
    return;
  }

  if (event.key === "ArrowRight") {
    event.preventDefault();
    if (!isExpanded(path) || node.children === undefined) {
      await expandNode(node);
      await focusPath(path);
      return;
    }
    const firstChild = node.children[0];
    if (firstChild) await focusPath(firstChild.path);
    return;
  }

  if (event.key === "ArrowLeft") {
    event.preventDefault();
    if (path !== "/" && isExpanded(path) && node.children !== undefined) {
      setExpanded(path, false);
      await focusPath(path);
      return;
    }
    if (item.parentPath) await focusPath(item.parentPath);
    return;
  }

  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    await navigateNode(node);
    await focusPath(path);
  }
}

const handleTreeKeyDown = (event: KeyboardEvent) => {
  const row = event.target instanceof HTMLElement ? event.target.closest<HTMLElement>(".tree-node") : null;
  const path = row?.dataset.treePath ?? currentFocusedPath.value;
  const node = findNodeByPath(props.data, path);
  if (node) void handleNodeKeyDown(node, event);
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
  if (token === syncToken) focusedPath.value = activePath.value;
}

watch([() => props.currentPath, () => props.data], () => {
  void syncCurrentPath();
}, {immediate: true});
</script>

<template>
  <div class="file-tree" role="tree" aria-label="文件树" tabindex="-1" @keydown="handleTreeKeyDown">
    <template v-if="data.length">
      <file-tree-node
          v-for="file in data"
          :key="file.path"
          :data="file"
          :current-path="activePath"
          :focused-path="currentFocusedPath"
          :expanded-paths="expandedPaths"
          :loading-paths="loadingPaths"
          :load-data="loadData"
          @toggle="toggleNode"
          @navigate="navigateNode"
          @node-focus="handleNodeFocus" />
    </template>
    <div v-else class="tree-empty">暂无目录</div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.file-tree {
  @apply flex h-full w-full flex-col overflow-auto bg-transparent py-1 pr-1 select-none text-sm;
  color: var(--app-text-muted);
}

.tree-empty {
  @apply flex h-20 items-center justify-center rounded-lg text-xs;
  color: var(--app-text-disabled);
}

.file-tree::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.file-tree::-webkit-scrollbar-thumb {
  @apply rounded-full;
  background: var(--app-border);
}

.file-tree::-webkit-scrollbar-track {
  @apply bg-transparent;
}
</style>
