<script setup lang="ts">
import type {FavoriteItem, FileTreeData, LoadData} from "../../class.ts";
import type {ExplorerEntry} from "../explorer/types.ts";
import FileTree from "../FileTree.vue";
import FavoritePanel from "./FavoritePanel.vue";

defineProps<{
  treeData: FileTreeData[];
  loadData: LoadData;
  currentPath: string;
  favorites: FavoriteItem[];
  favoritesLoading: boolean;
  favoritePaths: string[];
}>();

defineEmits<{
  (e: "drop-entries", payload: {entries: ExplorerEntry[]; target: FileTreeData; action: "copy" | "move"}): void;
  (e: "reorder-mount", payload: {source: FileTreeData; target: FileTreeData; placement: "before" | "after"}): void;
  (e: "open-new-tab", node: FileTreeData): void;
  (e: "open-favorite", favorite: FavoriteItem): void;
  (e: "open-favorite-new-tab", favorite: FavoriteItem): void;
  (e: "rename-favorite", payload: {favorite: FavoriteItem; name: string}): void;
  (e: "reorder-favorite", payload: {source: FavoriteItem; target: FavoriteItem; placement: "before" | "after"}): void;
  (e: "remove-favorite", favorite: FavoriteItem): void;
  (e: "refresh-favorites"): void;
  (e: "add-favorite", node: FileTreeData): void;
  (e: "remove-favorite-path", path: string): void;
  (e: "notice", payload: {message: string; kind?: "info" | "success" | "warning" | "error"; title?: string}): void;
}>();
</script>

<template>
  <aside class="sidebar">
    <favorite-panel
        :favorites="favorites"
        :loading="favoritesLoading"
        :current-path="currentPath"
        @open="favorite => $emit('open-favorite', favorite)"
        @open-new-tab="favorite => $emit('open-favorite-new-tab', favorite)"
        @rename="payload => $emit('rename-favorite', payload)"
        @reorder="payload => $emit('reorder-favorite', payload)"
        @remove="favorite => $emit('remove-favorite', favorite)"
        @refresh="$emit('refresh-favorites')"
        @notice="payload => $emit('notice', payload)" />
    <file-tree
        :data="treeData"
        :load-data="loadData"
        :current-path="currentPath"
        :favorite-paths="favoritePaths"
        @drop-entries="payload => $emit('drop-entries', payload)"
        @reorder-mount="payload => $emit('reorder-mount', payload)"
        @open-new-tab="node => $emit('open-new-tab', node)"
        @add-favorite="node => $emit('add-favorite', node)"
        @remove-favorite="path => $emit('remove-favorite-path', path)"
        @notice="payload => $emit('notice', payload)" />
  </aside>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.sidebar {
  @apply flex min-h-0 flex-col overflow-hidden px-2 py-2;
  background: var(--app-panel-muted);
}
</style>
