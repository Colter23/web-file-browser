# Web File Browser

网页本地文件浏览器。后端使用 Rust，前端在 `ui` 目录中使用 Vue。

项目边界和后续开发取舍见 [docs/PROJECT_BOUNDARY.md](docs/PROJECT_BOUNDARY.md)，长期路线图见 [docs/ROADMAP.md](docs/ROADMAP.md)，可执行任务清单见 [docs/TASK_BACKLOG.md](docs/TASK_BACKLOG.md)，后端 API 契约见 [docs/API_CONTRACT.md](docs/API_CONTRACT.md)，API 错误码见 [docs/API_ERRORS.md](docs/API_ERRORS.md)，Linux Docker 部署见 [docs/DOCKER_DEPLOY.md](docs/DOCKER_DEPLOY.md)。

当前仍处于开发期，模型或接口设计不合理时优先破坏式重构，不为旧接口形状长期保留适配层。项目注释、文档、日志、错误提示、前端提示性文本和管理界面文案默认使用中文；后续改动触及旧英文注释或提示文案时也应同步中文化。

## 协作约定

后续开发按后端和前端分工推进：本线程默认负责 Rust 后端、API 契约、部署脚本、接口文档和后端验证；前端页面、组件和交互由另一个 AI 负责。后端如果破坏式调整接口，需要同步更新文档，说明请求字段、响应字段、错误码和前端调用影响。

## 后端开发

```powershell
cargo run
```

默认监听 `http://localhost:8080`。

运行配置优先级为：默认值 < `data/config.json` < 环境变量。日常部署建议把多数运行参数写进 `data/config.json`，环境变量只用于 Docker 覆盖端口、UID/GID 或临时调试。

最小 `data/config.json` 示例：

```json
{
  "server": {
    "bind": "0.0.0.0",
    "port": 8080
  },
  "limits": {
    "maxDirPageSize": 2000,
    "maxUploadBytes": null
  },
  "conflictPolicy": "autoRename"
}
```

认证状态单独保存在 `data/auth.json`。首次进入 Web 页面且尚未设置密码时，前端应引导用户设置单管理员密码；后端只保存 Argon2 哈希。重置密码时停止服务，删除 `data/auth.json`，重启后重新进入 Web 页面设置即可。

可用环境变量：

- `PORT`：服务端口，默认 `8080`
- `WEB_FILE_BROWSER_BIND`：监听地址，默认 `0.0.0.0`
- `WEB_FILE_BROWSER_MAPPING_FILE`：路径映射文件，默认 `data/mappings.json`
- `WEB_FILE_BROWSER_CONFIG_FILE`：运行配置文件，默认 `data/config.json`
- `WEB_FILE_BROWSER_AUTH_FILE`：认证哈希文件，默认 `data/auth.json`
- `WEB_FILE_BROWSER_TRASH_DIR`：自管回收站目录，默认 `data/trash`
- `WEB_FILE_BROWSER_STATIC_DIR`：前端静态资源目录，默认 `ui/dist`
- `WEB_FILE_BROWSER_CORS_ORIGINS`：允许跨域访问的可信来源，逗号分隔，默认空表示只支持同源
- `WEB_FILE_BROWSER_TRUST_PROXY_HEADERS`：是否信任 `X-Forwarded-For`，默认 `false`；仅在服务只通过可信反向代理访问时启用
- `WEB_FILE_BROWSER_MAX_EDIT_BYTES`：在线编辑文件字节上限，默认 `2097152`
- `WEB_FILE_BROWSER_EDITABLE_EXTENSIONS`：允许在线编辑的扩展名白名单，逗号分隔，可写 `txt,md,json` 或 `.txt,.md`；默认空表示不限制扩展名
- `WEB_FILE_BROWSER_EDITABLE_MIME_TYPES`：允许在线编辑的 MIME 白名单，逗号分隔，支持 `text/*` 这类分组；默认空表示不限制 MIME
- `WEB_FILE_BROWSER_MAX_UPLOAD_BYTES`：上传和保存的单文件字节上限，默认不限制
- `WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE`：目录分页单次返回上限，默认 `2000`
- `WEB_FILE_BROWSER_MAX_DIR_CONCURRENCY`：目录扫描并发上限，默认 `4`
- `WEB_FILE_BROWSER_MAX_TRANSFER_CONCURRENCY`：文件传输并发上限，默认 `8`
- `WEB_FILE_BROWSER_MAX_IP_CONCURRENCY`：单 IP 受保护 API 并发上限，默认 `16`
- `WEB_FILE_BROWSER_MAX_TASK_CONCURRENCY`：后台任务执行并发上限，默认 `2`
- `WEB_FILE_BROWSER_TASK_HISTORY_LIMIT`：内存中保留的已结束后台任务数量，默认 `200`
- `WEB_FILE_BROWSER_TASK_SPEED_LIMIT_BYTES_PER_SEC`：后台任务单任务限速，默认不限制
- `WEB_FILE_BROWSER_MAX_EXTRACT_BYTES`：单次解压后的总字节上限，默认不限制
- `WEB_FILE_BROWSER_MAX_EXTRACT_FILES`：单次解压的条目数量上限，默认不限制
- `WEB_FILE_BROWSER_MAX_EXTRACT_DEPTH`：单个解压条目的路径深度上限，默认 `64`
- `WEB_FILE_BROWSER_INDEX_ENABLED`：是否启用后台搜索索引，默认 `false`
- `WEB_FILE_BROWSER_INDEX_REBUILD_ON_STARTUP`：启动时是否自动重建搜索索引，默认 `false`
- `WEB_FILE_BROWSER_INDEX_SCAN_DELAY_MS`：索引扫描每个目录后的节流延迟，默认 `2`
- `WEB_FILE_BROWSER_AUDIT_FILE`：审计日志 JSONL 文件，默认 `data/audit.jsonl`
- `WEB_FILE_BROWSER_AUDIT_MAX_BYTES`：审计日志单文件轮转上限，默认 `10485760`，设置为 `0` 可关闭轮转
- `WEB_FILE_BROWSER_AUDIT_RETENTION_FILES`：保留的审计轮转文件数量，默认 `8`，设置为 `0` 表示只保留当前审计文件
- `WEB_FILE_BROWSER_TRASH_RETENTION_DAYS`：回收站按天保留，默认不限制
- `WEB_FILE_BROWSER_TRASH_MAX_BYTES`：回收站总大小上限，默认不限制
- `WEB_FILE_BROWSER_CONFLICT_POLICY`：文件冲突策略，支持 `autoRename`、`reject`、`overwrite`，默认 `autoRename`

## 前端开发

```powershell
cd ui
yarn install
yarn dev
```

前端默认使用同源 `/api`。开发模式下 Vite 会把 `/api` 代理到 `http://127.0.0.1:8080`。如需修改 API 地址，可以设置 `VITE_API_BASE_URL`。
如果开发环境必须跨域直连后端，需要在后端设置 `WEB_FILE_BROWSER_CORS_ORIGINS`，例如 `http://localhost:5173`；不支持使用 `*`。

## Docker 部署

```bash
cp env.example .env
docker compose up -d --build
```

首次启动后请打开 Web 页面完成管理员密码设置。默认数据卷是 `./data:/app/data`，业务文件示例目录是 `./files:/mnt/files`。
在 Linux Docker 环境中可以运行 `scripts/docker-smoke.sh` 做自动冒烟验证。脚本会使用 `.smoke/docker` 临时目录，覆盖前端静态托管、登录、挂载、编辑保存、下载、上传、zip/tar.gz 压缩和解压、删除/恢复和指标接口；默认端口为 `18080`，运行结束后会清理容器和临时数据。
需要验证更接近真实规模的目录和传输场景时，可以运行 `scripts/docker-perf-smoke.sh`。脚本默认创建 1 万个目录项、上传下载 64 MiB 文件，并验证 Range、在线编辑保护、tar.gz 压缩解压、回收站恢复和指标接口；默认端口为 `18081`。该脚本已在真实 Linux Docker 环境按默认参数通过。

## 后端结构

- `src/main.rs`：启动入口
- `src/config.rs`：运行配置文件、环境变量和启动配置
- `src/app.rs`：Axum 应用组装、CORS、静态资源托管
- `src/error.rs`：统一错误响应
- `src/models.rs`：接口模型
- `src/routes/`：HTTP 路由层
- `src/services/`：路径解析、流式传输、挂载映射、认证、设置、文件操作、回收站等业务逻辑

## 已实现接口

- `GET /api/`
- `POST /api/auth/setup`
- `POST /api/auth/login`
- `POST /api/auth/logout`
- `GET /api/auth/session`
- `POST /api/auth/password`
- `GET /api/health`
- `GET /api/ready`
- `GET /api/metrics`
- `POST /api/audit/cleanup`
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
- `POST /api/tasks/archive`
- `POST /api/tasks/extract`
- `GET /api/tasks`
- `POST /api/tasks/cleanup`
- `GET /api/tasks/{id}`
- `POST /api/tasks/{id}/cancel`
- `GET /api/search?q=&mount=&type=&limit=&offset=`
- `GET /api/recent?limit=`
- `POST /api/index/rebuild`
- `POST /api/index/cancel`
- `GET /api/index/status`
- `GET /api/trash`
- `POST /api/trash/cleanup`
- `POST /api/trash/{id}/restore`
- `DELETE /api/trash/{id}`
- `POST /api/trash/empty`

路径映射会保存到 `data/mappings.json`。
运行配置默认读取 `data/config.json`；认证哈希保存到 `data/auth.json`。
`GET /api/file/{path...}` 只返回目录或文件元数据；文件内容读取请使用 `GET /api/content/{path...}`。
目录元数据默认分页，支持 `offset`、`limit`、`detail=basic|full`、`sort=name|modified|size`、`order=asc|desc`、`type=all|file|folder`、`includeHidden=true` 和 `includeTotal=true` 查询参数。默认不统计 `folderTotal/fileTotal`，以减少大目录额外开销；请求 `includeTotal=true` 时才返回总数。分页请求会返回 `hasMore`。按名称排序的轻量目录读取只保留当前页所需的排序窗口，不把整页之外的条目长期留在内存中。
`GET /api/file/{path...}` 元数据响应会返回 `ETag` 和可用时的 `Last-Modified`；条件请求支持 `If-None-Match` 和 `If-Modified-Since`，缓存命中时返回 `304`。默认 `detail=basic` 的目录 `Last-Modified` 来自目录自身修改时间，不为了响应头额外 `stat` 子项；没有 `If-None-Match` 且 `If-Modified-Since` 命中时，会跳过目录 JSON 序列化。
`GET /api/content/{path...}` 和 `GET /api/download/{path...}` 支持单段 `Range` 请求，并返回内容 `ETag`。
编辑器读取文件时应使用 `GET /api/content/{path...}?mode=edit`，后端会拒绝超过编辑上限或看起来不是文本的文件；普通预览仍可使用默认 `mode=raw` 保持流式读取。
`PUT /api/content/{path...}` 必须携带 `If-Match`，保存时会校验文件是否被外部修改，并受在线编辑上限保护；缺少版本标记返回 `428`，版本过期返回 `412`。在线编辑可通过扩展名或 MIME 白名单收紧允许范围，默认仍使用大小上限、文本采样和保存分块校验保护。
新建、上传、移动、复制和回收站恢复支持冲突策略，默认自动重命名；请求级可使用 `conflict=autoRename|reject|overwrite` 或 `conflictPolicy` 指定。显式覆盖文件时会尽量先保留旧目标，只有新文件落到目标路径后才清理旧文件。
删除文件会移动到 `data/trash`，并记录原路径、删除时间、操作者、文件类型和可低成本得到的大小；普通文件会记录 `sizeBytes`，文件夹不为了统计大小而额外预扫。
回收站恢复会按当前挂载配置重新解析原虚拟路径，要求目标父目录仍在可写挂载中；恢复会返回实际恢复路径，如果默认自动重命名生效，`restoredVirtualPath` 会是重命名后的路径。
回收站保留策略不会在启动时自动扫描回收站内容；`POST /api/trash/cleanup` 会显式执行清理，列表接口也会按间隔触发一次保留检查。容量清理会优先使用记录中的 `sizeBytes`，只有缺少大小的记录才按需计算。
批量复制、移动、删除、压缩和解压会创建后台任务，任务状态保存在内存中；大文件读写按块推进，并受任务并发/限速配置保护。复制任务会先写入同目录临时路径，成功后再移动或替换目标，取消或失败时清理临时输出；移动任务在每个条目执行真实移动前会重新检查取消状态。
任务状态会在已有处理循环中更新 `currentPath`，用于显示当前处理条目；任务结束后会清空，不会为了任务面板额外扫描目录。
后台任务只保留最近的已结束任务历史，默认上限为 200 条；排队和运行中的任务不会被历史清理移除。`POST /api/tasks/cleanup` 可主动清理当前已结束任务历史，不会删除排队或运行中的任务。
`POST /api/tasks/archive` 支持 `tarGz` 和 `zip`，输出文件会先写入同目录临时文件，完成后再原子移动或替换。`POST /api/tasks/extract` 支持 `zip`、`tar.gz` 和 `tgz`，默认在目标目录下创建压缩包同名文件夹，解压时会拒绝路径穿越、符号链接、不支持的条目类型和超过深度上限的条目。
搜索索引当前为可选内存索引，后续可替换为 SQLite 存储。
启用搜索索引不会自动触发启动全盘扫描；只有显式设置 `WEB_FILE_BROWSER_INDEX_REBUILD_ON_STARTUP=true` 才会在启动后重建索引。
手动重建索引可通过 `POST /api/index/cancel` 取消，扫描只在目录边界检查取消状态，不强行中断正在执行的单个文件系统调用。
创建、保存、上传、移动、删除、恢复和后台任务输出会对内存索引做轻量增量更新；复制文件夹和解压目录只刷新输出根路径，不会为了索引递归扫描全部子项。
挂载创建、更新和删除会清理相关挂载的内存索引，避免旧映射结果残留；不会自动扫描新挂载目录。
搜索查询分页只克隆当前页结果；最近文件查询也不会先克隆整个索引再排序。
未初始化管理员密码时，`GET /api/auth/session` 会返回 `authConfigured=false`，前端应展示首次设置页面并调用 `POST /api/auth/setup`。登录后可通过 `POST /api/auth/password` 修改单管理员密码；修改成功后旧会话会失效，当前请求会获得新会话。服务端会话与 Cookie 一样默认 7 天有效，登录、鉴权和指标统计会懒清理过期会话。
默认不开放跨域 CORS；只有 `WEB_FILE_BROWSER_CORS_ORIGINS` 显式配置的来源才能带凭据跨域访问 API。
`GET /api/health` 是轻量存活检查，只确认进程可响应；`GET /api/ready` 是就绪检查，会验证配置、认证哈希、映射、回收站、审计和静态入口文件是否可用。管理员密码尚未初始化时仍返回就绪，便于首次进入 Web 页面完成设置。
审计日志默认写入 append-only JSONL，达到 `WEB_FILE_BROWSER_AUDIT_MAX_BYTES` 后会在下一条审计写入前轮转为 `audit.<时间戳>.jsonl`。旧轮转文件只在发生新轮转后按 `WEB_FILE_BROWSER_AUDIT_RETENTION_FILES` 清理，不在启动时扫描审计目录。`POST /api/audit/cleanup` 可主动清理旧轮转文件，并会写入一条轻量审计记录。
`GET /api/metrics` 返回轻量内存指标快照：映射数量、活跃会话、任务状态汇总、目录扫描/文件传输/IP 并发占用、回收站条目数量和搜索索引状态；默认不递归估算回收站大小，避免指标接口触发额外磁盘遍历。单 IP 并发保护会顺手清理空闲 IP 记录，`trackedIps` 只反映仍有活跃请求的限流记录。
API 错误响应统一为 `{ "code": "...", "message": "..." }`，稳定错误码和前端处理建议见 [docs/API_ERRORS.md](docs/API_ERRORS.md)。

## 固定验证

```powershell
cargo fmt
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cd ui
yarn.cmd build
```

Linux Docker 环境可额外执行：

```bash
bash scripts/docker-smoke.sh
bash scripts/docker-perf-smoke.sh
```
