# Web File Browser

网页本地文件浏览器。后端使用 Rust，前端在 `ui` 目录中使用 Vue。

## 后端开发

```powershell
cargo run
```

默认监听 `http://localhost:8080`。

可用环境变量：

- `PORT`：服务端口，默认 `8080`
- `WEB_FILE_BROWSER_BIND`：监听地址，默认 `0.0.0.0`
- `WEB_FILE_BROWSER_MAPPING_FILE`：路径映射文件，默认 `data/mappings.json`
- `WEB_FILE_BROWSER_CONFIG_FILE`：应用配置文件，默认 `data/config.json`
- `WEB_FILE_BROWSER_TRASH_DIR`：自管回收站目录，默认 `data/trash`
- `WEB_FILE_BROWSER_ADMIN_PASSWORD`：首次启动时初始化管理员密码
- `WEB_FILE_BROWSER_STATIC_DIR`：前端静态资源目录，默认 `ui/dist`
- `WEB_FILE_BROWSER_MAX_UPLOAD_BYTES`：上传和保存的单文件字节上限，默认不限制
- `WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE`：目录分页单次返回上限，默认 `2000`
- `WEB_FILE_BROWSER_MAX_DIR_CONCURRENCY`：目录扫描并发上限，默认 `4`
- `WEB_FILE_BROWSER_MAX_TRANSFER_CONCURRENCY`：文件传输并发上限，默认 `8`
- `WEB_FILE_BROWSER_MAX_IP_CONCURRENCY`：单 IP 受保护 API 并发上限，默认 `16`
- `WEB_FILE_BROWSER_MAX_TASK_CONCURRENCY`：后台任务执行并发上限，默认 `2`
- `WEB_FILE_BROWSER_TASK_SPEED_LIMIT_BYTES_PER_SEC`：后台复制任务单任务限速，默认不限制
- `WEB_FILE_BROWSER_INDEX_ENABLED`：是否启用后台搜索索引，默认 `false`
- `WEB_FILE_BROWSER_INDEX_SCAN_DELAY_MS`：索引扫描每个目录后的节流延迟，默认 `2`
- `WEB_FILE_BROWSER_AUDIT_FILE`：审计日志 JSONL 文件，默认 `data/audit.jsonl`
- `WEB_FILE_BROWSER_TRASH_RETENTION_DAYS`：回收站按天保留，默认不限制
- `WEB_FILE_BROWSER_TRASH_MAX_BYTES`：回收站总大小上限，默认不限制

首次启用认证时建议这样启动：

```powershell
$env:WEB_FILE_BROWSER_ADMIN_PASSWORD="change-me"
cargo run
```

密码只会以 Argon2 哈希写入 `data/config.json`。配置文件已经存在管理员密码哈希后，后续启动不再读取该环境变量覆盖密码。

## 前端开发

```powershell
cd ui
yarn install
yarn dev
```

前端默认使用同源 `/api`。开发模式下 Vite 会把 `/api` 代理到 `http://127.0.0.1:8080`。如需修改 API 地址，可以设置 `VITE_API_BASE_URL`。

## 后端结构

- `src/main.rs`：启动入口
- `src/config.rs`：环境变量和启动配置
- `src/app.rs`：Axum 应用组装、CORS、静态资源托管
- `src/error.rs`：统一错误响应
- `src/models.rs`：接口模型
- `src/routes/`：HTTP 路由层
- `src/services/`：路径解析、流式传输、挂载映射、认证、设置、文件操作、回收站等业务逻辑

## 已实现接口

- `GET /api/`
- `POST /api/auth/login`
- `POST /api/auth/logout`
- `GET /api/auth/session`
- `GET /api/health`
- `GET /api/metrics`
- `GET /api/file`
- `GET /api/file/{path...}`
- `POST /api/file/{path...}`
- `PATCH /api/file/{path...}`
- `DELETE /api/file/{path...}`
- `GET /api/content/{path...}`
- `HEAD /api/content/{path...}`
- `PUT /api/content/{path...}`
- `POST /api/upload/{path...}`
- `GET /api/download/{path...}`
- `HEAD /api/download/{path...}`
- `GET /api/mapping`
- `GET /api/mapping/root`
- `POST /api/mapping`
- `PUT /api/mapping/{id}`
- `DELETE /api/mapping/{id}`
- `GET /api/settings`
- `POST /api/tasks/copy`
- `POST /api/tasks/move`
- `POST /api/tasks/delete`
- `GET /api/tasks`
- `GET /api/tasks/{id}`
- `POST /api/tasks/{id}/cancel`
- `GET /api/search?q=&mount=&type=&limit=&offset=`
- `GET /api/recent?limit=`
- `POST /api/index/rebuild`
- `GET /api/index/status`
- `GET /api/trash`
- `POST /api/trash/{id}/restore`
- `DELETE /api/trash/{id}`
- `POST /api/trash/empty`

路径映射会保存到 `data/mappings.json`。
`GET /api/file/{path...}` 只返回目录或文件元数据；文件内容读取请使用 `GET /api/content/{path...}`。
目录元数据默认分页，支持 `offset`、`limit`、`detail=basic|full`、`sort=name|modified|size`、`order=asc|desc`、`type=all|file|folder` 和 `includeHidden=true` 查询参数。分页请求会返回 `folderTotal`、`fileTotal` 和 `hasMore`。
`GET /api/content/{path...}` 和 `GET /api/download/{path...}` 支持单段 `Range` 请求。
删除文件会移动到 `data/trash`，并记录原路径、删除时间、操作者和文件类型。
批量复制、移动、删除会创建后台任务，任务状态保存在内存中；复制任务按块更新速度和已处理字节，并受任务并发/限速配置保护。搜索索引当前为可选内存索引，后续可替换为 SQLite 存储。

## 固定验证

```powershell
cargo fmt
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cd ui
yarn.cmd build
```
