<script setup lang="ts">
import {computed, onMounted, reactive, ref} from "vue";
import {useRouter} from "vue-router";
import type {IndexStatus, MetricsResponse, PathMapping, RuntimeSettings} from "../class";
import {
  cancelIndexRebuild,
  changePassword,
  createMapping,
  deleteMapping,
  getIndexStatus,
  getMappings,
  getMetrics,
  getSettings,
  rebuildIndex,
  updateMapping
} from "../network/api";
import {
  accentColorOptions,
  colorModeOptions,
  fileIconPaletteOptions,
  iconStyleOptions,
  useAppearanceStore
} from "../store/appearance.ts";
import {formatEntryDate, formatEntrySize} from "../utils/file-entry.ts";

const router = useRouter();
const appearanceStore = useAppearanceStore();
const mappings = ref<PathMapping[]>([]);
const settings = ref<RuntimeSettings | null>(null);
const indexStatus = ref<IndexStatus | null>(null);
const metrics = ref<MetricsResponse | null>(null);
const loading = ref(false);
const indexLoading = ref(false);
const indexActionLoading = ref(false);
const metricsLoading = ref(false);
const message = ref("");
const messageKind = ref<"success" | "error">("error");

const form = reactive<PathMapping>({
  mountPath: "",
  folderPath: "",
  remark: "",
  order: 0,
  writable: true
})

const passwordForm = reactive({
  currentPassword: "",
  newPassword: "",
  confirmPassword: ""
})

const indexBusy = computed(() => indexLoading.value || indexActionLoading.value);
const indexBuilding = computed(() => indexStatus.value?.state === "building");
const canRebuildIndex = computed(() => Boolean(indexStatus.value?.enabled) && !indexBusy.value && !indexBuilding.value);
const canCancelIndex = computed(() => Boolean(indexStatus.value?.enabled) && !indexBusy.value && indexBuilding.value);
const taskMetrics = computed(() => metrics.value?.tasks);
const limitMetrics = computed(() => metrics.value?.limits);

const load = async () => {
  loading.value = true;
  message.value = "";
  try {
    const [mappingData, settingData] = await Promise.all([getMappings(), getSettings()]);
    mappings.value = mappingData;
    settings.value = settingData;
    await Promise.all([loadIndexStatus(false), loadMetrics(false)]);
  } catch (error) {
    showError(error, "加载设置失败");
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

const conflictPolicyText = (policy: RuntimeSettings["conflictPolicy"]) => {
  return {
    autoRename: "自动重命名",
    reject: "拒绝",
    overwrite: "覆盖"
  }[policy] ?? policy;
}

const corsOriginsText = (origins: RuntimeSettings["corsAllowedOrigins"]) => {
  return origins.length ? origins.join("，") : "同源";
}

const listText = (values: string[]) => {
  return values.length ? values.join("，") : "不限制";
}

const indexStateText = (status: IndexStatus | null) => {
  if (!status) return "未知";
  if (!status.enabled || status.state === "disabled") return "未启用";
  return {
    idle: "空闲",
    building: "重建中",
    error: "异常"
  }[status.state] ?? status.state;
}

const indexStateClass = (status: IndexStatus | null) => {
  if (!status || !status.enabled || status.state === "disabled") return "disabled";
  return {
    idle: "idle",
    building: "building",
    error: "error"
  }[status.state] ?? "idle";
}

const optionalDateText = (value?: string | null) => value ? formatEntryDate(value) : "-";

const indexEntryCountText = (value?: number) => Number.isFinite(value) ? `${value} 项` : "-";

const isFiniteNumber = (value: unknown): value is number => typeof value === "number" && Number.isFinite(value);

const countText = (value?: number) => isFiniteNumber(value) ? String(value) : "-";

const bytesText = (value?: number) => formatEntrySize(value, "-");

const optionalBytesText = (value?: number) => isFiniteNumber(value) && value > 0 ? formatEntrySize(value) : "不限制";

const speedText = (value?: number) => `${formatEntrySize(value, "0 B")}/s`;

const limitUsageText = (active?: number, limit?: number) => {
  if (!isFiniteNumber(active) || !isFiniteNumber(limit)) return "-";
  return `${active}/${limit}`;
}

const limitUsageRatio = (active?: number, limit?: number) => {
  if (!isFiniteNumber(active) || !isFiniteNumber(limit) || !limit) return "0%";
  return `${Math.min(100, Math.max(0, active / limit * 100))}%`;
}

const showError = (error: unknown, fallback: string) => {
  messageKind.value = "error";
  message.value = error instanceof Error ? error.message : fallback;
}

const showSuccess = (text: string) => {
  messageKind.value = "success";
  message.value = text;
}

const loadIndexStatus = async (showFailure = true) => {
  indexLoading.value = true;
  try {
    indexStatus.value = await getIndexStatus();
  } catch (error) {
    indexStatus.value = null;
    if (showFailure) showError(error, "加载索引状态失败");
  } finally {
    indexLoading.value = false;
  }
}

const loadMetrics = async (showFailure = true) => {
  metricsLoading.value = true;
  try {
    metrics.value = await getMetrics();
    if (metrics.value.index) indexStatus.value = metrics.value.index;
  } catch (error) {
    metrics.value = null;
    if (showFailure) showError(error, "加载运行状态失败");
  } finally {
    metricsLoading.value = false;
  }
}

const requestIndexRebuild = async () => {
  if (!canRebuildIndex.value) return;
  message.value = "";
  indexActionLoading.value = true;
  try {
    await rebuildIndex();
    showSuccess("已开始重建搜索索引");
    await loadIndexStatus(false);
  } catch (error) {
    showError(error, "启动索引重建失败");
  } finally {
    indexActionLoading.value = false;
  }
}

const requestIndexCancel = async () => {
  if (!canCancelIndex.value) return;
  message.value = "";
  indexActionLoading.value = true;
  try {
    await cancelIndexRebuild();
    showSuccess("已发送取消索引重建请求");
    await loadIndexStatus(false);
  } catch (error) {
    showError(error, "取消索引重建失败");
  } finally {
    indexActionLoading.value = false;
  }
}

const addMapping = async () => {
  message.value = "";
  try {
    await createMapping({...form});
    resetForm();
    await load();
  } catch (error) {
    showError(error, "添加挂载失败");
  }
}

const saveMapping = async (mapping: PathMapping) => {
  if (mapping.id == null) return;
  message.value = "";
  try {
    await updateMapping(mapping.id, mapping);
    await load();
  } catch (error) {
    showError(error, "保存挂载失败");
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
    showError(error, "删除挂载失败");
  }
}

const resetPasswordForm = () => {
  passwordForm.currentPassword = "";
  passwordForm.newPassword = "";
  passwordForm.confirmPassword = "";
}

const savePassword = async () => {
  message.value = "";
  if (passwordForm.newPassword !== passwordForm.confirmPassword) {
    showError(null, "两次输入的新密码不一致");
    return;
  }
  try {
    await changePassword(passwordForm.currentPassword, passwordForm.newPassword);
    resetPasswordForm();
    showSuccess("管理员密码已更新");
  } catch (error) {
    showError(error, "修改密码失败");
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
        <h2>管理员密码</h2>
        <form class="password-form" @submit.prevent="savePassword">
          <input v-model="passwordForm.currentPassword" autocomplete="current-password" placeholder="当前密码" type="password" required>
          <input v-model="passwordForm.newPassword" autocomplete="new-password" minlength="8" placeholder="新密码" type="password" required>
          <input v-model="passwordForm.confirmPassword" autocomplete="new-password" minlength="8" placeholder="确认新密码" type="password" required>
          <button class="primary-button" type="submit">更新密码</button>
        </form>
      </section>

      <section class="panel appearance-panel">
        <h2>外观偏好</h2>
        <div class="preference-grid">
          <div class="preference-row">
            <span>亮暗模式</span>
            <div class="segmented-control" role="group" aria-label="亮暗模式">
              <button
                  v-for="option in colorModeOptions"
                  :key="option.value"
                  type="button"
                  :class="{active: appearanceStore.colorMode === option.value}"
                  @click="appearanceStore.setColorMode(option.value)">
                {{ option.label }}
              </button>
            </div>
          </div>
          <div class="preference-row">
            <span>界面图标</span>
            <div class="segmented-control" role="group" aria-label="界面图标">
              <button
                  v-for="option in iconStyleOptions"
                  :key="option.value"
                  type="button"
                  :class="{active: appearanceStore.iconStyle === option.value}"
                  @click="appearanceStore.setIconStyle(option.value)">
                {{ option.label }}
              </button>
            </div>
          </div>
          <div class="preference-row">
            <span>文件图标颜色</span>
            <div class="segmented-control" role="group" aria-label="文件图标颜色">
              <button
                  v-for="option in fileIconPaletteOptions"
                  :key="option.value"
                  type="button"
                  :class="{active: appearanceStore.fileIconPalette === option.value}"
                  @click="appearanceStore.setFileIconPalette(option.value)">
                {{ option.label }}
              </button>
            </div>
          </div>
          <div class="preference-row">
            <span>主题色</span>
            <div class="color-swatches" aria-label="主题色">
              <button
                  v-for="option in accentColorOptions"
                  :key="option.value"
                  type="button"
                  class="color-swatch"
                  :class="{active: appearanceStore.accentColor === option.value}"
                  :title="option.label"
                  :aria-label="option.label"
                  :style="{backgroundColor: option.color}"
                  @click="appearanceStore.setAccentColor(option.value)">
              </button>
            </div>
          </div>
        </div>
      </section>

      <section class="panel metrics-panel">
        <div class="section-heading">
          <h2>运行状态</h2>
          <button class="plain-button" :disabled="metricsLoading" @click="loadMetrics(true)">刷新状态</button>
        </div>
        <div v-if="metricsLoading && !metrics" class="empty">正在加载运行状态...</div>
        <template v-else>
          <div class="metrics-summary">
            <div class="metric-tile">
              <span>挂载</span>
              <strong>{{ countText(metrics?.mappings) }}</strong>
            </div>
            <div class="metric-tile">
              <span>会话</span>
              <strong>{{ countText(metrics?.activeSessions) }}</strong>
            </div>
            <div class="metric-tile">
              <span>回收站</span>
              <strong>{{ countText(metrics?.trashEntries) }}</strong>
            </div>
            <div class="metric-tile">
              <span>后台任务</span>
              <strong>{{ countText(taskMetrics?.total) }}</strong>
            </div>
          </div>

          <div class="metrics-grid">
            <section class="metric-section" aria-label="任务统计">
              <h3>任务统计</h3>
              <dl class="compact-meta">
                <div><dt>运行 / 排队</dt><dd>{{ countText(taskMetrics?.running) }} / {{ countText(taskMetrics?.queued) }}</dd></div>
                <div><dt>完成 / 失败 / 取消</dt><dd>{{ countText(taskMetrics?.completed) }} / {{ countText(taskMetrics?.failed) }} / {{ countText(taskMetrics?.cancelled) }}</dd></div>
                <div><dt>错误数</dt><dd>{{ countText(taskMetrics?.errorsTotal) }}</dd></div>
                <div><dt>已处理</dt><dd>{{ bytesText(taskMetrics?.processedBytes) }}</dd></div>
                <div><dt>当前速度</dt><dd>{{ speedText(taskMetrics?.currentSpeedBytesPerSec) }}</dd></div>
              </dl>
            </section>

            <section class="metric-section" aria-label="并发占用">
              <h3>并发占用</h3>
              <div class="limit-list">
                <div class="limit-row">
                  <div>
                    <span>目录扫描</span>
                    <strong>{{ limitUsageText(limitMetrics?.activeDirScans, limitMetrics?.dirScanLimit) }}</strong>
                  </div>
                  <span class="limit-bar"><span :style="{width: limitUsageRatio(limitMetrics?.activeDirScans, limitMetrics?.dirScanLimit)}"></span></span>
                </div>
                <div class="limit-row">
                  <div>
                    <span>文件传输</span>
                    <strong>{{ limitUsageText(limitMetrics?.activeTransfers, limitMetrics?.transferLimit) }}</strong>
                  </div>
                  <span class="limit-bar"><span :style="{width: limitUsageRatio(limitMetrics?.activeTransfers, limitMetrics?.transferLimit)}"></span></span>
                </div>
                <div class="limit-row">
                  <div>
                    <span>IP 请求</span>
                    <strong>{{ limitUsageText(limitMetrics?.activeIpRequests, limitMetrics?.ipLimit) }}</strong>
                  </div>
                  <span class="limit-bar"><span :style="{width: limitUsageRatio(limitMetrics?.activeIpRequests, limitMetrics?.ipLimit)}"></span></span>
                </div>
                <div class="limit-row tracked">
                  <div>
                    <span>已跟踪 IP</span>
                    <strong>{{ countText(limitMetrics?.trackedIps) }}</strong>
                  </div>
                </div>
              </div>
            </section>
          </div>
        </template>
      </section>

      <section class="panel index-panel">
        <div class="section-heading">
          <h2>搜索索引</h2>
          <div class="panel-actions">
            <button class="plain-button" :disabled="indexBusy" @click="loadIndexStatus(true)">刷新状态</button>
            <button class="primary-button" :disabled="!canRebuildIndex" @click="requestIndexRebuild">重建索引</button>
            <button v-if="indexBuilding" class="danger-button" :disabled="!canCancelIndex" @click="requestIndexCancel">取消重建</button>
          </div>
        </div>
        <div class="index-status-line">
          <span class="index-state" :class="indexStateClass(indexStatus)">{{ indexLoading ? "读取中" : indexStateText(indexStatus) }}</span>
          <span>已索引 {{ indexEntryCountText(indexStatus?.indexedEntries) }}</span>
        </div>
        <dl class="index-meta">
          <div><dt>上次开始</dt><dd>{{ optionalDateText(indexStatus?.lastStartedAt) }}</dd></div>
          <div><dt>上次完成</dt><dd>{{ optionalDateText(indexStatus?.lastFinishedAt) }}</dd></div>
          <div v-if="indexStatus?.lastError"><dt>错误</dt><dd>{{ indexStatus.lastError }}</dd></div>
        </dl>
      </section>

      <section class="panel" v-if="settings">
        <h2>服务配置</h2>
        <dl>
          <div><dt>监听地址</dt><dd>{{ settings.bindAddress }}:{{ settings.port }}</dd></div>
          <div><dt>映射文件</dt><dd>{{ settings.mappingFile }}</dd></div>
          <div><dt>配置文件</dt><dd>{{ settings.configFile }}</dd></div>
          <div><dt>回收站</dt><dd>{{ settings.trashDir }}</dd></div>
          <div><dt>静态目录</dt><dd>{{ settings.staticDir }}</dd></div>
          <div><dt>CORS 来源</dt><dd>{{ corsOriginsText(settings.corsAllowedOrigins) }}</dd></div>
          <div><dt>信任代理来源头</dt><dd>{{ settings.trustProxyHeaders ? "已启用" : "未启用" }}</dd></div>
          <div><dt>编辑上限</dt><dd>{{ bytesText(settings.maxEditBytes) }}</dd></div>
          <div><dt>可编辑扩展名</dt><dd>{{ listText(settings.editableExtensions) }}</dd></div>
          <div><dt>可编辑 MIME</dt><dd>{{ listText(settings.editableMimeTypes) }}</dd></div>
          <div><dt>上传上限</dt><dd>{{ optionalBytesText(settings.maxUploadBytes) }}</dd></div>
          <div><dt>目录分页上限</dt><dd>{{ settings.maxDirPageSize }}</dd></div>
          <div><dt>目录并发</dt><dd>{{ settings.maxDirConcurrency }}</dd></div>
          <div><dt>传输并发</dt><dd>{{ settings.maxTransferConcurrency }}</dd></div>
          <div><dt>IP 并发</dt><dd>{{ settings.maxIpConcurrency }}</dd></div>
          <div><dt>任务并发</dt><dd>{{ settings.maxTaskConcurrency }}</dd></div>
          <div><dt>任务历史</dt><dd>最近 {{ settings.taskHistoryLimit }} 条已结束任务</dd></div>
          <div><dt>任务限速</dt><dd>{{ settings.taskSpeedLimitBytesPerSec ? speedText(settings.taskSpeedLimitBytesPerSec) : "不限制" }}</dd></div>
          <div><dt>解压字节上限</dt><dd>{{ optionalBytesText(settings.maxExtractBytes) }}</dd></div>
          <div><dt>解压条目上限</dt><dd>{{ settings.maxExtractFiles ? settings.maxExtractFiles : "不限制" }}</dd></div>
          <div><dt>搜索索引</dt><dd>{{ settings.indexEnabled ? "已启用" : "未启用" }}</dd></div>
          <div><dt>启动重建索引</dt><dd>{{ settings.indexRebuildOnStartup ? "启用" : "关闭" }}</dd></div>
          <div><dt>审计日志</dt><dd>{{ settings.auditFile }}</dd></div>
          <div><dt>审计轮转</dt><dd>{{ optionalBytesText(settings.auditMaxBytes) }}</dd></div>
          <div><dt>审计保留</dt><dd>最近 {{ settings.auditRetentionFiles }} 个轮转文件</dd></div>
          <div><dt>回收站保留</dt><dd>{{ settings.trashRetentionDays ? `${settings.trashRetentionDays} 天` : "不限制" }}</dd></div>
          <div><dt>回收站容量</dt><dd>{{ optionalBytesText(settings.trashMaxBytes) }}</dd></div>
          <div><dt>冲突策略</dt><dd>{{ conflictPolicyText(settings.conflictPolicy) }}</dd></div>
          <div><dt>认证</dt><dd>{{ settings.authConfigured ? "已初始化" : "未初始化" }}</dd></div>
        </dl>
      </section>

      <div v-if="message" class="message" :class="{success: messageKind === 'success'}">{{ message }}</div>
    </main>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";
.settings-page {
  @apply min-h-screen;
  background: var(--app-bg);
  color: var(--app-text);
}

header {
  @apply h-14 border-b px-4 flex items-center justify-between;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

h1 {
  @apply text-xl font-semibold
}

main {
  @apply p-4 flex flex-col gap-4
}

.panel {
  @apply border rounded-lg p-4 flex flex-col gap-4;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

h2 {
  @apply text-base font-semibold
}

.section-heading {
  @apply flex flex-wrap items-center justify-between gap-3;
}

.panel-actions {
  @apply flex flex-wrap items-center gap-2;
}

.mapping-form,
.mapping-row,
.password-form {
  @apply grid gap-2 items-center
}

.mapping-form {
  grid-template-columns: 1fr 2fr 1fr 90px 90px 90px;
}

.mapping-row {
  grid-template-columns: 1fr 2fr 1fr 90px 90px 80px 80px;
}

.password-form {
  grid-template-columns: repeat(3, minmax(0, 1fr)) 100px;
}

.preference-grid {
  @apply grid gap-3;
}

.preference-row {
  @apply grid items-center gap-3;
  grid-template-columns: 9rem minmax(0, 1fr);
}

.preference-row > span {
  @apply text-sm font-medium;
  color: var(--app-text-subtle);
}

.segmented-control {
  @apply inline-flex w-fit rounded-md border p-0.5;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.segmented-control button {
  @apply h-8 rounded px-3 text-sm font-medium;
  color: var(--app-text-subtle);
}

.segmented-control button:hover {
  background: var(--app-control-hover);
}

.segmented-control button.active {
  background: var(--app-accent, #2563eb);
  @apply shadow-sm;
  color: var(--app-accent-contrast);
}

.color-swatches {
  @apply flex items-center gap-2;
}

.color-swatch {
  @apply h-7 w-7 rounded-full border-2 shadow ring-1 transition;
  border-color: var(--app-panel-solid);
  --tw-ring-color: var(--app-border);
}

.color-swatch.active {
  box-shadow: 0 0 0 3px var(--app-accent-soft, #eff6ff), 0 0 0 5px var(--app-accent-border, #bfdbfe);
}

.metrics-summary {
  @apply grid overflow-hidden rounded-md border;
  border-color: var(--app-border-soft);
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.metric-tile {
  @apply flex min-w-0 items-center justify-between gap-3 border-r px-3 py-2;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.metric-tile:last-child {
  @apply border-r-0;
}

.metric-tile span,
.limit-row span {
  @apply truncate text-xs;
  color: var(--app-text-subtle);
}

.metric-tile strong {
  @apply shrink-0 text-base font-semibold;
  color: var(--app-text);
}

.metrics-grid {
  @apply grid gap-4;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
}

.metric-section {
  @apply min-w-0 rounded-md border p-3;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.metric-section h3 {
  @apply mb-3 text-sm font-semibold;
  color: var(--app-text);
}

.compact-meta {
  @apply gap-1;
}

.compact-meta div {
  @apply py-0.5;
  grid-template-columns: 7rem minmax(0, 1fr);
}

.limit-list {
  @apply flex flex-col gap-3;
}

.limit-row {
  @apply grid gap-1.5;
}

.limit-row > div {
  @apply flex min-w-0 items-center justify-between gap-3;
}

.limit-row strong {
  @apply shrink-0 text-sm font-semibold;
  color: var(--app-text);
}

.limit-bar {
  @apply block h-1.5 overflow-hidden rounded-full;
  background: var(--app-control-solid);
}

.limit-bar span {
  @apply block h-full rounded-full transition-[width];
  background: var(--app-accent, #2563eb);
}

.limit-row.tracked {
  @apply pt-1;
}

.index-status-line {
  @apply flex flex-wrap items-center gap-3 text-sm;
  color: var(--app-text-muted);
}

.index-state {
  @apply inline-flex h-7 items-center rounded-full px-3 text-xs font-semibold;
  background: var(--app-panel-muted);
  color: var(--app-text-muted);
}

.index-state.idle {
  background: var(--app-success-soft);
  color: var(--app-success-text);
}

.index-state.building {
  background: var(--app-accent-soft);
  color: var(--app-accent);
}

.index-state.error {
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.index-state.disabled {
  background: var(--app-panel-muted);
  color: var(--app-text-subtle);
}

input {
  @apply h-9 min-w-0 rounded-md border px-2 text-sm outline-none;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text);
}

input:focus {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.check-field {
  @apply h-9 flex items-center gap-2 text-sm;
  color: var(--app-text-muted);
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
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.primary-button:hover {
  background: var(--app-accent-strong);
}

.plain-button,
.ghost-button {
  background: var(--app-panel-muted);
  color: var(--app-text-muted);
}

.plain-button:hover,
.ghost-button:hover {
  background: var(--app-control-hover);
}

.danger-button {
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.danger-button:hover {
  background: color-mix(in srgb, var(--app-danger) 16%, var(--app-panel-solid));
}

.primary-button:disabled,
.plain-button:disabled,
.danger-button:disabled,
.ghost-button:disabled {
  @apply cursor-not-allowed opacity-50;
}

.empty {
  @apply rounded-md border border-dashed px-3 py-6 text-center text-sm;
  border-color: var(--app-border-soft);
  color: var(--app-text-subtle);
}

dl {
  @apply grid gap-2 text-sm
}

.index-meta {
  @apply grid gap-2 text-sm;
}

dl div {
  @apply grid gap-3;
  grid-template-columns: 8rem minmax(0, 1fr);
}

dt {
  color: var(--app-text-subtle);
}

dd {
  @apply min-w-0 break-all
}

.message {
  @apply rounded-md border px-3 py-2 text-sm;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.message.success {
  border-color: var(--app-success-border);
  background: var(--app-success-soft);
  color: var(--app-success-text);
}

@media (max-width: 900px) {
  .mapping-form,
  .mapping-row,
  .password-form,
  .preference-row,
  .metrics-summary,
  .metrics-grid {
    grid-template-columns: 1fr;
  }

  .metric-tile {
    @apply border-r-0 border-b;
  }

  .metric-tile:last-child {
    @apply border-b-0;
  }
}
</style>
