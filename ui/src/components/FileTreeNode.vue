<script setup lang="ts">
import Icon from "./Icon.vue";

const props = defineProps({
  deep: {
    type: Number,
    default: 0
  },
  content: String
});

</script>

<template>
<div class="tree-node">
  <div class="node-indent" v-for="_ in deep"></div>
  <div class="node-fold-icon">
    <icon icon="icon-unfold" size="normal"></icon>
  </div>
  <div class="node-content">{{ content }}</div>
</div>
</template>

<style scoped lang="postcss">
.tree-node {
  @apply flex w-full h-7 px-1 rounded-md hover:bg-[#EBF3FF]
}
.node-indent {
  @apply w-7 grow-0 shrink-0
}
.node-fold-icon {
  @apply w-7 inline-flex grow-0 shrink-0 items-center justify-center cursor-pointer transition-transform -rotate-90
}
.node-content {
  @apply inline-flex grow items-center cursor-pointer
}
</style>