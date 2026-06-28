<script setup lang="ts">
import {computed, onBeforeUnmount, onMounted, watch, watchEffect} from "vue";
import {useI18n} from "./i18n";
import {resolveSystemColorMode, useAppearanceStore} from "./store/appearance.ts";

const appearanceStore = useAppearanceStore();
const {locale} = useI18n();
const colorSchemeQuery = typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window.matchMedia("(prefers-color-scheme: dark)")
    : null;
const resolvedColorMode = computed(() => appearanceStore.resolvedColorMode);

const syncSystemColorMode = () => {
  appearanceStore.setSystemColorMode(resolveSystemColorMode());
}

watchEffect(() => {
  Object.entries(appearanceStore.cssVars).forEach(([key, value]) => {
    document.documentElement.style.setProperty(key, value);
  });
});

watch(resolvedColorMode, mode => {
  document.documentElement.dataset.appTheme = mode;
  document.documentElement.style.colorScheme = mode;
}, {immediate: true});

watch(locale, value => {
  document.documentElement.lang = value;
}, {immediate: true});

onMounted(() => {
  syncSystemColorMode();
  colorSchemeQuery?.addEventListener("change", syncSystemColorMode);
});

onBeforeUnmount(() => {
  Object.keys(appearanceStore.cssVars).forEach(key => {
    document.documentElement.style.removeProperty(key);
  });
  document.documentElement.removeAttribute("data-app-theme");
  document.documentElement.style.removeProperty("color-scheme");
  colorSchemeQuery?.removeEventListener("change", syncSystemColorMode);
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
