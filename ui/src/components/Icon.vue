<script setup lang="ts">
import AppIcon from "./AppIcon.vue";

const props = withDefaults(defineProps<{
  icon: string;
  color?: string;
  size?: string | "large" | "small" | "normal";
}>(), {
  size: "normal"
});
</script>

<template>
  <app-icon class="icon" :icon="props.icon" :color="props.color" :size="props.size" />
</template>
