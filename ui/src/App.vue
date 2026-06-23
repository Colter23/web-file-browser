<script setup lang="ts">
import {useAppearanceStore} from "./store/appearance.ts";

const appearanceStore = useAppearanceStore();
</script>

<template>
  <div class="app-root" :style="appearanceStore.cssVars">
    <router-view></router-view>
  </div>
</template>

<style scoped lang="postcss">
.app-root {
  min-height: 100vh;
}

</style>
