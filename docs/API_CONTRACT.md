# 后端 API 契约

这份文档面向前端协作者，记录当前可直接对接的后端 API。项目仍处于开发期，接口可以破坏式重构；一旦后端调整请求字段、响应字段、错误码或默认行为，应同步更新本文档。

## 通用约定

- API 前缀为 `/api`，前端默认同源调用，不需要额外配置 base URL。
- 除 `GET /api/`、`GET /api/health`、`GET /api/ready`、`POST /api/auth/setup`、`POST /api/auth/login`、`GET /api/auth/session` 外，其余接口默认要求登录。
- 认证使用 Cookie：登录成功后后端写入 `wfb_session`，前端请求需要带凭据。
- 错误响应统一为 `{ "code": "...", "message": "..." }`，错误码详见 [API_ERRORS.md](API_ERRORS.md)。
- 路径参数 `{path...}` 使用虚拟路径去掉开头 `/` 后的部分，例如虚拟路径 `/files/a.txt` 对应 `/api/file/files/a.txt`。
- 写操作冲突策略支持 `autoRename`、`reject`、`overwrite`。默认来自后端配置，当前默认是 `autoRename`。
- 写操作可用查询参数 `conflictPolicy=` 或 `conflict=` 指定冲突策略；部分 JSON 请求体也支持 `conflictPolicy`。
- 文件内容传输走流式接口。`GET /api/file/*` 只返回元数据，不返回文件字节。
- 时间字段当前多为 Unix 秒字符串或 RFC3339 字符串，前端展示时应按字段语义转换，不要假设所有时间格式相同。

## 认证

### `POST /api/auth/setup`

仅在管理员密码尚未初始化时可用。前端应先调用 `GET /api/auth/session`，当 `authConfigured=false` 时展示首次设置密码页面。

请求：

```json
{
  "password": "管理员密码"
}
```

成功返回 `200`，并通过 `Set-Cookie` 写入会话：

```json
{
  "authenticated": true,
  "authConfigured": true
}
```

常见错误：`400 BAD_REQUEST` 密码长度少于 8 个字符，`409 CONFLICT` 管理员密码已经初始化。后端只把 Argon2 哈希保存到 `data/auth.json`，不会保存明文密码。

### `POST /api/auth/login`

请求：

```json
{
  "password": "管理员密码"
}
```

成功返回 `200`，并通过 `Set-Cookie` 写入会话：

```json
{
  "authenticated": true,
  "authConfigured": true
}
```

常见错误：`401 UNAUTHORIZED` 密码错误，`409 CONFLICT` 管理员密码未初始化。未初始化时前端应转到首次设置页面并调用 `POST /api/auth/setup`。

### `POST /api/auth/logout`

成功返回 `200`，并清理会话 Cookie：

```json
{
  "authenticated": false,
  "authConfigured": true
}
```

### `GET /api/auth/session`

返回当前会话状态。未登录也可调用。

```json
{
  "authenticated": false,
  "authConfigured": false
}
```

### `POST /api/auth/password`

请求：

```json
{
  "currentPassword": "旧密码",
  "newPassword": "新密码"
}
```

成功后旧会话全部失效，当前响应会写入新会话 Cookie。新密码至少 8 个字符，不能与旧密码相同。

## 挂载配置

### `GET /api/mapping`

返回挂载列表：

```json
[
  {
    "id": 1,
    "mountPath": "/files",
    "folderPath": "/mnt/files",
    "remark": "文件目录",
    "order": 0,
    "writable": true
  }
]
```

### `POST /api/mapping`

请求：

```json
{
  "mountPath": "/files",
  "folderPath": "/mnt/files",
  "remark": "文件目录",
  "order": 0,
  "writable": true
}
```

成功返回 `201`，响应体是新建映射的数字 `id`。

### `PUT /api/mapping/{id}`

请求体同创建映射。成功返回 `200` 空响应体。

### `DELETE /api/mapping/{id}`

成功返回 `200` 空响应体。

### `GET /api/mapping/root`

返回虚拟挂载树，用于设置页或目录树展示：

```json
{
  "type": "virtual",
  "name": "/",
  "path": "/",
  "children": []
}
```

## 文件元数据和目录浏览

### `GET /api/file`

返回虚拟根目录元数据。

### `GET /api/file/{path...}`

返回目录或文件元数据。目录查询参数：

- `offset`：分页起点，默认 `0`。
- `limit`：分页大小，会被 `WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE` 截断；`0` 会返回错误。
- `detail`：`basic` 或 `full`，默认 `basic`。
- `sort`：`name`、`modified`、`size`，默认 `name`。`modified` 和 `size` 需要 `detail=full`。
- `order`：`asc` 或 `desc`，默认 `asc`。
- `type`：`all`、`file`、`folder`，默认 `all`。
- `includeHidden=true`：返回隐藏文件。
- `includeTotal=true`：返回 `folderTotal` 和 `fileTotal`；默认不统计总数。

目录响应：

```json
{
  "path": "/files",
  "folder": [
    {
      "name": "docs",
      "path": "/files/docs",
      "modified": "1760000000",
      "type": "folder"
    }
  ],
  "file": [
    {
      "name": "a.txt",
      "path": "/files/a.txt",
      "modified": "1760000000",
      "size": 12,
      "extension": "txt",
      "type": "file"
    }
  ],
  "offset": 0,
  "limit": 200,
  "hasMore": false
}
```

文件响应是单个 `FileInfo` 对象。

响应头：元数据接口会尽量返回 `ETag` 和 `Last-Modified`；前端可带 `If-None-Match` 或 `If-Modified-Since`，命中时返回 `304`。

## 文件创建、移动和删除

### `POST /api/file/{path...}`

在指定父目录下新建文件或文件夹。

请求：

```json
{
  "type": "file",
  "name": "new.txt",
  "conflictPolicy": "reject"
}
```

`type` 支持 `file` 和 `folder`。成功返回 `201`：

```json
{
  "path": "/files/new.txt"
}
```

### `PATCH /api/file/{path...}`

重命名或移动条目。

```json
{
  "targetPath": "/files/new-name.txt",
  "conflictPolicy": "autoRename"
}
```

成功返回实际路径：

```json
{
  "path": "/files/new-name.txt"
}
```

不能跨挂载点移动。

### `DELETE /api/file/{path...}`

默认删除到应用自管回收站。带 `permanent=true` 时直接永久删除，不生成回收站记录。成功返回：

```json
{
  "path": "/files/new-name.txt"
}
```

## 文件内容、保存、上传和下载

### `GET /api/content/{path...}`

流式读取文件内容，默认用于预览或原始读取。支持单段 `Range` 请求。

常用查询参数：

- `mode=raw`：默认模式，流式读取。
- `mode=edit`：编辑器读取，会检查文件大小和文本/二进制安全。

响应头通常包含：

- `Content-Type`
- `Content-Length` 或分段传输
- `ETag`
- `Accept-Ranges: bytes`
- `Content-Range`，仅 `206` 时返回

### `HEAD /api/content/{path...}`

只返回内容相关响应头。普通 HEAD 只读取元数据；`mode=edit` 会执行编辑安全检查。

### `PUT /api/content/{path...}`

保存文件内容。必须携带 `If-Match`，建议使用读取内容时返回的 `ETag`。

```http
PUT /api/content/files/a.txt
If-Match: W/"..."
Content-Type: text/plain
```

成功返回新的 `ETag` 和路径：

```json
{
  "path": "/files/a.txt"
}
```

常见错误：`428 PRECONDITION_REQUIRED` 缺少 `If-Match`，`412 PRECONDITION_FAILED` 文件已被外部修改，`413 PAYLOAD_TOO_LARGE` 超过保存上限，`415 UNSUPPORTED_MEDIA_TYPE` 不适合在线编辑。

### `POST /api/upload/{path...}`

multipart 流式上传到目标目录。字段名使用 `file`，可重复上传多个文件。

```bash
curl -F "file=@a.bin;filename=a.bin" /api/upload/files
```

成功返回 `201`：

```json
{
  "files": [
    {
      "path": "/files/a.bin"
    }
  ]
}
```

上传不受 Axum 默认 body limit 限制，实际上限由 `WEB_FILE_BROWSER_MAX_UPLOAD_BYTES` 控制。

### `GET /api/download/{path...}`

附件下载文件，支持单段 `Range`。用于下载按钮时优先使用此接口。

### `HEAD /api/download/{path...}`

只返回下载相关响应头。

## 后台任务

后台任务状态字段：

```json
{
  "id": "uuid",
  "kind": "copy",
  "state": "running",
  "progress": 0.5,
  "processedBytes": 1048576,
  "totalBytes": 2097152,
  "speedBytesPerSec": 123456.0,
  "processedItems": 1,
  "totalItems": 2,
  "currentPath": "/files/a.bin",
  "errors": [
    {
      "path": "/files/b.bin",
      "message": "错误信息"
    }
  ],
  "startedAt": "2026-06-23T00:00:00Z",
  "finishedAt": null,
  "createdAt": "2026-06-23T00:00:00Z",
  "cancelled": false
}
```

`kind` 支持 `copy`、`move`、`delete`、`archive`、`extract`。`state` 支持 `queued`、`running`、`completed`、`failed`、`cancelled`。

### `GET /api/tasks`

返回任务列表，包含运行中和最近已结束任务。

### `GET /api/tasks/{id}`

返回单个任务状态。

### `POST /api/tasks/{id}/cancel`

取消排队或运行中的任务。已结束任务返回 `409 CONFLICT`。

### `POST /api/tasks/cleanup`

清理已结束任务历史，不影响排队或运行中任务。

```json
{
  "removed": 3
}
```

### `POST /api/tasks/copy`

```json
{
  "sources": ["/files/a.txt"],
  "targetPath": "/files/backup",
  "conflictPolicy": "autoRename"
}
```

返回：

```json
{
  "id": "uuid"
}
```

### `POST /api/tasks/move`

请求体同复制任务。移动任务不能跨挂载点移动。

### `POST /api/tasks/delete`

```json
{
  "paths": ["/files/a.txt", "/files/old"],
  "permanent": false
}
```

默认删除进入回收站；`permanent=true` 时直接永久删除。

### `POST /api/tasks/archive`

```json
{
  "sources": ["/files/a.txt", "/files/folder"],
  "targetPath": "/files",
  "outputName": "bundle.tar.gz",
  "format": "tarGz",
  "conflictPolicy": "reject"
}
```

`format` 支持 `tarGz` 和 `zip`。`outputName` 可省略，后端会生成默认文件名。

### `POST /api/tasks/extract`

```json
{
  "sourcePath": "/files/bundle.tar.gz",
  "targetPath": "/files",
  "folderName": "bundle",
  "conflictPolicy": "reject"
}
```

支持 `zip`、`tar.gz` 和 `tgz`。不支持覆盖解压目录。

## 回收站

### `GET /api/trash`

返回回收站记录列表，按删除时间倒序：

```json
[
  {
    "id": "uuid",
    "originalVirtualPath": "/files/a.txt",
    "originalRealPath": "/mnt/files/a.txt",
    "trashPath": "/mnt/files/.web-file-browser-trash/uuid/a.txt",
    "sizeBytes": 12,
    "deletedAt": "2026-06-23T00:00:00Z",
    "actor": "admin",
    "kind": "file"
  }
]
```

### `POST /api/trash/{id}/restore`

可带 `conflictPolicy=` 或 `conflict=` 查询参数。成功返回：

```json
{
  "record": {},
  "restoredVirtualPath": "/files/a (1).txt",
  "restoredRealPath": "/mnt/files/a (1).txt"
}
```

恢复会按当前挂载配置重新解析原虚拟路径，目标父目录必须仍在可写挂载中。

### `POST /api/trash/batch/restore`

批量恢复回收站记录。单条失败不会中断后续记录，适合前端多选操作：

```json
{
  "ids": ["uuid-1", "uuid-2"],
  "conflictPolicy": "autoRename"
}
```

成功返回：

```json
{
  "restored": [],
  "errors": [
    {
      "id": "uuid-2",
      "message": "错误信息"
    }
  ],
  "success": 1,
  "failed": 1
}
```

### `DELETE /api/trash/{id}`

永久删除单条回收站记录。成功返回 `204`。

### `POST /api/trash/batch/purge`

批量永久删除回收站记录。单条失败不会中断后续记录：

```json
{
  "ids": ["uuid-1", "uuid-2"]
}
```

成功返回：

```json
{
  "purged": ["uuid-1"],
  "errors": [
    {
      "id": "uuid-2",
      "message": "错误信息"
    }
  ],
  "success": 1,
  "failed": 1
}
```

### `POST /api/trash/empty`

清空回收站：

```json
{
  "removed": 10
}
```

### `POST /api/trash/cleanup`

按配置的保留天数或容量上限清理回收站：

```json
{
  "removed": 2
}
```

## 搜索和最近文件

搜索索引默认关闭。索引关闭时，重建和取消接口会返回错误；搜索结果可能为空。

### `GET /api/search`

查询参数：

- `q`：搜索关键词。
- `mount`：限定挂载点，例如 `/files`。
- `type`：`file` 或 `folder`。
- `offset`：分页起点，默认 `0`。
- `limit`：分页大小，受 `WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE` 限制。

响应：

```json
{
  "items": [
    {
      "name": "a.txt",
      "path": "/files/a.txt",
      "extension": "txt",
      "modified": "1760000000",
      "size": 12,
      "type": "file",
      "mountPath": "/files"
    }
  ],
  "total": 1,
  "offset": 0,
  "limit": 50
}
```

### `GET /api/recent?limit=50`

返回最近文件数组，单项结构同 `SearchResult`。

### `POST /api/index/rebuild`

开始重建索引，成功返回 `202` 空响应体。索引正在重建时返回 `409 CONFLICT`。

### `POST /api/index/cancel`

取消正在重建的索引，成功返回 `202` 空响应体。

### `GET /api/index/status`

```json
{
  "enabled": true,
  "state": "idle",
  "indexedEntries": 100,
  "lastStartedAt": "2026-06-23T00:00:00Z",
  "lastFinishedAt": "2026-06-23T00:00:10Z",
  "lastError": null
}
```

## 设置、健康检查和指标

### `GET /api/settings`

返回运行配置快照，字段来自 `RuntimeSettings`。前端设置页可用于展示当前限制，不建议把这些字段当作可写配置。

### `GET /api/health`

轻量存活检查，公开访问。

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### `GET /api/ready`

就绪检查，公开访问。目录或关键文件父目录不可用时返回 `503`，但仍返回 JSON。管理员密码尚未初始化时仍返回 `200`，`auth` 检查项会提示等待首次设置，便于前端完成初始化流程。

```json
{
  "status": "ok",
  "version": "0.1.0",
  "checks": [
    {
      "name": "auth",
      "status": "ok",
      "message": "管理员密码已初始化"
    },
    {
      "name": "authStore",
      "status": "ok",
      "message": "目录可写"
    }
  ]
}
```

### `GET /api/metrics`

轻量运行时指标快照，不递归扫描磁盘。

```json
{
  "mappings": 1,
  "activeSessions": 1,
  "trashEntries": 0,
  "tasks": {
    "total": 0,
    "queued": 0,
    "running": 0,
    "completed": 0,
    "failed": 0,
    "cancelled": 0,
    "errorsTotal": 0,
    "processedBytes": 0,
    "currentSpeedBytesPerSec": 0.0
  },
  "limits": {
    "dirScanLimit": 4,
    "activeDirScans": 0,
    "transferLimit": 8,
    "activeTransfers": 0,
    "ipLimit": 16,
    "trackedIps": 1,
    "activeIpRequests": 1
  },
  "index": {
    "enabled": false,
    "state": "disabled",
    "indexedEntries": 0,
    "lastStartedAt": null,
    "lastFinishedAt": null,
    "lastError": null
  }
}
```

## 前端对接建议

- 目录页默认请求 `detail=basic&limit=合理页大小`，只有需要完整排序或属性面板时再请求 `detail=full`。
- 文件编辑流程应先 `GET /api/content/*?mode=edit` 保存响应头 `ETag`，再用 `PUT /api/content/*` 携带 `If-Match`。
- 下载按钮使用 `/api/download/*`；预览或编辑读取使用 `/api/content/*`。
- 写操作遇到 `409 CONFLICT` 时，根据用户选择改用 `autoRename`、`reject` 或 `overwrite` 重试。
- 后台任务创建后应轮询 `GET /api/tasks/{id}` 或刷新 `GET /api/tasks`，不要假设任务立即完成。
- `429 TOO_MANY_REQUESTS` 应做退避，不要立即并发重试。
- `401 UNAUTHORIZED` 应统一跳转登录或刷新会话状态。
