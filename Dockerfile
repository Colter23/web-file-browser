# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim AS ui-builder
WORKDIR /app/ui

# 先安装前端依赖，利用 Docker 缓存减少重复下载。
COPY ui/package.json ui/yarn.lock ./
RUN corepack enable && yarn install --frozen-lockfile

COPY ui/ ./
RUN yarn build

FROM rust:1-bookworm AS backend-builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY --from=ui-builder /app/ui/dist ./ui/dist
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

ARG WFB_UID=1000
ARG WFB_GID=1000

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid "${WFB_GID}" web-file-browser \
    && useradd --uid "${WFB_UID}" --gid "${WFB_GID}" --create-home --no-log-init web-file-browser \
    && mkdir -p /app/data /app/ui/dist /mnt/files \
    && chown -R web-file-browser:web-file-browser /app /mnt/files

WORKDIR /app

COPY --from=backend-builder /app/target/release/web-file-browser /usr/local/bin/web-file-browser
COPY --from=ui-builder /app/ui/dist /app/ui/dist

ENV WEB_FILE_BROWSER_BIND=0.0.0.0 \
    PORT=8080 \
    WEB_FILE_BROWSER_STATIC_DIR=/app/ui/dist \
    WEB_FILE_BROWSER_MAPPING_FILE=/app/data/mappings.json \
    WEB_FILE_BROWSER_CONFIG_FILE=/app/data/config.json \
    WEB_FILE_BROWSER_TRASH_DIR=/app/data/trash \
    WEB_FILE_BROWSER_AUDIT_FILE=/app/data/audit.jsonl

EXPOSE 8080

USER web-file-browser
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 CMD curl -fsS http://127.0.0.1:8080/api/ready >/dev/null || exit 1
ENTRYPOINT ["web-file-browser"]
