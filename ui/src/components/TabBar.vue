<script setup lang="ts">

import Card from "./Card.vue";
import TabItem from "./TabItem.vue";
import {ref} from "vue";

const files = ref<{[key: string]: string}>({
  "/": "Home",
  "/resources": "Resources"
})
const selectedFile = ref("/")

function closeHandler(payload: {path: string}) {
  delete files.value[payload.path]
}
</script>

<template>
  <card class="tab-bar">
    <div class="tab-item-group">
      <tab-item v-for="(file, path) in files" :name="file" :path="path" :open="selectedFile===path" @close="closeHandler" @click="selectedFile=path"/>
    </div>
  </card>
</template>

<style scoped lang="postcss">
.tab-bar {
  @apply mx-6 my-4 h-12 p-1.5;
}
.tab-item-group {
  @apply h-full flex justify-start gap-x-1.5
}

</style>