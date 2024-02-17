<script setup lang="ts">
import Icon from "../Icon.vue";
import CodeEditor from "./CodeEditor.vue";
import {computed, Ref, ref} from "vue";
import editorConfig from "../../assets/editor-config.json"
import {useFileStore} from "../../store";


const fileStore = useFileStore();

const tabItem = ['语言', '主题', '设置'];

const settingList = ['VS', 'OneDark'];

const fileInfo = {
  name: "test.json",
  path: "AAA/test.json",
  modified: "2024/2/2 10:00:00",
  size: "100 KB",
  extension: "json"
}

const themeFlag = ref(false);
const languageFlag = ref(false);
const settingFlag = ref(false);

const toolMenuBox = ref();

const currentTheme = ref('dracula');
const currentMode = ref('json');


const themeClass = computed(() => {
  return `ace-${currentTheme.value.replaceAll('_', '-')}`;
})


const changeMode = (mode: string) => {
  currentMode.value = mode;
}

const changeTheme = (theme: string) => {
  currentTheme.value = theme;
}

const addClickListener = (el: Element, flag: Ref) => {
  el.addEventListener("click", () => {
    if (!flag.value) flag.value = true;
    const handle = (e: MouseEvent) => {
      if (!el.contains(e.target as Node)) {
        flag.value = false;
        document.removeEventListener("click", handle);
      }
    }
    document.addEventListener("click", handle);
  })
}

const vClickOutside = {
  mounted: (el: Element, binding: object) => {
    if (binding.value == '主题') {
      addClickListener(el, themeFlag);
    } else if (binding.value == '语言') {
      addClickListener(el, languageFlag);
    } else if (binding.value == '设置') {
      addClickListener(el, settingFlag);
    }
  }
}

const itemActive = (select: string) => {
  return (languageFlag.value && select == '语言') ||
      (themeFlag.value && select == '主题') ||
      (settingFlag.value && select == '设置');
}

</script>

<template>
  <div class="editor-box">
    <div class="tool-bar mask" :class="themeClass">
      <div class="tool-left">
        <div class="tool-item" v-for="item in tabItem" v-click-outside="item" :class="itemActive(item)?'item-active':''">
          <span class="pointer-events-none">{{ item }}</span>
        </div>
      </div>
      <div class="tool-center">
        <span class="ace_variable">{{ fileInfo.name }}</span>
      </div>
      <div class="tool-right">
        <div class="tool-button-icon hover:bg-white/30">
          <icon icon="icon-save-fill" size="large" color="#ffffff" />
        </div>

        <div class="tool-button-icon hover:bg-white/30">
          <icon icon="icon-add" size="large" color="#ffffff" />
        </div>

        <div class="tool-button-icon hover:bg-red-600/50" @click="fileStore.showEditor = false">
          <icon icon="icon-close" size="large" color="#ffffff" />
        </div>
      </div>
    </div>

    <code-editor class="border-t-[1px] border-b-[1px] border-white/50" :mode="currentMode" :theme="currentTheme"></code-editor>

    <div class="footer-bar mask" :class="themeClass">
      <div class="footer-left">
        <div>*未保存</div>
        <div>{{ fileInfo.modified }}</div>
      </div>
      <div class="footer-right">
        <div>{{ currentMode }}</div>
        <div>{{ fileInfo.size }}</div>
      </div>
    </div>

    <div class="tool-menu" ref="toolMenuBox">
      <div class="tool-menu-item tool-menu-language" :class="`${languageFlag? 'inline-flex' : 'hidden'} ${themeClass}`">
        <div class="tool-menu-item-option" v-for="mode in editorConfig.mode" @click="changeMode(mode.key)">
          <icon icon="icon-file" color="#ffffff" />
          <span>{{ mode.name }}</span>
        </div>
      </div>

      <div class="tool-menu-item tool-menu-theme" :class="`${themeFlag? 'inline-flex' : 'hidden'} ${themeClass}`">
        <div class="ml-1 font-bold">Light</div>
        <div class="tool-menu-item-option" v-for="theme in editorConfig.theme.light" @click="changeTheme(theme.key)">
          <div class="w-2 grow-0 shrink-0"></div>
          <span>{{ theme.name }}</span>
        </div>
        <div class="ml-1 mt-1 font-bold">Dark</div>
        <div class="tool-menu-item-option" v-for="theme in editorConfig.theme.dark" @click="changeTheme(theme.key)">
          <div class="w-2 grow-0 shrink-0"></div>
          <span>{{ theme.name }}</span>
        </div>
      </div>

      <div class="tool-menu-item tool-menu-setting" :class="`${settingFlag? 'inline-flex' : 'hidden'} ${themeClass}`">
        <div class="tool-menu-item-option" v-for="setting in settingList">
          <icon icon="icon-file" color="#ffffff" />
          <span>{{ setting }}</span>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped lang="postcss">
.editor-box {
  @apply relative w-full h-full flex flex-col text-white
}

.mask {
  @apply before:block before:absolute before:w-full before:h-full before:bg-black/10 before:z-0 before:pointer-events-none
}

.tool-bar {
  @apply relative w-full h-10 flex justify-between
}
.tool-left {
  @apply flex h-full px-3 py-1 gap-1
}
.tool-center {
  @apply flex items-center justify-center
}
.tool-right {
  @apply flex h-full px-1 gap-1 items-center
}
.tool-button-icon {
  @apply  inline-flex h-7 w-7 items-center justify-center rounded
}
.tool-item {
  @apply flex items-center justify-center w-20 text-base rounded cursor-pointer hover:bg-white/20
}
.item-active {
  @apply bg-white/20
}

.footer-bar {
  @apply relative flex items-center justify-between w-full h-7 text-sm
}
.footer-left {
  @apply inline-flex gap-4 px-2 z-[1]
}
.footer-right {
  @apply inline-flex gap-4 px-2 z-[1]
}

.tool-menu {
  @apply absolute z-10
}
.tool-menu-item {
  @apply absolute flex-col p-1 max-h-80 overflow-y-scroll border-white/15 border-[1px] translate-y-9 rounded shadow-2xl
}

.tool-menu-item::-webkit-scrollbar{
  @apply w-2
}
.tool-menu-item::-webkit-scrollbar-thumb {
  @apply w-0 bg-white/80 rounded
}

.tool-menu-language {
  @apply translate-x-3 w-40
}
.tool-menu-theme {
  @apply translate-x-24 w-56
}
.tool-menu-setting {
  @apply translate-x-44
}
.tool-menu-item-option {
  @apply inline-flex items-center gap-2 px-2 w-full hover:bg-white/20 rounded-sm cursor-pointer
}
</style>