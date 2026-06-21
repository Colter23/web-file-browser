<script setup lang="ts">
import ace from "ace-builds";
import "ace-builds/src-noconflict/ext-language_tools";
import "ace-builds/esm-resolver";
import {onBeforeUnmount, onMounted, ref, watch} from "vue";

ace.config.setModuleUrl("ace/mode/base_worker", "ace/worker-base.js");
ace.config.setModuleUrl("ace/mode/json_worker", "ace/worker-json.js");
ace.config.setModuleUrl("ace/mode/xml_worker", "ace/worker-xml.js");
ace.config.setModuleUrl("ace/mode/yaml_worker", "ace/worker-yaml.js");

interface CodeEditorProps {
  mode: string;
  theme: string;
  content: string;
  fontSize?: number;
  wrap?: boolean;
  tabSize?: number;
}

const props = withDefaults(defineProps<CodeEditorProps>(), {
  mode: "text",
  theme: "dracula",
  content: "",
  fontSize: 16,
  wrap: true,
  tabSize: 2
})

const emit = defineEmits<{
  (e: "change", content: string): void;
  (e: "save"): void;
}>()

const editorRef = ref<HTMLElement | null>(null);
let editor: ReturnType<typeof ace.edit> | null = null;

onMounted(() => {
  if (!editorRef.value) return;
  editor = ace.edit(editorRef.value);
  let syncing = false;

  editor.setOptions({
    theme: "ace/theme/" + props.theme,
    fontSize: props.fontSize,
    mode: "ace/mode/" + props.mode,
    value: props.content,

    enableBasicAutocompletion: true,
    enableSnippets: true,
    enableLiveAutocompletion: true
  });
  editor.session.setUseWrapMode(props.wrap);
  editor.session.setTabSize(props.tabSize);
  editor.commands.addCommand({
    name: "saveFile",
    bindKey: {win: "Ctrl-S", mac: "Command-S"},
    exec: () => emit("save")
  });

  watch(() => props.theme, (theme: string) => {
    editor?.setTheme("ace/theme/" + theme);
  });
  watch(() => props.mode, (mode: string) => {
    editor?.session.setMode("ace/mode/" + mode);
  });
  watch(() => props.content, (content: string) => {
    if (!editor || editor.getValue() === content) return;
    syncing = true;
    editor.session.setValue(content);
    syncing = false;
  });
  watch(() => props.fontSize, (fontSize: number) => {
    editor?.setOption("fontSize", fontSize);
  });
  watch(() => props.wrap, (wrap: boolean) => {
    editor?.session.setUseWrapMode(wrap);
  });
  watch(() => props.tabSize, (tabSize: number) => {
    editor?.session.setTabSize(tabSize);
  });

  editor.session.on("change", () => {
    if (editor && !syncing) {
      emit("change", editor.getValue());
    }
  });
})

onBeforeUnmount(() => {
  editor?.destroy();
  editor = null;
});

defineExpose({
  focus: () => editor?.focus()
})
</script>

<template>
  <div ref="editorRef" class="code-editor"></div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
.code-editor {
  @apply w-full h-full
}
</style>
