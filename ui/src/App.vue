<script setup lang="ts">
import {onBeforeUnmount, watchEffect} from "vue";
import {useAppearanceStore} from "./store/appearance.ts";

const appearanceStore = useAppearanceStore();

watchEffect(() => {
  Object.entries(appearanceStore.cssVars).forEach(([key, value]) => {
    document.documentElement.style.setProperty(key, value);
  });
});

onBeforeUnmount(() => {
  Object.keys(appearanceStore.cssVars).forEach(key => {
    document.documentElement.style.removeProperty(key);
  });
});
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
