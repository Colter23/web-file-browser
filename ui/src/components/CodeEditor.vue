<script setup lang="ts">
import * as monaco from "monaco-editor"
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import {onMounted} from "vue";
import {languages} from "monaco-editor";

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

</script>

<template>
  <div class="editor-box">
    <div class="tool-bar">
      <div>
        <label for="theme">主题:</label>
        <select id="theme">
          <option>vs</option>
          <option>hc-black</option>
          <option>vs-dark</option>
        </select>
      </div>
      <div>
        <label for="language">语言:</label>
        <select id="language">
          <option>json</option>
          <option>yaml</option>
          <option>html</option>
        </select>
      </div>

    </div>
    <div id="editor"></div>
  </div>

</template>

<style scoped lang="postcss">
.editor-box {
  @apply w-full h-full
}
.tool-bar {
  @apply w-full h-9
}

#editor {
  @apply w-full h-full
}
</style>