<script setup lang="ts">
import Icon from "./Icon.vue";
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
  loadData: LoadData;
}>(), {
  deep: 0
});

const emit = defineEmits<{
  (e: "toggle", node: FileTreeData): void;
  (e: "navigate", node: FileTreeData): void;
  (e: "node-focus", node: FileTreeData): void;
}>();

const normalizedPath = computed(() => normalizePathText(props.data.path));
const active = computed(() => normalizedPath.value === normalizePathText(props.currentPath || "/"));
const focused = computed(() => normalizedPath.value === normalizePathText(props.focusedPath || "/"));
const expanded = computed(() => props.expandedPaths.has(normalizedPath.value));
const loading = computed(() => props.loadingPaths.has(normalizedPath.value));
const hasChildren = computed(() => Boolean(props.data.children?.length));
const nodeIcon = computed(() => normalizedPath.value === "/" ? "icon-home-fill" : expanded.value ? "icon-folder-open-fill" : "icon-folder-fill");
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
        :class="{active, loading}"
        :style="nodeStyle"
        role="treeitem"
        :tabindex="focused ? 0 : -1"
        :data-tree-path="normalizedPath"
        :aria-selected="active"
        :aria-expanded="expanded"
        :title="data.path"
        @click="handleRowClick"
        @focus="emit('node-focus', data)">
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
        <icon v-if="loading" icon="icon-refresh" size="0.8rem" />
        <icon v-else icon="icon-unfold" size="0.72rem" />
      </button>
      <span class="node-icon" aria-hidden="true">
        <icon :icon="nodeIcon" size="1.05rem" />
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
          :load-data="loadData"
          @toggle="node => emit('toggle', node)"
          @navigate="node => emit('navigate', node)"
          @node-focus="node => emit('node-focus', node)" />
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
  @apply flex h-7 w-full min-w-max items-center rounded-md border border-transparent pr-2 text-left text-slate-700 outline-none hover:bg-blue-50/80;
  padding-left: calc(var(--tree-depth) * 1rem + 0.125rem);
}

.tree-node:focus-visible {
  @apply border-blue-300 bg-blue-50 ring-2 ring-blue-100;
}

.tree-node.active {
  @apply border-blue-200 bg-blue-100 text-blue-950;
}

.tree-node.loading {
  @apply cursor-wait text-slate-500;
}

.node-spacer {
  @apply h-full w-0.5 shrink-0;
}

.fold-button {
  @apply mr-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded text-slate-500 transition hover:bg-white/80 hover:text-slate-800 disabled:pointer-events-none disabled:opacity-40;
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
  @apply mr-1.5 inline-flex h-5 w-5 shrink-0 items-center justify-center text-amber-500;
}

.tree-node.active .node-icon {
  @apply text-blue-600;
}

.node-name {
  @apply min-w-0 max-w-52 truncate text-[13px] leading-none;
}

.tree-children {
  @apply flex flex-col;
}
</style>
