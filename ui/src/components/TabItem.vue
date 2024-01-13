<script setup lang="ts">
import SubCard from "./SubCard.vue";

const emit = defineEmits(['close'])

const props = defineProps({
  name: {
    type: String,
    require: true
  },
  path: {
    type: String,
    require: true
  },
  open: {
    type: Boolean,
    require: false,
    default: false
  }
});


</script>

<template>
  <sub-card class="tab-item" :class="{open}">
    <svg class="tab-icon w-4 fill-black" xmlns="http://www.w3.org/2000/svg" fill="none" version="1.1" width="12.5" height="10.25" viewBox="0 0 12.5 10.25"><g><path d="M12,1.7875L6.39062,1.7875L4.55781,0.034375C4.53438,0.0125,4.50313,0,4.47188,0L0.5,0C0.223438,0,0,0.223438,0,0.5L0,9.75C0,10.0266,0.223438,10.25,0.5,10.25L12,10.25C12.2766,10.25,12.5,10.0266,12.5,9.75L12.5,2.2875C12.5,2.01094,12.2766,1.7875,12,1.7875Z" fill-opacity="1"/></g></svg>
    <div class="grow mx-2 truncate">{{ props.name }}</div>
    <div class="w-6 ml-2 p-1.5 rounded hover:bg-gray-200/70" @click="emit('close', { path })">
      <svg class="h-full w-3 fill-black" xmlns="http://www.w3.org/2000/svg" fill="none" version="1.1" width="9.998254776000977" height="9.990312576293945"
           viewBox="0 0 9.998254776000977 9.990312576293945"><g><path d="M5.70563,4.97831L9.85063,0.854814C10.0466,0.659814,10.0476,0.343814,9.85263,0.147814C9.65713,-0.048686,9.34163,-0.049186,9.14563,0.145814L4.99963,4.26981L0.889628,0.147314C0.695127,-0.0476859,0.378627,-0.0486858,0.182627,0.146314C-0.0128729,0.341314,-0.0133729,0.657814,0.181627,0.853814L4.29063,4.97531L0.147627,9.09631C-0.048373,9.29131,-0.0493729,9.60731,0.145627,9.80331C0.243127,9.90131,0.371627,9.95081,0.500127,9.95081C0.627627,9.95081,0.755127,9.90231,0.852627,9.80531L4.99663,5.68331L9.14413,9.84331C9.24163,9.94131,9.36963,9.99031,9.49813,9.99031C9.62613,9.99031,9.75363,9.94131,9.85113,9.84431C10.0466,9.64931,10.0471,9.33331,9.85213,9.13731L5.70563,4.97831Z" fill-opacity="1"/></g></svg>
    </div>
  </sub-card>
</template>

<style scoped lang="postcss">
.tab-item {
  @apply flex pl-2.5 pr-0.5 min-w-28 max-w-60 h-full text-base items-center border-2 border-white hover:border-[#3662EC] cursor-pointer active:bg-[#3662EC] active:text-white
}

.tab-item.open {
  @apply bg-[#3662EC] text-white
}

.tab-icon {
 @apply h-full
}
</style>