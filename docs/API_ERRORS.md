# API 错误码

后端错误响应统一返回 JSON：

```json
{
  "code": "BAD_REQUEST",
  "reason": "PATH_SEGMENT_INVALID",
  "message": "路径不能包含 .. 或路径分隔符",
  "params": {
    "segment": ".."
  }
}
```

`code` 是 HTTP/大类错误，适合脚本和通用分支判断；`reason` 是更细的业务错误 key，前端国际化应优先使用它；`message` 是面向管理员的中文兜底提示；`params` 是可选的插值参数。当前项目仍处于开发期，可以破坏式调整接口；但新增或修改错误时，应优先保持 `code` 和高频 `reason` 稳定，并同步更新本文档和相关测试。

## 通用规则

- 受保护的 `/api/*` 接口未登录时返回 `UNAUTHORIZED`。
- `GET /api/health` 和登录接口不要求认证。
- `GET /api/ready` 是就绪检查，失败时返回非 2xx，但仍使用 JSON 描述检查项。
- 后台任务创建接口通常先返回任务 `id`，单个路径失败会写入任务 `errors` 数组；任务错误项同样包含 `code`、`reason`、`message` 和可选 `params`。
- 回收站批量恢复/永久删除的 `errors` 数组也包含 `code`、`reason`、`message` 和可选 `params`。
- `GET /api/file/*`、`GET /api/content/*`、`GET /api/download/*` 可能返回 `304 Not Modified`，这不是错误响应，通常没有错误 JSON。
- `DELETE` 永久清理成功可能返回 `204 No Content`，也没有错误 JSON。

## 稳定错误码

| HTTP 状态 | code | 常见场景 | 处理建议 |
| --- | --- | --- | --- |
| 400 | `BAD_REQUEST` | 参数格式错误、路径非法、任务路径或输出名为空、跨挂载移动、压缩包条目非法、搜索索引未启用 | 前端优先按 `reason` 翻译，调用方修正请求参数 |
| 401 | `UNAUTHORIZED` | 未登录、会话失效、管理员密码错误 | 跳转登录页或提示重新登录 |
| 403 | `FORBIDDEN` | 只读挂载写入、无写入权限 | 展示只读提示，避免继续重试写操作 |
| 404 | `NOT_FOUND` | 文件、映射、任务、回收站记录不存在 | 刷新当前视图或提示目标已不存在 |
| 409 | `CONFLICT` | 名称冲突、任务已取消、任务已结束不能取消、索引正在重建、当前没有可取消的索引重建、禁止覆盖目录 | 根据场景提示用户选择自动重命名、拒绝或显式覆盖 |
| 413 | `PAYLOAD_TOO_LARGE` | 上传、保存或解压超过配置上限 | 展示大小限制，必要时调整环境变量 |
| 415 | `UNSUPPORTED_MEDIA_TYPE` | 在线编辑二进制文件或不允许的文件类型 | 提示文件不适合在线编辑 |
| 416 | `RANGE_NOT_SATISFIABLE` | Range 头非法、越界或多段 Range | 重新发起完整读取或修正单段 Range |
| 428 | `PRECONDITION_REQUIRED` | 保存内容时缺少 `If-Match` | 重新打开文件，带最新 ETag 保存 |
| 412 | `PRECONDITION_FAILED` | 文件已被外部修改或 ETag 过期 | 提示刷新后再保存，避免覆盖外部修改 |
| 429 | `TOO_MANY_REQUESTS` | 目录扫描、文件传输或单 IP 并发超过限制 | 稍后重试，前端避免并发风暴 |
| 500 | `INTERNAL_ERROR` | 文件系统异常、配置写入异常、任务内部异常 | 展示 `message`，查看服务日志和挂载权限 |

## 前端处理建议

- 网络层优先读取 `code`，缺少 `code` 时再按 HTTP 状态兜底。
- 展示文案优先读取 `reason` 并使用 `params` 插值；没有对应翻译时展示后端中文 `message`。
- `UNAUTHORIZED` 统一触发会话刷新或跳转登录。
- `PRECONDITION_REQUIRED` 和 `PRECONDITION_FAILED` 属于编辑保存保护，不应静默重试。
- `RANGE_NOT_SATISFIABLE` 只影响预览或下载分段读取，可以退回完整请求。
- `TOO_MANY_REQUESTS` 不应立即并发重试，应做短暂退避。
- 高频接口如目录列表、预览和下载不应额外写审计日志；错误提示也应避免制造新的高频请求。

## 后端维护要求

- 新增 `AppError` 变体时，需要补充错误码表。
- 调整 HTTP 状态、`code` 或高频 `reason` 时，需要更新 `src/error.rs` 和相关 API 冒烟测试。
- 业务错误尽量使用已有稳定 `code`，只为前端需要区分展示或处理的高频场景新增 `reason`。
- 面向管理员的 `message` 使用中文；协议字段、HTTP 头、环境变量、`code` 和 `reason` 保持英文。

## 高频 reason

| reason | 典型场景 | params |
| --- | --- | --- |
| `AUTH_REQUIRED` | 未登录访问受保护接口 | 无 |
| `ADMIN_PASSWORD_NOT_CONFIGURED` | 登录时管理员密码尚未初始化 | 无 |
| `ADMIN_PASSWORD_INCORRECT` | 登录密码错误 | 无 |
| `PASSWORD_TOO_SHORT` | 首次设置或修改密码长度不足 | `field`, `minLength` |
| `MOUNT_READONLY` | 对只读挂载执行写操作 | 无 |
| `PATH_NOT_FOUND` | 文件浏览、下载、编辑等操作中的虚拟路径不存在 | `path` |
| `MAPPING_FOLDER_PATH_NOT_FOUND` | 创建或更新挂载时，本地 `folderPath` 不存在或不可访问 | `path` |
| `PATH_SEGMENT_INVALID` | 路径片段包含 `..`、反斜杠或分隔符 | `segment` |
| `PATH_OUTSIDE_MOUNT` | 真实路径 canonical 后越过挂载根 | 无 |
| `PAGE_SIZE_MUST_BE_POSITIVE` | `limit=0` | `field` |
| `CROSS_MOUNT_MOVE_FORBIDDEN` | 跨挂载移动文件 | `sourcePath`, `targetPath` |
| `PATH_ALREADY_EXISTS` | 冲突策略为拒绝或落盘前目标已存在 | `path` |
| `OVERWRITE_DIR_FORBIDDEN` | 显式覆盖目录 | 无 |
| `TRANSFER_CONCURRENCY_LIMITED` | 文件传输并发超过限制 | `limit` |
| `DIR_SCAN_CONCURRENCY_LIMITED` | 目录扫描并发超过限制 | `limit` |
| `IP_CONCURRENCY_LIMITED` | 单 IP 并发超过限制 | `limit` |
| `RANGE_START_OUT_OF_BOUNDS` | Range 起点超过文件大小 | `start`, `size` |
| `UPLOAD_SIZE_LIMIT_EXCEEDED` | 上传或保存请求超过大小限制 | `maxBytes`, `contentLength` 或 `writtenBytes` |
| `EDIT_SIZE_LIMIT_EXCEEDED` | 文件超过在线编辑上限 | `path`, `size`, `maxBytes` |
| `EDIT_FILE_TYPE_NOT_ALLOWED` | 扩展名或 MIME 不允许在线编辑 | `path` |
| `EDIT_FILE_NOT_TEXT` | 文件内容探测不像文本 | `path` |
| `IF_MATCH_REQUIRED` | 保存内容缺少 `If-Match` | 无 |
| `FILE_MODIFIED` | 保存前 ETag 已变化 | `path` |
| `SEARCH_INDEX_DISABLED` | 搜索索引关闭时请求重建 | 无 |
| `SEARCH_INDEX_SCANNING` | 索引正在重建 | 无 |
| `TASK_FIELD_REQUIRED` | 任务创建缺少必填字段 | `field` |
| `TASK_LIST_EMPTY` | 任务创建列表为空 | `field` |
| `TASK_SCHEDULER_UNAVAILABLE` | 后台任务调度器不可用 | 无 |
| `TRASH_RECORD_NOT_FOUND` | 回收站记录不存在 | `id` |
| `TRASH_RESTORE_TARGET_EXISTS` | 回收站恢复目标已存在 | `path` |
