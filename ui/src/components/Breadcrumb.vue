<script setup lang="ts">

import SubCard from "./SubCard.vue";
import Icon from "./Icon.vue";
import {onMounted, ref} from "vue";


const path: string = "/AAAA/BBBBB/CCCCCCCCCC/DDDDDDDDDDDD"

const pathList = path.split("/")

const pathBox = ref()
const input = ref()
const pathInput = ref()

const isInput = ref(false)

onMounted(() => {
  const pathBoxEl = pathBox.value as Element
  pathBoxEl.addEventListener("wheel", (e) => {
    e.preventDefault()
    pathBoxEl.scrollLeft += e.deltaY
  })
})

const changeInput = (e: Event) => {
  e.stopPropagation()
  isInput.value = true

  const handle = (e: MouseEvent) => {
    if (!input.value.contains(e.target as Node)) {
      isInput.value = false
      document.removeEventListener("mousedown", handle)
    }
  }
  document.addEventListener("mousedown", handle)
}


</script>

<template>
  <sub-card class="path-card">
    <div ref="pathBox" class="path-box" :class="isInput? 'hidden':'flex'">
      <div class="flex items-center gap-1" v-for="(pathItem, index) in pathList">
        <div class="path-item"><span>{{ pathItem }}</span></div>
        <div class="separator" v-if="index != pathList.length - 1"><icon icon="icon-unfold" class="-rotate-90" /></div>
      </div>
    </div>
    <div class="grow h-full min-w-14 cursor-text" :class="isInput? 'hidden':''" @click="changeInput">
    </div>
    <div ref="input" class="grow" :class="isInput? 'flex':'hidden'">
      <input ref="pathInput" class="w-full h-10 px-3" :value="path">
    </div>

  </sub-card>
</template>

<style scoped lang="postcss">
.path-card {
  @apply flex grow shrink min-w-48 h-10 items-center gap-1
}
.path-box {
  @apply px-2 shrink overflow-x-scroll
}
.path-item {
  @apply inline-flex items-center h-7 px-2 hover:bg-blue-100 rounded cursor-pointer
}
.separator {
  @apply w-4 h-7 inline-flex items-center justify-center
}
.path-box::-webkit-scrollbar {
  display: none;
}
</style>