<script setup lang="ts">
import type {AppIconStyle} from "../class.ts";
import AppIcon from "./AppIcon.vue";

const props = withDefaults(defineProps<{
  icon: string;
  color?: string;
  size?: string | "large" | "small" | "normal";
  iconStyle?: AppIconStyle;
}>(), {
  size: "normal"
});
</script>

<template>
  <app-icon class="icon" :icon="props.icon" :color="props.color" :size="props.size" :icon-style="props.iconStyle" />
</template>
