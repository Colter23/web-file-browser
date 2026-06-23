#!/usr/bin/env bash
set -euo pipefail

log() {
  printf '\n[性能冒烟] %s\n' "$*"
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

run_step() {
  local label="$1"
  shift
  local started
  local finished
  started=$(date +%s)
  log "$label"
  "$@"
  finished=$(date +%s)
  log "$label 完成，用时 $((finished - started)) 秒"
}

wait_ready() {
  for _ in $(seq 1 90); do
    if curl -fsS "${BASE_URL}/api/ready" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done

  compose logs --no-color --tail=160 || true
  fail "容器未在预期时间内就绪"
}

wait_task() {
  local task_id="$1"
  local deadline=$(( $(date +%s) + TASK_WAIT_SECONDS ))
  local body
  local state

  while [[ $(date +%s) -lt ${deadline} ]]; do
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

  fail "后台任务未在 ${TASK_WAIT_SECONDS} 秒内结束: ${task_id}"
}

write_mebibytes() {
  local source="$1"
  local target="$2"
  local size_mb="$3"

  dd if="${source}" of="${target}" bs=1M count="${size_mb}" status=none
}

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd -P)

require_command docker
require_command curl
require_command jq
require_command cmp
require_command dd
docker compose version >/dev/null 2>&1 || fail "需要 Docker Compose v2"

PROJECT_NAME=${WEB_FILE_BROWSER_PERF_PROJECT:-web-file-browser-perf-smoke}
SMOKE_ROOT=${WEB_FILE_BROWSER_PERF_ROOT:-"${REPO_ROOT}/.smoke/perf"}
SMOKE_PARENT=$(dirname "${SMOKE_ROOT}")
mkdir -p "${SMOKE_PARENT}"
SMOKE_ROOT=$(cd "${SMOKE_PARENT}" && pwd -P)/$(basename "${SMOKE_ROOT}")
case "${SMOKE_ROOT}" in
  "${REPO_ROOT}/.smoke/"*) ;;
  *) fail "为避免误删数据，WEB_FILE_BROWSER_PERF_ROOT 必须位于 ${REPO_ROOT}/.smoke/ 下" ;;
esac

PORT=${WEB_FILE_BROWSER_PERF_PORT:-18081}
ADMIN_PASSWORD=${WEB_FILE_BROWSER_PERF_PASSWORD:-web-file-browser-perf-password}
DIR_ENTRIES=${WEB_FILE_BROWSER_PERF_DIR_ENTRIES:-10000}
DIR_PAGE_LIMIT=${WEB_FILE_BROWSER_PERF_DIR_PAGE_LIMIT:-200}
LARGE_FILE_MB=${WEB_FILE_BROWSER_PERF_FILE_MB:-64}
ARCHIVE_FILE_MB=${WEB_FILE_BROWSER_PERF_ARCHIVE_MB:-16}
TASK_WAIT_SECONDS=${WEB_FILE_BROWSER_PERF_TASK_WAIT_SECONDS:-900}
BASE_URL="http://127.0.0.1:${PORT}"
COOKIE_JAR="${SMOKE_ROOT}/cookies.txt"
ENV_FILE="${SMOKE_ROOT}/compose.env"
OVERRIDE_FILE="${SMOKE_ROOT}/compose.override.yml"
LARGE_SOURCE_FILE="${SMOKE_ROOT}/large-upload-source.bin"
LARGE_DOWNLOAD_FILE="${SMOKE_ROOT}/large-download.bin"
RANGE_EXPECTED_FILE="${SMOKE_ROOT}/range-expected.bin"
RANGE_DOWNLOAD_FILE="${SMOKE_ROOT}/range-download.bin"
EDIT_ERROR_FILE="${SMOKE_ROOT}/edit-error.json"
ARCHIVE_SOURCE_FILE="${SMOKE_ROOT}/files/archive-source.bin"
ARCHIVE_DOWNLOAD_FILE="${SMOKE_ROOT}/archive-download.bin"
LARGE_DIR="${SMOKE_ROOT}/files/large-dir"

compose() {
  docker compose \
    --project-name "${PROJECT_NAME}" \
    --env-file "${ENV_FILE}" \
    -f "${REPO_ROOT}/docker-compose.yml" \
    -f "${OVERRIDE_FILE}" \
    "$@"
}

cleanup() {
  if [[ "${WEB_FILE_BROWSER_PERF_KEEP:-0}" == "1" ]]; then
    log "已按 WEB_FILE_BROWSER_PERF_KEEP=1 保留容器和临时目录: ${SMOKE_ROOT}"
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
WEB_FILE_BROWSER_ADMIN_PASSWORD=${ADMIN_PASSWORD}
WEB_FILE_BROWSER_UID=${WEB_FILE_BROWSER_UID:-$(id -u)}
WEB_FILE_BROWSER_GID=${WEB_FILE_BROWSER_GID:-$(id -g)}
WEB_FILE_BROWSER_CONFLICT_POLICY=autoRename
WEB_FILE_BROWSER_INDEX_ENABLED=false
WEB_FILE_BROWSER_INDEX_REBUILD_ON_STARTUP=false
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

log "登录并创建 /mnt/files 挂载"
LOGIN_PAYLOAD=$(jq -n --arg password "${ADMIN_PASSWORD}" '{password: $password}')
curl -fsS \
  -c "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d "${LOGIN_PAYLOAD}" \
  "${BASE_URL}/api/auth/login" >/dev/null

curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"mountPath":"/files","folderPath":"/mnt/files","remark":"Docker 性能冒烟挂载","order":0,"writable":true}' \
  "${BASE_URL}/api/mapping" >/dev/null

create_large_directory() {
  mkdir -p "${LARGE_DIR}"
  local index=0
  local name
  while [[ ${index} -lt ${DIR_ENTRIES} ]]; do
    name=$(printf 'file-%05d.txt' "${index}")
    : >"${LARGE_DIR}/${name}"
    index=$((index + 1))
  done
}

verify_large_directory() {
  local body
  local total_body
  local count
  local has_more
  local file_total

  body=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/file/files/large-dir?detail=basic&limit=${DIR_PAGE_LIMIT}&offset=0")
  count=$(printf '%s' "${body}" | json_query '.file | length')
  has_more=$(printf '%s' "${body}" | json_query '.hasMore')
  printf '%s' "${body}" | jq -e 'has("folderTotal") == false and has("fileTotal") == false' >/dev/null \
    || fail "默认目录分页不应返回总数"

  local expected_count=${DIR_ENTRIES}
  if [[ ${expected_count} -gt ${DIR_PAGE_LIMIT} ]]; then
    expected_count=${DIR_PAGE_LIMIT}
  fi

  if [[ ${count} -ne ${expected_count} ]]; then
    fail "目录分页数量不符合预期: got=${count}, want=${expected_count}"
  fi
  if [[ ${DIR_ENTRIES} -gt ${DIR_PAGE_LIMIT} && "${has_more}" != "true" ]]; then
    fail "大目录分页应返回 hasMore=true"
  fi

  total_body=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/file/files/large-dir?detail=basic&limit=1&includeTotal=true")
  file_total=$(printf '%s' "${total_body}" | json_query '.fileTotal')
  if [[ ${file_total} -ne ${DIR_ENTRIES} ]]; then
    fail "includeTotal 统计不符合预期: got=${file_total}, want=${DIR_ENTRIES}"
  fi
}

verify_large_transfer() {
  write_mebibytes /dev/urandom "${LARGE_SOURCE_FILE}" "${LARGE_FILE_MB}"
  curl -fsS \
    -b "${COOKIE_JAR}" \
    -F "file=@${LARGE_SOURCE_FILE};filename=large-upload.bin" \
    "${BASE_URL}/api/upload/files?conflictPolicy=reject" >/dev/null

  curl -fsS \
    -b "${COOKIE_JAR}" \
    "${BASE_URL}/api/download/files/large-upload.bin" \
    -o "${LARGE_DOWNLOAD_FILE}"
  cmp "${LARGE_SOURCE_FILE}" "${LARGE_DOWNLOAD_FILE}"

  dd if="${LARGE_SOURCE_FILE}" of="${RANGE_EXPECTED_FILE}" bs=1 skip=1024 count=1024 status=none
  curl -fsS \
    -b "${COOKIE_JAR}" \
    -H "Range: bytes=1024-2047" \
    "${BASE_URL}/api/download/files/large-upload.bin" \
    -o "${RANGE_DOWNLOAD_FILE}"
  cmp "${RANGE_EXPECTED_FILE}" "${RANGE_DOWNLOAD_FILE}"

  local status
  status=$(curl -sS \
    -o "${EDIT_ERROR_FILE}" \
    -w "%{http_code}" \
    -b "${COOKIE_JAR}" \
    "${BASE_URL}/api/content/files/large-upload.bin?mode=edit")
  case "${status}" in
    413|415) ;;
    *) fail "大文件在线编辑保护返回码不符合预期: ${status}" ;;
  esac
}

verify_archive_extract() {
  write_mebibytes /dev/zero "${ARCHIVE_SOURCE_FILE}" "${ARCHIVE_FILE_MB}"

  local archive_response
  local archive_task_id
  archive_response=$(curl -fsS \
    -b "${COOKIE_JAR}" \
    -H "Content-Type: application/json" \
    -d '{"sources":["/files/archive-source.bin"],"targetPath":"/files","outputName":"archive-source.tar.gz","format":"tarGz","conflictPolicy":"reject"}' \
    "${BASE_URL}/api/tasks/archive")
  archive_task_id=$(printf '%s' "${archive_response}" | json_query '.id')
  wait_task "${archive_task_id}"

  local extract_response
  local extract_task_id
  extract_response=$(curl -fsS \
    -b "${COOKIE_JAR}" \
    -H "Content-Type: application/json" \
    -d '{"sourcePath":"/files/archive-source.tar.gz","targetPath":"/files","folderName":"archive-extracted","conflictPolicy":"reject"}' \
    "${BASE_URL}/api/tasks/extract")
  extract_task_id=$(printf '%s' "${extract_response}" | json_query '.id')
  wait_task "${extract_task_id}"

  curl -fsS \
    -b "${COOKIE_JAR}" \
    "${BASE_URL}/api/download/files/archive-extracted/archive-source.bin" \
    -o "${ARCHIVE_DOWNLOAD_FILE}"
  cmp "${ARCHIVE_SOURCE_FILE}" "${ARCHIVE_DOWNLOAD_FILE}"
}

verify_cross_mount_trash_restore() {
  curl -fsS \
    -X DELETE \
    -b "${COOKIE_JAR}" \
    "${BASE_URL}/api/file/files/large-upload.bin" >/dev/null

  local trash_response
  local trash_id
  trash_response=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/trash")
  trash_id=$(printf '%s' "${trash_response}" | json_query '.[0].id')
  curl -fsS \
    -X POST \
    -b "${COOKIE_JAR}" \
    "${BASE_URL}/api/trash/${trash_id}/restore" >/dev/null

  curl -fsS \
    -b "${COOKIE_JAR}" \
    "${BASE_URL}/api/download/files/large-upload.bin" \
    -o "${LARGE_DOWNLOAD_FILE}"
  cmp "${LARGE_SOURCE_FILE}" "${LARGE_DOWNLOAD_FILE}"
}

run_step "生成 ${DIR_ENTRIES} 个文件的大目录" create_large_directory
run_step "验证大目录分页和按需总数" verify_large_directory
run_step "验证 ${LARGE_FILE_MB} MiB 上传、下载、Range 和编辑保护" verify_large_transfer
run_step "验证 ${ARCHIVE_FILE_MB} MiB tar.gz 压缩和解压" verify_archive_extract
run_step "验证跨挂载回收站删除和恢复" verify_cross_mount_trash_restore

log "验证指标接口"
curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/metrics" >/dev/null

log "Docker 性能冒烟通过"
