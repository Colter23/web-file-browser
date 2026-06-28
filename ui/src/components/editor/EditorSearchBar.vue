<script setup lang="ts">
import {useI18n} from "../../i18n";
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
  readOnly: boolean;
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

const {t} = useI18n();

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
  <div v-if="visible" class="search-bar" @click.stop @keydown.esc.prevent.stop="emit('close')">
    <div class="search-fields">
      <input
          :ref="setSearchInputRef"
          :value="searchText"
          class="search-input"
          type="text"
          :placeholder="t('editor.findPlaceholder')"
          @keydown.enter.prevent.stop="emit('search-input', $event)"
          @input="updateSearchText">
      <input
          v-if="replaceVisible"
          :ref="setReplaceInputRef"
          :value="replaceText"
          class="search-input replace-input"
          type="text"
          :placeholder="t('editor.replacePlaceholder')"
          @keydown.enter.prevent.stop="emit('replace-current')"
          @input="updateReplaceText">
    </div>
    <div class="search-actions">
      <span v-if="searchStatusText" class="search-status">{{ searchStatusText }}</span>
      <button :title="t('editor.previousMatch')" :disabled="!canFind" @click="emit('search', true)">
        <icon icon="action.up" />
      </button>
      <button :title="t('editor.nextMatch')" :disabled="!canFind" @click="emit('search', false)">
        <icon icon="action.down" />
      </button>
      <button v-if="!replaceVisible && !readOnly" :title="t('editor.showReplace')" @click="emit('show-replace')">
        <icon icon="action.rename" />
      </button>
      <button v-if="replaceVisible" class="text-tool" :title="t('editor.replaceCurrent')" :disabled="!canReplace" @click="emit('replace-current')">{{ t("editor.replace") }}</button>
      <button v-if="replaceVisible" class="text-tool" :title="t('editor.replaceAll')" :disabled="!canReplace" @click="emit('replace-all')">{{ t("editor.replaceAllShort") }}</button>
      <button class="text-tool" :class="{active: caseSensitive}" :title="t('editor.caseSensitive')" @click="emit('toggle-option', 'case')">Aa</button>
      <button class="text-tool" :class="{active: wholeWord}" :title="t('editor.wholeWord')" @click="emit('toggle-option', 'word')">W</button>
      <button class="text-tool" :class="{active: regex}" :title="t('editor.regex')" @click="emit('toggle-option', 'regex')">.*</button>
      <button :title="t('editor.closeFind')" @click="emit('close')">
        <icon icon="action.close" />
      </button>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.search-bar {
  @apply relative z-20 flex shrink-0 flex-wrap items-center justify-between gap-2 rounded-md border px-2 py-1.5 text-xs shadow-sm;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

.search-fields {
  @apply flex min-w-64 grow items-center gap-2;
}

.search-input {
  @apply h-8 min-w-0 flex-1 rounded-md border px-2 text-sm outline-none;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text);
}

.search-input::placeholder {
  color: var(--app-text-disabled);
}

.search-input:focus {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.replace-input {
  border-color: var(--app-border);
}

.search-actions {
  @apply flex min-w-0 shrink-0 items-center gap-1;
  color: var(--app-text-muted);
}

.search-actions button {
  @apply inline-flex h-8 min-w-8 items-center justify-center rounded-md border px-2 text-xs font-medium disabled:cursor-not-allowed disabled:opacity-40;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.search-actions button:disabled {
  background: var(--app-control-solid);
}

.search-actions button:hover:not(:disabled) {
  background: var(--app-accent-hover, #eff6ff);
}

.search-actions button.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.search-actions .text-tool {
  @apply min-w-9;
}

.search-status {
  @apply max-w-32 truncate px-1;
  color: var(--app-warning-text);
}

@media (max-width: 760px) {
  .search-fields {
    @apply min-w-full;
  }

  .search-actions {
    @apply w-full justify-end;
  }
}
</style>
