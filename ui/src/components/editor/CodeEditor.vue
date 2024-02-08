<script setup lang="ts">
import ace from "ace-builds";
import "ace-builds/src-noconflict/ext-language_tools";
import "ace-builds/esm-resolver";
import {onMounted, watch} from "vue";


// 导入语言工作文件（用于语法检测）
ace.config.setModuleUrl("ace/mode/base_worker", "ace/worker-base.js");
ace.config.setModuleUrl("ace/mode/json_worker", "ace/worker-json.js");
ace.config.setModuleUrl("ace/mode/xml_worker", "ace/worker-xml.js");
ace.config.setModuleUrl("ace/mode/yaml_worker", "ace/worker-yaml.js");


interface CodeEditorProps {
  mode: string;
  theme: string;
}

const props = withDefaults(defineProps<CodeEditorProps>(), {
  mode: "text",
  theme: "dracula"
})


onMounted(() => {
  // 创建编辑器
  const editor = ace.edit("editor");

  // 初始化编辑器
  editor.setOptions({
    theme: "ace/theme/" + props.theme,
    fontSize: 18,
    mode: "ace/mode/" + props.mode,

    enableBasicAutocompletion: true,
    enableSnippets: true,
    enableLiveAutocompletion: true
  });

  watch(() => props.theme, (theme: string) => {
    editor.setTheme("ace/theme/" + theme);
  });
  watch(() => props.mode, (mode: string) => {
    editor.session.setMode("ace/mode/" + mode);
  });
})
</script>

<template>
  <div id="editor"></div>
</template>

<style scoped lang="postcss">
#editor {
  @apply w-full h-full
}
</style>