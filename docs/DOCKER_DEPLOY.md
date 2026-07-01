# Linux Docker 部署说明

这份说明面向局域网部署。默认仍保持单管理员、JSON 基础配置、文件传输流式处理，不引入公网分享或多用户能力。

## 镜像内容

Dockerfile 使用三阶段构建：

- 前端阶段：在 Node 镜像中构建 `ui/dist`。
- 后端阶段：在 Rust 镜像中构建 release 二进制。
- 运行阶段：保留 Rust 二进制和前端静态文件，健康检查由应用二进制自身完成。

运行镜像默认：

- 工作目录：`/app`
- 静态文件：`/app/ui/dist`
- 数据目录：`/app/data`
- 文件挂载示例目录：`/mnt/files`
- 服务端口：`8080`
- 健康检查：`GET /api/ready`

## 快速启动

复制环境变量模板：

```bash
cp env.example .env
```

修改 `.env` 中的 UID/GID：

```bash
id -u
id -g
```

准备宿主机目录：

```bash
mkdir -p data files
chown -R "$(id -u):$(id -g)" data files
```

构建并启动：

```bash
docker compose up -d --build
```

可选：把常用运行配置写入 `data/config.json`，避免 `.env` 越来越长：

```json
{
  "server": {
    "bind": "0.0.0.0",
    "port": 8080
  },
  "storage": {
    "favoritesFile": "data/favorites.json"
  },
  "auth": {
    "sessionTtlSeconds": 604800,
    "secureCookie": false
  },
  "limits": {
    "maxUploadBytes": null,
    "maxDirPageSize": 2000,
    "maxDirConcurrency": 4,
    "maxTransferConcurrency": 8,
    "maxIpConcurrency": 16
  },
  "editor": {
    "maxEditBytes": 2097152,
    "editableExtensions": [],
    "editableMimeTypes": []
  },
  "tasks": {
    "maxConcurrency": 2,
    "historyLimit": 200,
    "speedLimitBytesPerSec": null
  },
  "archive": {
    "maxArchiveBytes": null,
    "maxArchiveFiles": null,
    "maxExtractBytes": null,
    "maxExtractFiles": null,
    "maxExtractDepth": 64
  },
  "index": {
    "enabled": false,
    "rebuildOnStartup": false,
    "scanDelayMs": 2
  },
  "audit": {
    "enabled": true,
    "maxBytes": 10485760,
    "retentionFiles": 8
  },
  "trash": {
    "retentionDays": null,
    "maxBytes": null
  },
  "conflictPolicy": "autoRename"
}
```

打开：

```text
http://服务器IP:8080
```

首次启动后打开 Web 页面，按提示设置单管理员密码。后端只会把 Argon2 哈希写入 `data/auth.json`，不会保存明文密码。需要重置密码时，停止容器，删除 `data/auth.json`，再启动容器并重新进入 Web 页面设置。

镜像内置 Docker `HEALTHCHECK`，会通过 `web-file-browser --healthcheck` 按当前监听配置访问容器内的 `/api/ready`。管理员密码尚未初始化时仍视为就绪，便于首次进入 Web 页面完成设置；如果 `/app/data`、认证哈希、映射、回收站、审计日志目录或静态入口文件不可用，容器会显示为 `unhealthy`。

## 自动冒烟验证

项目提供 Linux Docker 冒烟脚本：

```bash
bash scripts/docker-smoke.sh
```

脚本会构建并启动 Compose 项目，使用 `.smoke/docker` 下的临时 `data` 和 `files` 目录，不会复用当前开发环境的 `data/`。默认端口是 `18080`，默认项目名是 `web-file-browser-smoke`。

脚本会验证：

- `/api/ready` 就绪检查。
- 前端静态入口托管。
- 首次设置单管理员密码并登录。
- 创建 `/mnt/files` 挂载。
- 新建文件、读取 ETag、`If-Match` 保存和下载校验。
- 上传文件。
- 后台 `zip` 压缩和解压。
- 后台 `tar.gz` 压缩和解压。
- 删除到回收站并恢复。
- 运行配置热生效：目录分页上限和上传大小上限。
- 文件夹收藏：创建、检查、更新、重排和删除。
- 直接永久删除，不生成回收站记录。
- 回收站批量恢复和批量永久清理。
- 轻量搜索索引：开启、重建、查询和关闭。
- 启动配置保存后的 `restartPending` 提示。
- `/api/metrics` 指标接口。

可选环境变量：

- `WEB_FILE_BROWSER_SMOKE_PORT`：冒烟服务端口，默认 `18080`。
- `WEB_FILE_BROWSER_SMOKE_PASSWORD`：冒烟管理员密码，默认 `web-file-browser-smoke-password`。
- `WEB_FILE_BROWSER_SMOKE_PROJECT`：Docker Compose 项目名，默认 `web-file-browser-smoke`。
- `WEB_FILE_BROWSER_SMOKE_KEEP=1`：失败或完成后保留容器和 `.smoke/docker` 临时目录，便于排查。

运行脚本需要宿主机安装 Docker Compose v2、`curl`、`grep`、`jq` 和 `cmp`。

## 性能冒烟验证

普通冒烟通过后，可以继续运行偏性能和规模的 Docker 冒烟：

```bash
bash scripts/docker-perf-smoke.sh
```

脚本使用 `.smoke/perf` 下的临时 `data` 和 `files` 目录，默认端口是 `18081`，默认项目名是 `web-file-browser-perf-smoke`。它会构建并启动真实 Compose 项目，然后验证：

- 1 万个文件的大目录分页。
- 默认目录分页不返回总数，`includeTotal=true` 才返回总数。
- 64 MiB 文件上传、下载和内容一致性。
- 单段 `Range` 下载。
- 大文件在线编辑保护。
- 16 MiB 文件的 `tar.gz` 压缩和解压。
- 删除到自管回收站并恢复。
- `/api/metrics` 指标接口。

可选环境变量：

- `WEB_FILE_BROWSER_PERF_PORT`：性能冒烟服务端口，默认 `18081`。
- `WEB_FILE_BROWSER_PERF_PASSWORD`：性能冒烟管理员密码，默认 `web-file-browser-perf-password`。
- `WEB_FILE_BROWSER_PERF_PROJECT`：Docker Compose 项目名，默认 `web-file-browser-perf-smoke`。
- `WEB_FILE_BROWSER_PERF_ROOT`：临时目录，默认 `.smoke/perf`，必须位于仓库 `.smoke/` 下。
- `WEB_FILE_BROWSER_PERF_DIR_ENTRIES`：大目录文件数，默认 `10000`。
- `WEB_FILE_BROWSER_PERF_DIR_PAGE_LIMIT`：目录分页校验页大小，默认 `200`。
- `WEB_FILE_BROWSER_PERF_FILE_MB`：上传下载大文件大小，默认 `64`。
- `WEB_FILE_BROWSER_PERF_ARCHIVE_MB`：压缩解压源文件大小，默认 `16`。
- `WEB_FILE_BROWSER_PERF_TASK_WAIT_SECONDS`：后台任务等待上限，默认 `900`。
- `WEB_FILE_BROWSER_PERF_KEEP=1`：失败或完成后保留容器和 `.smoke/perf` 临时目录，便于排查。

运行脚本需要宿主机安装 Docker Compose v2、`curl`、`jq`、`cmp` 和 `dd`。脚本会创建较多临时文件并读写随机数据，建议在 Linux Docker 宿主机本地磁盘上运行，不要在 Windows Docker Desktop 的 Windows 路径挂载上判断最终性能。

当前默认参数已经在真实 Linux Docker 环境通过：1 万项目录创建和分页、64 MiB 上传下载、Range、编辑保护、16 MiB tar.gz 压缩解压、回收站恢复和指标接口均已覆盖。

## 已验证的 Linux Docker 问题

真实 Linux Docker 冒烟已经通过，并修复过以下部署问题：

- 前端构建阶段必须在 `yarn install` 前复制 `ui/.yarnrc.yml`，否则 Yarn 4 构建时找不到 `node_modules` 安装状态。
- 运行镜像不再安装 `curl`，健康检查改为应用二进制 `web-file-browser --healthcheck`，减少镜像构建耗时和运行依赖。
- 运行镜像直接使用数字 UID/GID，不在构建阶段创建固定用户或用户组，避免宿主机传入 `0:0` 时因为 GID 已存在导致构建失败。
- 回收站恢复已兼容跨挂载点移动；当 `rename` 因跨设备失败时，会回退为复制后清理，适配 `/app/data/trash` 和 `/mnt/files` 分属不同 Docker bind mount 的场景。
- 大文件 multipart 上传已关闭框架默认 body limit，继续由 `WEB_FILE_BROWSER_MAX_UPLOAD_BYTES` 控制项目级上传上限，避免 64 MiB 流式上传被提前拦截为 `400`。

## 卷和目录

Compose 示例默认挂载：

- `./data:/app/data`
- `./files:/mnt/files`

`/app/data` 保存运行时数据：

- `config.json`
- `auth.json`
- `mappings.json`
- `favorites.json`
- `trash/`
- `audit.jsonl`

`config.json` 是运行配置；`auth.json` 是认证状态文件，通常不需要手动修改；`favorites.json` 保存文件夹收藏快捷入口。

运行配置有两类：

- 可在线编辑并热生效：会话有效期、新会话 Secure Cookie、上传/编辑大小上限、目录分页上限、目录/传输/IP 并发、任务并发/历史/限速、压缩输入和解压限制、搜索索引开关、回收站保留策略、审计开关和轮转策略、默认冲突策略。Web 设置页会通过 `/api/settings` 写入 `config.json` 并更新内存快照，后续请求立即使用新值。
- 可在线保存但需要重启：监听地址、端口、静态目录、CORS、代理头信任、认证文件路径、挂载文件路径、收藏文件路径、回收站根目录、审计文件路径、启动时重建索引。保存后设置接口会返回 `restartPending=true`。
- 只能展示：配置文件路径本身由默认值或 `WEB_FILE_BROWSER_CONFIG_FILE` / `WEB_FILE_BROWSER_CONFIG` 决定，不能写入当前配置文件来改变自己。

不做配置文件自动监听。如果手动编辑了 `config.json`，可在 Web 设置页触发重新加载，或调用 `POST /api/settings/reload`。由环境变量控制的字段优先级最高，不能通过 Web 设置页覆盖。

`/mnt/files` 是业务文件目录示例。进入管理界面新增挂载时，`folderPath` 可以填写 `/mnt/files`。

只读目录可以这样挂载：

```yaml
volumes:
  - /srv/media:/mnt/media:ro
```

同时建议在应用挂载配置里把对应挂载点设为不可写。Docker 只读挂载会从文件系统层阻止写入，应用只读开关会提前返回更清晰的错误。

## UID/GID 和权限

容器默认使用 `1000:1000` 运行。镜像不会在构建阶段创建固定用户名，而是直接使用数字 UID/GID，因此 `0:0` 这类宿主机返回值不会导致构建失败。真实部署时建议使用一个非 root 的宿主机用户运行，并让这个用户只拥有需要暴露给应用的目录：

```env
WEB_FILE_BROWSER_UID=1000
WEB_FILE_BROWSER_GID=1000
```

不建议把 `WEB_FILE_BROWSER_UID` / `WEB_FILE_BROWSER_GID` 设置为 `0:0`。这是一个文件管理器，Web 登录后的操作会真实修改 bind mount 里的宿主机文件；如果以 root 运行，误操作、漏洞或被盗会话的影响范围都会变大，新建文件也容易变成 root 所有，后续迁移和维护更麻烦。遇到权限不足时，优先在宿主机上用 `chown`、`chgrp`、目录 ACL 或只读挂载来授权，而不是让容器长期以 root 运行。

Compose 示例额外启用了 `no-new-privileges:true` 和 `cap_drop: [ALL]`。应用监听容器内 `8080` 端口，不需要特权端口，也不需要额外 Linux capability；文件读写能力只由容器进程的 UID/GID 和挂载目录权限决定。

如果容器无法写入 `/app/data` 或业务文件目录，优先检查宿主机目录所有者和权限：

```bash
ls -ld data files
chown -R "$WEB_FILE_BROWSER_UID:$WEB_FILE_BROWSER_GID" data files
```

如果 `./data` 或 `./files` 没有提前创建，Docker/Compose 可能会用 root 创建宿主机目录，非 root 容器随后就会写入失败。因此首次启动前建议手动创建并授权：

```bash
mkdir -p data files
chown -R "$WEB_FILE_BROWSER_UID:$WEB_FILE_BROWSER_GID" data files
chmod 700 data
```

`data/` 里包含认证哈希、挂载配置、回收站索引和审计日志，单机部署时建议用较严格的权限；业务文件目录 `files/` 则按实际共享需求设置权限。如果需要只读共享目录，优先使用 Docker `:ro` 挂载，并在应用挂载配置里同步关闭写入。

## 常用环境变量

- `WEB_FILE_BROWSER_CORS_ORIGINS`：允许跨域访问的可信来源，默认空表示同源，不支持 `*`。
- `WEB_FILE_BROWSER_TRUST_PROXY_HEADERS`：是否信任 `X-Forwarded-For`，默认 `false`；仅在容器只通过可信反向代理访问时启用。
- `WEB_FILE_BROWSER_AUTH_SESSION_TTL_SECONDS`：管理员会话有效期，默认 `604800` 秒，只影响后续新会话。
- `WEB_FILE_BROWSER_AUTH_SECURE_COOKIE`：是否给会话 Cookie 增加 `Secure` 标记，默认 `false`；仅在通过 HTTPS 访问时启用。
- `WEB_FILE_BROWSER_CONFLICT_POLICY`：冲突策略，默认 `autoRename`。
- `WEB_FILE_BROWSER_MAX_UPLOAD_BYTES`：上传和保存上限，默认不限制。
- `WEB_FILE_BROWSER_MAX_EDIT_BYTES`：在线编辑上限，默认 `2097152`。
- `WEB_FILE_BROWSER_EDITABLE_EXTENSIONS`：在线编辑扩展名白名单，默认空表示不限制扩展名。
- `WEB_FILE_BROWSER_EDITABLE_MIME_TYPES`：在线编辑 MIME 白名单，默认空表示不限制 MIME，可写 `text/*`。
- `WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE`：目录分页上限，默认 `2000`。
- `WEB_FILE_BROWSER_TASK_HISTORY_LIMIT`：内存中保留的已结束后台任务数量，默认 `200`。
- `WEB_FILE_BROWSER_AUDIT_ENABLED`：是否写入审计 JSONL，默认 `true`。
- `WEB_FILE_BROWSER_AUDIT_MAX_BYTES`：审计 JSONL 单文件轮转上限，默认 `10485760`，设置为 `0` 可关闭轮转。
- `WEB_FILE_BROWSER_AUDIT_RETENTION_FILES`：保留的审计轮转文件数量，默认 `8`，设置为 `0` 表示只保留当前审计文件。
- `WEB_FILE_BROWSER_MAX_ARCHIVE_BYTES`：单次压缩输入文件总字节上限，默认不限制。
- `WEB_FILE_BROWSER_MAX_ARCHIVE_FILES`：单次压缩输入文件数量上限，默认不限制。
- `WEB_FILE_BROWSER_MAX_EXTRACT_BYTES`：单次解压后的总字节上限，默认不限制。
- `WEB_FILE_BROWSER_MAX_EXTRACT_FILES`：单次解压的条目数量上限，默认不限制。
- `WEB_FILE_BROWSER_MAX_EXTRACT_DEPTH`：单个解压条目的路径深度上限，默认 `64`。
- `WEB_FILE_BROWSER_INDEX_ENABLED`：是否启用搜索索引，默认 `false`。
- `WEB_FILE_BROWSER_INDEX_REBUILD_ON_STARTUP`：启动是否重建索引，默认 `false`。

跨域开发示例：

```env
WEB_FILE_BROWSER_CORS_ORIGINS=http://localhost:5173,http://192.168.1.10:5173
```

反向代理部署时，只有代理会覆盖并重新设置 `X-Forwarded-For`，且客户端无法绕过代理直连容器时，才启用：

```env
WEB_FILE_BROWSER_TRUST_PROXY_HEADERS=true
```

## 备份和迁移

基础备份至少包含：

- `data/config.json`
- `data/auth.json`
- `data/mappings.json`
- `data/favorites.json`
- `data/trash/`
- `data/audit.jsonl`
- 实际业务文件挂载目录

迁移到新机器时：

1. 停止旧容器。
2. 复制 `data/` 和业务文件目录。
3. 保持目标机器上的挂载路径与 `mappings.json` 中的路径一致，或启动后在设置页修正。
4. 使用相同 UID/GID 或重新 `chown` 目录。
5. 启动新容器。

不要把 `ui/dist`、`target`、`node_modules` 当作运行数据备份，它们可以通过镜像重新构建。

## 升级

保留 `data/` 和业务文件目录，重新构建镜像：

```bash
docker compose build --pull
docker compose up -d
```

如果新增环境变量，先更新 `.env`。运行配置默认以 `data/config.json` 为准，认证哈希保存在 `data/auth.json`。

## 性能注意

- 文件内容传输由后端流式处理，Docker 卷应尽量直接挂载本机磁盘目录。
- 大目录浏览默认分页，前端和脚本不要默认请求过大的 `limit`。
- 搜索索引默认不在启动时扫描挂载目录，避免容器重启后立刻打满磁盘。
- 审计旧轮转文件只在发生新轮转后清理，不在容器启动时扫描审计目录。
- 回收站普通文件会记录 `sizeBytes` 供容量清理使用；文件夹不为了统计大小提前递归扫描。
- `/api/ready` 只做必要目录读写探测，不扫描挂载目录或回收站内容。
- 不建议把整个根目录挂给应用；按需要挂载明确目录，减少误操作范围。
