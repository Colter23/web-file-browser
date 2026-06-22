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

修改 `.env` 中的管理员初始密码和 UID/GID：

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

打开：

```text
http://服务器IP:8080
```

首次启动时，`WEB_FILE_BROWSER_ADMIN_PASSWORD` 会用于初始化管理员密码哈希。`data/config.json` 已经存在管理员密码哈希后，后续启动不会用环境变量覆盖密码。

镜像内置 Docker `HEALTHCHECK`，会通过 `web-file-browser --healthcheck` 访问容器内的 `http://127.0.0.1:8080/api/ready`。如果管理员密码没有初始化，或 `/app/data`、回收站、审计日志目录、静态文件目录不可用，容器会显示为 `unhealthy`。

## 自动冒烟验证

项目提供 Linux Docker 冒烟脚本：

```bash
bash scripts/docker-smoke.sh
```

脚本会构建并启动 Compose 项目，使用 `.smoke/docker` 下的临时 `data` 和 `files` 目录，不会复用当前开发环境的 `data/`。默认端口是 `18080`，默认项目名是 `web-file-browser-smoke`。

脚本会验证：

- `/api/ready` 就绪检查。
- 前端静态入口托管。
- 单管理员登录。
- 创建 `/mnt/files` 挂载。
- 新建文件、读取 ETag、`If-Match` 保存和下载校验。
- 上传文件。
- 后台 `zip` 压缩和解压。
- 后台 `tar.gz` 压缩和解压。
- 删除到回收站并恢复。
- `/api/metrics` 指标接口。

可选环境变量：

- `WEB_FILE_BROWSER_SMOKE_PORT`：冒烟服务端口，默认 `18080`。
- `WEB_FILE_BROWSER_SMOKE_PASSWORD`：冒烟管理员密码，默认 `web-file-browser-smoke-password`。
- `WEB_FILE_BROWSER_SMOKE_PROJECT`：Docker Compose 项目名，默认 `web-file-browser-smoke`。
- `WEB_FILE_BROWSER_SMOKE_KEEP=1`：失败或完成后保留容器和 `.smoke/docker` 临时目录，便于排查。

运行脚本需要宿主机安装 Docker Compose v2、`curl`、`grep`、`jq` 和 `cmp`。

## 卷和目录

Compose 示例默认挂载：

- `./data:/app/data`
- `./files:/mnt/files`

`/app/data` 保存运行时数据：

- `config.json`
- `mappings.json`
- `trash/`
- `audit.jsonl`

`/mnt/files` 是业务文件目录示例。进入管理界面新增挂载时，`folderPath` 可以填写 `/mnt/files`。

只读目录可以这样挂载：

```yaml
volumes:
  - /srv/media:/mnt/media:ro
```

同时建议在应用挂载配置里把对应挂载点设为不可写。Docker 只读挂载会从文件系统层阻止写入，应用只读开关会提前返回更清晰的错误。

## UID/GID 和权限

容器默认使用 `1000:1000` 运行。真实部署时建议把 `.env` 中的值改为宿主机拥有文件目录的用户：

```env
WEB_FILE_BROWSER_UID=1000
WEB_FILE_BROWSER_GID=1000
```

如果容器无法写入 `/app/data` 或业务文件目录，优先检查宿主机目录所有者和权限：

```bash
ls -ld data files
chown -R "$WEB_FILE_BROWSER_UID:$WEB_FILE_BROWSER_GID" data files
```

## 常用环境变量

- `WEB_FILE_BROWSER_ADMIN_PASSWORD`：首次启动初始化管理员密码。
- `WEB_FILE_BROWSER_CORS_ORIGINS`：允许跨域访问的可信来源，默认空表示同源，不支持 `*`。
- `WEB_FILE_BROWSER_TRUST_PROXY_HEADERS`：是否信任 `X-Forwarded-For`，默认 `false`；仅在容器只通过可信反向代理访问时启用。
- `WEB_FILE_BROWSER_CONFLICT_POLICY`：冲突策略，默认 `autoRename`。
- `WEB_FILE_BROWSER_MAX_UPLOAD_BYTES`：上传和保存上限，默认不限制。
- `WEB_FILE_BROWSER_MAX_EDIT_BYTES`：在线编辑上限，默认 `2097152`。
- `WEB_FILE_BROWSER_EDITABLE_EXTENSIONS`：在线编辑扩展名白名单，默认空表示不限制扩展名。
- `WEB_FILE_BROWSER_EDITABLE_MIME_TYPES`：在线编辑 MIME 白名单，默认空表示不限制 MIME，可写 `text/*`。
- `WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE`：目录分页上限，默认 `2000`。
- `WEB_FILE_BROWSER_TASK_HISTORY_LIMIT`：内存中保留的已结束后台任务数量，默认 `200`。
- `WEB_FILE_BROWSER_AUDIT_MAX_BYTES`：审计 JSONL 单文件轮转上限，默认 `10485760`，设置为 `0` 可关闭。
- `WEB_FILE_BROWSER_AUDIT_RETENTION_FILES`：保留的审计轮转文件数量，默认 `8`，设置为 `0` 表示只保留当前审计文件。
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
- `data/mappings.json`
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

如果新增环境变量，先更新 `.env`。配置和认证仍以 `data/config.json` 为准。

## 性能注意

- 文件内容传输由后端流式处理，Docker 卷应尽量直接挂载本机磁盘目录。
- 大目录浏览默认分页，前端和脚本不要默认请求过大的 `limit`。
- 搜索索引默认不在启动时扫描挂载目录，避免容器重启后立刻打满磁盘。
- 审计旧轮转文件只在发生新轮转后清理，不在容器启动时扫描审计目录。
- 回收站普通文件会记录 `sizeBytes` 供容量清理使用；文件夹不为了统计大小提前递归扫描。
- `/api/ready` 只做必要目录读写探测，不扫描挂载目录或回收站内容。
- 不建议把整个根目录挂给应用；按需要挂载明确目录，减少误操作范围。
