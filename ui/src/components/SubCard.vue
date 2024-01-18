<script setup lang="ts">

</script>

<template>
  <div class="sub-card">
    <slot></slot>
  </div>
</template>

<style scoped lang="postcss">
.sub-card {
  @apply bg-white/50 rounded-lg backdrop-blur shadow;
}
</style>