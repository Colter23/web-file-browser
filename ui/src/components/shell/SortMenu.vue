<script setup lang="ts">
import {computed, nextTick, onBeforeUnmount, onMounted, ref} from "vue";
import type {DirSortKey, DirSortOrder} from "../../class";
import {useMenuKeyboardNavigation} from "../../composables/useMenuKeyboardNavigation.ts";
import {useOutsidePointerDown} from "../../composables/useOutsidePointerDown.ts";
import type {MessageKey} from "../../i18n";
import {useI18n} from "../../i18n";
import Icon from "../Icon.vue";

type SortOption = {
  key: DirSortKey;
  labelKey: MessageKey;
  descriptionKey: MessageKey;
  icon: string;
  hintKey?: MessageKey;
  hint?: string;
}

type OrderOption = {
  key: DirSortOrder;
  labelKey: MessageKey;
  icon: string;
}

const props = defineProps<{
  sortKey: DirSortKey;
  sortOrder: DirSortOrder;
}>();

const emit = defineEmits<{
  (e: "set-sort-key", key: DirSortKey): void;
  (e: "set-sort-order", order: DirSortOrder): void;
}>();

const {t} = useI18n();
const sortMenuRef = ref<HTMLElement | null>(null);
const sortMenuPanelRef = ref<HTMLElement | null>(null);
const sortButtonRef = ref<HTMLButtonElement | null>(null);
const open = ref(false);

const sortOptions: SortOption[] = [
  {key: "name", labelKey: "sort.name", descriptionKey: "sort.byName", icon: "sort.name", hint: "A-Z"},
  {key: "type", labelKey: "sort.type", descriptionKey: "sort.byType", icon: "sort.type", hintKey: "sort.type"},
  {key: "modified", labelKey: "sort.modified", descriptionKey: "sort.byModified", icon: "sort.modified", hintKey: "sort.time"},
  {key: "size", labelKey: "sort.size", descriptionKey: "sort.bySize", icon: "sort.size", hintKey: "sort.size"}
];

const orderOptions: OrderOption[] = [
  {key: "asc", labelKey: "sort.asc", icon: "sort.asc"},
  {key: "desc", labelKey: "sort.desc", icon: "sort.desc"}
];

const activeSortOption = computed(() => sortOptions.find(option => option.key === props.sortKey) ?? sortOptions[0]);
const activeSortLabel = computed(() => t(activeSortOption.value.labelKey));
const activeSortIcon = computed(() => activeSortOption.value.icon);
const activeOrderLabel = computed(() => props.sortOrder === "asc" ? t("sort.asc") : t("sort.desc"));
const buttonTitle = computed(() => t("sort.buttonTitle", {field: activeSortLabel.value, order: activeOrderLabel.value}));

const orderDescription = (order: DirSortOrder) => {
  if (props.sortKey === "modified") return order === "asc" ? t("sort.oldToNew") : t("sort.newToOld");
  if (props.sortKey === "size") return order === "asc" ? t("sort.smallToLarge") : t("sort.largeToSmall");
  return order === "asc" ? t("sort.aToZ") : t("sort.zToA");
}

const close = () => {
  open.value = false;
}

const focusButton = async () => {
  await nextTick();
  sortButtonRef.value?.focus({preventScroll: true});
}

const {
  focusMenuButton,
  handleMenuKeyDown
} = useMenuKeyboardNavigation({
  menuRef: sortMenuPanelRef,
  onEscape: () => {
    close();
    void focusButton();
  }
});

const focusActiveOption = async () => {
  await nextTick();
  const activeIndex = sortOptions.findIndex(option => option.key === props.sortKey);
  focusMenuButton(activeIndex >= 0 ? activeIndex : 0);
}

const openMenu = () => {
  open.value = true;
  void focusActiveOption();
}

const toggle = async () => {
  open.value = !open.value;
  if (open.value) await focusActiveOption();
}

const selectSortKey = (key: DirSortKey) => {
  if (key === props.sortKey) return;
  emit("set-sort-key", key);
}

const selectSortOrder = (order: DirSortOrder) => {
  if (order === props.sortOrder) return;
  emit("set-sort-order", order);
}

const handleButtonKeyDown = (event: KeyboardEvent) => {
  if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
  event.preventDefault();
  if (!open.value) openMenu();
  else void focusActiveOption();
}

useOutsidePointerDown({
  refs: [sortMenuRef],
  enabled: () => open.value,
  onOutsidePointerDown: close
});

const handleDocumentKeyDown = (event: KeyboardEvent) => {
  if (event.key === "Escape") close();
}

onMounted(() => {
  window.addEventListener("keydown", handleDocumentKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleDocumentKeyDown);
});
</script>

<template>
  <div ref="sortMenuRef" class="sort-menu">
    <button
        ref="sortButtonRef"
        class="sort-button"
        :class="{active: open}"
        :title="buttonTitle"
        aria-haspopup="menu"
        :aria-expanded="open"
        @click="toggle"
        @keydown="handleButtonKeyDown">
      <icon :icon="activeSortIcon" />
      <span class="sort-button-label">
        <span>{{ activeSortLabel }}</span>
        <small>{{ activeOrderLabel }}</small>
      </span>
      <icon icon="action.down" class="sort-caret icon-motion-caret" :class="{'is-open': open}" />
    </button>
    <div v-if="open" ref="sortMenuPanelRef" class="sort-menu-panel" role="menu" :aria-label="t('sort.aria')" @keydown="handleMenuKeyDown">
      <section class="sort-menu-section">
        <p class="menu-group-title">{{ t("sort.field") }}</p>
        <div class="sort-field-list">
          <button
              v-for="option in sortOptions"
              :key="option.key"
              class="sort-field-item"
              :class="{active: sortKey === option.key}"
              role="menuitemradio"
              :aria-checked="sortKey === option.key"
              tabindex="-1"
              @click="selectSortKey(option.key)">
            <span class="sort-option-icon"><icon :icon="option.icon" /></span>
            <span class="sort-menu-copy">
              <strong>{{ t(option.labelKey) }}</strong>
              <small>{{ t(option.descriptionKey) }}</small>
            </span>
            <span class="sort-option-hint">{{ option.hintKey ? t(option.hintKey) : option.hint }}</span>
            <span class="sort-check"><icon v-if="sortKey === option.key" icon="action.check" size="small" /></span>
          </button>
        </div>
      </section>
      <section class="sort-menu-section">
        <p class="menu-group-title">{{ t("sort.order") }}</p>
        <div class="sort-order-toggle">
          <button
              v-for="option in orderOptions"
              :key="option.key"
              class="sort-order-button"
              :class="{active: sortOrder === option.key}"
              role="menuitemradio"
              :aria-checked="sortOrder === option.key"
              tabindex="-1"
            @click="selectSortOrder(option.key)">
            <icon :icon="option.icon" />
            <span class="sort-menu-copy">
              <strong>{{ t(option.labelKey) }}</strong>
              <small>{{ orderDescription(option.key) }}</small>
            </span>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.sort-menu {
  @apply relative shrink-0;
}

.sort-button {
  @apply inline-flex h-10 shrink-0 items-center justify-center gap-2 rounded-md border px-3 text-sm;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.sort-button:hover {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-hover, #eff6ff);
}

.sort-button.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
}

.sort-button:focus-visible {
  @apply outline-none;
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 3px var(--app-accent-ring, rgba(37, 99, 235, 0.22));
}

.sort-button-label {
  @apply inline-flex min-w-0 items-center gap-1.5;
}

.sort-button-label > span {
  @apply truncate;
  max-width: 4.5rem;
}

.sort-button-label small {
  @apply rounded px-1.5 py-0.5 text-[0.68rem] font-medium leading-none;
  background: var(--app-panel-muted);
  color: var(--app-text-subtle);
}

.sort-button.active .sort-button-label small,
.sort-button:hover .sort-button-label small {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 11%, transparent);
  color: var(--app-accent, #2563eb);
}

.sort-caret {
  @apply text-[0.65rem];
  color: var(--app-text-subtle);
}

.sort-menu-panel {
  @apply absolute right-0 top-[calc(100%+0.35rem)] z-50 w-[20.5rem] overflow-hidden rounded-md border p-2;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: var(--app-menu-shadow);
}

.sort-menu-section + .sort-menu-section {
  @apply mt-2;
}

.menu-group-title {
  @apply px-1 pb-1 text-[0.68rem] font-medium;
  color: var(--app-text-subtle);
}

.sort-field-list {
  @apply grid gap-1;
}

.sort-field-item {
  @apply grid w-full grid-cols-[2rem_minmax(0,1fr)_auto_1.25rem] items-center gap-2 rounded-md border px-2 py-1.5 text-left text-sm;
  border-color: transparent;
  color: var(--app-text-muted);
}

.sort-field-item:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-hover);
}

.sort-field-item.active {
  border-color: var(--app-accent-border, #bfdbfe);
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, var(--app-panel-solid));
  color: var(--app-accent, #2563eb);
}

.sort-field-item:focus-visible,
.sort-order-button:focus-visible {
  @apply outline-none;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, var(--app-panel-solid));
  color: var(--app-accent, #2563eb);
  box-shadow: inset 0 0 0 1px var(--app-accent-border, #bfdbfe);
}

.sort-option-icon {
  @apply grid size-8 place-items-center rounded-md;
  background: var(--app-control);
  color: var(--app-text-muted);
}

.sort-field-item.active .sort-option-icon {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 14%, transparent);
  color: var(--app-accent, #2563eb);
}

.sort-option-hint {
  @apply rounded border px-1.5 py-0.5 text-[0.68rem] font-medium;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
  color: var(--app-text-subtle);
}

.sort-field-item.active .sort-option-hint {
  border-color: color-mix(in srgb, var(--app-accent, #2563eb) 25%, var(--app-border-soft));
  background: color-mix(in srgb, var(--app-accent, #2563eb) 10%, transparent);
  color: var(--app-accent, #2563eb);
}

.sort-check {
  @apply grid size-5 place-items-center rounded-full text-xs font-semibold;
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, transparent);
  color: var(--app-accent, #2563eb);
}

.sort-menu-copy {
  @apply flex min-w-0 flex-col;
}

.sort-menu-copy strong {
  @apply truncate text-sm font-medium;
}

.sort-menu-copy small {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.sort-field-item.active .sort-menu-copy small,
.sort-order-button.active .sort-menu-copy small {
  color: var(--app-accent, #2563eb);
}

.sort-order-toggle {
  @apply grid grid-cols-2 gap-1 rounded-md border p-0.5;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
}

.sort-order-button {
  @apply grid grid-cols-[1.1rem_minmax(0,1fr)] items-center gap-1.5 rounded px-2 py-1 text-left text-[0.8rem];
  color: var(--app-text-muted);
}

.sort-order-button .sort-menu-copy strong {
  @apply text-[0.8rem];
}

.sort-order-button .sort-menu-copy small {
  @apply text-[0.7rem];
}

.sort-order-button:hover {
  background: var(--app-control-hover);
}

.sort-order-button.active {
  background: color-mix(in srgb, var(--app-accent, #2563eb) 12%, var(--app-panel-solid));
  color: var(--app-accent, #2563eb);
  box-shadow:
      inset 0 0 0 1px var(--app-accent-border, #bfdbfe),
      0 1px 2px rgba(15, 23, 42, 0.08);
}
</style>
