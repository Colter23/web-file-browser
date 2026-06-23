<script setup lang="ts">
import {computed, nextTick, ref, watch} from "vue";
import FileTreeNode from "./FileTreeNode.vue";
import type {FileTreeData, LoadData} from "../class.ts";
import type {ExplorerEntry} from "./explorer/types.ts";
import {isSameOrDescendantPath, normalizePathText} from "../utils/file-path.ts";
import {
  getActiveInternalEntryDragEntries,
  hasInternalEntryDragData,
  readInternalEntryDragData
} from "../utils/internal-entry-drag.ts";

const props = defineProps<{
  data: FileTreeData[];
  loadData: LoadData;
  currentPath: string;
}>();

const emit = defineEmits<{
  (e: "drop-entries", payload: {entries: ExplorerEntry[]; target: FileTreeData; action: "copy" | "move"}): void;
}>();

const expandedPaths = ref<Set<string>>(new Set(["/"]));
const loadingPaths = ref<Set<string>>(new Set());
const focusedPath = ref("/");
const treeRef = ref<HTMLElement | null>(null);
const dropTargetPath = ref("");
const activePath = computed(() => normalizePathText(props.currentPath || "/"));
let syncToken = 0;
let dropExpandTimer = 0;
let dropExpandPath = "";

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

const rowElement = (path: string) => {
  const normalized = normalizePathText(path);
  return Array.from(treeRef.value?.querySelectorAll<HTMLElement>(".tree-node") ?? [])
      .find(item => item.dataset.treePath === normalized) ?? null;
}

const revealPath = async (path: string, focus = false) => {
  const normalized = normalizePathText(path);
  await nextTick();
  const row = rowElement(normalized);
  row?.scrollIntoView({block: "nearest", inline: "nearest"});
  if (focus) row?.focus();
}

const focusPath = async (path: string) => {
  const normalized = normalizePathText(path);
  focusedPath.value = normalized;
  await revealPath(normalized, true);
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

const isCopyDrop = (event: DragEvent) => Boolean(event.ctrlKey || event.metaKey);

const clearDropExpandTimer = () => {
  if (!dropExpandTimer) return;
  window.clearTimeout(dropExpandTimer);
  dropExpandTimer = 0;
  dropExpandPath = "";
}

const clearTreeDropTarget = () => {
  dropTargetPath.value = "";
  clearDropExpandTimer();
}

const canDropOnNode = (node: FileTreeData, entries: ExplorerEntry[]) => {
  if (node.isFile || !entries.length) return false;
  const targetPath = normalizePathText(node.path);
  return !entries.some(entry => entry.type === "folder" && isSameOrDescendantPath(targetPath, entry.path));
}

const scheduleDropExpand = (node: FileTreeData) => {
  const path = normalizePathText(node.path);
  if (isExpanded(path) || isLoading(path)) return;
  if (dropExpandTimer && dropExpandPath === path) return;
  clearDropExpandTimer();
  dropExpandPath = path;
  dropExpandTimer = window.setTimeout(() => {
    dropExpandTimer = 0;
    dropExpandPath = "";
    void expandNode(node);
  }, 650);
}

const handleNodeDragOver = (node: FileTreeData, event: DragEvent) => {
  if (!hasInternalEntryDragData(event.dataTransfer) || node.isFile) return;
  const entries = getActiveInternalEntryDragEntries();
  if (!canDropOnNode(node, entries)) {
    if (event.dataTransfer) event.dataTransfer.dropEffect = "none";
    clearTreeDropTarget();
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  dropTargetPath.value = normalizePathText(node.path);
  scheduleDropExpand(node);
  if (event.dataTransfer) event.dataTransfer.dropEffect = isCopyDrop(event) ? "copy" : "move";
}

const handleNodeDragLeave = (node: FileTreeData, event: DragEvent) => {
  const path = normalizePathText(node.path);
  if (dropTargetPath.value !== path) return;
  const target = rowElement(path);
  const related = event.relatedTarget;
  if (related instanceof Node && target?.contains(related)) return;
  clearTreeDropTarget();
}

const handleTreeDragLeave = (event: DragEvent) => {
  const related = event.relatedTarget;
  if (related instanceof Node && treeRef.value?.contains(related)) return;
  clearTreeDropTarget();
}

const handleNodeDrop = (node: FileTreeData, event: DragEvent) => {
  if (!hasInternalEntryDragData(event.dataTransfer)) return;
  event.preventDefault();
  event.stopPropagation();
  const entries = readInternalEntryDragData(event.dataTransfer);
  if (canDropOnNode(node, entries)) {
    emit("drop-entries", {entries, target: node, action: isCopyDrop(event) ? "copy" : "move"});
  }
  clearTreeDropTarget();
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
  if (token === syncToken) await revealPath(activePath.value);
}

watch([() => props.currentPath, () => props.data], () => {
  void syncCurrentPath();
}, {immediate: true});
</script>

<template>
  <div ref="treeRef" class="file-tree" role="tree" aria-label="文件树" tabindex="-1" @keydown="handleTreeKeyDown" @dragleave="handleTreeDragLeave" @dragend="clearTreeDropTarget" @drop="clearTreeDropTarget">
    <template v-if="data.length">
      <file-tree-node
          v-for="file in data"
          :key="file.path"
          :data="file"
          :current-path="activePath"
          :focused-path="currentFocusedPath"
          :expanded-paths="expandedPaths"
          :loading-paths="loadingPaths"
          :drop-target-path="dropTargetPath"
          :load-data="loadData"
          @toggle="toggleNode"
          @navigate="navigateNode"
          @node-focus="handleNodeFocus"
          @node-drag-over="handleNodeDragOver"
          @node-drag-leave="handleNodeDragLeave"
          @node-drop="handleNodeDrop" />
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
