<script setup lang="ts">
import * as monaco from "monaco-editor"
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import {computed, Events, onMounted, ref} from "vue";
import {languages} from "monaco-editor";
import Icon from "./Icon.vue";

// 语法检测
self.MonacoEnvironment = {
  getWorker(workerId, label) {
    switch (label) {
      case 'json':
        return new jsonWorker();
      case 'css':
      case 'scss':
      case 'less':
        return new cssWorker();
      case 'html':
      case 'razor':
      case 'handlebars':
        return new htmlWorker();
      case 'typescript':
      case 'javascript':
        return new tsWorker();
      default: return new EditorWorker();
    }
  },
};


onMounted(() => {
  const editorBox = document.getElementById('editor');

  monaco.editor.create(editorBox as HTMLElement, {
    value: '',
    language: 'plaintext',
    theme: 'vs-dark', //官方自带三种主题vs, hc-black, or vs-dark
    automaticLayout: true,
    fontSize: 18, //字体大小
    wordWrap: 'off', // 自动换行
  });

  // console.log(languages.getLanguages())

})

const themeFlag = ref(false)
const languageFlag = ref(false)
const settingFlag = ref(false)

const toolMenuBox = ref()

function showSelect(select: string) {
  const languageClick = () => {
    languageFlag.value = false
    document.removeEventListener('mousedown', languageClick)
  }
  const themeClick = () => {
    themeFlag.value = false
    document.removeEventListener('mousedown', themeClick)
  }
  const settingClick = () => {
    settingFlag.value = false
    document.removeEventListener('mousedown', settingClick)
  }

  if (select == '主题') {
    if (!themeFlag.value) themeFlag.value = true
    document.addEventListener('mousedown', themeClick)
  } else if (select == '语言') {
    if (!languageFlag.value) languageFlag.value = true
    document.addEventListener('mousedown', languageClick)
  } else if (select == '设置') {
    if (!settingFlag.value) settingFlag.value = true
    document.addEventListener('mousedown', settingClick)
  }
}

const tabItem = ['语言', '主题', '设置']

const languageList = ['Json', 'Java', 'JavaScript', 'CSS']
const themeList = ['VS', 'OneDark']
const settingList = ['VS', 'OneDark']

const itemActive = (select: string) => {
  return (languageFlag.value && select == '语言') ||
  (themeFlag.value && select == '主题') ||
  (settingFlag.value && select == '设置')
}

</script>

<template>
  <div class="editor-box">
    <div class="tool-bar">
      <div class="tool-left">
        <div class="tool-item" v-for="item in tabItem" @click="showSelect(item)" :class="itemActive(item)?'item-active':''">
          <span class="pointer-events-none">{{ item }}</span>
        </div>
      </div>
      <div class="tool-center">
        <span>test.json</span>
      </div>
      <div class="tool-right">
        <div class="tool-button-icon hover:bg-white/30">
          <icon icon="icon-save-fill" size="large" color="#ffffff" />
        </div>

        <div class="tool-button-icon hover:bg-white/30">
          <icon icon="icon-add" size="large" color="#ffffff" />
        </div>

        <div class="tool-button-icon hover:bg-red-600/50">
          <icon icon="icon-close" size="large" color="#ffffff" />
        </div>
      </div>
    </div>
    <div id="editor"></div>

    <div class="footer-bar">
      <div class="footer-left">
        <div>*未保存</div>
        <div>2024/2/2 10:00:00</div>
      </div>
      <div class="footer-right">
        <div>Json</div>
        <div>100 KB</div>
      </div>
    </div>

    <div id="tool-menu" ref="toolMenuBox">
      <div class="tool-menu-item tool-menu-language" :class="languageFlag? 'inline-flex' : 'hidden'">
        <div class="tool-menu-item-option" v-for="mode in languageList">
          <icon icon="icon-file" color="#ffffff" />
          <span>{{ mode }}</span>
        </div>
      </div>

      <div class="tool-menu-item tool-menu-theme" :class="themeFlag? 'inline-flex' : 'hidden'">
        <div class="tool-menu-item-option" v-for="theme in themeList">
          <icon icon="icon-file" color="#ffffff" />
          <span>{{ theme }}</span>
        </div>
      </div>

      <div class="tool-menu-item tool-menu-setting" :class="settingFlag? 'inline-flex' : 'hidden'">
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
.tool-bar {
  @apply w-full h-10 flex justify-between text-[#CDCFD5] bg-[#11111B]
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
  @apply flex items-center justify-between w-full h-7 bg-[#11111B] text-sm text-white/80
}
.footer-left {
  @apply inline-flex gap-4 px-2
}
.footer-right {
  @apply inline-flex gap-4 px-2
}

#tool-menu {
  @apply absolute
}
.tool-menu-item {
  @apply absolute flex-col p-1 w-40 bg-[#1E1E2E] border-white/15 border-[1px] cursor-pointer translate-y-9 rounded shadow-2xl
}
.tool-menu-language {
  @apply translate-x-3
}
.tool-menu-theme {
  @apply translate-x-24
}
.tool-menu-setting {
  @apply translate-x-44
}
.tool-menu-item-option {
  @apply inline-flex items-center gap-2 px-2 w-full hover:bg-white/20 rounded-sm
}
#editor {
  @apply w-full h-full border-t-[1px] border-b-[1px] border-[#61616C]
}
</style>