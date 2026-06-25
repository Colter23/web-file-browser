<script setup lang="ts">
import {computed, shallowRef, useAttrs, watch} from "vue";
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
  size: "normal",
  strokeWidth: 2
});

const appearanceStore = useAppearanceStore();
const attrs = useAttrs();
const resolvedIcon = shallowRef<AppIconDefinition>();
let resolveRunId = 0;

defineOptions({
  inheritAttrs: false
});

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
const passthroughAttrs = computed(() => {
  const {class: _class, style: _style, ...rest} = attrs;
  return rest;
});

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
  color: props.color || undefined,
  transform: resolvedIcon.value?.transform
}));
const rootClass = computed(() => [attrs.class, "app-icon", iconClass.value]);
const fallbackClass = computed(() => [attrs.class, "app-icon", "app-icon-missing"]);
const rootStyle = computed(() => [attrs.style, iconStyle.value]);
</script>

<template>
  <component
      :is="iconComponent"
      v-if="iconComponent"
      v-bind="passthroughAttrs"
      :class="rootClass"
      aria-hidden="true"
      :style="rootStyle"
      :stroke-width="strokeWidth" />
  <svg v-else-if="symbolHref" v-bind="passthroughAttrs" :class="rootClass" aria-hidden="true" :style="rootStyle">
    <use :href="symbolHref" :xlink:href="symbolHref" />
  </svg>
  <span v-else v-bind="passthroughAttrs" :class="fallbackClass" aria-hidden="true" :style="rootStyle" />
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
