<script setup lang="ts">
import Icon from "../Icon.vue";
import CodeEditor from "./CodeEditor.vue";
import {computed, Ref, ref, watch} from "vue";
import type {DirectiveBinding} from "vue";
import editorConfig from "../../assets/editor-config.json"
import {useFileStore} from "../../store";
import {FileInfo} from "../../class.ts";
import {getFile, saveFile} from "../../network/file-api.ts";
import {checkFileLanguageMode} from "../../utils/common.ts";

const fileStore = useFileStore();
const tabItem = ['语言', '主题', '设置'];
const settingList = ['VS', 'OneDark'];

const fileInfo = ref<FileInfo | null>(null);
const themeFlag = ref(false);
const languageFlag = ref(false);
const settingFlag = ref(false);
const currentTheme = ref('dracula');
const currentMode = ref('text');
const content = ref("")
const contentEtag = ref("");
const isChange = ref(false);
const saving = ref(false);
const statusText = ref("");

const themeClass = computed(() => {
  return `ace-${currentTheme.value.replace(/_/g, '-')}`;
})

const changeMode = (mode: string) => {
  currentMode.value = mode;
}

const changeTheme = (theme: string) => {
  currentTheme.value = theme;
}

const addClickListener = (el: Element, flag: Ref<boolean>) => {
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
  mounted: (el: Element, binding: DirectiveBinding<string>) => {
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

const loadCurrentFile = async () => {
  if (!fileStore.showEditor || fileStore.currentFile == null) return;
  fileInfo.value = fileStore.currentFile;
  currentMode.value = checkFileLanguageMode(fileStore.currentFile.extension);
  statusText.value = "";
  try {
    const file = await getFile(fileStore.currentFile.path);
    content.value = file.content;
    contentEtag.value = file.etag;
    isChange.value = false;
  } catch (error) {
    window.alert(error instanceof Error ? error.message : "打开文件失败");
    fileStore.showEditor = false;
    fileStore.currentFile = null;
    content.value = "";
    contentEtag.value = "";
    isChange.value = false;
  }
}

watch(() => [fileStore.showEditor, fileStore.currentFile?.path], loadCurrentFile);

const onContentChange = (value: string) => {
  content.value = value;
  isChange.value = true;
  statusText.value = "";
}

const save = async () => {
  if (!fileInfo.value || saving.value) return;
  saving.value = true;
  try {
    if (!contentEtag.value) {
      throw new Error("文件版本信息缺失，请重新打开文件后再保存");
    }
    const saved = await saveFile(fileInfo.value.path, content.value, contentEtag.value);
    contentEtag.value = saved.etag;
    isChange.value = false;
    statusText.value = "已保存";
  } catch (error) {
    window.alert(error instanceof Error ? error.message : "保存失败");
  } finally {
    saving.value = false;
  }
}

const close = () => {
  if (isChange.value && !window.confirm("放弃未保存的修改？")) {
    return;
  }
  fileStore.showEditor = false;
  fileStore.currentFile = null;
  isChange.value = false;
  contentEtag.value = "";
  statusText.value = "";
}
</script>

<template>
  <div class="editor-box">
    <div class="tool-bar mask" :class="themeClass">
      <div class="tool-left">
        <div class="tool-item" v-for="item in tabItem" :key="item" v-click-outside="item" :class="itemActive(item)?'item-active':''">
          <span class="pointer-events-none">{{ item }}</span>
        </div>
      </div>
      <div class="tool-center">
        <span class="ace_variable">{{ fileInfo?.name ?? "" }}</span>
      </div>
      <div class="tool-right">
        <div class="tool-button-icon hover:bg-white/30" @click="save()">
          <icon icon="icon-save-fill" size="large" color="#ffffff" />
        </div>

        <div class="tool-button-icon hover:bg-red-600/50" @click="close()">
          <icon icon="icon-close" size="large" color="#ffffff" />
        </div>
      </div>
    </div>

    <code-editor
        class="border-t-[1px] border-b-[1px] border-white/50"
        :mode="currentMode"
        :theme="currentTheme"
        :content="content"
        @change="onContentChange">
    </code-editor>

    <div class="footer-bar mask" :class="themeClass">
      <div class="footer-left">
        <div v-show="isChange">*未保存</div>
        <div v-show="statusText">{{ statusText }}</div>
        <div>{{ fileInfo?.modified ?? "" }}</div>
      </div>
      <div class="footer-right">
        <div>{{ currentMode }}</div>
        <div>{{ fileInfo?.size ?? 0 }}</div>
      </div>
    </div>

    <div class="tool-menu">
      <div class="tool-menu-item tool-menu-language" :class="`${languageFlag? 'inline-flex' : 'hidden'} ${themeClass}`">
        <div class="tool-menu-item-option" v-for="mode in editorConfig.mode" :key="mode.key" @click="changeMode(mode.key)">
          <icon icon="icon-file" color="#ffffff" />
          <span>{{ mode.name }}</span>
        </div>
      </div>

      <div class="tool-menu-item tool-menu-theme" :class="`${themeFlag? 'inline-flex' : 'hidden'} ${themeClass}`">
        <div class="ml-1 font-bold">Light</div>
        <div class="tool-menu-item-option" v-for="theme in editorConfig.theme.light" :key="theme.key" @click="changeTheme(theme.key)">
          <div class="w-2 grow-0 shrink-0"></div>
          <span>{{ theme.name }}</span>
        </div>
        <div class="ml-1 mt-1 font-bold">Dark</div>
        <div class="tool-menu-item-option" v-for="theme in editorConfig.theme.dark" :key="theme.key" @click="changeTheme(theme.key)">
          <div class="w-2 grow-0 shrink-0"></div>
          <span>{{ theme.name }}</span>
        </div>
      </div>

      <div class="tool-menu-item tool-menu-setting" :class="`${settingFlag? 'inline-flex' : 'hidden'} ${themeClass}`">
        <div class="tool-menu-item-option" v-for="setting in settingList" :key="setting">
          <icon icon="icon-file" color="#ffffff" />
          <span>{{ setting }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
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
