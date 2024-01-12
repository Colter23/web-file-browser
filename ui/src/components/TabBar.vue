<script setup lang="ts">

import Card from "./Card.vue";
import TabItem from "./TabItem.vue";
import {ref} from "vue";


const files = ref(["Resource", "Download"])

function closeHandler(payload: object) {
  console.log(files.value.indexOf(payload.name))
  console.log(files.value.toString())
  const index = files.value.indexOf(payload.name)

  files.value = files.value.splice(index + (index >= files.value.length ? 0 : 1), 1)
  console.log(files.value.toString())
}

</script>

<template>
  <card class="tab-bar">
    <div class="tab-item-group">
      <tab-item v-for="file in files" :name="file" @close="closeHandler"/>
    </div>
  </card>
</template>

<style scoped>
.tab-bar {
  @apply mx-6 my-4 h-12 p-1.5;
}
.tab-item-group {
  @apply h-full flex justify-start gap-x-1.5
}

</style>