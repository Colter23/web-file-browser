<script setup lang="ts">

import {ref} from "vue";

const props = defineProps({
  width: {
    type: Number,
    default: 300
  }
});

const split = ref()

const widthRef = ref(props.width)

const handleDown = () => {
  document.addEventListener("mouseup", handleUp)
  document.addEventListener("mousemove", handleMove)
}
const handleUp = () => {
  document.removeEventListener("mouseup", handleUp)
  document.removeEventListener("mousemove", handleMove)
}

const handleMove = (e) => {
  widthRef.value = e.clientX - split.value.getBoundingClientRect().left
}

</script>

<template>
  <div class="flex" ref="split">
    <div :style="'width: ' + widthRef + 'px'">
      <slot name="left"></slot>
    </div>
    <div class="w-1 h-90 mx-1 bg-amber-500 cursor-col-resize" @mousedown="handleDown"></div>
    <div class="grow">
      <slot name="right"></slot>
    </div>
  </div>

</template>

<style scoped lang="postcss">

</style>