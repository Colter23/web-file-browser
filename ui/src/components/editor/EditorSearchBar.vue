<script setup lang="ts">
import Icon from "../Icon.vue";
import type {EditorInputRefSetter, EditorSearchOptionName} from "./types.ts";

defineProps<{
  visible: boolean;
  replaceVisible: boolean;
  searchText: string;
  replaceText: string;
  searchStatusText: string;
  caseSensitive: boolean;
  wholeWord: boolean;
  regex: boolean;
  canFind: boolean;
  canReplace: boolean;
  setSearchInputRef: EditorInputRefSetter;
  setReplaceInputRef: EditorInputRefSetter;
}>();

const emit = defineEmits<{
  (e: "update:searchText", value: string): void;
  (e: "update:replaceText", value: string): void;
  (e: "show-replace"): void;
  (e: "search", backwards: boolean): void;
  (e: "search-input", event: KeyboardEvent): void;
  (e: "replace-current"): void;
  (e: "replace-all"): void;
  (e: "toggle-option", option: EditorSearchOptionName): void;
  (e: "clear-status"): void;
  (e: "close"): void;
}>();

const updateSearchText = (event: Event) => {
  const input = event.target as HTMLInputElement | null;
  emit("update:searchText", input?.value ?? "");
  emit("clear-status");
}

const updateReplaceText = (event: Event) => {
  const input = event.target as HTMLInputElement | null;
  emit("update:replaceText", input?.value ?? "");
}
</script>

<template>
  <div v-if="visible" class="search-bar" @click.stop>
    <div class="search-fields">
      <input
          :ref="setSearchInputRef"
          :value="searchText"
          class="search-input"
          type="text"
          placeholder="查找"
          @keydown.enter.prevent="emit('search-input', $event)"
          @input="updateSearchText">
      <input
          v-if="replaceVisible"
          :ref="setReplaceInputRef"
          :value="replaceText"
          class="search-input replace-input"
          type="text"
          placeholder="替换为"
          @keydown.enter.prevent="emit('replace-current')"
          @input="updateReplaceText">
    </div>
    <div class="search-actions">
      <span v-if="searchStatusText" class="search-status">{{ searchStatusText }}</span>
      <button title="上一个 (Shift+Enter)" :disabled="!canFind" @click="emit('search', true)">
        <icon icon="icon-back_android" class="rotate-90" />
      </button>
      <button title="下一个 (Enter)" :disabled="!canFind" @click="emit('search', false)">
        <icon icon="icon-back_android" class="-rotate-90" />
      </button>
      <button v-if="!replaceVisible" title="显示替换 (Ctrl+H)" @click="emit('show-replace')">
        <icon icon="icon-renamebox" />
      </button>
      <button v-if="replaceVisible" class="text-tool" title="替换当前" :disabled="!canReplace" @click="emit('replace-current')">替换</button>
      <button v-if="replaceVisible" class="text-tool" title="全部替换" :disabled="!canReplace" @click="emit('replace-all')">全部</button>
      <button class="text-tool" :class="{active: caseSensitive}" title="区分大小写" @click="emit('toggle-option', 'case')">Aa</button>
      <button class="text-tool" :class="{active: wholeWord}" title="全词匹配" @click="emit('toggle-option', 'word')">W</button>
      <button class="text-tool" :class="{active: regex}" title="正则表达式" @click="emit('toggle-option', 'regex')">.*</button>
      <button title="关闭查找" @click="emit('close')">
        <icon icon="icon-close" />
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.search-bar {
  @apply relative z-20 flex shrink-0 items-center justify-between gap-2 rounded-md border border-slate-200 bg-white px-2 py-1.5 text-xs shadow-sm;
}

.search-fields {
  @apply flex min-w-0 grow items-center gap-2;
}

.search-input {
  @apply h-8 min-w-0 flex-1 rounded-md border border-slate-200 bg-white px-2 text-sm text-slate-900 outline-none placeholder:text-slate-400 focus:border-blue-500 focus:ring-2 focus:ring-blue-100;
}

.replace-input {
  @apply border-slate-300;
}

.search-actions {
  @apply flex shrink-0 items-center gap-1 text-slate-600;
}

.search-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border border-slate-200 bg-white px-2 text-xs font-medium text-slate-600 hover:bg-blue-50 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-white;
}

.search-actions button.active {
  @apply border-blue-300 bg-blue-50 text-blue-700;
}

.search-actions .text-tool {
  @apply min-w-9;
}

.search-status {
  @apply max-w-28 truncate px-1 text-amber-600;
}
</style>
