<script setup lang="ts">
const props = defineProps({
  deep: {
    type: Number,
    default: 0
  }
});

// const retract = 0

</script>

<template>
<div class="tree-node">
  <div class="tree-indent" v-for="_ in deep"></div>
  <div class="inline-flex"></div>
  <div>AAA</div>
</div>
</template>

<style scoped>
.tree-node {
  @apply flex w-full h-7 rounded-md bg-amber-500
}
.tree-indent {
  @apply w-6 grow-0 shrink-0
}
</style>