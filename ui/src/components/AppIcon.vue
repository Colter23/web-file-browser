<script setup lang="ts">
import {computed, shallowRef, watch} from "vue";
import {useAppearanceStore} from "../store/appearance.ts";
import {resolveAppIcon} from "./icon-registry.ts";
import type {AppIconDefinition} from "./icon-packs/types.ts";

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
const resolvedIcon = shallowRef<AppIconDefinition>();
let resolveRunId = 0;

watch(
    () => [appearanceStore.iconStyle, props.icon] as const,
    async ([style, icon]) => {
      const runId = ++resolveRunId;
      const nextIcon = await resolveAppIcon(style, icon);
      if (runId === resolveRunId) resolvedIcon.value = nextIcon;
    },
    {immediate: true}
);

const iconComponent = computed(() => resolvedIcon.value?.kind === "component" ? resolvedIcon.value.component : undefined);
const symbolHref = computed(() => resolvedIcon.value?.kind === "symbol" ? `#${resolvedIcon.value.symbol}` : "");
const iconClass = computed(() => resolvedIcon.value?.className);

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
  transform: resolvedIcon.value?.transform
}));
</script>

<template>
  <component
      :is="iconComponent"
      v-if="iconComponent"
      class="app-icon"
      :class="iconClass"
      aria-hidden="true"
      :style="iconStyle"
      :stroke-width="strokeWidth" />
  <svg v-else-if="symbolHref" class="app-icon" :class="iconClass" aria-hidden="true" :style="iconStyle">
    <use :href="symbolHref" :xlink:href="symbolHref" />
  </svg>
  <span v-else class="app-icon app-icon-missing" aria-hidden="true" :style="iconStyle" />
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
