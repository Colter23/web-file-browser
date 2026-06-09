use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{ETAG, IF_NONE_MATCH, LAST_MODIFIED},
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};
use time::{OffsetDateTime, format_description::well_known::Rfc2822};

use crate::{
    app::AppState,
    error::AppError,
    models::{CreateEntryRequest, FileOperationResponse, MoveEntryRequest, UploadResponse},
    services::{
        file_ops,
        path_resolver::{
            self, DirectoryDetail, DirectoryListOptions, DirectorySort, EntryFilter, MetadataEntry,
            SortOrder,
        },
        transfer,
    },
};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryQuery {
    offset: Option<usize>,
    limit: Option<usize>,
    detail: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    #[serde(rename = "type")]
    entry_type: Option<String>,
    include_hidden: Option<bool>,
}

impl DirectoryQuery {
    fn into_options(self, max_page_size: usize) -> Result<DirectoryListOptions, AppError> {
        let limit = self.limit.unwrap_or(max_page_size).min(max_page_size);
        if limit == 0 {
            return Err(AppError::bad_request("分页大小必须大于 0"));
        }
        Ok(DirectoryListOptions {
            offset: self.offset.unwrap_or(0),
            limit: Some(limit),
            detail: parse_detail(self.detail.as_deref())?,
            sort: parse_sort(self.sort.as_deref())?,
            order: parse_order(self.order.as_deref())?,
            filter: parse_filter(self.entry_type.as_deref())?,
            include_hidden: self.include_hidden.unwrap_or(false),
        })
    }
}

fn parse_detail(value: Option<&str>) -> Result<DirectoryDetail, AppError> {
    match value.unwrap_or("basic") {
        "basic" => Ok(DirectoryDetail::Basic),
        "full" => Ok(DirectoryDetail::Full),
        other => Err(AppError::bad_request(format!("不支持的 detail: {other}"))),
    }
}

fn parse_sort(value: Option<&str>) -> Result<DirectorySort, AppError> {
    match value.unwrap_or("name") {
        "name" => Ok(DirectorySort::Name),
        "modified" => Ok(DirectorySort::Modified),
        "size" => Ok(DirectorySort::Size),
        other => Err(AppError::bad_request(format!("不支持的 sort: {other}"))),
    }
}

fn parse_order(value: Option<&str>) -> Result<SortOrder, AppError> {
    match value.unwrap_or("asc") {
        "asc" => Ok(SortOrder::Asc),
        "desc" => Ok(SortOrder::Desc),
        other => Err(AppError::bad_request(format!("不支持的 order: {other}"))),
    }
}

fn parse_filter(value: Option<&str>) -> Result<EntryFilter, AppError> {
    match value.unwrap_or("all") {
        "all" => Ok(EntryFilter::All),
        "file" => Ok(EntryFilter::File),
        "folder" => Ok(EntryFilter::Folder),
        other => Err(AppError::bad_request(format!("不支持的 type: {other}"))),
    }
}

pub fn file_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/file",
            get(get_root_metadata)
                .post(create_root_entry)
                .patch(move_root_entry)
                .delete(delete_root_entry),
        )
        .route(
            "/file/",
            get(get_root_metadata)
                .post(create_root_entry)
                .patch(move_root_entry)
                .delete(delete_root_entry),
        )
        .route(
            "/file/{*path}",
            get(get_metadata_by_path)
                .post(create_entry)
                .patch(move_entry)
                .delete(delete_entry),
        )
        .route(
            "/content",
            get(get_root_content)
                .head(head_root_content)
                .put(save_root_content),
        )
        .route(
            "/content/",
            get(get_root_content)
                .head(head_root_content)
                .put(save_root_content),
        )
        .route(
            "/content/{*path}",
            get(get_content)
                .head(head_content)
                .put(save_content_by_path),
        )
        .route(
            "/download",
            get(download_root_file).head(head_download_root_file),
        )
        .route(
            "/download/",
            get(download_root_file).head(head_download_root_file),
        )
        .route(
            "/download/{*path}",
            get(download_file).head(head_download_file),
        )
        .route("/upload", post(upload_root_file))
        .route("/upload/", post(upload_root_file))
        .route("/upload/{*path}", post(upload_file))
}

async fn get_root_metadata(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<DirectoryQuery>,
) -> Result<Response, AppError> {
    respond_metadata(state, String::new(), query, headers).await
}

async fn get_metadata_by_path(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    Query(query): Query<DirectoryQuery>,
) -> Result<Response, AppError> {
    respond_metadata(state, path, query, headers).await
}

async fn respond_metadata(
    state: Arc<AppState>,
    path: String,
    query: DirectoryQuery,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let _permit = state.limits.acquire_dir_scan()?;
    let snapshot = state.mapping_store.snapshot().await;
    let options = query.into_options(state.runtime_settings.max_dir_page_size)?;
    match path_resolver::metadata(snapshot, path, options).await? {
        MetadataEntry::Folder(data) => metadata_json_response(&headers, data),
        MetadataEntry::File(info) => metadata_json_response(&headers, info),
    }
}

fn metadata_json_response<T>(request_headers: &HeaderMap, value: T) -> Result<Response, AppError>
where
    T: serde::Serialize,
{
    let bytes = serde_json::to_vec(&value)?;
    let etag_value = weak_etag(&bytes);
    let not_modified = request_headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .any(|candidate| candidate.trim() == etag_value)
        })
        .unwrap_or(false);

    let mut response = if not_modified {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        Json(value).into_response()
    };
    response.headers_mut().insert(
        ETAG,
        HeaderValue::from_str(&etag_value)
            .map_err(|error| AppError::internal(format!("生成 ETag 失败: {error}")))?,
    );
    if let Some(last_modified) = last_modified_from_json(&bytes)? {
        response.headers_mut().insert(
            LAST_MODIFIED,
            HeaderValue::from_str(&last_modified)
                .map_err(|error| AppError::internal(format!("生成 Last-Modified 失败: {error}")))?,
        );
    }
    Ok(response)
}

fn weak_etag(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("W/\"{:016x}\"", hasher.finish())
}

fn last_modified_from_json(bytes: &[u8]) -> Result<Option<String>, AppError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    let mut max_seconds = None;
    collect_modified_seconds(&value, &mut max_seconds);
    let Some(seconds) = max_seconds else {
        return Ok(None);
    };
    let modified = OffsetDateTime::from_unix_timestamp(seconds)
        .map_err(|error| AppError::internal(format!("生成 Last-Modified 失败: {error}")))?
        .format(&Rfc2822)
        .map_err(|error| AppError::internal(format!("生成 Last-Modified 失败: {error}")))?;
    Ok(Some(modified))
}

fn collect_modified_seconds(value: &serde_json::Value, max_seconds: &mut Option<i64>) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(seconds) = object
                .get("modified")
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse::<i64>().ok())
            {
                *max_seconds = Some(
                    max_seconds
                        .map(|current| current.max(seconds))
                        .unwrap_or(seconds),
                );
            }
            for value in object.values() {
                collect_modified_seconds(value, max_seconds);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_modified_seconds(value, max_seconds);
            }
        }
        _ => {}
    }
}

async fn create_root_entry(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<FileOperationResponse>), AppError> {
    create_entry_at_path(state, String::new(), request).await
}

async fn create_entry(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Json(request): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<FileOperationResponse>), AppError> {
    create_entry_at_path(state, path, request).await
}

async fn create_entry_at_path(
    state: Arc<AppState>,
    path: String,
    request: CreateEntryRequest,
) -> Result<(StatusCode, Json<FileOperationResponse>), AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let response = file_ops::create_entry(snapshot, path, request).await?;
    audit_ignore(&state, "create", Some(&response.path), None).await;
    Ok((StatusCode::CREATED, Json(response)))
}

async fn move_root_entry(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MoveEntryRequest>,
) -> Result<Json<FileOperationResponse>, AppError> {
    move_entry_at_path(state, String::new(), request).await
}

async fn move_entry(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Json(request): Json<MoveEntryRequest>,
) -> Result<Json<FileOperationResponse>, AppError> {
    move_entry_at_path(state, path, request).await
}

async fn move_entry_at_path(
    state: Arc<AppState>,
    path: String,
    request: MoveEntryRequest,
) -> Result<Json<FileOperationResponse>, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let response = file_ops::move_entry(snapshot, path, request).await?;
    audit_ignore(&state, "move", Some(&response.path), None).await;
    Ok(Json(response))
}

async fn delete_root_entry(
    State(state): State<Arc<AppState>>,
) -> Result<Json<FileOperationResponse>, AppError> {
    delete_entry_at_path(state, String::new()).await
}

async fn delete_entry(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<FileOperationResponse>, AppError> {
    delete_entry_at_path(state, path).await
}

async fn delete_entry_at_path(
    state: Arc<AppState>,
    path: String,
) -> Result<Json<FileOperationResponse>, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let target = file_ops::resolve_delete_target(snapshot, path).await?;
    let original_virtual_path = target.virtual_path.clone();
    let original_real_path = target.real_path.to_string_lossy().to_string();
    state
        .trash
        .move_to_trash(
            target.real_path,
            original_virtual_path.clone(),
            original_real_path,
            "admin".to_string(),
        )
        .await?;
    audit_ignore(&state, "delete", Some(&original_virtual_path), None).await;

    Ok(Json(FileOperationResponse {
        path: original_virtual_path,
    }))
}

async fn get_root_content(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_content_at_path(state, String::new(), headers, false).await
}

async fn get_content(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_content_at_path(state, path, headers, false).await
}

async fn head_root_content(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_content_at_path(state, String::new(), headers, true).await
}

async fn head_content(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_content_at_path(state, path, headers, true).await
}

async fn stream_content_at_path(
    state: Arc<AppState>,
    path: String,
    headers: HeaderMap,
    head_only: bool,
) -> Result<Response, AppError> {
    let _permit = state.limits.acquire_transfer()?;
    let snapshot = state.mapping_store.snapshot().await;
    let resolved = path_resolver::resolve_existing(snapshot, path).await?;
    transfer::stream_existing_file(resolved, headers, false, head_only).await
}

async fn save_root_content(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<FileOperationResponse>, AppError> {
    save_content_at_path(state, String::new(), headers, body).await
}

async fn save_content_by_path(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<FileOperationResponse>, AppError> {
    save_content_at_path(state, path, headers, body).await
}

async fn save_content_at_path(
    state: Arc<AppState>,
    path: String,
    headers: HeaderMap,
    body: Body,
) -> Result<Json<FileOperationResponse>, AppError> {
    let _permit = state.limits.acquire_transfer()?;
    let snapshot = state.mapping_store.snapshot().await;
    let response = transfer::save_content(
        snapshot,
        path,
        headers,
        body,
        state.runtime_settings.max_upload_bytes,
    )
    .await?;
    audit_ignore(&state, "save", Some(&response.0.path), None).await;
    Ok(response)
}

async fn download_root_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_download_at_path(state, String::new(), headers, false).await
}

async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_download_at_path(state, path, headers, false).await
}

async fn head_download_root_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_download_at_path(state, String::new(), headers, true).await
}

async fn head_download_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    stream_download_at_path(state, path, headers, true).await
}

async fn stream_download_at_path(
    state: Arc<AppState>,
    path: String,
    headers: HeaderMap,
    head_only: bool,
) -> Result<Response, AppError> {
    let _permit = state.limits.acquire_transfer()?;
    let snapshot = state.mapping_store.snapshot().await;
    let resolved = path_resolver::resolve_existing(snapshot, path).await?;
    let audit_path = resolved.virtual_path.clone();
    let response = transfer::stream_existing_file(resolved, headers, true, head_only).await?;
    if !head_only {
        audit_ignore(&state, "download", Some(&audit_path), None).await;
    }
    Ok(response)
}

async fn upload_root_file(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    upload_file_at_path(state, String::new(), multipart).await
}

async fn upload_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    upload_file_at_path(state, path, multipart).await
}

async fn upload_file_at_path(
    state: Arc<AppState>,
    path: String,
    multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let _permit = state.limits.acquire_transfer()?;
    let snapshot = state.mapping_store.snapshot().await;
    let response = transfer::upload_multipart(
        snapshot,
        path,
        multipart,
        state.runtime_settings.max_upload_bytes,
    )
    .await?;
    let detail = format!("files={}", response.1.0.files.len());
    audit_ignore(&state, "upload", None, Some(&detail)).await;
    Ok(response)
}

async fn audit_ignore(state: &AppState, action: &str, path: Option<&str>, detail: Option<&str>) {
    if let Err(error) = state.audit.record("admin", action, path, detail).await {
        tracing::warn!("写入审计日志失败: {error}");
    }
}
