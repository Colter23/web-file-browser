<script setup lang="ts">
import SubCard from "./SubCard.vue";
import Icon from "./Icon.vue";

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
    <Icon icon="icon-folder-fill" size="normal" />
    <div class="grow mx-2 truncate">{{ props.name }}</div>
    <div class="w-6 ml-2 p-1.5 rounded hover:bg-gray-200/70" @click.stop="emit('close', { path })">
      <Icon icon="icon-delete" size="small" />
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