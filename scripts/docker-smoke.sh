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

wait_index_idle() {
  local body
  local state

  for _ in $(seq 1 90); do
    body=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/index/status")
    state=$(printf '%s' "${body}" | json_query '.state')
    case "${state}" in
      idle)
        return 0
        ;;
      disabled|failed|cancelled)
        printf '%s\n' "${body}" >&2
        fail "搜索索引未成功进入 idle 状态"
        ;;
    esac
    sleep 1
  done

  fail "搜索索引未在预期时间内完成重建"
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
TOO_LARGE_UPLOAD_FILE="${SMOKE_ROOT}/too-large-upload.txt"
HTTP_ERROR_FILE="${SMOKE_ROOT}/http-error.json"

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

log "验证运行配置热生效"
SETTINGS_RESPONSE=$(curl -fsS \
  -X PATCH \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"runtime":{"maxUploadBytes":4,"maxDirPageSize":2}}' \
  "${BASE_URL}/api/settings")
printf '%s' "${SETTINGS_RESPONSE}" | jq -e \
  '.runtime.maxUploadBytes == 4 and .runtime.maxDirPageSize == 2 and .restartPending == false' >/dev/null \
  || fail "运行配置 PATCH 后响应不符合预期"

mkdir -p "${SMOKE_ROOT}/files/settings-page"
: >"${SMOKE_ROOT}/files/settings-page/a.txt"
: >"${SMOKE_ROOT}/files/settings-page/b.txt"
: >"${SMOKE_ROOT}/files/settings-page/c.txt"
PAGE_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/file/files/settings-page?detail=basic&limit=99")
PAGE_COUNT=$(printf '%s' "${PAGE_RESPONSE}" | json_query '.file | length')
PAGE_HAS_MORE=$(printf '%s' "${PAGE_RESPONSE}" | json_query '.hasMore')
if [[ "${PAGE_COUNT}" -ne 2 || "${PAGE_HAS_MORE}" != "true" ]]; then
  printf '%s\n' "${PAGE_RESPONSE}" >&2
  fail "目录分页上限没有热生效"
fi

printf '12345' >"${TOO_LARGE_UPLOAD_FILE}"
UPLOAD_STATUS=$(curl -sS \
  -o "${HTTP_ERROR_FILE}" \
  -w "%{http_code}" \
  -b "${COOKIE_JAR}" \
  -F "file=@${TOO_LARGE_UPLOAD_FILE};filename=too-large-after-settings.txt" \
  "${BASE_URL}/api/upload/files?conflictPolicy=reject")
if [[ "${UPLOAD_STATUS}" != "413" ]]; then
  cat "${HTTP_ERROR_FILE}" >&2 || true
  fail "上传上限没有热生效: ${UPLOAD_STATUS}"
fi
[[ ! -e "${SMOKE_ROOT}/files/too-large-after-settings.txt" ]] || fail "超限上传不应落盘"

log "验证文件夹收藏"
curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"type":"folder","name":"favorite-a","conflictPolicy":"reject"}' \
  "${BASE_URL}/api/file/files" >/dev/null
curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"type":"folder","name":"favorite-b","conflictPolicy":"reject"}' \
  "${BASE_URL}/api/file/files" >/dev/null

FAVORITE_A_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"path":"/files/favorite-a","name":"收藏 A","order":20}' \
  "${BASE_URL}/api/favorites")
FAVORITE_B_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"path":"/files/favorite-b","name":"收藏 B","order":10}' \
  "${BASE_URL}/api/favorites")
FAVORITE_A_ID=$(printf '%s' "${FAVORITE_A_RESPONSE}" | json_query '.id')
FAVORITE_B_ID=$(printf '%s' "${FAVORITE_B_RESPONSE}" | json_query '.id')
FAVORITE_LIST=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/favorites?check=true")
printf '%s' "${FAVORITE_LIST}" | jq -e \
  'length == 2 and .[0].path == "/files/favorite-b" and .[0].missing == false' >/dev/null \
  || fail "收藏夹列表不符合预期"

curl -fsS \
  -X PATCH \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"name":"收藏 A 更新","order":5}' \
  "${BASE_URL}/api/favorites/${FAVORITE_A_ID}" >/dev/null
REORDER_PAYLOAD=$(jq -n \
  --arg favorite_a "${FAVORITE_A_ID}" \
  --arg favorite_b "${FAVORITE_B_ID}" \
  '{items:[{id:$favorite_a,order:5},{id:$favorite_b,order:15}]}')
curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d "${REORDER_PAYLOAD}" \
  "${BASE_URL}/api/favorites/reorder" >/dev/null
FAVORITE_LIST=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/favorites")
printf '%s' "${FAVORITE_LIST}" | jq -e \
  --arg favorite_a "${FAVORITE_A_ID}" \
  '.[0].id == $favorite_a and .[0].name == "收藏 A 更新"' >/dev/null \
  || fail "收藏夹更新或重排不符合预期"

curl -fsS -X DELETE -b "${COOKIE_JAR}" "${BASE_URL}/api/favorites/${FAVORITE_A_ID}" >/dev/null
curl -fsS -X DELETE -b "${COOKIE_JAR}" "${BASE_URL}/api/favorites/${FAVORITE_B_ID}" >/dev/null
FAVORITE_LIST=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/favorites")
printf '%s' "${FAVORITE_LIST}" | jq -e 'length == 0' >/dev/null \
  || fail "收藏夹删除后列表应为空"

log "验证直接永久删除"
curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"type":"file","name":"permanent-delete.txt","conflictPolicy":"reject"}' \
  "${BASE_URL}/api/file/files" >/dev/null
curl -fsS \
  -X DELETE \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/file/files/permanent-delete.txt?permanent=true" >/dev/null
DELETE_STATUS=$(curl -sS \
  -o "${HTTP_ERROR_FILE}" \
  -w "%{http_code}" \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/file/files/permanent-delete.txt")
[[ "${DELETE_STATUS}" == "404" ]] || fail "永久删除后文件仍可访问: ${DELETE_STATUS}"
TRASH_COUNT=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/trash" | json_query 'length')
[[ "${TRASH_COUNT}" -eq 0 ]] || fail "永久删除不应新增回收站记录"

log "验证回收站批量恢复和批量清理"
for name in batch-trash-a.txt batch-trash-b.txt; do
  curl -fsS \
    -b "${COOKIE_JAR}" \
    -H "Content-Type: application/json" \
    -d "{\"type\":\"file\",\"name\":\"${name}\",\"conflictPolicy\":\"reject\"}" \
    "${BASE_URL}/api/file/files" >/dev/null
  curl -fsS \
    -X DELETE \
    -b "${COOKIE_JAR}" \
    "${BASE_URL}/api/file/files/${name}" >/dev/null
done
TRASH_RESPONSE=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/trash")
BATCH_TRASH_A_ID=$(printf '%s' "${TRASH_RESPONSE}" | jq -er '.[] | select(.originalVirtualPath == "/files/batch-trash-a.txt") | .id')
BATCH_TRASH_B_ID=$(printf '%s' "${TRASH_RESPONSE}" | jq -er '.[] | select(.originalVirtualPath == "/files/batch-trash-b.txt") | .id')
BATCH_PAYLOAD=$(jq -n \
  --arg a "${BATCH_TRASH_A_ID}" \
  --arg b "${BATCH_TRASH_B_ID}" \
  '{ids:[$a,$b],conflictPolicy:"reject"}')
BATCH_RESTORE_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d "${BATCH_PAYLOAD}" \
  "${BASE_URL}/api/trash/batch/restore")
printf '%s' "${BATCH_RESTORE_RESPONSE}" | jq -e \
  '.success == 2 and .failed == 0 and (.restored | length) == 2' >/dev/null \
  || fail "回收站批量恢复结果不符合预期"
curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/file/files/batch-trash-a.txt" >/dev/null
curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/file/files/batch-trash-b.txt" >/dev/null

for name in batch-trash-a.txt batch-trash-b.txt; do
  curl -fsS \
    -X DELETE \
    -b "${COOKIE_JAR}" \
    "${BASE_URL}/api/file/files/${name}" >/dev/null
done
TRASH_RESPONSE=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/trash")
BATCH_TRASH_A_ID=$(printf '%s' "${TRASH_RESPONSE}" | jq -er '.[] | select(.originalVirtualPath == "/files/batch-trash-a.txt") | .id')
BATCH_TRASH_B_ID=$(printf '%s' "${TRASH_RESPONSE}" | jq -er '.[] | select(.originalVirtualPath == "/files/batch-trash-b.txt") | .id')
BATCH_PAYLOAD=$(jq -n --arg a "${BATCH_TRASH_A_ID}" --arg b "${BATCH_TRASH_B_ID}" '{ids:[$a,$b]}')
BATCH_PURGE_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d "${BATCH_PAYLOAD}" \
  "${BASE_URL}/api/trash/batch/purge")
printf '%s' "${BATCH_PURGE_RESPONSE}" | jq -e \
  '.success == 2 and .failed == 0 and (.purged | length) == 2' >/dev/null \
  || fail "回收站批量清理结果不符合预期"
TRASH_LEFT=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/trash" | jq -er \
  '[.[] | select(.originalVirtualPath == "/files/batch-trash-a.txt" or .originalVirtualPath == "/files/batch-trash-b.txt")] | length')
[[ "${TRASH_LEFT}" -eq 0 ]] || fail "批量清理后不应残留对应回收站记录"

log "验证轻量搜索索引"
printf 'smoke search body\n' >"${SMOKE_ROOT}/files/smoke-needle.txt"
SEARCH_SETTINGS_RESPONSE=$(curl -fsS \
  -X PATCH \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"runtime":{"indexEnabled":true,"indexScanDelayMs":0}}' \
  "${BASE_URL}/api/settings")
printf '%s' "${SEARCH_SETTINGS_RESPONSE}" | jq -e '.runtime.indexEnabled == true' >/dev/null \
  || fail "搜索索引开关没有热生效"
curl -fsS \
  -X POST \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/index/rebuild" >/dev/null
wait_index_idle
SEARCH_RESPONSE=$(curl -fsS \
  -b "${COOKIE_JAR}" \
  "${BASE_URL}/api/search?q=smoke-needle")
printf '%s' "${SEARCH_RESPONSE}" | jq -e \
  '.total >= 1 and ([.items[] | select(.path == "/files/smoke-needle.txt")] | length) >= 1' >/dev/null \
  || fail "搜索索引没有返回预期文件"
SEARCH_SETTINGS_RESPONSE=$(curl -fsS \
  -X PATCH \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"runtime":{"indexEnabled":false}}' \
  "${BASE_URL}/api/settings")
printf '%s' "${SEARCH_SETTINGS_RESPONSE}" | jq -e '.runtime.indexEnabled == false' >/dev/null \
  || fail "搜索索引关闭配置没有热生效"
INDEX_STATUS=$(curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/index/status")
printf '%s' "${INDEX_STATUS}" | jq -e '.enabled == false and .state == "disabled"' >/dev/null \
  || fail "搜索索引关闭后状态不符合预期"

log "验证启动配置保存提示"
STARTUP_SETTINGS_RESPONSE=$(curl -fsS \
  -X PATCH \
  -b "${COOKIE_JAR}" \
  -H "Content-Type: application/json" \
  -d '{"startup":{"indexRebuildOnStartup":true}}' \
  "${BASE_URL}/api/settings")
printf '%s' "${STARTUP_SETTINGS_RESPONSE}" | jq -e \
  '.startup.indexRebuildOnStartup == true
   and .activeStartup.indexRebuildOnStartup == false
   and .restartPending == true
   and (.restartPendingFields | index("startup.indexRebuildOnStartup") != null)' >/dev/null \
  || fail "启动配置保存提示不符合预期"

log "验证指标接口"
curl -fsS -b "${COOKIE_JAR}" "${BASE_URL}/api/metrics" >/dev/null

log "Docker 冒烟通过"
