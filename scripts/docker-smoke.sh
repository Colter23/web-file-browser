#!/usr/bin/env bash
set -euo pipefail

log() {
  printf '\n[冒烟] %s\n' "$*"
}

fail() {
  printf '\n[失败] %s\n' "$*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "缺少命令: $1"
}

json_query() {
  jq -er "$1"
}

wait_ready() {
  for _ in $(seq 1 90); do
    if curl -fsS "${BASE_URL}/api/ready" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done

  compose logs --no-color --tail=120 || true
  fail "容器未在预期时间内就绪"
}

wait_task() {
  local task_id="$1"
  local body
  local state

  for _ in $(seq 1 120); do
    body=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/tasks/${task_id}")
    state=$(printf '%s' "${body}" | json_query '.state')
    case "${state}" in
      completed)
        return 0
        ;;
      failed|cancelled)
        printf '%s\n' "${body}" >&2
        fail "后台任务未成功完成: ${task_id}"
        ;;
    esac
    sleep 1
  done

  fail "后台任务未在预期时间内结束: ${task_id}"
}

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd -P)

require_command docker
require_command curl
require_command grep
require_command jq
require_command cmp
docker compose version >/dev/null 2>&1 || fail "需要 Docker Compose v2"

PROJECT_NAME=${WEB_FILE_BROWSER_SMOKE_PROJECT:-web-file-browser-smoke}
SMOKE_ROOT=${WEB_FILE_BROWSER_SMOKE_ROOT:-"${REPO_ROOT}/.smoke/docker"}
SMOKE_PARENT=$(dirname "${SMOKE_ROOT}")
mkdir -p "${SMOKE_PARENT}"
SMOKE_ROOT=$(cd "${SMOKE_PARENT}" && pwd -P)/$(basename "${SMOKE_ROOT}")
case "${SMOKE_ROOT}" in
  "${REPO_ROOT}/.smoke/"*) ;;
  *) fail "为避免误删数据，WEB_FILE_BROWSER_SMOKE_ROOT 必须位于 ${REPO_ROOT}/.smoke/ 下" ;;
esac

PORT=${WEB_FILE_BROWSER_SMOKE_PORT:-18080}
ADMIN_PASSWORD=${WEB_FILE_BROWSER_SMOKE_PASSWORD:-web-file-browser-smoke-password}
BASE_URL="http://127.0.0.1:${PORT}"
COOKIE_JAR="${SMOKE_ROOT}/cookies.txt"
ENV_FILE="${SMOKE_ROOT}/compose.env"
OVERRIDE_FILE="${SMOKE_ROOT}/compose.override.yml"
EXPECTED_FILE="${SMOKE_ROOT}/expected.txt"
DOWNLOADED_FILE="${SMOKE_ROOT}/downloaded.txt"
EXTRACTED_FILE="${SMOKE_ROOT}/extracted.txt"
TAR_EXTRACTED_FILE="${SMOKE_ROOT}/tar-extracted.txt"
UPLOAD_FILE="${SMOKE_ROOT}/upload.txt"
HEADERS_FILE="${SMOKE_ROOT}/headers.txt"
FRONTEND_FILE="${SMOKE_ROOT}/frontend.html"

compose() {
  docker compose \
    --project-name "${PROJECT_NAME}" \
    --env-file "${ENV_FILE}" \
    -f "${REPO_ROOT}/docker-compose.yml" \
    -f "${OVERRIDE_FILE}" \
    "$@"
}

cleanup() {
  if [[ "${WEB_FILE_BROWSER_SMOKE_KEEP:-0}" == "1" ]]; then
    log "已按 WEB_FILE_BROWSER_SMOKE_KEEP=1 保留容器和临时目录: ${SMOKE_ROOT}"
    return
  fi

  if [[ -f "${ENV_FILE}" && -f "${OVERRIDE_FILE}" ]]; then
    compose down --volumes --remove-orphans >/dev/null 2>&1 || true
  fi
  rm -rf "${SMOKE_ROOT}"
}

trap 'status=$?; cleanup; exit ${status}' EXIT

rm -rf "${SMOKE_ROOT}"
mkdir -p "${SMOKE_ROOT}/data" "${SMOKE_ROOT}/files"

cat >"${ENV_FILE}" <<EOF
WEB_FILE_BROWSER_PORT=${PORT}
WEB_FILE_BROWSER_UID=${WEB_FILE_BROWSER_UID:-$(id -u)}
WEB_FILE_BROWSER_GID=${WEB_FILE_BROWSER_GID:-$(id -g)}
EOF

cat >"${OVERRIDE_FILE}" <<EOF
services:
  web-file-browser:
    volumes:
      - "${SMOKE_ROOT}/data:/app/data"
      - "${SMOKE_ROOT}/files:/mnt/files"
EOF

log "构建并启动 Docker Compose 项目 ${PROJECT_NAME}"
compose up -d --build

log "等待 /api/ready 就绪"
wait_ready

log "验证前端静态文件托管"
curl -fsS "${BASE_URL}/" -o "${FRONTEND_FILE}"
grep -q 'id="app"' "${FRONTEND_FILE}" || fail "首页没有返回前端入口 HTML"

log "首次设置管理员密码、登录并创建 /mnt/files 挂载"
LOGIN_PAYLOAD=$(jq -n --arg password "${ADMIN_PASSWORD}" '{password: $password}')
curl -fsS \
  -c "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d "${LOGIN_PAYLOAD}" \
  "${BASE_URL}/api/auth/setup" >/dev/null

curl -fsS \
  -c "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d "${LOGIN_PAYLOAD}" \
  "${BASE_URL}/api/auth/login" >/dev/null

curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"mountPath":"/files","folderPath":"/mnt/files","remark":"Docker 冒烟挂载","order":0,"writable":true}' \
  "${BASE_URL}/api/mapping" >/dev/null

curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/file/files?detail=basic" >/dev/null

log "验证新建、编辑保存和下载"
curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"type":"file","name":"hello.txt","conflictPolicy":"reject"}' \
  "${BASE_URL}/api/file/files" >/dev/null

curl -fsS \
  -D "${HEADERS_FILE}" \
  -o /dev/null \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/content/files/hello.txt?mode=edit"
ETAG=$(tr -d '\r' <"${HEADERS_FILE}" | awk 'tolower($1)=="etag:" {print $2; exit}')
[[ -n "${ETAG}" ]] || fail "内容接口未返回 ETag"

printf 'Docker smoke body\n' >"${EXPECTED_FILE}"
curl -fsS \
  -X PUT \
  -b "${COOKIE_JAR}" \
  -H "If-Match: ${ETAG}" \
  --data-binary @"${EXPECTED_FILE}" \
  "${BASE_URL}/api/content/files/hello.txt" >/dev/null

curl -fsS \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/download/files/hello.txt" \
  -o "${DOWNLOADED_FILE}"
cmp "${EXPECTED_FILE}" "${DOWNLOADED_FILE}"

log "验证上传"
printf 'uploaded from docker smoke\n' >"${UPLOAD_FILE}"
curl -fsS \
  -b "${COOKIE_JAR}" \
  -F "file=@${UPLOAD_FILE};filename=upload.txt" \
  "${BASE_URL}/api/upload/files?conflictPolicy=reject" >/dev/null
curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/file/files/upload.txt" >/dev/null

log "验证后台 zip 压缩和解压"
ARCHIVE_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"sources":["/files/hello.txt","/files/upload.txt"],"targetPath":"/files","outputName":"bundle.zip","format":"zip","conflictPolicy":"reject"}' \
  "${BASE_URL}/api/tasks/archive")
ARCHIVE_TASK_ID=$(printf '%s' "${ARCHIVE_RESPONSE}" | json_query '.id')
wait_task "${ARCHIVE_TASK_ID}"
curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/file/files/bundle.zip" >/dev/null

EXTRACT_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"sourcePath":"/files/bundle.zip","targetPath":"/files","folderName":"unzipped","conflictPolicy":"reject"}' \
  "${BASE_URL}/api/tasks/extract")
EXTRACT_TASK_ID=$(printf '%s' "${EXTRACT_RESPONSE}" | json_query '.id')
wait_task "${EXTRACT_TASK_ID}"
curl -fsS \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/download/files/unzipped/hello.txt" \
  -o "${EXTRACTED_FILE}"
cmp "${EXPECTED_FILE}" "${EXTRACTED_FILE}"

log "验证后台 tar.gz 压缩和解压"
TAR_ARCHIVE_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"sources":["/files/upload.txt"],"targetPath":"/files","outputName":"upload.tar.gz","format":"tarGz","conflictPolicy":"reject"}' \
  "${BASE_URL}/api/tasks/archive")
TAR_ARCHIVE_TASK_ID=$(printf '%s' "${TAR_ARCHIVE_RESPONSE}" | json_query '.id')
wait_task "${TAR_ARCHIVE_TASK_ID}"
curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/file/files/upload.tar.gz" >/dev/null

TAR_EXTRACT_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"sourcePath":"/files/upload.tar.gz","targetPath":"/files","folderName":"untarred","conflictPolicy":"reject"}' \
  "${BASE_URL}/api/tasks/extract")
TAR_EXTRACT_TASK_ID=$(printf '%s' "${TAR_EXTRACT_RESPONSE}" | json_query '.id')
wait_task "${TAR_EXTRACT_TASK_ID}"
curl -fsS \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/download/files/untarred/upload.txt" \
  -o "${TAR_EXTRACTED_FILE}"
cmp "${UPLOAD_FILE}" "${TAR_EXTRACTED_FILE}"

log "验证删除到回收站和恢复"
curl -fsS \
  -X DELETE \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/file/files/hello.txt" >/dev/null
TRASH_RESPONSE=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/trash")
TRASH_ID=$(printf '%s' "${TRASH_RESPONSE}" | json_query '.[0].id')
curl -fsS \
  -X POST \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/trash/${TRASH_ID}/restore" >/dev/null
curl -fsS \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/download/files/hello.txt" \
  -o "${DOWNLOADED_FILE}"
cmp "${EXPECTED_FILE}" "${DOWNLOADED_FILE}"

log "验证指标接口"
curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/metrics" >/dev/null

log "Docker 冒烟通过"
