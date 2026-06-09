<script setup lang="ts">
import {onMounted, reactive, ref} from "vue";
import {useRouter} from "vue-router";
import {PathMapping, RuntimeSettings} from "../class";
import {createMapping, deleteMapping, getMappings, getSettings, updateMapping} from "../network/api";

const router = useRouter();
const mappings = ref<PathMapping[]>([]);
const settings = ref<RuntimeSettings | null>(null);
const loading = ref(false);
const message = ref("");

const form = reactive<PathMapping>({
  mountPath: "",
  folderPath: "",
  remark: "",
  order: 0,
  writable: true
})

const load = async () => {
  loading.value = true;
  message.value = "";
  try {
    const [mappingData, settingData] = await Promise.all([getMappings(), getSettings()]);
    mappings.value = mappingData;
    settings.value = settingData;
  } catch (error) {
    message.value = error instanceof Error ? error.message : "加载设置失败";
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const resetForm = () => {
  form.mountPath = "";
  form.folderPath = "";
  form.remark = "";
  form.order = 0;
  form.writable = true;
}

const addMapping = async () => {
  message.value = "";
  try {
    await createMapping({...form});
    resetForm();
    await load();
  } catch (error) {
    message.value = error instanceof Error ? error.message : "添加挂载失败";
  }
}

const saveMapping = async (mapping: PathMapping) => {
  if (mapping.id == null) return;
  message.value = "";
  try {
    await updateMapping(mapping.id, mapping);
    await load();
  } catch (error) {
    message.value = error instanceof Error ? error.message : "保存挂载失败";
  }
}

const removeMapping = async (mapping: PathMapping) => {
  if (mapping.id == null) return;
  if (!window.confirm(`删除挂载 ${mapping.mountPath}？`)) return;
  message.value = "";
  try {
    await deleteMapping(mapping.id);
    await load();
  } catch (error) {
    message.value = error instanceof Error ? error.message : "删除挂载失败";
  }
}
</script>

<template>
  <div class="settings-page">
    <header>
      <button class="ghost-button" @click="router.push('/')">返回</button>
      <h1>设置</h1>
      <button class="ghost-button" @click="load">刷新</button>
    </header>

    <main>
      <section class="panel">
        <h2>挂载目录</h2>
        <form class="mapping-form" @submit.prevent="addMapping">
          <input v-model="form.mountPath" placeholder="/repo" required>
          <input v-model="form.folderPath" placeholder="D:\Files" required>
          <input v-model="form.remark" placeholder="备注">
          <input v-model.number="form.order" type="number" placeholder="排序">
          <label class="check-field">
            <input v-model="form.writable" type="checkbox">
            <span>可写</span>
          </label>
          <button class="primary-button" type="submit">添加</button>
        </form>

        <div class="mapping-list">
          <div v-for="mapping in mappings" :key="mapping.id" class="mapping-row">
            <input v-model="mapping.mountPath">
            <input v-model="mapping.folderPath">
            <input v-model="mapping.remark">
            <input v-model.number="mapping.order" type="number">
            <label class="check-field">
              <input v-model="mapping.writable" type="checkbox">
              <span>可写</span>
            </label>
            <button class="plain-button" @click="saveMapping(mapping)">保存</button>
            <button class="danger-button" @click="removeMapping(mapping)">删除</button>
          </div>
          <div v-if="!mappings.length && !loading" class="empty">暂无挂载目录</div>
        </div>
      </section>

      <section class="panel" v-if="settings">
        <h2>服务配置</h2>
        <dl>
          <div><dt>监听地址</dt><dd>{{ settings.bindAddress }}:{{ settings.port }}</dd></div>
          <div><dt>映射文件</dt><dd>{{ settings.mappingFile }}</dd></div>
          <div><dt>配置文件</dt><dd>{{ settings.configFile }}</dd></div>
          <div><dt>回收站</dt><dd>{{ settings.trashDir }}</dd></div>
          <div><dt>静态目录</dt><dd>{{ settings.staticDir }}</dd></div>
          <div><dt>上传上限</dt><dd>{{ settings.maxUploadBytes ? `${settings.maxUploadBytes} bytes` : "不限制" }}</dd></div>
          <div><dt>目录分页上限</dt><dd>{{ settings.maxDirPageSize }}</dd></div>
          <div><dt>目录并发</dt><dd>{{ settings.maxDirConcurrency }}</dd></div>
          <div><dt>传输并发</dt><dd>{{ settings.maxTransferConcurrency }}</dd></div>
          <div><dt>IP 并发</dt><dd>{{ settings.maxIpConcurrency }}</dd></div>
          <div><dt>任务并发</dt><dd>{{ settings.maxTaskConcurrency }}</dd></div>
          <div><dt>任务限速</dt><dd>{{ settings.taskSpeedLimitBytesPerSec ? `${settings.taskSpeedLimitBytesPerSec} bytes/s` : "不限制" }}</dd></div>
          <div><dt>搜索索引</dt><dd>{{ settings.indexEnabled ? "已启用" : "未启用" }}</dd></div>
          <div><dt>审计日志</dt><dd>{{ settings.auditFile }}</dd></div>
          <div><dt>回收站保留</dt><dd>{{ settings.trashRetentionDays ? `${settings.trashRetentionDays} 天` : "不限制" }}</dd></div>
          <div><dt>回收站容量</dt><dd>{{ settings.trashMaxBytes ? `${settings.trashMaxBytes} bytes` : "不限制" }}</dd></div>
          <div><dt>认证</dt><dd>{{ settings.authConfigured ? "已初始化" : "未初始化" }}</dd></div>
        </dl>
      </section>

      <div v-if="message" class="message">{{ message }}</div>
    </main>
  </div>
</template>

<style scoped lang="postcss">
.settings-page {
  @apply min-h-screen bg-slate-100 text-slate-900
}

header {
  @apply h-14 bg-white border-b border-slate-200 px-4 flex items-center justify-between
}

h1 {
  @apply text-xl font-semibold
}

main {
  @apply p-4 flex flex-col gap-4
}

.panel {
  @apply bg-white border border-slate-200 rounded-lg p-4 flex flex-col gap-4
}

h2 {
  @apply text-base font-semibold
}

.mapping-form,
.mapping-row {
  @apply grid gap-2 items-center
}

.mapping-form {
  grid-template-columns: 1fr 2fr 1fr 90px 90px 90px;
}

.mapping-row {
  grid-template-columns: 1fr 2fr 1fr 90px 90px 80px 80px;
}

input {
  @apply h-9 min-w-0 rounded-md border border-slate-300 px-2 text-sm outline-none focus:border-blue-500
}

.check-field {
  @apply h-9 flex items-center gap-2 text-sm text-slate-700
}

.check-field input {
  @apply h-4 w-4
}

.mapping-list {
  @apply flex flex-col gap-2
}

.primary-button,
.plain-button,
.danger-button,
.ghost-button {
  @apply h-9 rounded-md px-3 text-sm font-medium
}

.primary-button {
  @apply bg-blue-600 text-white hover:bg-blue-700
}

.plain-button,
.ghost-button {
  @apply bg-slate-100 text-slate-700 hover:bg-slate-200
}

.danger-button {
  @apply bg-red-50 text-red-700 hover:bg-red-100
}

.empty {
  @apply rounded-md border border-dashed border-slate-300 px-3 py-6 text-center text-sm text-slate-500
}

dl {
  @apply grid gap-2 text-sm
}

dl div {
  @apply grid gap-3;
  grid-template-columns: 8rem minmax(0, 1fr);
}

dt {
  @apply text-slate-500
}

dd {
  @apply min-w-0 break-all
}

.message {
  @apply rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700
}

@media (max-width: 900px) {
  .mapping-form,
  .mapping-row {
    grid-template-columns: 1fr;
  }
}
</style>
