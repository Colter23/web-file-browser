# API 错误码

后端错误响应统一返回 JSON：

```json
{
  "code": "BAD_REQUEST",
  "message": "路径不能包含 .. 或路径分隔符"
}
```

`code` 用于前端和脚本判断错误类型，`message` 是面向管理员的中文提示。当前项目仍处于开发期，可以破坏式调整接口；但新增或修改错误时，应优先保持 `code` 稳定，并同步更新本文档和相关测试。

## 通用规则

- 受保护的 `/api/*` 接口未登录时返回 `UNAUTHORIZED`。
- `GET /api/health` 和登录接口不要求认证。
- `GET /api/ready` 是就绪检查，失败时返回非 2xx，但仍使用 JSON 描述检查项。
- 后台任务创建接口通常先返回任务 `id`，单个路径失败会写入任务 `errors` 数组；任务错误中的 `message` 仍使用中文。
- `GET /api/file/*`、`GET /api/content/*`、`GET /api/download/*` 可能返回 `304 Not Modified`，这不是错误响应，通常没有错误 JSON。
- `DELETE` 永久清理成功可能返回 `204 No Content`，也没有错误 JSON。

## 稳定错误码

| HTTP 状态 | code | 常见场景 | 处理建议 |
| --- | --- | --- | --- |
| 400 | `BAD_REQUEST` | 参数格式错误、路径非法、任务路径或输出名为空、跨挂载移动、压缩包条目非法、搜索索引未启用 | 前端展示 `message`，调用方修正请求参数 |
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
- `UNAUTHORIZED` 统一触发会话刷新或跳转登录。
- `PRECONDITION_REQUIRED` 和 `PRECONDITION_FAILED` 属于编辑保存保护，不应静默重试。
- `RANGE_NOT_SATISFIABLE` 只影响预览或下载分段读取，可以退回完整请求。
- `TOO_MANY_REQUESTS` 不应立即并发重试，应做短暂退避。
- 高频接口如目录列表、预览和下载不应额外写审计日志；错误提示也应避免制造新的高频请求。

## 后端维护要求

- 新增 `AppError` 变体时，需要补充错误码表。
- 调整 HTTP 状态或 `code` 时，需要更新 `src/error.rs` 的错误响应测试。
- 业务错误尽量使用已有稳定 code，不为每一种中文提示新增细碎 code。
- 面向管理员的 `message` 使用中文；协议字段、HTTP 头、环境变量和 code 保持英文。
