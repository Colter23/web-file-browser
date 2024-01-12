<script setup lang="ts">

import TabBar from "./components/TabBar.vue";
import FileTree from "./components/FileTree.vue";
</script>

<template>
  <tab-bar></tab-bar>
  <file-tree></file-tree>
</template>

<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}
.logo:hover {
  filter: drop-shadow(0 0 2em #646cffaa);
}
.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
