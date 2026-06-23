<script setup lang="ts">
import Icon from "./Icon.vue";
import FileTypeIcon from "./FileTypeIcon.vue";
import {computed} from "vue";
import type {FileTreeData, LoadData} from "../class.ts";
import {normalizePathText} from "../utils/file-path.ts";

const props = withDefaults(defineProps<{
  deep?: number;
  data: FileTreeData;
  currentPath: string;
  focusedPath: string;
  expandedPaths: Set<string>;
  loadingPaths: Set<string>;
  dropTargetPath: string;
  loadData: LoadData;
}>(), {
  deep: 0
});

const emit = defineEmits<{
  (e: "toggle", node: FileTreeData): void;
  (e: "navigate", node: FileTreeData): void;
  (e: "node-focus", node: FileTreeData): void;
  (e: "node-context-menu", node: FileTreeData, event: MouseEvent): void;
  (e: "node-drag-over", node: FileTreeData, event: DragEvent): void;
  (e: "node-drag-leave", node: FileTreeData, event: DragEvent): void;
  (e: "node-drop", node: FileTreeData, event: DragEvent): void;
}>();

const normalizedPath = computed(() => normalizePathText(props.data.path));
const active = computed(() => normalizedPath.value === normalizePathText(props.currentPath || "/"));
const focused = computed(() => normalizedPath.value === normalizePathText(props.focusedPath || "/"));
const expanded = computed(() => props.expandedPaths.has(normalizedPath.value));
const loading = computed(() => props.loadingPaths.has(normalizedPath.value));
const dropTarget = computed(() => Boolean(props.dropTargetPath) && normalizedPath.value === normalizePathText(props.dropTargetPath));
const hasChildren = computed(() => Boolean(props.data.children?.length));
const nodeIcon = computed(() => normalizedPath.value === "/" ? "file.home" : "file.folder");
const nodeStyle = computed(() => ({"--tree-depth": props.deep}));

const handleRowClick = (event: MouseEvent) => {
  const target = event.currentTarget instanceof HTMLElement ? event.currentTarget : null;
  target?.focus();
  emit("node-focus", props.data);
  emit("navigate", props.data);
}
const handleToggle = (event: MouseEvent) => {
  const row = event.currentTarget instanceof HTMLElement ? event.currentTarget.closest<HTMLElement>(".tree-node") : null;
  row?.focus();
  emit("node-focus", props.data);
  emit("toggle", props.data);
}
</script>

<template>
  <div class="tree-node-wrap" role="none">
    <div
        class="tree-node"
        :class="{active, loading, dropTarget}"
        :style="nodeStyle"
        role="treeitem"
        :tabindex="focused ? 0 : -1"
        :data-tree-path="normalizedPath"
        :aria-selected="active"
        :aria-expanded="expanded"
        :title="data.path"
        @click="handleRowClick"
        @focus="emit('node-focus', data)"
        @contextmenu.prevent.stop="emit('node-context-menu', data, $event)"
        @dragover="emit('node-drag-over', data, $event)"
        @dragleave="emit('node-drag-leave', data, $event)"
        @drop="emit('node-drop', data, $event)">
      <span class="node-spacer" aria-hidden="true"></span>
      <button
          type="button"
          class="fold-button"
          :class="{expanded, placeholder: normalizedPath === '/' && !hasChildren}"
          :disabled="loading || (normalizedPath === '/' && !hasChildren)"
          tabindex="-1"
          aria-hidden="true"
          title="展开或折叠"
          @pointerdown.prevent
          @click.stop="handleToggle">
        <icon v-if="loading" icon="action.refresh" size="0.8rem" />
        <icon v-else icon="action.down" size="0.72rem" />
      </button>
      <span class="node-icon" aria-hidden="true">
        <icon v-if="normalizedPath === '/'" :icon="nodeIcon" size="1.05rem" />
        <file-type-icon v-else kind="folder" :open="expanded" size="1.05rem" />
      </span>
      <span class="node-name">{{ data.name }}</span>
    </div>

    <div v-if="expanded && hasChildren" class="tree-children" role="group">
      <file-tree-node
          v-for="file in data.children"
          :key="file.path"
          :deep="deep + 1"
          :data="file"
          :current-path="currentPath"
          :focused-path="focusedPath"
          :expanded-paths="expandedPaths"
          :loading-paths="loadingPaths"
          :drop-target-path="dropTargetPath"
          :load-data="loadData"
          @toggle="node => emit('toggle', node)"
          @navigate="node => emit('navigate', node)"
          @node-focus="node => emit('node-focus', node)"
          @node-context-menu="(node, event) => emit('node-context-menu', node, event)"
          @node-drag-over="(node, event) => emit('node-drag-over', node, event)"
          @node-drag-leave="(node, event) => emit('node-drag-leave', node, event)"
          @node-drop="(node, event) => emit('node-drop', node, event)" />
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.tree-node-wrap {
  @apply min-w-max;
}

.tree-node {
  --tree-depth: 0;
  @apply flex h-7 w-full min-w-max items-center rounded-sm border border-transparent pr-2 text-left outline-none;
  padding-left: calc(var(--tree-depth) * 1rem + 0.125rem);
  color: var(--app-text-muted);
}

.tree-node:hover {
  background: var(--app-accent-hover, #eaf4ff);
}

.tree-node:focus-visible {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.tree-node.active {
  color: color-mix(in srgb, var(--app-accent, #2563eb) 62%, var(--app-text));
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-selected, #dceeff);
}

.tree-node.dropTarget {
  color: var(--app-accent, #2563eb);
  border-color: var(--app-accent, #2563eb);
  background: var(--app-accent-soft, #eff6ff);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.tree-node.loading {
  @apply cursor-wait;
  color: var(--app-text-subtle);
}

.node-spacer {
  @apply h-full w-0.5 shrink-0;
}

.fold-button {
  @apply mr-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-sm transition disabled:pointer-events-none disabled:opacity-40;
  color: var(--app-text-subtle);
}

.fold-button:hover {
  background: var(--app-control-hover);
  color: var(--app-text);
}

.fold-button :deep(.icon) {
  @apply transition-transform;
}

.fold-button:not(.expanded) :deep(.icon) {
  @apply -rotate-90;
}

.fold-button.loading :deep(.icon),
.tree-node.loading .fold-button :deep(.icon) {
  @apply animate-spin;
}

.fold-button.placeholder {
  @apply opacity-0;
}

.node-icon {
  @apply mr-1.5 inline-flex h-5 w-5 shrink-0 items-center justify-center;
  color: color-mix(in srgb, var(--app-warning) 88%, var(--app-text-muted));
}

.tree-node.active .node-icon {
  color: var(--app-accent, #2563eb);
}

.node-name {
  @apply min-w-0 max-w-52 truncate text-[13px] leading-none;
}

.tree-children {
  @apply flex flex-col;
}
</style>
