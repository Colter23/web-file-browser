<script setup lang="ts">
import ace from "ace-builds";
import "ace-builds/src-noconflict/ext-language_tools";
import "ace-builds/esm-resolver";
import {onMounted, watch} from "vue";

ace.config.setModuleUrl("ace/mode/base_worker", "ace/worker-base.js");
ace.config.setModuleUrl("ace/mode/json_worker", "ace/worker-json.js");
ace.config.setModuleUrl("ace/mode/xml_worker", "ace/worker-xml.js");
ace.config.setModuleUrl("ace/mode/yaml_worker", "ace/worker-yaml.js");

interface CodeEditorProps {
  mode: string;
  theme: string;
  content: string;
}

const props = withDefaults(defineProps<CodeEditorProps>(), {
  mode: "text",
  theme: "dracula",
  content: ""
})

const emit = defineEmits<{
  (e: "change", content: string): void;
}>()

onMounted(() => {
  const editor = ace.edit("editor");
  let syncing = false;

  editor.setOptions({
    theme: "ace/theme/" + props.theme,
    fontSize: 18,
    mode: "ace/mode/" + props.mode,
    value: props.content,

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
  watch(() => props.content, (content: string) => {
    if (editor.getValue() === content) return;
    syncing = true;
    editor.session.setValue(content);
    syncing = false;
  });

  editor.session.on("change", () => {
    if (!syncing) {
      emit("change", editor.getValue());
    }
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
