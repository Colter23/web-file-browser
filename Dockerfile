# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim AS ui-builder
WORKDIR /app/ui

# 先安装前端依赖，利用 Docker 缓存减少重复下载。
# .yarnrc.yml 必须在安装前复制，否则 Yarn 4 会使用默认 linker，构建时找不到 node_modules 安装状态。
COPY ui/package.json ui/yarn.lock ui/.yarnrc.yml ./
RUN corepack enable && yarn install --immutable

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

RUN mkdir -p /app/data /app/ui/dist /mnt/files \
    && chown -R "${WFB_UID}:${WFB_GID}" /app /mnt/files

WORKDIR /app

COPY --from=backend-builder /app/target/release/web-file-browser /usr/local/bin/web-file-browser
COPY --from=ui-builder /app/ui/dist /app/ui/dist

ENV WEB_FILE_BROWSER_BIND=0.0.0.0 \
    PORT=8080 \
    WEB_FILE_BROWSER_STATIC_DIR=/app/ui/dist \
    WEB_FILE_BROWSER_MAPPING_FILE=/app/data/mappings.json \
    WEB_FILE_BROWSER_CONFIG_FILE=/app/data/config.json \
    WEB_FILE_BROWSER_AUTH_FILE=/app/data/auth.json \
    WEB_FILE_BROWSER_TRASH_DIR=/app/data/trash \
    WEB_FILE_BROWSER_AUDIT_FILE=/app/data/audit.jsonl

EXPOSE 8080

USER ${WFB_UID}:${WFB_GID}
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 CMD ["web-file-browser", "--healthcheck"]
ENTRYPOINT ["web-file-browser"]
