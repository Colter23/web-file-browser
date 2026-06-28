<script setup lang="ts">
import type {StyleValue} from "vue";
import Icon from "../Icon.vue";

defineProps<{
  muted: boolean;
  volume: number;
  volumeStyle: StyleValue;
  volumeTitle: string;
  muteButtonTitle: string;
  audibleVolumePercent: string;
  variant?: "inline" | "floating";
}>();

const emit = defineEmits<{
  (e: "toggleMute"): void;
  (e: "updateVolume", event: Event): void;
}>();
</script>

<template>
  <div class="audio-volume" :class="`variant-${variant ?? 'inline'}`" :aria-label="volumeTitle">
    <button :title="muteButtonTitle" @click="emit('toggleMute')">
      <icon :icon="muted || volume === 0 ? 'action.volume-muted' : 'action.volume'" color="currentColor" />
    </button>
    <div class="audio-volume-slider" :style="volumeStyle">
      <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          :value="muted ? 0 : volume"
          :aria-label="volumeTitle"
          :title="volumeTitle"
          @input="emit('updateVolume', $event)">
      <span class="audio-volume-badge">{{ audibleVolumePercent }}</span>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.audio-volume {
  @apply flex shrink-0 items-center gap-1;
}

.audio-volume.variant-inline {
  @apply w-32 cursor-pointer;
}

.audio-volume.variant-floating {
  @apply relative w-8 cursor-pointer overflow-visible;
}

.audio-volume-slider {
  @apply relative min-w-0 flex-1;
}

.audio-volume.variant-floating::before {
  content: "";
  @apply pointer-events-auto absolute right-1/2 bottom-full h-3 w-32;
  transform: translateX(50%);
}

.audio-volume.variant-floating .audio-volume-slider {
  @apply pointer-events-none absolute right-1/2 bottom-[calc(100%+0.25rem)] w-28 rounded-full border px-2 py-1 opacity-0 shadow-xl backdrop-blur-xl;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 94%, transparent);
  transform: translate(50%, 0.375rem) scale(0.98);
  transform-origin: bottom center;
  transition: opacity 140ms ease, transform 140ms ease;
}

.audio-volume.variant-floating:hover .audio-volume-slider {
  @apply pointer-events-auto opacity-100;
  transform: translate(50%, 0) scale(1);
}

.audio-volume button {
  @apply inline-flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md border border-transparent text-sm;
  color: var(--app-text-muted);
}

.audio-volume.variant-floating button {
  @apply h-7 w-7 rounded-full;
}

.audio-volume button:hover:not(:disabled) {
  border-color: var(--app-border-soft);
  background: var(--app-accent-hover, #eff6ff);
  color: var(--app-text);
}

.audio-volume button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.audio-volume input[type="range"] {
  @apply h-4 w-full cursor-pointer appearance-none bg-transparent;
}

.audio-volume input[type="range"]::-webkit-slider-runnable-track {
  @apply h-1 rounded-full;
  cursor: pointer;
  background: linear-gradient(
      to right,
      var(--app-accent, #2563eb) 0%,
      var(--app-accent, #2563eb) var(--audio-progress, 0%),
      var(--app-border-soft) var(--audio-progress, 0%),
      var(--app-border-soft) 100%
  );
}

.audio-volume input[type="range"]::-webkit-slider-thumb {
  @apply h-3.5 w-3.5 appearance-none rounded-full border;
  margin-top: -0.3125rem;
  cursor: pointer;
  border-color: var(--app-panel-solid);
  background: var(--app-accent, #2563eb);
  box-shadow: 0 1px 5px rgba(15, 23, 42, 0.25);
}

.audio-volume input[type="range"]::-moz-range-track {
  @apply h-1 rounded-full;
  cursor: pointer;
  background: var(--app-border-soft);
}

.audio-volume input[type="range"]::-moz-range-progress {
  @apply h-1 rounded-full;
  cursor: pointer;
  background: var(--app-accent, #2563eb);
}

.audio-volume input[type="range"]::-moz-range-thumb {
  @apply h-3.5 w-3.5 rounded-full border-0;
  cursor: pointer;
  background: var(--app-accent, #2563eb);
}

.audio-volume-badge {
  @apply pointer-events-none absolute rounded-full px-2 py-0.5 text-[0.68rem] font-semibold tabular-nums opacity-0;
  bottom: calc(100% + 0.25rem);
  left: clamp(1.25rem, var(--audio-volume, 0%), calc(100% - 1.25rem));
  background: color-mix(in srgb, var(--app-control-solid) 88%, transparent);
  color: var(--app-text);
  box-shadow: 0 1px 6px rgba(15, 23, 42, 0.12);
  transform: translate(-50%, 0.25rem);
  transition: opacity 120ms ease, transform 120ms ease;
}

.audio-volume:hover .audio-volume-badge {
  opacity: 1;
  transform: translate(-50%, 0);
}
</style>
