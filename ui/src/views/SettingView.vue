<script setup lang="ts">
import {computed, nextTick, onMounted, reactive, ref} from "vue";
import {useRouter} from "vue-router";
import type {
  HealthResponse,
  IndexStatus,
  MetricsResponse,
  PathMapping,
  ReadinessResponse,
  RuntimeSettings,
  RuntimeSettingsPatch,
  SettingsResponse,
  StartupSettings,
  StartupSettingsPatch
} from "../class";
import {
  cancelIndexRebuild,
  changePassword,
  cleanupAudit,
  createMapping,
  deleteMapping,
  getHealth,
  getIndexStatus,
  getMappings,
  getMetrics,
  getReadiness,
  getSettings,
  rebuildIndex,
  reloadSettings,
  reorderMappings,
  updateMapping,
  updateSettings
} from "../network/api";
import Icon from "../components/Icon.vue";
import OperationPanelShell from "../components/operations/OperationPanelShell.vue";
import ShellNotice from "../components/shell/ShellNotice.vue";
import {useShellNotice} from "../composables/useShellNotice.ts";
import {formatEntryDate} from "../utils/file-entry.ts";

type MappingDialogMode = "create" | "edit";

interface RuntimeSettingsForm {
  authSessionTtlSeconds: string;
  authSecureCookie: boolean;
  maxEditBytes: string;
  editableExtensions: string;
  editableMimeTypes: string;
  maxUploadBytes: string;
  maxDirPageSize: string;
  maxDirConcurrency: string;
  maxTransferConcurrency: string;
  maxIpConcurrency: string;
  maxTaskConcurrency: string;
  taskHistoryLimit: string;
  taskSpeedLimitBytesPerSec: string;
  maxArchiveBytes: string;
  maxArchiveFiles: string;
  maxExtractBytes: string;
  maxExtractFiles: string;
  maxExtractDepth: string;
  indexEnabled: boolean;
  indexScanDelayMs: string;
  auditEnabled: boolean;
  auditMaxBytes: string;
  auditRetentionFiles: string;
  trashRetentionDays: string;
  trashMaxBytes: string;
  conflictPolicy: RuntimeSettings["conflictPolicy"];
}

interface StartupSettingsForm {
  bindAddress: string;
  port: string;
  mappingFile: string;
  configFile: string;
  authFile: string;
  favoritesFile: string;
  trashDir: string;
  staticDir: string;
  corsAllowedOrigins: string;
  trustProxyHeaders: boolean;
  auditFile: string;
  indexRebuildOnStartup: boolean;
}

const router = useRouter();
const {
  notice: shellNotice,
  show: showShellNotice,
  showError: showErrorNotice,
  close: closeShellNotice,
  stopTimer: stopShellNoticeTimer,
  resumeTimer: resumeShellNoticeTimer
} = useShellNotice();

const mappings = ref<PathMapping[]>([]);
const settingsSnapshot = ref<SettingsResponse | null>(null);
const indexStatus = ref<IndexStatus | null>(null);
const metrics = ref<MetricsResponse | null>(null);
const health = ref<HealthResponse | null>(null);
const readiness = ref<ReadinessResponse | null>(null);
const loading = ref(false);
const saving = ref(false);
const reloadingSettings = ref(false);
const indexLoading = ref(false);
const indexActionLoading = ref(false);
const metricsLoading = ref(false);
const auditCleanupLoading = ref(false);
const passwordSaving = ref(false);
const mappingSavingId = ref<number | null>(null);
const mappingRefreshing = ref(false);
const mappingReorderLoading = ref(false);
const mappingDialogOpen = ref(false);
const mappingDialogMode = ref<MappingDialogMode>("create");
const mappingDeleteTarget = ref<PathMapping | null>(null);
const draggingMappingId = ref<number | null>(null);
const dragOverMappingId = ref<number | null>(null);
const dragOverPlacement = ref<"before" | "after">("before");
const mappingMountInputRef = ref<HTMLInputElement | null>(null);

const mappingForm = reactive<PathMapping>({
  mountPath: "",
  folderPath: "",
  remark: "",
  order: 0,
  writable: true
});

const passwordForm = reactive({
  currentPassword: "",
  newPassword: "",
  confirmPassword: ""
});

const runtimeForm = reactive<RuntimeSettingsForm>({
  authSessionTtlSeconds: "",
  authSecureCookie: false,
  maxEditBytes: "",
  editableExtensions: "",
  editableMimeTypes: "",
  maxUploadBytes: "",
  maxDirPageSize: "",
  maxDirConcurrency: "",
  maxTransferConcurrency: "",
  maxIpConcurrency: "",
  maxTaskConcurrency: "",
  taskHistoryLimit: "",
  taskSpeedLimitBytesPerSec: "",
  maxArchiveBytes: "",
  maxArchiveFiles: "",
  maxExtractBytes: "",
  maxExtractFiles: "",
  maxExtractDepth: "",
  indexEnabled: false,
  indexScanDelayMs: "",
  auditEnabled: true,
  auditMaxBytes: "",
  auditRetentionFiles: "",
  trashRetentionDays: "",
  trashMaxBytes: "",
  conflictPolicy: "autoRename"
});

const startupForm = reactive<StartupSettingsForm>({
  bindAddress: "",
  port: "",
  mappingFile: "",
  configFile: "",
  authFile: "",
  favoritesFile: "",
  trashDir: "",
  staticDir: "",
  corsAllowedOrigins: "",
  trustProxyHeaders: false,
  auditFile: "",
  indexRebuildOnStartup: false
});

const navItems = [
  {id: "overview", label: "概览", icon: "action.properties"},
  {id: "mappings", label: "挂载目录", icon: "file.folder"},
  {id: "index", label: "索引与安全", icon: "action.search"},
  {id: "config", label: "系统配置", icon: "action.tools"}
] as const;

type SettingsPageId = typeof navItems[number]["id"];

const conflictPolicyOptions: {value: RuntimeSettings["conflictPolicy"]; label: string; description: string}[] = [
  {value: "autoRename", label: "自动重命名", description: "发生冲突时自动追加序号"},
  {value: "reject", label: "拒绝", description: "冲突时取消本次写入"},
  {value: "overwrite", label: "覆盖", description: "冲突时覆盖已有内容"}
];

const startupFieldLabels: Record<keyof StartupSettings, string> = {
  bindAddress: "监听地址",
  port: "端口",
  mappingFile: "挂载配置文件",
  configFile: "配置文件",
  authFile: "认证文件",
  favoritesFile: "收藏文件",
  trashDir: "回收站目录",
  staticDir: "静态文件目录",
  corsAllowedOrigins: "CORS 来源",
  trustProxyHeaders: "信任代理来源头",
  auditFile: "审计日志文件",
  indexRebuildOnStartup: "启动时重建索引"
};

const readinessCheckLabels: Record<string, string> = {
  auth: "管理员认证",
  configStore: "运行配置",
  authStore: "认证文件",
  favoritesStore: "收藏文件",
  mappingStore: "挂载配置",
  trash: "回收站目录",
  audit: "审计日志",
  staticFiles: "静态资源"
};

const runtimeSettings = computed(() => settingsSnapshot.value?.runtime ?? null);
const startupSettings = computed(() => settingsSnapshot.value?.startup ?? null);
const activeStartupSettings = computed(() => settingsSnapshot.value?.activeStartup ?? null);
const activeSettingsPage = ref<SettingsPageId>("overview");
const activeNavItem = computed(() => navItems.find(item => item.id === activeSettingsPage.value) ?? navItems[0]);
const envLockedSet = computed(() => new Set(settingsSnapshot.value?.envLocked ?? []));
const restartPendingSet = computed(() => new Set(settingsSnapshot.value?.restartPendingFields ?? []));
const taskMetrics = computed(() => metrics.value?.tasks);
const limitMetrics = computed(() => metrics.value?.limits);
const readinessOkCount = computed(() => readiness.value?.checks.filter(check => check.status === "ok").length ?? 0);
const readinessIssueCount = computed(() => readiness.value?.checks.filter(check => check.status !== "ok").length ?? 0);
const indexBusy = computed(() => indexLoading.value || indexActionLoading.value);
const indexBuilding = computed(() => indexStatus.value?.state === "scanning" || indexStatus.value?.state === "building");
const canRebuildIndex = computed(() => Boolean(indexStatus.value?.enabled) && !indexBusy.value && !indexBuilding.value);
const canCancelIndex = computed(() => Boolean(indexStatus.value?.enabled) && !indexBusy.value && indexBuilding.value);
const runtimeDirty = computed(() => {
  const runtime = runtimeSettings.value;
  if (!runtime) return false;
  return runtimeFormSignature() !== runtimeSettingsSignature(runtime);
});
const startupDirty = computed(() => {
  const startup = startupSettings.value;
  if (!startup) return false;
  return startupFormSignature() !== startupSettingsSignature(startup);
});
const settingsDirty = computed(() => runtimeDirty.value || startupDirty.value);
const canSaveSettings = computed(() => Boolean(settingsSnapshot.value) && settingsDirty.value && !saving.value);
const sortedMappings = computed(() => [...mappings.value].sort((left, right) => {
  const orderDelta = (left.order ?? 0) - (right.order ?? 0);
  if (orderDelta !== 0) return orderDelta;
  return left.mountPath.localeCompare(right.mountPath, "zh-Hans-CN");
}));
const writableMappingCount = computed(() => mappings.value.filter(mapping => mapping.writable).length);
const readonlyMappingCount = computed(() => mappings.value.length - writableMappingCount.value);
const nextMappingOrder = computed(() => {
  if (!mappings.value.length) return 10;
  return Math.max(...mappings.value.map(mapping => mapping.order ?? 0)) + 10;
});
const mappingBusy = computed(() => loading.value || mappingRefreshing.value || mappingReorderLoading.value || mappingSavingId.value != null);
const mappingDialogTitle = computed(() => mappingDialogMode.value === "edit" ? "修改挂载" : "添加挂载");
const mappingDialogSubtitle = computed(() => mappingDialogMode.value === "edit"
    ? "调整虚拟路径、目录和读写权限。"
    : "把本地或容器目录显示为一个虚拟路径。");
const mappingDialogSubmitText = computed(() => mappingDialogMode.value === "edit" ? "保存修改" : "添加挂载");

const selectSettingsPage = (id: SettingsPageId) => {
  activeSettingsPage.value = id;
}

const load = async () => {
  loading.value = true;
  closeShellNotice();
  try {
    const [mappingData, settingData] = await Promise.all([getMappings(), getSettings()]);
    mappings.value = mappingData;
    if (!mappingForm.mountPath && !mappingForm.folderPath && !mappingForm.remark && mappingForm.order === 0) {
      mappingForm.order = nextMappingOrder.value;
    }
    applySettingsSnapshot(settingData);
    await Promise.all([loadIndexStatus(false), loadMetrics(false)]);
  } catch (error) {
    showError(error, "加载设置失败");
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const applySettingsSnapshot = (snapshot: SettingsResponse) => {
  settingsSnapshot.value = snapshot;
  syncRuntimeForm(snapshot.runtime);
  syncStartupForm(snapshot.startup);
}

const syncRuntimeForm = (runtime: RuntimeSettings) => {
  runtimeForm.authSessionTtlSeconds = numberText(runtime.authSessionTtlSeconds);
  runtimeForm.authSecureCookie = runtime.authSecureCookie;
  runtimeForm.maxEditBytes = numberText(runtime.maxEditBytes);
  runtimeForm.editableExtensions = listInputText(runtime.editableExtensions);
  runtimeForm.editableMimeTypes = listInputText(runtime.editableMimeTypes);
  runtimeForm.maxUploadBytes = optionalNumberText(runtime.maxUploadBytes);
  runtimeForm.maxDirPageSize = numberText(runtime.maxDirPageSize);
  runtimeForm.maxDirConcurrency = numberText(runtime.maxDirConcurrency);
  runtimeForm.maxTransferConcurrency = numberText(runtime.maxTransferConcurrency);
  runtimeForm.maxIpConcurrency = numberText(runtime.maxIpConcurrency);
  runtimeForm.maxTaskConcurrency = numberText(runtime.maxTaskConcurrency);
  runtimeForm.taskHistoryLimit = numberText(runtime.taskHistoryLimit);
  runtimeForm.taskSpeedLimitBytesPerSec = optionalNumberText(runtime.taskSpeedLimitBytesPerSec);
  runtimeForm.maxArchiveBytes = optionalNumberText(runtime.maxArchiveBytes);
  runtimeForm.maxArchiveFiles = optionalNumberText(runtime.maxArchiveFiles);
  runtimeForm.maxExtractBytes = optionalNumberText(runtime.maxExtractBytes);
  runtimeForm.maxExtractFiles = optionalNumberText(runtime.maxExtractFiles);
  runtimeForm.maxExtractDepth = numberText(runtime.maxExtractDepth);
  runtimeForm.indexEnabled = runtime.indexEnabled;
  runtimeForm.indexScanDelayMs = numberText(runtime.indexScanDelayMs);
  runtimeForm.auditEnabled = runtime.auditEnabled;
  runtimeForm.auditMaxBytes = optionalNumberText(runtime.auditMaxBytes);
  runtimeForm.auditRetentionFiles = numberText(runtime.auditRetentionFiles);
  runtimeForm.trashRetentionDays = optionalNumberText(runtime.trashRetentionDays);
  runtimeForm.trashMaxBytes = optionalNumberText(runtime.trashMaxBytes);
  runtimeForm.conflictPolicy = runtime.conflictPolicy;
}

const syncStartupForm = (startup: StartupSettings) => {
  startupForm.bindAddress = startup.bindAddress;
  startupForm.port = numberText(startup.port);
  startupForm.mappingFile = startup.mappingFile;
  startupForm.configFile = startup.configFile;
  startupForm.authFile = startup.authFile;
  startupForm.favoritesFile = startup.favoritesFile;
  startupForm.trashDir = startup.trashDir;
  startupForm.staticDir = startup.staticDir;
  startupForm.corsAllowedOrigins = listInputText(startup.corsAllowedOrigins);
  startupForm.trustProxyHeaders = startup.trustProxyHeaders;
  startupForm.auditFile = startup.auditFile;
  startupForm.indexRebuildOnStartup = startup.indexRebuildOnStartup;
}

const resetMappingForm = () => {
  mappingForm.id = undefined;
  mappingForm.mountPath = "";
  mappingForm.folderPath = "";
  mappingForm.remark = "";
  mappingForm.order = nextMappingOrder.value;
  mappingForm.writable = true;
}

const focusMappingDialogInput = (selectText = false) => {
  void nextTick(() => {
    mappingMountInputRef.value?.focus();
    if (selectText) mappingMountInputRef.value?.select();
  });
}

const openMappingDialog = () => {
  resetMappingForm();
  mappingDialogMode.value = "create";
  mappingDialogOpen.value = true;
  focusMappingDialogInput();
}

const openEditMappingDialog = (mapping: PathMapping) => {
  mappingForm.id = mapping.id;
  mappingForm.mountPath = mapping.mountPath;
  mappingForm.folderPath = mapping.folderPath;
  mappingForm.remark = mapping.remark ?? "";
  mappingForm.order = mapping.order ?? 0;
  mappingForm.writable = mapping.writable;
  mappingDialogMode.value = "edit";
  mappingDialogOpen.value = true;
  focusMappingDialogInput(true);
}

const closeMappingDialog = () => {
  if (mappingBusy.value) return;
  mappingDialogOpen.value = false;
}

const openMappingDeleteConfirm = (mapping: PathMapping) => {
  if (mappingBusy.value) return;
  mappingDeleteTarget.value = mapping;
}

const closeMappingDeleteConfirm = () => {
  if (mappingBusy.value) return;
  mappingDeleteTarget.value = null;
}

const resetPasswordForm = () => {
  passwordForm.currentPassword = "";
  passwordForm.newPassword = "";
  passwordForm.confirmPassword = "";
}

const showError = (error: unknown, fallback: string) => {
  showErrorNotice(error, fallback);
}

const showSuccess = (text: string) => {
  showShellNotice(text, "success");
}

const showWarning = (text: string) => {
  showShellNotice(text, "warning");
}

const formText = (value: unknown) => value == null ? "" : String(value);
const trimFormText = (value: unknown) => formText(value).trim();
const numberText = (value?: number | null) => value == null ? "" : String(value);
const optionalNumberText = (value?: number | null) => value == null ? "" : String(value);
const listInputText = (values: string[]) => values.join("\n");
const parseListInput = (value: unknown) => formText(value)
    .split(/[\n,，]/)
    .map(item => item.trim())
    .filter(Boolean);
const normalizeMountPath = (value: string) => {
  const normalized = value.trim().replace(/\\/g, "/").replace(/\/{2,}/g, "/");
  if (!normalized) return "";
  return normalized.startsWith("/") ? normalized : `/${normalized}`;
}
const buildMappingPayload = (mapping: PathMapping): PathMapping => {
  const order = Number(mapping.order ?? 0);
  const payload: PathMapping = {
    mountPath: normalizeMountPath(mapping.mountPath),
    folderPath: mapping.folderPath.trim(),
    remark: mapping.remark?.trim() ?? "",
    order: Number.isFinite(order) ? order : 0,
    writable: Boolean(mapping.writable)
  };
  if (mapping.id != null) payload.id = mapping.id;
  if (!payload.mountPath) throw new Error("需要填写虚拟路径");
  if (!payload.folderPath) throw new Error("需要填写本地或容器目录");
  return payload;
}

const parseInteger = (value: unknown, label: string, min = 1, max = Number.MAX_SAFE_INTEGER) => {
  const text = trimFormText(value);
  const number = Number(text);
  if (!text || !Number.isInteger(number) || number < min || number > max) {
    throw new Error(`${label}需要填写 ${min}${max === Number.MAX_SAFE_INTEGER ? " 以上" : `-${max}`} 的整数`);
  }
  return number;
}

const parseOptionalInteger = (value: unknown, label: string, min = 1) => {
  const text = trimFormText(value);
  if (!text) return null;
  return parseInteger(text, label, min);
}

const arraysEqual = (left: string[], right: string[]) => left.length === right.length && left.every((item, index) => item === right[index]);
const isRuntimeLocked = (field: keyof RuntimeSettingsPatch) => envLockedSet.value.has(`runtime.${String(field)}`);
const isStartupLocked = (field: keyof StartupSettingsPatch) => envLockedSet.value.has(`startup.${String(field)}`);
const isStartupPending = (field: keyof StartupSettings) => restartPendingSet.value.has(`startup.${String(field)}`);

const runtimeFormSignature = () => JSON.stringify({
  authSessionTtlSeconds: trimFormText(runtimeForm.authSessionTtlSeconds),
  authSecureCookie: runtimeForm.authSecureCookie,
  maxEditBytes: trimFormText(runtimeForm.maxEditBytes),
  editableExtensions: parseListInput(runtimeForm.editableExtensions),
  editableMimeTypes: parseListInput(runtimeForm.editableMimeTypes),
  maxUploadBytes: trimFormText(runtimeForm.maxUploadBytes),
  maxDirPageSize: trimFormText(runtimeForm.maxDirPageSize),
  maxDirConcurrency: trimFormText(runtimeForm.maxDirConcurrency),
  maxTransferConcurrency: trimFormText(runtimeForm.maxTransferConcurrency),
  maxIpConcurrency: trimFormText(runtimeForm.maxIpConcurrency),
  maxTaskConcurrency: trimFormText(runtimeForm.maxTaskConcurrency),
  taskHistoryLimit: trimFormText(runtimeForm.taskHistoryLimit),
  taskSpeedLimitBytesPerSec: trimFormText(runtimeForm.taskSpeedLimitBytesPerSec),
  maxArchiveBytes: trimFormText(runtimeForm.maxArchiveBytes),
  maxArchiveFiles: trimFormText(runtimeForm.maxArchiveFiles),
  maxExtractBytes: trimFormText(runtimeForm.maxExtractBytes),
  maxExtractFiles: trimFormText(runtimeForm.maxExtractFiles),
  maxExtractDepth: trimFormText(runtimeForm.maxExtractDepth),
  indexEnabled: runtimeForm.indexEnabled,
  indexScanDelayMs: trimFormText(runtimeForm.indexScanDelayMs),
  auditEnabled: runtimeForm.auditEnabled,
  auditMaxBytes: trimFormText(runtimeForm.auditMaxBytes),
  auditRetentionFiles: trimFormText(runtimeForm.auditRetentionFiles),
  trashRetentionDays: trimFormText(runtimeForm.trashRetentionDays),
  trashMaxBytes: trimFormText(runtimeForm.trashMaxBytes),
  conflictPolicy: runtimeForm.conflictPolicy
});

const runtimeSettingsSignature = (runtime: RuntimeSettings) => JSON.stringify({
  authSessionTtlSeconds: numberText(runtime.authSessionTtlSeconds),
  authSecureCookie: runtime.authSecureCookie,
  maxEditBytes: numberText(runtime.maxEditBytes),
  editableExtensions: runtime.editableExtensions,
  editableMimeTypes: runtime.editableMimeTypes,
  maxUploadBytes: optionalNumberText(runtime.maxUploadBytes),
  maxDirPageSize: numberText(runtime.maxDirPageSize),
  maxDirConcurrency: numberText(runtime.maxDirConcurrency),
  maxTransferConcurrency: numberText(runtime.maxTransferConcurrency),
  maxIpConcurrency: numberText(runtime.maxIpConcurrency),
  maxTaskConcurrency: numberText(runtime.maxTaskConcurrency),
  taskHistoryLimit: numberText(runtime.taskHistoryLimit),
  taskSpeedLimitBytesPerSec: optionalNumberText(runtime.taskSpeedLimitBytesPerSec),
  maxArchiveBytes: optionalNumberText(runtime.maxArchiveBytes),
  maxArchiveFiles: optionalNumberText(runtime.maxArchiveFiles),
  maxExtractBytes: optionalNumberText(runtime.maxExtractBytes),
  maxExtractFiles: optionalNumberText(runtime.maxExtractFiles),
  maxExtractDepth: numberText(runtime.maxExtractDepth),
  indexEnabled: runtime.indexEnabled,
  indexScanDelayMs: numberText(runtime.indexScanDelayMs),
  auditEnabled: runtime.auditEnabled,
  auditMaxBytes: optionalNumberText(runtime.auditMaxBytes),
  auditRetentionFiles: numberText(runtime.auditRetentionFiles),
  trashRetentionDays: optionalNumberText(runtime.trashRetentionDays),
  trashMaxBytes: optionalNumberText(runtime.trashMaxBytes),
  conflictPolicy: runtime.conflictPolicy
});

const startupFormSignature = () => JSON.stringify({
  bindAddress: trimFormText(startupForm.bindAddress),
  port: trimFormText(startupForm.port),
  mappingFile: trimFormText(startupForm.mappingFile),
  configFile: trimFormText(startupForm.configFile),
  authFile: trimFormText(startupForm.authFile),
  favoritesFile: trimFormText(startupForm.favoritesFile),
  trashDir: trimFormText(startupForm.trashDir),
  staticDir: trimFormText(startupForm.staticDir),
  corsAllowedOrigins: parseListInput(startupForm.corsAllowedOrigins),
  trustProxyHeaders: startupForm.trustProxyHeaders,
  auditFile: trimFormText(startupForm.auditFile),
  indexRebuildOnStartup: startupForm.indexRebuildOnStartup
});

const startupSettingsSignature = (startup: StartupSettings) => JSON.stringify({
  bindAddress: startup.bindAddress,
  port: numberText(startup.port),
  mappingFile: startup.mappingFile,
  configFile: startup.configFile,
  authFile: startup.authFile,
  favoritesFile: startup.favoritesFile,
  trashDir: startup.trashDir,
  staticDir: startup.staticDir,
  corsAllowedOrigins: startup.corsAllowedOrigins,
  trustProxyHeaders: startup.trustProxyHeaders,
  auditFile: startup.auditFile,
  indexRebuildOnStartup: startup.indexRebuildOnStartup
});

const buildRuntimeDraft = (): RuntimeSettings => ({
  authSessionTtlSeconds: parseInteger(runtimeForm.authSessionTtlSeconds, "会话有效期"),
  authSecureCookie: runtimeForm.authSecureCookie,
  maxEditBytes: parseInteger(runtimeForm.maxEditBytes, "编辑上限"),
  editableExtensions: parseListInput(runtimeForm.editableExtensions),
  editableMimeTypes: parseListInput(runtimeForm.editableMimeTypes),
  maxUploadBytes: parseOptionalInteger(runtimeForm.maxUploadBytes, "上传上限"),
  maxDirPageSize: parseInteger(runtimeForm.maxDirPageSize, "目录分页上限"),
  maxDirConcurrency: parseInteger(runtimeForm.maxDirConcurrency, "目录并发"),
  maxTransferConcurrency: parseInteger(runtimeForm.maxTransferConcurrency, "传输并发"),
  maxIpConcurrency: parseInteger(runtimeForm.maxIpConcurrency, "单 IP 并发"),
  maxTaskConcurrency: parseInteger(runtimeForm.maxTaskConcurrency, "任务并发"),
  taskHistoryLimit: parseInteger(runtimeForm.taskHistoryLimit, "任务历史"),
  taskSpeedLimitBytesPerSec: parseOptionalInteger(runtimeForm.taskSpeedLimitBytesPerSec, "任务限速"),
  maxArchiveBytes: parseOptionalInteger(runtimeForm.maxArchiveBytes, "压缩输入字节上限"),
  maxArchiveFiles: parseOptionalInteger(runtimeForm.maxArchiveFiles, "压缩条目上限"),
  maxExtractBytes: parseOptionalInteger(runtimeForm.maxExtractBytes, "解压字节上限"),
  maxExtractFiles: parseOptionalInteger(runtimeForm.maxExtractFiles, "解压条目上限"),
  maxExtractDepth: parseInteger(runtimeForm.maxExtractDepth, "解压深度上限"),
  indexEnabled: runtimeForm.indexEnabled,
  indexScanDelayMs: parseInteger(runtimeForm.indexScanDelayMs, "索引扫描延迟", 0),
  auditEnabled: runtimeForm.auditEnabled,
  auditMaxBytes: parseOptionalInteger(runtimeForm.auditMaxBytes, "审计轮转大小"),
  auditRetentionFiles: parseInteger(runtimeForm.auditRetentionFiles, "审计保留数量", 0),
  trashRetentionDays: parseOptionalInteger(runtimeForm.trashRetentionDays, "回收站保留天数"),
  trashMaxBytes: parseOptionalInteger(runtimeForm.trashMaxBytes, "回收站容量上限"),
  conflictPolicy: runtimeForm.conflictPolicy
});

const buildStartupDraft = (): StartupSettings => ({
  bindAddress: trimFormText(startupForm.bindAddress),
  port: parseInteger(startupForm.port, "端口", 1, 65535),
  mappingFile: trimFormText(startupForm.mappingFile),
  configFile: trimFormText(startupForm.configFile),
  authFile: trimFormText(startupForm.authFile),
  favoritesFile: trimFormText(startupForm.favoritesFile),
  trashDir: trimFormText(startupForm.trashDir),
  staticDir: trimFormText(startupForm.staticDir),
  corsAllowedOrigins: parseListInput(startupForm.corsAllowedOrigins),
  trustProxyHeaders: startupForm.trustProxyHeaders,
  auditFile: trimFormText(startupForm.auditFile),
  indexRebuildOnStartup: startupForm.indexRebuildOnStartup
});

const buildRuntimePatch = (): RuntimeSettingsPatch => {
  if (!runtimeSettings.value) return {};
  const current = runtimeSettings.value;
  const draft = buildRuntimeDraft();
  const patch: RuntimeSettingsPatch = {};

  if (!isRuntimeLocked("authSessionTtlSeconds") && draft.authSessionTtlSeconds !== current.authSessionTtlSeconds) patch.authSessionTtlSeconds = draft.authSessionTtlSeconds;
  if (!isRuntimeLocked("authSecureCookie") && draft.authSecureCookie !== current.authSecureCookie) patch.authSecureCookie = draft.authSecureCookie;
  if (!isRuntimeLocked("maxEditBytes") && draft.maxEditBytes !== current.maxEditBytes) patch.maxEditBytes = draft.maxEditBytes;
  if (!isRuntimeLocked("editableExtensions") && !arraysEqual(draft.editableExtensions, current.editableExtensions)) patch.editableExtensions = draft.editableExtensions;
  if (!isRuntimeLocked("editableMimeTypes") && !arraysEqual(draft.editableMimeTypes, current.editableMimeTypes)) patch.editableMimeTypes = draft.editableMimeTypes;
  if (!isRuntimeLocked("maxUploadBytes") && draft.maxUploadBytes !== current.maxUploadBytes) patch.maxUploadBytes = draft.maxUploadBytes ?? null;
  if (!isRuntimeLocked("maxDirPageSize") && draft.maxDirPageSize !== current.maxDirPageSize) patch.maxDirPageSize = draft.maxDirPageSize;
  if (!isRuntimeLocked("maxDirConcurrency") && draft.maxDirConcurrency !== current.maxDirConcurrency) patch.maxDirConcurrency = draft.maxDirConcurrency;
  if (!isRuntimeLocked("maxTransferConcurrency") && draft.maxTransferConcurrency !== current.maxTransferConcurrency) patch.maxTransferConcurrency = draft.maxTransferConcurrency;
  if (!isRuntimeLocked("maxIpConcurrency") && draft.maxIpConcurrency !== current.maxIpConcurrency) patch.maxIpConcurrency = draft.maxIpConcurrency;
  if (!isRuntimeLocked("maxTaskConcurrency") && draft.maxTaskConcurrency !== current.maxTaskConcurrency) patch.maxTaskConcurrency = draft.maxTaskConcurrency;
  if (!isRuntimeLocked("taskHistoryLimit") && draft.taskHistoryLimit !== current.taskHistoryLimit) patch.taskHistoryLimit = draft.taskHistoryLimit;
  if (!isRuntimeLocked("taskSpeedLimitBytesPerSec") && draft.taskSpeedLimitBytesPerSec !== current.taskSpeedLimitBytesPerSec) patch.taskSpeedLimitBytesPerSec = draft.taskSpeedLimitBytesPerSec ?? null;
  if (!isRuntimeLocked("maxArchiveBytes") && draft.maxArchiveBytes !== current.maxArchiveBytes) patch.maxArchiveBytes = draft.maxArchiveBytes ?? null;
  if (!isRuntimeLocked("maxArchiveFiles") && draft.maxArchiveFiles !== current.maxArchiveFiles) patch.maxArchiveFiles = draft.maxArchiveFiles ?? null;
  if (!isRuntimeLocked("maxExtractBytes") && draft.maxExtractBytes !== current.maxExtractBytes) patch.maxExtractBytes = draft.maxExtractBytes ?? null;
  if (!isRuntimeLocked("maxExtractFiles") && draft.maxExtractFiles !== current.maxExtractFiles) patch.maxExtractFiles = draft.maxExtractFiles ?? null;
  if (!isRuntimeLocked("maxExtractDepth") && draft.maxExtractDepth !== current.maxExtractDepth) patch.maxExtractDepth = draft.maxExtractDepth;
  if (!isRuntimeLocked("indexEnabled") && draft.indexEnabled !== current.indexEnabled) patch.indexEnabled = draft.indexEnabled;
  if (!isRuntimeLocked("indexScanDelayMs") && draft.indexScanDelayMs !== current.indexScanDelayMs) patch.indexScanDelayMs = draft.indexScanDelayMs;
  if (!isRuntimeLocked("auditEnabled") && draft.auditEnabled !== current.auditEnabled) patch.auditEnabled = draft.auditEnabled;
  if (!isRuntimeLocked("auditMaxBytes") && draft.auditMaxBytes !== current.auditMaxBytes) patch.auditMaxBytes = draft.auditMaxBytes ?? null;
  if (!isRuntimeLocked("auditRetentionFiles") && draft.auditRetentionFiles !== current.auditRetentionFiles) patch.auditRetentionFiles = draft.auditRetentionFiles;
  if (!isRuntimeLocked("trashRetentionDays") && draft.trashRetentionDays !== current.trashRetentionDays) patch.trashRetentionDays = draft.trashRetentionDays ?? null;
  if (!isRuntimeLocked("trashMaxBytes") && draft.trashMaxBytes !== current.trashMaxBytes) patch.trashMaxBytes = draft.trashMaxBytes ?? null;
  if (!isRuntimeLocked("conflictPolicy") && draft.conflictPolicy !== current.conflictPolicy) patch.conflictPolicy = draft.conflictPolicy;

  return patch;
}

const buildStartupPatch = (): StartupSettingsPatch => {
  if (!startupSettings.value) return {};
  const current = startupSettings.value;
  const draft = buildStartupDraft();
  const patch: StartupSettingsPatch = {};

  if (!isStartupLocked("bindAddress") && draft.bindAddress !== current.bindAddress) patch.bindAddress = draft.bindAddress;
  if (!isStartupLocked("port") && draft.port !== current.port) patch.port = draft.port;
  if (!isStartupLocked("mappingFile") && draft.mappingFile !== current.mappingFile) patch.mappingFile = draft.mappingFile;
  if (!isStartupLocked("authFile") && draft.authFile !== current.authFile) patch.authFile = draft.authFile;
  if (!isStartupLocked("favoritesFile") && draft.favoritesFile !== current.favoritesFile) patch.favoritesFile = draft.favoritesFile;
  if (!isStartupLocked("trashDir") && draft.trashDir !== current.trashDir) patch.trashDir = draft.trashDir;
  if (!isStartupLocked("staticDir") && draft.staticDir !== current.staticDir) patch.staticDir = draft.staticDir;
  if (!isStartupLocked("corsAllowedOrigins") && !arraysEqual(draft.corsAllowedOrigins, current.corsAllowedOrigins)) patch.corsAllowedOrigins = draft.corsAllowedOrigins;
  if (!isStartupLocked("trustProxyHeaders") && draft.trustProxyHeaders !== current.trustProxyHeaders) patch.trustProxyHeaders = draft.trustProxyHeaders;
  if (!isStartupLocked("auditFile") && draft.auditFile !== current.auditFile) patch.auditFile = draft.auditFile;
  if (!isStartupLocked("indexRebuildOnStartup") && draft.indexRebuildOnStartup !== current.indexRebuildOnStartup) patch.indexRebuildOnStartup = draft.indexRebuildOnStartup;

  return patch;
}

const saveSettings = async () => {
  if (!canSaveSettings.value) return;
  closeShellNotice();
  saving.value = true;
  try {
    const runtime = buildRuntimePatch();
    const startup = buildStartupPatch();
    const request = {
      ...(Object.keys(runtime).length ? {runtime} : {}),
      ...(Object.keys(startup).length ? {startup} : {})
    };
    if (!Object.keys(request).length) {
      showWarning("没有需要保存的配置");
      return;
    }
    const next = await updateSettings(request);
    applySettingsSnapshot(next);
    showSuccess(next.restartPending ? "配置已保存，部分启动配置需要重启后生效" : "配置已保存");
    await Promise.all([loadIndexStatus(false), loadMetrics(false)]);
  } catch (error) {
    showError(error, "保存配置失败");
  } finally {
    saving.value = false;
  }
}

const requestReloadSettings = async () => {
  if (reloadingSettings.value) return;
  closeShellNotice();
  reloadingSettings.value = true;
  try {
    const next = await reloadSettings();
    applySettingsSnapshot(next);
    showSuccess(next.restartPending ? "已从配置文件重载，启动配置仍需重启后进入当前进程" : "已从配置文件重载");
    await Promise.all([loadIndexStatus(false), loadMetrics(false)]);
  } catch (error) {
    showError(error, "重载配置失败");
  } finally {
    reloadingSettings.value = false;
  }
}

const resetSettingsForms = () => {
  if (!settingsSnapshot.value) return;
  syncRuntimeForm(settingsSnapshot.value.runtime);
  syncStartupForm(settingsSnapshot.value.startup);
  showWarning("已撤销未保存的配置改动");
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
    const [metricsData, healthData, readinessData] = await Promise.all([
      getMetrics(),
      getHealth(),
      getReadiness()
    ]);
    metrics.value = metricsData;
    health.value = healthData;
    readiness.value = readinessData;
    if (metrics.value.index) indexStatus.value = metrics.value.index;
  } catch (error) {
    metrics.value = null;
    health.value = null;
    readiness.value = null;
    if (showFailure) showError(error, "加载运行状态失败");
  } finally {
    metricsLoading.value = false;
  }
}

const requestIndexRebuild = async () => {
  if (!canRebuildIndex.value) return;
  closeShellNotice();
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
  closeShellNotice();
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

const requestAuditCleanup = async () => {
  if (auditCleanupLoading.value) return;
  closeShellNotice();
  auditCleanupLoading.value = true;
  try {
    const result = await cleanupAudit();
    showSuccess(result.removed > 0 ? `已清理 ${result.removed} 个旧审计日志文件` : "没有需要清理的旧审计日志文件");
    await loadMetrics(false);
  } catch (error) {
    showError(error, "清理旧审计日志失败");
  } finally {
    auditCleanupLoading.value = false;
  }
}

const loadMappings = async (showResult = true) => {
  mappingRefreshing.value = true;
  try {
    mappings.value = await getMappings();
    if (!mappingForm.mountPath && !mappingForm.folderPath && !mappingForm.remark) {
      mappingForm.order = nextMappingOrder.value;
    }
    if (showResult) showSuccess("挂载目录已刷新");
  } catch (error) {
    showError(error, "刷新挂载目录失败");
  } finally {
    mappingRefreshing.value = false;
  }
}

const submitMappingDialog = async () => {
  closeShellNotice();
  const editing = mappingDialogMode.value === "edit";
  try {
    const payload = buildMappingPayload(mappingForm);
    if (editing && payload.id == null) return;
    mappingSavingId.value = payload.id ?? -1;
    if (editing) {
      await updateMapping(payload.id as number, payload);
    } else {
      await createMapping(payload);
    }
    resetMappingForm();
    mappingDialogOpen.value = false;
    await loadMappings(false);
    showSuccess(editing ? "挂载目录已保存" : "挂载目录已添加");
  } catch (error) {
    showError(error, editing ? "保存挂载失败" : "添加挂载失败");
  } finally {
    mappingSavingId.value = null;
  }
}

const confirmRemoveMapping = async () => {
  const mapping = mappingDeleteTarget.value;
  if (!mapping || mapping.id == null) return;
  closeShellNotice();
  mappingSavingId.value = mapping.id;
  try {
    await deleteMapping(mapping.id);
    mappingDeleteTarget.value = null;
    await loadMappings(false);
    showSuccess("挂载目录已删除");
  } catch (error) {
    showError(error, "删除挂载失败");
  } finally {
    mappingSavingId.value = null;
  }
}

const commitMappingOrder = async (nextMappings: PathMapping[], activeId: number) => {
  const items = nextMappings
      .filter(item => item.id != null)
      .map((item, index) => ({id: item.id as number, order: (index + 1) * 10}));

  closeShellNotice();
  mappingSavingId.value = activeId;
  mappingReorderLoading.value = true;
  try {
    await reorderMappings(items);
    await loadMappings(false);
  } catch (error) {
    showError(error, "调整挂载顺序失败");
  } finally {
    mappingSavingId.value = null;
    mappingReorderLoading.value = false;
  }
}

const handleMappingDragStart = (mapping: PathMapping, event: DragEvent) => {
  if (mapping.id == null || mappingBusy.value) {
    event.preventDefault();
    return;
  }
  draggingMappingId.value = mapping.id;
  event.dataTransfer?.setData("text/plain", String(mapping.id));
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}

const updateMappingDragPlacement = (mapping: PathMapping, event: DragEvent) => {
  if (mapping.id == null || draggingMappingId.value == null || draggingMappingId.value === mapping.id) return;
  event.preventDefault();
  const row = event.currentTarget instanceof HTMLElement ? event.currentTarget : null;
  const rect = row?.getBoundingClientRect();
  dragOverMappingId.value = mapping.id;
  dragOverPlacement.value = rect && event.clientY > rect.top + rect.height / 2 ? "after" : "before";
  if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
}

const handleMappingDrop = async (mapping: PathMapping, event: DragEvent) => {
  updateMappingDragPlacement(mapping, event);
  const sourceId = draggingMappingId.value;
  const targetId = mapping.id;
  const placement = dragOverPlacement.value;
  draggingMappingId.value = null;
  dragOverMappingId.value = null;
  if (sourceId == null || targetId == null || sourceId === targetId) return;

  const nextMappings = [...sortedMappings.value];
  const sourceIndex = nextMappings.findIndex(item => item.id === sourceId);
  const targetIndex = nextMappings.findIndex(item => item.id === targetId);
  if (sourceIndex < 0 || targetIndex < 0) return;
  const [source] = nextMappings.splice(sourceIndex, 1);
  const targetIndexAfterRemove = nextMappings.findIndex(item => item.id === targetId);
  const insertIndex = placement === "after" ? targetIndexAfterRemove + 1 : targetIndexAfterRemove;
  nextMappings.splice(insertIndex, 0, source);
  await commitMappingOrder(nextMappings, sourceId);
}

const handleMappingDragEnd = () => {
  draggingMappingId.value = null;
  dragOverMappingId.value = null;
}

const savePassword = async () => {
  if (passwordSaving.value) return;
  closeShellNotice();
  if (passwordForm.newPassword !== passwordForm.confirmPassword) {
    showError(null, "两次输入的新密码不一致");
    return;
  }
  passwordSaving.value = true;
  try {
    await changePassword(passwordForm.currentPassword, passwordForm.newPassword);
    resetPasswordForm();
    showSuccess("管理员密码已更新");
  } catch (error) {
    showError(error, "修改密码失败");
  } finally {
    passwordSaving.value = false;
  }
}

const serviceStatusText = (status?: string | null) => {
  if (!status) return "未知";
  return {
    ok: "正常",
    notReady: "未就绪",
    error: "异常"
  }[status] ?? status;
}

const serviceStatusClass = (status?: string | null) => {
  if (!status) return "disabled";
  if (status === "ok") return "ok";
  if (status === "notReady") return "warning";
  return "error";
}

const readinessCheckLabel = (name: string) => readinessCheckLabels[name] ?? name;

const indexStateText = (status: IndexStatus | null) => {
  if (!status) return "未知";
  if (!status.enabled || status.state === "disabled") return "未启用";
  return {
    idle: "空闲",
    scanning: "重建中",
    building: "重建中",
    error: "异常"
  }[status.state] ?? status.state;
}

const indexStateClass = (status: IndexStatus | null) => {
  if (!status || !status.enabled || status.state === "disabled") return "disabled";
  return {
    idle: "idle",
    scanning: "building",
    building: "building",
    error: "error"
  }[status.state] ?? "idle";
}

const optionalDateText = (value?: string | null) => value ? formatEntryDate(value) : "-";
const countText = (value?: number) => typeof value === "number" && Number.isFinite(value) ? String(value) : "-";

const limitUsageText = (active?: number, limit?: number) => {
  if (!Number.isFinite(active) || !Number.isFinite(limit)) return "-";
  return `${active}/${limit}`;
}

const limitUsageRatio = (active?: number, limit?: number) => {
  if (!Number.isFinite(active) || !Number.isFinite(limit) || !limit) return "0%";
  return `${Math.min(100, Math.max(0, Number(active) / Number(limit) * 100))}%`;
}

const startupFieldName = (fieldPath: string) => {
  const key = fieldPath.replace(/^startup\./, "") as keyof StartupSettings;
  return startupFieldLabels[key] ?? fieldPath;
}

const startupFieldValue = (fieldPath: string, source?: StartupSettings | null) => {
  if (!source) return "-";
  const key = fieldPath.replace(/^startup\./, "") as keyof StartupSettings;
  const value = source[key];
  if (Array.isArray(value)) return value.length ? value.join("，") : "同源";
  if (typeof value === "boolean") return value ? "启用" : "关闭";
  return String(value ?? "-");
}
</script>

<template>
  <div class="settings-page">
    <header class="settings-topbar">
      <button class="icon-button" title="返回文件浏览器" @click="router.push('/')">
        <Icon icon="action.previous" />
      </button>
      <div class="settings-title">
        <h1>设置</h1>
        <span>{{ settingsSnapshot?.restartPending ? "有启动配置等待重启生效" : `当前：${activeNavItem.label}` }}</span>
      </div>
    </header>

    <main class="settings-shell">
      <aside class="settings-sidebar" aria-label="设置分组">
        <button
            v-for="item in navItems"
            :key="item.id"
            type="button"
            class="nav-item"
            :class="{active: activeSettingsPage === item.id}"
            @click="selectSettingsPage(item.id)">
          <Icon :icon="item.icon" />
          <span>{{ item.label }}</span>
        </button>
      </aside>

      <section class="settings-content">
        <div v-if="activeSettingsPage === 'config'" class="settings-actionbar">
          <div class="settings-actionbar-copy">
            <strong>{{ activeNavItem.label }}</strong>
            <span>{{ settingsSnapshot?.restartPending ? "启动配置等待重启生效" : "保存会应用当前未提交的配置改动" }}</span>
          </div>
          <div class="topbar-actions">
            <button class="plain-button" :disabled="loading || saving" @click="load">
              <Icon class="icon-motion-spin" :class="{'is-spinning': loading}" icon="action.refresh" />
              刷新
            </button>
            <button class="plain-button" :disabled="reloadingSettings || saving" @click="requestReloadSettings">
              <Icon class="icon-motion-spin" :class="{'is-spinning': reloadingSettings}" icon="action.refresh" />
              重载配置
            </button>
            <button class="plain-button" :disabled="!settingsDirty || saving" @click="resetSettingsForms">
              撤销
            </button>
            <button class="primary-button" :disabled="!canSaveSettings" @click="saveSettings">
              <Icon icon="action.save" />
              保存更改
            </button>
          </div>
        </div>

        <section v-if="activeSettingsPage === 'overview'" id="overview" class="settings-panel hero-panel">
          <div class="panel-heading">
            <div>
              <p class="eyebrow">Overview</p>
              <h2>服务概览</h2>
            </div>
            <div class="panel-actions">
              <button class="plain-button" :disabled="metricsLoading" @click="loadMetrics(true)">
                <Icon class="icon-motion-spin" :class="{'is-spinning': metricsLoading}" icon="action.refresh" />
                刷新状态
              </button>
              <button
                  class="plain-button"
                  :disabled="auditCleanupLoading"
                  title="删除超过保留数量的旧审计日志文件，不会删除当前正在写入的日志"
                  @click="requestAuditCleanup">
                <Icon icon="action.clean" />
                清理旧审计日志
              </button>
            </div>
          </div>

          <div v-if="settingsSnapshot?.restartPending" class="restart-banner">
            <Icon icon="action.warning" />
            <div>
              <strong>启动配置已保存，但当前进程还在使用旧配置。</strong>
              <span>重启服务后，下次启动配置会进入当前生效配置。</span>
            </div>
          </div>

          <div class="status-grid">
            <article class="status-tile">
              <span>服务存活</span>
              <strong class="status-pill" :class="serviceStatusClass(health?.status)">{{ serviceStatusText(health?.status) }}</strong>
              <small>版本 {{ health?.version ?? "-" }}</small>
            </article>
            <article class="status-tile">
              <span>服务就绪</span>
              <strong class="status-pill" :class="serviceStatusClass(readiness?.status)">{{ serviceStatusText(readiness?.status) }}</strong>
              <small>{{ readiness?.checks?.length ? `${readinessOkCount}/${readiness.checks.length} 项正常` : "暂无检查项" }}</small>
            </article>
            <article class="status-tile">
              <span>挂载目录</span>
              <strong>{{ countText(metrics?.mappings) }}</strong>
              <small>页面已加载 {{ mappings.length }} 项</small>
            </article>
            <article class="status-tile">
              <span>后台任务</span>
              <strong>{{ countText(taskMetrics?.total) }}</strong>
              <small>运行 {{ countText(taskMetrics?.running) }}，排队 {{ countText(taskMetrics?.queued) }}</small>
            </article>
          </div>

          <div class="overview-grid">
            <section class="inline-section">
              <div class="inline-section-heading">
                <div>
                  <h3>服务依赖检查</h3>
                  <p>检查登录、配置文件、挂载配置、回收站、审计日志和静态资源是否可用。</p>
                </div>
                <span v-if="readiness?.checks?.length" class="section-badge" :class="{warning: readinessIssueCount > 0}">
                  {{ readinessIssueCount > 0 ? `${readinessIssueCount} 项异常` : `${readinessOkCount} 项正常` }}
                </span>
              </div>
              <div v-if="readiness?.checks?.length" class="check-list">
                <div v-for="check in readiness.checks" :key="check.name" class="check-row">
                  <span class="check-dot" :class="serviceStatusClass(check.status)"></span>
                  <div class="check-name">
                    <strong>{{ readinessCheckLabel(check.name) }}</strong>
                    <small v-if="readinessCheckLabel(check.name) !== check.name">{{ check.name }}</small>
                  </div>
                  <span>{{ check.message }}</span>
                </div>
              </div>
              <p v-else class="empty-inline">暂无就绪检查信息</p>
            </section>

            <section class="inline-section">
              <div class="inline-section-heading">
                <div>
                  <h3>并发占用</h3>
                  <p>用于判断目录扫描、文件传输和请求限流是否接近上限。</p>
                </div>
              </div>
              <div class="limit-list">
                <div class="limit-row">
                  <div><span>目录扫描</span><strong>{{ limitUsageText(limitMetrics?.activeDirScans, limitMetrics?.dirScanLimit) }}</strong></div>
                  <span class="limit-bar"><span :style="{width: limitUsageRatio(limitMetrics?.activeDirScans, limitMetrics?.dirScanLimit)}"></span></span>
                </div>
                <div class="limit-row">
                  <div><span>文件传输</span><strong>{{ limitUsageText(limitMetrics?.activeTransfers, limitMetrics?.transferLimit) }}</strong></div>
                  <span class="limit-bar"><span :style="{width: limitUsageRatio(limitMetrics?.activeTransfers, limitMetrics?.transferLimit)}"></span></span>
                </div>
                <div class="limit-row">
                  <div><span>IP 请求</span><strong>{{ limitUsageText(limitMetrics?.activeIpRequests, limitMetrics?.ipLimit) }}</strong></div>
                  <span class="limit-bar"><span :style="{width: limitUsageRatio(limitMetrics?.activeIpRequests, limitMetrics?.ipLimit)}"></span></span>
                </div>
              </div>
            </section>
          </div>
        </section>

        <section v-if="activeSettingsPage === 'config'" id="config" class="config-split">
          <section id="runtime" class="settings-panel config-pane runtime-pane">
          <div class="panel-heading">
            <div>
              <p class="eyebrow">Runtime</p>
              <h2>运行配置</h2>
            </div>
            <span class="section-badge config-badge immediate">保存后立即生效</span>
          </div>

          <div class="setting-group">
            <h3>索引与审计</h3>
            <div class="form-grid">
              <label class="switch-field" :class="{disabled: isRuntimeLocked('indexEnabled')}">
                <input v-model="runtimeForm.indexEnabled" type="checkbox" :disabled="isRuntimeLocked('indexEnabled')">
                <span class="switch-track"><span></span></span>
                <span class="switch-copy">
                  <strong>启用搜索索引</strong>
                  <small>影响后续索引任务</small>
                </span>
              </label>
              <label class="setting-field">
                <span>索引扫描延迟 <small>毫秒</small></span>
                <input v-model="runtimeForm.indexScanDelayMs" type="number" min="0" :disabled="isRuntimeLocked('indexScanDelayMs')">
              </label>
              <label class="switch-field" :class="{disabled: isRuntimeLocked('auditEnabled')}">
                <input v-model="runtimeForm.auditEnabled" type="checkbox" :disabled="isRuntimeLocked('auditEnabled')">
                <span class="switch-track"><span></span></span>
                <span class="switch-copy">
                  <strong>启用审计日志</strong>
                  <small>关闭后不再记录新的审计事件</small>
                </span>
              </label>
              <label class="setting-field">
                <span>审计轮转大小 <small>字节，空为不限制</small></span>
                <input v-model="runtimeForm.auditMaxBytes" type="number" min="1" :disabled="isRuntimeLocked('auditMaxBytes')">
              </label>
              <label class="setting-field">
                <span>审计保留数量</span>
                <input v-model="runtimeForm.auditRetentionFiles" type="number" min="0" :disabled="isRuntimeLocked('auditRetentionFiles')">
              </label>
            </div>
          </div>

          <div class="setting-group">
            <h3>回收站与冲突</h3>
            <div class="form-grid">
              <label class="setting-field">
                <span>回收站保留天数 <small>空为不限制</small></span>
                <input v-model="runtimeForm.trashRetentionDays" type="number" min="1" :disabled="isRuntimeLocked('trashRetentionDays')">
              </label>
              <label class="setting-field">
                <span>回收站容量 <small>字节，空为不限制</small></span>
                <input v-model="runtimeForm.trashMaxBytes" type="number" min="1" :disabled="isRuntimeLocked('trashMaxBytes')">
              </label>
              <label class="setting-field wide">
                <span>默认冲突策略</span>
                <select v-model="runtimeForm.conflictPolicy" :disabled="isRuntimeLocked('conflictPolicy')">
                  <option v-for="option in conflictPolicyOptions" :key="option.value" :value="option.value">
                    {{ option.label }} - {{ option.description }}
                  </option>
                </select>
              </label>
            </div>
          </div>

          <div class="setting-group">
            <h3>会话与访问</h3>
            <div class="form-grid">
              <label class="setting-field">
                <span>会话有效期 <small>秒</small></span>
                <input v-model="runtimeForm.authSessionTtlSeconds" type="number" min="1" :disabled="isRuntimeLocked('authSessionTtlSeconds')">
              </label>
              <label class="switch-field" :class="{disabled: isRuntimeLocked('authSecureCookie')}">
                <input v-model="runtimeForm.authSecureCookie" type="checkbox" :disabled="isRuntimeLocked('authSecureCookie')">
                <span class="switch-track"><span></span></span>
                <span class="switch-copy">
                  <strong>Secure Cookie</strong>
                  <small>仅在 HTTPS 下发送新会话 Cookie</small>
                </span>
              </label>
            </div>
          </div>

          <div class="setting-group">
            <h3>编辑与上传</h3>
            <div class="form-grid">
              <label class="setting-field">
                <span>编辑上限 <small>字节</small></span>
                <input v-model="runtimeForm.maxEditBytes" type="number" min="1" :disabled="isRuntimeLocked('maxEditBytes')">
              </label>
              <label class="setting-field">
                <span>上传上限 <small>空为不限制</small></span>
                <input v-model="runtimeForm.maxUploadBytes" type="number" min="1" :disabled="isRuntimeLocked('maxUploadBytes')">
              </label>
              <label class="setting-field wide">
                <span>可编辑扩展名 <small>每行一个，空为不限制</small></span>
                <textarea v-model="runtimeForm.editableExtensions" :disabled="isRuntimeLocked('editableExtensions')" rows="4"></textarea>
              </label>
              <label class="setting-field wide">
                <span>可编辑 MIME <small>每行一个，空为不限制</small></span>
                <textarea v-model="runtimeForm.editableMimeTypes" :disabled="isRuntimeLocked('editableMimeTypes')" rows="4"></textarea>
              </label>
            </div>
          </div>

          <div class="setting-group">
            <h3>浏览与并发</h3>
            <div class="form-grid">
              <label class="setting-field">
                <span>目录分页上限</span>
                <input v-model="runtimeForm.maxDirPageSize" type="number" min="1" :disabled="isRuntimeLocked('maxDirPageSize')">
              </label>
              <label class="setting-field">
                <span>目录扫描并发</span>
                <input v-model="runtimeForm.maxDirConcurrency" type="number" min="1" :disabled="isRuntimeLocked('maxDirConcurrency')">
              </label>
              <label class="setting-field">
                <span>文件传输并发</span>
                <input v-model="runtimeForm.maxTransferConcurrency" type="number" min="1" :disabled="isRuntimeLocked('maxTransferConcurrency')">
              </label>
              <label class="setting-field">
                <span>单 IP 并发</span>
                <input v-model="runtimeForm.maxIpConcurrency" type="number" min="1" :disabled="isRuntimeLocked('maxIpConcurrency')">
              </label>
            </div>
          </div>

          <div class="setting-group">
            <h3>任务与压缩</h3>
            <div class="form-grid">
              <label class="setting-field">
                <span>任务并发</span>
                <input v-model="runtimeForm.maxTaskConcurrency" type="number" min="1" :disabled="isRuntimeLocked('maxTaskConcurrency')">
              </label>
              <label class="setting-field">
                <span>任务历史</span>
                <input v-model="runtimeForm.taskHistoryLimit" type="number" min="1" :disabled="isRuntimeLocked('taskHistoryLimit')">
              </label>
              <label class="setting-field">
                <span>任务限速 <small>字节/秒，空为不限制</small></span>
                <input v-model="runtimeForm.taskSpeedLimitBytesPerSec" type="number" min="1" :disabled="isRuntimeLocked('taskSpeedLimitBytesPerSec')">
              </label>
              <label class="setting-field">
                <span>压缩输入上限 <small>字节，空为不限制</small></span>
                <input v-model="runtimeForm.maxArchiveBytes" type="number" min="1" :disabled="isRuntimeLocked('maxArchiveBytes')">
              </label>
              <label class="setting-field">
                <span>压缩条目上限 <small>空为不限制</small></span>
                <input v-model="runtimeForm.maxArchiveFiles" type="number" min="1" :disabled="isRuntimeLocked('maxArchiveFiles')">
              </label>
              <label class="setting-field">
                <span>解压字节上限 <small>空为不限制</small></span>
                <input v-model="runtimeForm.maxExtractBytes" type="number" min="1" :disabled="isRuntimeLocked('maxExtractBytes')">
              </label>
              <label class="setting-field">
                <span>解压条目上限 <small>空为不限制</small></span>
                <input v-model="runtimeForm.maxExtractFiles" type="number" min="1" :disabled="isRuntimeLocked('maxExtractFiles')">
              </label>
              <label class="setting-field">
                <span>解压深度上限</span>
                <input v-model="runtimeForm.maxExtractDepth" type="number" min="1" :disabled="isRuntimeLocked('maxExtractDepth')">
              </label>
            </div>
          </div>

          </section>

          <section id="startup" class="settings-panel config-pane startup-pane">
          <div class="panel-heading">
            <div>
              <p class="eyebrow">Startup</p>
              <h2>启动配置</h2>
            </div>
            <span class="section-badge config-badge restart" :class="{warning: settingsSnapshot?.restartPending}">
              {{ settingsSnapshot?.restartPending ? "等待重启生效" : "保存后重启生效" }}
            </span>
          </div>

          <div v-if="settingsSnapshot?.restartPendingFields.length" class="pending-list">
            <div v-for="field in settingsSnapshot.restartPendingFields" :key="field" class="pending-row">
              <strong>{{ startupFieldName(field) }}</strong>
              <span>当前：{{ startupFieldValue(field, activeStartupSettings) }}</span>
              <span>下次：{{ startupFieldValue(field, startupSettings) }}</span>
            </div>
          </div>

          <div class="setting-group">
            <h3>服务与入口</h3>
            <div class="form-grid">
              <label class="setting-field" :class="{pending: isStartupPending('bindAddress')}">
                <span>监听地址</span>
                <input v-model="startupForm.bindAddress" :disabled="isStartupLocked('bindAddress')">
              </label>
              <label class="setting-field" :class="{pending: isStartupPending('port')}">
                <span>端口</span>
                <input v-model="startupForm.port" type="number" min="1" max="65535" :disabled="isStartupLocked('port')">
              </label>
              <label class="setting-field wide" :class="{pending: isStartupPending('staticDir')}">
                <span>静态文件目录</span>
                <input v-model="startupForm.staticDir" :disabled="isStartupLocked('staticDir')">
              </label>
              <label class="setting-field wide readonly-field">
                <span>配置文件 <small>不支持在线修改</small></span>
                <input v-model="startupForm.configFile" disabled>
              </label>
            </div>
          </div>

          <div class="setting-group">
            <h3>数据文件</h3>
            <div class="form-grid">
              <label class="setting-field wide" :class="{pending: isStartupPending('mappingFile')}">
                <span>挂载配置文件</span>
                <input v-model="startupForm.mappingFile" :disabled="isStartupLocked('mappingFile')">
              </label>
              <label class="setting-field wide" :class="{pending: isStartupPending('authFile')}">
                <span>认证文件</span>
                <input v-model="startupForm.authFile" :disabled="isStartupLocked('authFile')">
              </label>
              <label class="setting-field wide" :class="{pending: isStartupPending('favoritesFile')}">
                <span>收藏文件</span>
                <input v-model="startupForm.favoritesFile" :disabled="isStartupLocked('favoritesFile')">
              </label>
              <label class="setting-field wide" :class="{pending: isStartupPending('trashDir')}">
                <span>回收站目录</span>
                <input v-model="startupForm.trashDir" :disabled="isStartupLocked('trashDir')">
              </label>
              <label class="setting-field wide" :class="{pending: isStartupPending('auditFile')}">
                <span>审计日志文件</span>
                <input v-model="startupForm.auditFile" :disabled="isStartupLocked('auditFile')">
              </label>
            </div>
          </div>

          <div class="setting-group">
            <h3>网络与启动行为</h3>
            <div class="form-grid">
              <label class="setting-field wide" :class="{pending: isStartupPending('corsAllowedOrigins')}">
                <span>CORS 来源 <small>每行一个，空为同源</small></span>
                <textarea v-model="startupForm.corsAllowedOrigins" rows="4" :disabled="isStartupLocked('corsAllowedOrigins')"></textarea>
              </label>
              <label class="switch-field" :class="{pending: isStartupPending('trustProxyHeaders'), disabled: isStartupLocked('trustProxyHeaders')}">
                <input v-model="startupForm.trustProxyHeaders" type="checkbox" :disabled="isStartupLocked('trustProxyHeaders')">
                <span class="switch-track"><span></span></span>
                <span class="switch-copy">
                  <strong>信任代理来源头</strong>
                  <small>位于反向代理后方时使用</small>
                </span>
              </label>
              <label class="switch-field" :class="{pending: isStartupPending('indexRebuildOnStartup'), disabled: isStartupLocked('indexRebuildOnStartup')}">
                <input v-model="startupForm.indexRebuildOnStartup" type="checkbox" :disabled="isStartupLocked('indexRebuildOnStartup')">
                <span class="switch-track"><span></span></span>
                <span class="switch-copy">
                  <strong>启动时重建索引</strong>
                  <small>下次启动时执行</small>
                </span>
              </label>
            </div>
          </div>
          </section>
        </section>

        <section v-if="activeSettingsPage === 'mappings'" id="mappings" class="settings-panel">
          <div class="panel-heading">
            <div>
              <p class="eyebrow">Mounts</p>
              <h2>挂载目录</h2>
            </div>
            <div class="panel-actions">
              <button class="plain-button" :disabled="mappingBusy" @click="loadMappings(true)">
                <Icon class="icon-motion-spin" :class="{'is-spinning': mappingRefreshing}" icon="action.refresh" />
                刷新
              </button>
              <button class="primary-button" :disabled="mappingBusy" @click="openMappingDialog">
                <Icon icon="action.add" />
                添加挂载
              </button>
            </div>
          </div>

          <div class="mapping-summary">
            <div>
              <span>挂载数量</span>
              <strong>{{ mappings.length }}</strong>
            </div>
            <div>
              <span>可写目录</span>
              <strong>{{ writableMappingCount }}</strong>
            </div>
            <div>
              <span>只读目录</span>
              <strong>{{ readonlyMappingCount }}</strong>
            </div>
            <div>
              <span>配置文件</span>
              <strong>{{ startupSettings?.mappingFile || "未读取" }}</strong>
            </div>
          </div>

          <div class="mapping-list-heading">
            <div>
              <strong>当前挂载</strong>
              <span>按显示顺序排列，拖动把手后会立即保存</span>
            </div>
            <span>{{ mappings.length }} 项</span>
          </div>

          <div class="mapping-table" role="table" aria-label="当前挂载目录">
            <div class="mapping-table-head" role="row">
              <span>顺序</span>
              <span>虚拟路径</span>
              <span>本地/容器目录</span>
              <span>备注</span>
              <span>权限</span>
              <span>操作</span>
            </div>
            <div
                v-for="mapping in sortedMappings"
                :key="mapping.id"
                class="mapping-table-row"
                :class="{
                  readonly: !mapping.writable,
                  dragging: draggingMappingId === mapping.id,
                  'drop-before': dragOverMappingId === mapping.id && dragOverPlacement === 'before',
                  'drop-after': dragOverMappingId === mapping.id && dragOverPlacement === 'after'
                }"
                role="row"
                @dragover="updateMappingDragPlacement(mapping, $event)"
                @drop="handleMappingDrop(mapping, $event)">
              <div
                  class="mapping-handle-cell"
                  title="拖动调整顺序"
                  :draggable="!mappingBusy"
                  @dragstart="handleMappingDragStart(mapping, $event)"
                  @dragend="handleMappingDragEnd">
                <Icon icon="action.drag-handle" />
              </div>
              <div class="mapping-cell primary" :title="mapping.mountPath">
                <strong>{{ mapping.mountPath }}</strong>
              </div>
              <div class="mapping-cell" :title="mapping.folderPath">
                <span>{{ mapping.folderPath }}</span>
              </div>
              <div class="mapping-cell" :class="{muted: !mapping.remark}" :title="mapping.remark || '无备注'">
                <span>{{ mapping.remark || "无备注" }}</span>
              </div>
              <div class="mapping-access-cell">
                <span class="access-badge" :class="{readonly: !mapping.writable}">
                  {{ mapping.writable ? "可写" : "只读" }}
                </span>
              </div>
              <div class="mapping-row-actions">
                <button class="plain-button" :disabled="mappingBusy" @click="openEditMappingDialog(mapping)">
                  <Icon icon="action.edit" />
                  修改
                </button>
                <button class="danger-button" :disabled="mappingBusy" @click="openMappingDeleteConfirm(mapping)">
                  <Icon icon="action.trash" />
                  删除
                </button>
              </div>
            </div>
            <div v-if="!mappings.length && !loading" class="mapping-empty">
              <Icon icon="file.folder" />
              <strong>暂无挂载目录</strong>
              <span>添加第一个挂载后会显示在文件树根目录。</span>
            </div>
          </div>

          <operation-panel-shell
              v-if="mappingDialogOpen"
              as="form"
              icon="action.new-folder"
              :title="mappingDialogTitle"
              :subtitle="mappingDialogSubtitle"
              width="properties"
              @close="closeMappingDialog"
              @submit="submitMappingDialog">
              <div class="mapping-dialog-body">
                <label class="setting-field wide">
                  <span>虚拟路径</span>
                  <input ref="mappingMountInputRef" v-model="mappingForm.mountPath" placeholder="/files" required>
                </label>
                <label class="setting-field wide">
                  <span>本地/容器目录</span>
                  <input v-model="mappingForm.folderPath" placeholder="D:\\Files 或 /mnt/files" required>
                </label>
                <label class="setting-field wide">
                  <span>备注</span>
                  <input v-model="mappingForm.remark" placeholder="可选">
                </label>
                <label class="switch-field mapping-writable-toggle">
                  <input v-model="mappingForm.writable" type="checkbox">
                  <span class="switch-track"><span></span></span>
                  <span class="switch-copy">
                    <strong>允许写入</strong>
                    <small>关闭后上传、编辑、删除等写操作会被阻止</small>
                  </span>
                </label>
              </div>
              <template #actions>
                <button class="plain-button" type="button" :disabled="mappingBusy" @click="closeMappingDialog">取消</button>
                <button class="primary-button" :disabled="mappingBusy" type="submit">
                  <Icon :icon="mappingDialogMode === 'edit' ? 'action.save' : 'action.add'" />
                  {{ mappingDialogSubmitText }}
                </button>
              </template>
          </operation-panel-shell>

          <operation-panel-shell
              v-if="mappingDeleteTarget"
              icon="action.delete"
              variant="red"
              width="delete"
              title="删除挂载？"
              :subtitle="`将移除 ${mappingDeleteTarget.mountPath} 的挂载配置，不会删除真实目录。`"
              @close="closeMappingDeleteConfirm">
            <div class="mapping-delete-summary">
              <div>
                <span>虚拟路径</span>
                <strong>{{ mappingDeleteTarget.mountPath }}</strong>
              </div>
              <div>
                <span>本地/容器目录</span>
                <strong>{{ mappingDeleteTarget.folderPath }}</strong>
              </div>
            </div>
            <template #actions>
              <button class="plain-button" type="button" :disabled="mappingBusy" @click="closeMappingDeleteConfirm">取消</button>
              <button class="danger-button" type="button" :disabled="mappingBusy" @click="confirmRemoveMapping">
                <Icon icon="action.trash" />
                确认删除
              </button>
            </template>
          </operation-panel-shell>
        </section>

        <section v-if="activeSettingsPage === 'index'" id="index" class="settings-panel">
          <div class="panel-heading">
            <div>
              <p class="eyebrow">Index & Security</p>
              <h2>索引与安全</h2>
            </div>
          </div>

          <div class="index-security-layout">
            <section class="inline-section index-card">
              <div class="subsection-heading">
                <div>
                  <h3>搜索索引</h3>
                  <span>查看索引状态，手动触发重建或取消正在进行的重建任务。</span>
                </div>
                <div class="panel-actions">
                  <button class="plain-button" :disabled="indexBusy" @click="loadIndexStatus(true)">
                    <Icon class="icon-motion-spin" :class="{'is-spinning': indexLoading}" icon="action.refresh" />
                    刷新状态
                  </button>
                  <button class="primary-button" :disabled="!canRebuildIndex" @click="requestIndexRebuild">重建索引</button>
                  <button v-if="indexBuilding" class="danger-button" :disabled="!canCancelIndex" @click="requestIndexCancel">取消重建</button>
                </div>
              </div>
              <div class="index-summary">
                <span class="index-state" :class="indexStateClass(indexStatus)">{{ indexLoading ? "读取中" : indexStateText(indexStatus) }}</span>
                <span>已索引 {{ countText(indexStatus?.indexedEntries) }} 项</span>
                <span>上次开始 {{ optionalDateText(indexStatus?.lastStartedAt) }}</span>
                <span>上次完成 {{ optionalDateText(indexStatus?.lastFinishedAt) }}</span>
              </div>
              <div v-if="indexBuilding" class="index-progress" aria-label="索引重建进行中">
                <span></span>
              </div>
              <p v-if="indexStatus?.lastError" class="error-text">{{ indexStatus.lastError }}</p>
            </section>

            <section class="inline-section security-card">
              <div class="subsection-heading">
                <div>
                  <h3>管理员密码</h3>
                  <span>更新当前管理员账号的登录密码。</span>
                </div>
                <span class="section-badge">{{ settingsSnapshot?.authConfigured ? "已初始化" : "未初始化" }}</span>
              </div>
              <form class="password-form" @submit.prevent="savePassword">
                <input v-model="passwordForm.currentPassword" :disabled="passwordSaving" autocomplete="current-password" placeholder="当前密码" type="password" required>
                <input v-model="passwordForm.newPassword" :disabled="passwordSaving" autocomplete="new-password" minlength="8" placeholder="新密码" type="password" required>
                <input v-model="passwordForm.confirmPassword" :disabled="passwordSaving" autocomplete="new-password" minlength="8" placeholder="确认新密码" type="password" required>
                <button class="primary-button" :disabled="passwordSaving" type="submit">
                  <Icon icon="action.save" />
                  {{ passwordSaving ? "更新中" : "更新密码" }}
                </button>
              </form>
            </section>
          </div>
        </section>
      </section>
    </main>

    <Transition name="shell-notice-pop" mode="out-in">
      <div
          v-if="shellNotice.visible"
          :key="shellNotice.id"
          class="shell-notice-layer">
        <shell-notice
            :kind="shellNotice.kind"
            :title="shellNotice.title"
            :message="shellNotice.message"
            @close="closeShellNotice"
            @pause="stopShellNoticeTimer"
            @resume="resumeShellNoticeTimer" />
      </div>
    </Transition>
  </div>
</template>

<style scoped lang="postcss">
@reference "tailwindcss";

.settings-page {
  @apply min-h-screen;
  background: var(--app-bg);
  color: var(--app-text);
}

.settings-topbar {
  @apply sticky top-0 z-30 flex h-14 items-center gap-3 border-b px-4;
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 94%, transparent);
  backdrop-filter: blur(18px);
}

.settings-title {
  @apply min-w-0 flex-1;
}

.settings-title h1 {
  @apply text-base font-semibold leading-tight;
}

.settings-title span {
  @apply block truncate text-xs;
  color: var(--app-text-subtle);
}

.topbar-actions,
.panel-actions {
  @apply flex shrink-0 flex-wrap items-center gap-2;
}

.settings-shell {
  @apply grid gap-4 p-4;
  grid-template-columns: 13rem minmax(0, 1fr);
  align-items: start;
}

.settings-sidebar {
  @apply sticky top-[4.5rem] flex h-[calc(100vh-5.5rem)] flex-col gap-1 overflow-auto rounded-lg border p-2;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
}

.nav-item {
  @apply flex h-9 w-full items-center gap-2 rounded-md border-0 px-3 text-left text-sm font-medium no-underline outline-none transition;
  background: transparent;
  color: var(--app-text-muted);
}

.nav-item:hover {
  background: var(--app-control-hover);
  color: var(--app-text);
}

.nav-item.active {
  background: var(--app-accent-soft, #eff6ff);
  color: var(--app-accent, #2563eb);
  box-shadow: inset 3px 0 0 var(--app-accent, #2563eb);
}

.settings-content {
  @apply flex min-w-0 w-full flex-col gap-4;
  max-width: 72rem;
  justify-self: center;
}

.settings-actionbar {
  @apply sticky z-20 flex min-w-0 flex-wrap items-center justify-between gap-3 rounded-lg border px-3 py-2;
  top: calc(3.5rem + 0.75rem);
  border-color: var(--app-border-soft);
  background: color-mix(in srgb, var(--app-panel-solid) 94%, transparent);
  backdrop-filter: blur(16px);
  box-shadow: 0 10px 28px color-mix(in srgb, var(--app-shadow, rgba(15, 23, 42, 0.12)) 10%, transparent);
}

.settings-actionbar-copy {
  @apply min-w-0;
}

.settings-actionbar-copy strong,
.settings-actionbar-copy span {
  @apply block truncate;
}

.settings-actionbar-copy strong {
  @apply text-sm font-semibold;
  color: var(--app-text);
}

.settings-actionbar-copy span {
  @apply text-xs;
  color: var(--app-text-subtle);
}

.settings-actionbar .topbar-actions {
  @apply justify-end;
}

.config-split {
  @apply grid min-w-0 gap-4;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-items: start;
}

.config-pane {
  @apply grid min-w-0 gap-3;
}

.startup-pane {
  order: 1;
}

.runtime-pane {
  order: 2;
}

.config-split .form-grid {
  grid-template-columns: 1fr;
}

.config-split .setting-field.wide {
  @apply col-span-1;
}

.settings-panel {
  @apply scroll-mt-20 rounded-lg border p-4;
  border-color: var(--app-border-soft);
  background: var(--app-panel-solid);
  box-shadow: 0 14px 36px color-mix(in srgb, var(--app-shadow, rgba(15, 23, 42, 0.12)) 16%, transparent);
}

.hero-panel {
  background:
      linear-gradient(135deg, var(--app-accent-tint), transparent 46%),
      var(--app-panel-solid);
}

.panel-heading {
  @apply mb-4 flex flex-wrap items-start justify-between gap-3;
}

.eyebrow {
  @apply mb-1 text-[11px] font-semibold uppercase tracking-wide;
  color: var(--app-accent, #2563eb);
}

.panel-heading h2 {
  @apply text-base font-semibold;
}

.section-badge {
  @apply inline-flex h-7 items-center rounded-full px-3 text-xs font-semibold;
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.section-badge.warning {
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.section-badge.config-badge {
  @apply rounded-md border px-2.5;
}

.config-badge.immediate {
  border-color: var(--app-success-border);
  background: var(--app-success-soft);
  color: var(--app-success-text);
}

.config-badge.restart {
  border-color: var(--app-warning-border);
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.restart-banner {
  @apply mb-4 flex items-start gap-3 rounded-md border px-3 py-2 text-sm;
  border-color: var(--app-warning-border);
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.restart-banner strong,
.restart-banner span {
  @apply block;
}

.status-grid {
  @apply grid gap-3;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.status-tile {
  @apply min-w-0 rounded-md border px-3 py-2;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.status-tile > span,
.status-tile small {
  @apply block truncate text-xs;
  color: var(--app-text-subtle);
}

.status-tile strong {
  @apply mt-1 block truncate text-lg font-semibold;
  color: var(--app-text);
}

.status-pill {
  @apply inline-flex h-7 w-fit max-w-full items-center rounded-full px-3 text-xs font-semibold;
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.status-pill.ok {
  background: var(--app-success-soft);
  color: var(--app-success-text);
}

.status-pill.warning {
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.status-pill.error {
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.overview-grid {
  @apply mt-4 grid gap-4;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
}

.inline-section {
  @apply min-w-0 rounded-md border p-3;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.inline-section h3,
.setting-group h3 {
  @apply mb-3 text-sm font-semibold;
  color: var(--app-text);
}

.inline-section-heading {
  @apply mb-3 flex min-w-0 flex-wrap items-start justify-between gap-3;
}

.inline-section-heading h3 {
  @apply mb-0;
}

.inline-section-heading p {
  @apply mt-1 text-xs leading-5;
  color: var(--app-text-subtle);
}

.inline-section-heading .section-badge {
  @apply shrink-0;
}

.check-list,
.limit-list {
  @apply grid gap-2;
}

.check-row {
  @apply grid min-w-0 items-center gap-2 rounded px-2 py-1.5 text-xs;
  grid-template-columns: 0.5rem minmax(6.5rem, 0.48fr) minmax(0, 1fr);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.check-row strong,
.check-row span:last-child {
  @apply min-w-0 truncate;
}

.check-name {
  @apply grid min-w-0 gap-0.5;
}

.check-name strong {
  color: var(--app-text);
}

.check-name small {
  @apply min-w-0 truncate text-[11px];
  color: var(--app-text-subtle);
}

.check-dot {
  @apply h-2 w-2 rounded-full;
  background: var(--app-text-subtle);
}

.check-dot.ok {
  background: var(--app-success);
}

.check-dot.warning {
  background: var(--app-warning);
}

.check-dot.error {
  background: var(--app-danger);
}

.limit-row {
  @apply grid gap-1.5;
}

.limit-row > div {
  @apply flex min-w-0 items-center justify-between gap-3 text-xs;
}

.limit-row span {
  color: var(--app-text-subtle);
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

.setting-group {
  @apply border-t pt-4 first:border-t-0 first:pt-0;
  border-color: var(--app-divider);
}

.config-pane .setting-group {
  @apply rounded-lg border p-3;
  border-color: color-mix(in srgb, var(--app-border-soft) 82%, var(--app-accent, #2563eb));
  background: color-mix(in srgb, var(--app-panel-muted) 76%, var(--app-panel-solid));
  box-shadow:
      inset 0 1px 0 color-mix(in srgb, white 42%, transparent),
      0 8px 20px color-mix(in srgb, var(--app-shadow, rgba(15, 23, 42, 0.12)) 8%, transparent);
}

.config-pane .setting-group h3 {
  @apply mb-3 flex items-center gap-2 border-b pb-2 text-[13px];
  border-color: color-mix(in srgb, var(--app-divider) 82%, transparent);
}

.config-pane .setting-group h3::before {
  @apply block h-3.5 w-1 rounded-full;
  content: "";
  background: var(--app-accent, #2563eb);
  opacity: 0.78;
}

.config-pane .setting-field:has(input:disabled)::after,
.config-pane .setting-field:has(textarea:disabled)::after,
.config-pane .setting-field:has(select:disabled)::after,
.config-pane .switch-field.disabled::after {
  @apply mt-1 block rounded-md border px-2 py-1 text-xs;
  content: "由环境变量控制，不能在页面中覆盖";
  border-color: var(--app-warning-border);
  background: color-mix(in srgb, var(--app-warning-soft) 72%, transparent);
  color: var(--app-warning-text);
}

.config-pane .readonly-field:has(input:disabled)::after {
  content: "配置文件路径不支持在线修改，需要通过环境变量指定后重启";
}

.config-pane .switch-field.disabled {
  @apply flex-wrap;
}

.config-pane .switch-field.disabled::after {
  flex-basis: 100%;
  margin-left: 3.25rem;
}

.form-grid {
  @apply grid gap-3;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.setting-field {
  @apply grid min-w-0 gap-1.5;
}

.setting-field.wide {
  @apply col-span-2;
}

.setting-field > span {
  @apply text-sm font-medium;
  color: var(--app-text-muted);
}

.setting-field small {
  @apply ml-1 text-xs font-normal;
  color: var(--app-text-subtle);
}

.setting-field.pending input,
.setting-field.pending textarea,
.setting-field.pending select,
.switch-field.pending {
  border-color: var(--app-warning-border);
  box-shadow: inset 3px 0 0 var(--app-warning);
}

input,
textarea,
select {
  @apply min-w-0 rounded-md border px-2 text-sm outline-none;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text);
}

input,
select {
  @apply h-9;
}

textarea {
  @apply resize-y py-2;
}

input:focus,
textarea:focus,
select:focus {
  border-color: var(--app-accent, #2563eb);
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

input:disabled,
textarea:disabled,
select:disabled {
  @apply cursor-not-allowed opacity-70;
  color: var(--app-text-subtle);
}

.readonly-field input {
  background: var(--app-panel-muted);
}

.pending-list {
  @apply mb-4 grid gap-2 rounded-md border p-3;
  border-color: var(--app-warning-border);
  background: var(--app-warning-soft);
}

.pending-row {
  @apply grid min-w-0 gap-2 text-xs;
  grid-template-columns: 8rem minmax(0, 1fr) minmax(0, 1fr);
  color: var(--app-warning-text);
}

.pending-row strong,
.pending-row span {
  @apply min-w-0 truncate;
}

.password-form {
  @apply grid gap-2 items-center;
}

.mapping-summary {
  @apply mb-4 grid gap-3;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.mapping-summary > div {
  @apply min-w-0 rounded-md border px-3 py-2;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.mapping-summary span {
  @apply block truncate text-xs;
  color: var(--app-text-subtle);
}

.mapping-summary strong {
  @apply mt-1 block truncate text-sm font-semibold;
  color: var(--app-text);
}

.mapping-list-heading strong {
  @apply block truncate text-sm font-semibold;
  color: var(--app-text);
}

.mapping-list-heading span {
  @apply block truncate text-xs;
  color: var(--app-text-subtle);
}

.mapping-list-heading {
  @apply mb-2 flex min-w-0 items-end justify-between gap-3 px-1;
}

.mapping-list-heading > span {
  @apply shrink-0 text-xs;
}

.mapping-table {
  @apply min-w-0 overflow-x-auto rounded-md border;
  border-color: var(--app-border-soft);
  background: var(--app-panel-muted);
}

.mapping-table-head,
.mapping-table-row {
  @apply grid min-w-[54rem] items-center gap-2;
  grid-template-columns: 4.75rem minmax(8rem, 0.8fr) minmax(13rem, 1.4fr) minmax(8rem, 0.7fr) 6.5rem 11rem;
}

.mapping-table-head {
  @apply min-h-11 border-b px-3 py-3 text-xs font-semibold;
  border-color: var(--app-divider);
  color: var(--app-text-subtle);
  background: var(--app-panel-solid);
}

.mapping-table-row {
  @apply relative border-b px-3 py-2 transition last:border-b-0;
  border-color: var(--app-divider);
  background: var(--app-control-solid);
}

.mapping-table-row.dragging {
  @apply opacity-55;
}

.mapping-table-row.drop-before::before,
.mapping-table-row.drop-after::after {
  @apply absolute left-0 right-0 h-0.5;
  content: "";
  background: var(--app-accent, #2563eb);
}

.mapping-table-row.drop-before::before {
  top: 0;
}

.mapping-table-row.drop-after::after {
  bottom: 0;
}

.mapping-table-row:hover {
  background: var(--app-control);
}

.mapping-table-row.readonly {
  box-shadow: inset 3px 0 0 var(--app-warning);
}

.mapping-handle-cell {
  @apply flex h-8 w-8 cursor-grab items-center justify-center rounded-md;
  color: var(--app-text-subtle);
}

.mapping-handle-cell:active {
  @apply cursor-grabbing;
}

.mapping-cell {
  @apply min-w-0 truncate text-sm;
  color: var(--app-text-muted);
}

.mapping-cell span {
  @apply truncate;
}

.mapping-cell strong {
  @apply truncate font-semibold;
  color: var(--app-text);
}

.mapping-cell.muted {
  color: var(--app-text-subtle);
}

.access-badge {
  @apply inline-flex h-7 items-center rounded-full px-2.5 text-xs font-semibold;
  background: var(--app-success-soft);
  color: var(--app-success-text);
}

.access-badge.readonly {
  background: var(--app-warning-soft);
  color: var(--app-warning-text);
}

.mapping-access-cell {
  @apply flex min-w-0 items-center gap-2;
}

.mapping-row-actions {
  @apply flex items-center justify-end gap-2;
}

.mapping-row-actions .plain-button,
.mapping-row-actions .danger-button {
  @apply px-2.5;
}

.mapping-dialog-body {
  @apply grid w-full gap-3;
  grid-template-columns: minmax(0, 1fr);
  justify-items: stretch;
}

.mapping-dialog-body .setting-field,
.mapping-dialog-body .switch-field {
  @apply w-full;
  grid-column: 1 / -1;
  justify-self: stretch;
}

.mapping-dialog-body .setting-field.wide {
  @apply col-span-1;
}

.mapping-writable-toggle {
  @apply min-h-14;
}

.mapping-writable-toggle .switch-copy {
  @apply flex-1;
}

.switch-field {
  @apply flex w-full cursor-pointer items-center gap-2.5 rounded-md border px-3 py-2 text-xs transition;
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
}

.switch-field:hover {
  border-color: color-mix(in srgb, var(--app-accent-border, #bfdbfe) 46%, var(--app-border-soft));
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 30%, var(--app-control-solid));
}

.switch-field.disabled {
  @apply cursor-not-allowed opacity-70;
}

.switch-field.disabled:hover {
  border-color: var(--app-border-soft);
  background: var(--app-control-solid);
}

.switch-field input {
  @apply sr-only;
}

.switch-track {
  @apply flex h-5 w-9 shrink-0 items-center rounded-full p-0.5 transition-colors;
  background: var(--app-control-hover);
}

.switch-track span {
  @apply h-4 w-4 rounded-full transition-transform;
  background: var(--app-panel-solid);
  box-shadow: 0 1px 3px color-mix(in srgb, var(--app-shadow, rgba(15, 23, 42, 0.2)) 28%, transparent);
}

.switch-field input:checked + .switch-track {
  background: var(--app-accent, #2563eb);
}

.switch-field input:checked + .switch-track span {
  transform: translateX(1rem);
}

.switch-field:focus-within {
  border-color: var(--app-accent-border, #bfdbfe);
}

.switch-field:focus-within .switch-track {
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--app-accent, #2563eb) 16%, transparent);
}

.switch-copy {
  @apply flex min-w-0 flex-col gap-0.5 text-xs;
  color: var(--app-text-subtle);
}

.switch-copy strong {
  @apply text-[0.8125rem] font-semibold leading-4;
  color: var(--app-text);
}

.switch-copy small {
  @apply leading-4;
}

.mapping-delete-summary {
  @apply grid gap-2 rounded-md border p-3 text-xs;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.mapping-delete-summary div {
  @apply grid min-w-0 gap-1;
}

.mapping-delete-summary span {
  @apply font-medium;
}

.mapping-delete-summary strong {
  @apply truncate text-sm;
}

.mapping-empty {
  @apply grid place-items-center gap-1 rounded-md border border-dashed p-8 text-center;
  border-color: var(--app-border-soft);
  color: var(--app-text-subtle);
}

.mapping-empty .icon {
  @apply mb-1 h-10 w-10;
  color: var(--app-accent, #2563eb);
}

.mapping-empty strong {
  @apply text-sm;
  color: var(--app-text);
}

.index-security-layout {
  @apply grid gap-4;
  grid-template-columns: minmax(0, 1.2fr) minmax(20rem, 0.8fr);
  align-items: start;
}

.subsection-heading {
  @apply mb-3 flex min-w-0 flex-wrap items-start justify-between gap-3;
}

.subsection-heading > div {
  @apply min-w-0;
}

.subsection-heading h3 {
  @apply mb-0 text-sm font-semibold;
  color: var(--app-text);
}

.subsection-heading span:not(.section-badge) {
  @apply mt-1 block text-xs;
  color: var(--app-text-subtle);
}

.subsection-heading .section-badge {
  @apply shrink-0;
}

.index-summary {
  @apply flex flex-wrap items-center gap-3 text-sm;
  color: var(--app-text-muted);
}

.index-progress {
  @apply mt-3 h-2 overflow-hidden rounded-full;
  background: color-mix(in srgb, var(--app-accent-soft, #eff6ff) 52%, var(--app-control-solid));
}

.index-progress span {
  @apply block h-full rounded-full;
  width: 38%;
  background: linear-gradient(90deg, transparent, var(--app-accent, #2563eb), transparent);
  animation: index-progress-slide 1.1s ease-in-out infinite;
}

@keyframes index-progress-slide {
  from {
    transform: translateX(-110%);
  }

  to {
    transform: translateX(270%);
  }
}

.index-state {
  @apply inline-flex h-7 items-center rounded-full px-3 text-xs font-semibold;
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.index-state.idle {
  background: var(--app-success-soft);
  color: var(--app-success-text);
}

.index-state.building {
  background: var(--app-accent-soft);
  color: var(--app-accent, #2563eb);
}

.index-state.error {
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.index-state.disabled,
.status-pill.disabled {
  background: var(--app-control-solid);
  color: var(--app-text-subtle);
}

.error-text {
  @apply mt-3 rounded-md border px-3 py-2 text-sm;
  border-color: var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.icon-button,
.primary-button,
.plain-button,
.danger-button {
  @apply inline-flex h-9 shrink-0 items-center justify-center gap-2 rounded-md px-3 text-sm font-medium outline-none transition;
}

.icon-button {
  @apply w-9 px-0;
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.icon-button:hover,
.plain-button:hover {
  background: var(--app-control-hover);
  color: var(--app-text);
}

.primary-button {
  background: var(--app-accent, #2563eb);
  color: var(--app-accent-contrast);
}

.primary-button:hover {
  background: var(--app-accent-strong);
}

.plain-button {
  border: 1px solid var(--app-border-soft);
  background: var(--app-control-solid);
  color: var(--app-text-muted);
}

.danger-button {
  border: 1px solid var(--app-danger-border);
  background: var(--app-danger-soft);
  color: var(--app-danger-text);
}

.danger-button:hover {
  background: color-mix(in srgb, var(--app-danger) 16%, var(--app-panel-solid));
}

.icon-button:focus-visible,
.primary-button:focus-visible,
.plain-button:focus-visible,
.danger-button:focus-visible,
.nav-item:focus-visible {
  box-shadow: 0 0 0 2px var(--app-accent-ring, rgba(37, 99, 235, 0.2));
}

.icon-button:disabled,
.primary-button:disabled,
.plain-button:disabled,
.danger-button:disabled {
  @apply cursor-not-allowed opacity-50;
}

.empty,
.empty-inline {
  @apply rounded-md border border-dashed px-3 py-6 text-center text-sm;
  border-color: var(--app-border-soft);
  color: var(--app-text-subtle);
}

.empty-inline {
  @apply py-4;
}

.shell-notice-layer {
  @apply pointer-events-none fixed inset-x-0 bottom-12 z-[70] flex justify-center px-4;
}

.shell-notice-layer :deep(.shell-notice) {
  @apply pointer-events-auto;
}

.shell-notice-pop-enter-active,
.shell-notice-pop-leave-active {
  transition:
      opacity 0.14s ease,
      transform 0.16s cubic-bezier(0.2, 0, 0, 1);
}

.shell-notice-pop-enter-from,
.shell-notice-pop-leave-to {
  opacity: 0;
  transform: translateY(0.5rem) scale(0.98);
}

@media (prefers-reduced-motion: reduce) {
  .shell-notice-pop-enter-active,
  .shell-notice-pop-leave-active {
    transition: none;
  }
}

@media (max-width: 1120px) {
  .settings-shell {
    grid-template-columns: 11rem minmax(0, 1fr);
  }

  .status-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .overview-grid {
    grid-template-columns: 1fr;
  }

  .index-security-layout {
    grid-template-columns: 1fr;
  }

  .mapping-summary {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

}

@media (max-width: 760px) {
  .settings-topbar {
    @apply h-auto flex-wrap py-2;
  }

  .topbar-actions {
    @apply w-full;
  }

  .settings-actionbar {
    @apply items-start;
  }

  .settings-actionbar .topbar-actions {
    @apply justify-start;
  }

  .settings-shell {
    @apply p-3;
    grid-template-columns: 1fr;
  }

  .settings-sidebar {
    @apply static h-auto flex-row overflow-x-auto;
  }

  .config-split {
    grid-template-columns: 1fr;
  }

  .nav-item {
    @apply shrink-0;
  }

  .form-grid,
  .status-grid,
  .mapping-summary,
  .password-form,
  .pending-row {
    grid-template-columns: 1fr;
  }

  .setting-field.wide {
    @apply col-span-1;
  }

}
</style>
