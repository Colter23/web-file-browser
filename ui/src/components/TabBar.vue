<script setup lang="ts">

import Card from "./Card.vue";
import TabItem from "./TabItem.vue";
import {ref} from "vue";
import SubCard from "./SubCard.vue";
import Icon from "./Icon.vue";

const emit = defineEmits<{
  (e: "change", path: string): void;
  (e: "close", path: string): void;
}>()
const files = ref<{[key: string]: string}>({
  "/": "Home",
  "/resources": "Resources"
})
const selectedFile = ref("/")

function closeHandler(payload: {path: string}) {
  delete files.value[payload.path]
  emit("close", payload.path)
}

const changeHandler = (path: string): void => {
  selectedFile.value = path
  emit("change", path)
}
</script>

<template>
  <card class="tab-bar">
    <div class="tab-item-group">
      <tab-item v-for="(file, path) in files" :name="file" :path="path" :open="selectedFile===path" @close="closeHandler" @click="changeHandler(path)"/>
      <sub-card class="add-btn" @click="files['/test']='TEST'">
        <icon icon="icon-add" size="normal" />
      </sub-card>
    </div>
  </card>
</template>

<style scoped lang="postcss">
.tab-bar {
  @apply h-12 p-1.5;
}
.tab-item-group {
  @apply h-full flex justify-start gap-x-1.5
}
.add-btn {
  @apply flex p-1.5 h-full text-base items-center border-2 border-white hover:border-[#3662EC] cursor-pointer active:bg-[#3662EC] active:text-white
}

</style>