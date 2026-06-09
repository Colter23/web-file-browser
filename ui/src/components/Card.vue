<script setup lang="ts">

</script>

<template>
  <div class="card">
    <slot></slot>
  </div>
</template>

<style scoped lang="postcss">
.card {
  @apply bg-white/50 border-2 border-white rounded-xl backdrop-blur shadow-lg;
}
</style>