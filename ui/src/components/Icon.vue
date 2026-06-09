<script setup lang="ts">
import {computed} from "vue";

const props = withDefaults(defineProps<{
  icon: string;
  color?: string;
  size?: string | "large" | "small" | "normal"
}>(), {
  color: "#333333",
  size: "normal"
})

const iconClassName = computed(() => {
  return `#${props.icon}`;
});

const iconSize = computed(() => {
  let size = "1rem";
  if (props.size === "large") {
    size = "1.5rem";
  } else if (props.size === "small") {
    size = "0.75rem";
  } else if (props.size === "normal") {
    size = "1rem";
  } else if (/^\d+$/.test(props.size)) {
    // 纯数字，添加单位
    size = `${props.size}rem`
  } else if (props.size) {
    size = props.size
  }
  return {
    width: size,
    height: size
  }
})
</script>

<template>
  <svg class="icon" aria-hidden="true" :style="iconSize">
    <use :xlink:href="iconClassName" :fill="props.color" />
  </svg>
</template>

<style scoped>
.icon {
  position: relative;
  fill: currentColor;
  vertical-align: -2px;
}
</style>