<script setup lang="ts">
import {computed} from "vue";
import {useAppearanceStore} from "../store/appearance.ts";
import {
  resolveClassicSymbolName,
  resolveClassicSymbolTransform,
  resolveLucideIcon
} from "./icon-registry.ts";

type IconSize = string | "large" | "small" | "normal";

const props = withDefaults(defineProps<{
  icon: string;
  color?: string;
  size?: IconSize;
  strokeWidth?: number;
}>(), {
  color: "currentColor",
  size: "normal",
  strokeWidth: 2
});

const appearanceStore = useAppearanceStore();

const classicSymbolName = computed(() => resolveClassicSymbolName(props.icon));
const useClassicSymbol = computed(() => appearanceStore.iconStyle === "classic" && Boolean(classicSymbolName.value));
const iconComponent = computed(() => useClassicSymbol.value ? undefined : resolveLucideIcon(props.icon));
const symbolHref = computed(() => `#${classicSymbolName.value || props.icon}`);

const normalizedSize = computed(() => {
  if (props.size === "large") return "1.5rem";
  if (props.size === "small") return "0.75rem";
  if (props.size === "normal") return "1rem";
  if (/^\d+$/.test(props.size)) return `${props.size}rem`;
  return props.size;
});

const iconStyle = computed(() => ({
  width: normalizedSize.value,
  height: normalizedSize.value,
  color: props.color,
  transform: useClassicSymbol.value ? resolveClassicSymbolTransform(props.icon) : undefined
}));
</script>

<template>
  <component
      :is="iconComponent"
      v-if="iconComponent"
      class="app-icon app-icon-lucide"
      aria-hidden="true"
      :style="iconStyle"
      :stroke-width="strokeWidth" />
  <svg v-else class="app-icon app-icon-symbol" aria-hidden="true" :style="iconStyle">
    <use :href="symbolHref" :xlink:href="symbolHref" />
  </svg>
</template>

<style scoped>
.app-icon {
  display: inline-block;
  flex-shrink: 0;
  transform-box: fill-box;
  transform-origin: center;
  vertical-align: -0.125em;
}

.app-icon-lucide {
  fill: none;
  stroke: currentColor;
}

.app-icon-symbol {
  fill: currentColor;
}
</style>
