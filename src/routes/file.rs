use axum::extract::DefaultBodyLimit;
use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED},
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
    models::{
        ConflictPolicy, CreateEntryRequest, FileInfo, FileOperationResponse, FolderData,
        MoveEntryRequest, UploadResponse,
    },
    services::{
        file_ops,
        path_resolver::{
            self, DirectoryDetail, DirectoryListOptions, DirectorySort, EntryFilter,
            MappingSnapshot, MetadataEntry, SortOrder,
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
    include_total: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteQuery {
    #[serde(default, alias = "conflict")]
    conflict_policy: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentQuery {
    mode: Option<String>,
}

impl WriteQuery {
    fn parse_conflict_policy(&self) -> Result<Option<ConflictPolicy>, AppError> {
        self.conflict_policy
            .as_deref()
            .map(|value| {
                value
                    .parse()
                    .map_err(|_| AppError::bad_request(format!("不支持的 conflictPolicy: {value}")))
            })
            .transpose()
    }
}

impl ContentQuery {
    fn requires_edit_policy(&self) -> Result<bool, AppError> {
        match self.mode.as_deref().unwrap_or("raw") {
            "raw" => Ok(false),
            "edit" => Ok(true),
            other => Err(AppError::bad_request(format!(
                "不支持的 content mode: {other}"
            ))),
        }
    }
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
            include_total: self.include_total.unwrap_or(false),
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
        .route(
            "/upload",
            post(upload_root_file).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/upload/",
            post(upload_root_file).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/upload/{*path}",
            post(upload_file).layer(DefaultBodyLimit::disable()),
        )
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
    let snapshot = state.mapping_store.snapshot().await;
    let options = query.into_options(state.runtime_settings.max_dir_page_size)?;
    if should_precheck_basic_not_modified(&headers, options)
        && let Some(modified) =
            path_resolver::basic_metadata_modified(snapshot.clone(), path.clone()).await?
    {
        let last_modified = last_modified_header(modified_seconds(&modified))?;
        if last_modified
            .as_deref()
            .is_some_and(|last_modified| if_modified_since_allows_304(&headers, last_modified))
        {
            let mut response = StatusCode::NOT_MODIFIED.into_response();
            insert_metadata_headers(response.headers_mut(), None, last_modified.as_deref())?;
            return Ok(response);
        }
    }
    let _permit = state.limits.acquire_dir_scan()?;
    match path_resolver::metadata(snapshot, path, options).await? {
        MetadataEntry::Folder { data, modified } => {
            let last_modified = folder_last_modified_header(&data, modified.as_deref())?;
            metadata_json_response(&headers, data, last_modified)
        }
        MetadataEntry::File(info) => {
            let last_modified = file_last_modified_header(&info)?;
            metadata_json_response(&headers, info, last_modified)
        }
    }
}

fn should_precheck_basic_not_modified(
    request_headers: &HeaderMap,
    options: DirectoryListOptions,
) -> bool {
    options.detail == DirectoryDetail::Basic
        && !request_headers.contains_key(IF_NONE_MATCH)
        && request_headers.contains_key(IF_MODIFIED_SINCE)
}

fn metadata_json_response<T>(
    request_headers: &HeaderMap,
    value: T,
    last_modified: Option<String>,
) -> Result<Response, AppError>
where
    T: serde::Serialize,
{
    if !request_headers.contains_key(IF_NONE_MATCH)
        && last_modified.as_deref().is_some_and(|last_modified| {
            if_modified_since_allows_304(request_headers, last_modified)
        })
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        insert_metadata_headers(response.headers_mut(), None, last_modified.as_deref())?;
        return Ok(response);
    }

    let bytes = serde_json::to_vec(&value)?;
    let etag_value = weak_etag(&bytes);
    let not_modified = match if_none_match_result(request_headers, &etag_value) {
        Some(matches) => matches,
        None => last_modified.as_deref().is_some_and(|last_modified| {
            if_modified_since_allows_304(request_headers, last_modified)
        }),
    };

    let mut response = if not_modified {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        let mut response = Response::new(Body::from(bytes));
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        response
    };
    insert_metadata_headers(
        response.headers_mut(),
        Some(&etag_value),
        last_modified.as_deref(),
    )?;
    Ok(response)
}

fn insert_metadata_headers(
    headers: &mut HeaderMap,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<(), AppError> {
    if let Some(etag) = etag {
        headers.insert(
            ETAG,
            HeaderValue::from_str(etag)
                .map_err(|error| AppError::internal(format!("生成 ETag 失败: {error}")))?,
        );
    }
    if let Some(last_modified) = last_modified {
        headers.insert(
            LAST_MODIFIED,
            HeaderValue::from_str(last_modified)
                .map_err(|error| AppError::internal(format!("生成 Last-Modified 失败: {error}")))?,
        );
    }
    Ok(())
}

fn folder_last_modified_header(
    data: &FolderData,
    directory_modified: Option<&str>,
) -> Result<Option<String>, AppError> {
    let max_seconds = data
        .folder
        .iter()
        .filter_map(|folder| modified_seconds(&folder.modified))
        .chain(
            data.file
                .iter()
                .filter_map(|file| modified_seconds(&file.modified)),
        )
        .chain(directory_modified.and_then(modified_seconds))
        .max();
    last_modified_header(max_seconds)
}

fn file_last_modified_header(info: &FileInfo) -> Result<Option<String>, AppError> {
    last_modified_header(modified_seconds(&info.modified))
}

fn modified_seconds(value: &str) -> Option<i64> {
    value.parse().ok()
}

fn last_modified_header(seconds: Option<i64>) -> Result<Option<String>, AppError> {
    let Some(seconds) = seconds else {
        return Ok(None);
    };
    let modified = OffsetDateTime::from_unix_timestamp(seconds)
        .map_err(|error| AppError::internal(format!("生成 Last-Modified 失败: {error}")))?
        .format(&Rfc2822)
        .map_err(|error| AppError::internal(format!("生成 Last-Modified 失败: {error}")))?;
    Ok(Some(modified))
}

fn if_none_match_result(request_headers: &HeaderMap, etag_value: &str) -> Option<bool> {
    request_headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .any(|candidate| etag_candidate_matches(candidate.trim(), etag_value))
        })
}

fn etag_candidate_matches(candidate: &str, etag_value: &str) -> bool {
    candidate == "*"
        || candidate == etag_value
        || strip_weak_prefix(candidate) == strip_weak_prefix(etag_value)
}

fn strip_weak_prefix(value: &str) -> &str {
    value.strip_prefix("W/").unwrap_or(value)
}

fn if_modified_since_allows_304(request_headers: &HeaderMap, last_modified: &str) -> bool {
    let Some(request_time) = request_headers
        .get(IF_MODIFIED_SINCE)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_http_date)
    else {
        return false;
    };
    let Some(last_modified) = parse_http_date(last_modified) else {
        return false;
    };

    last_modified <= request_time
}

fn parse_http_date(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc2822).ok()
}

fn weak_etag(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("W/\"{:016x}\"", hasher.finish())
}

async fn create_root_entry(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WriteQuery>,
    Json(request): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<FileOperationResponse>), AppError> {
    create_entry_at_path(state, String::new(), query, request).await
}

async fn create_entry(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(query): Query<WriteQuery>,
    Json(request): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<FileOperationResponse>), AppError> {
    create_entry_at_path(state, path, query, request).await
}

async fn create_entry_at_path(
    state: Arc<AppState>,
    path: String,
    query: WriteQuery,
    request: CreateEntryRequest,
) -> Result<(StatusCode, Json<FileOperationResponse>), AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let policy = write_policy(&state, &query, request.conflict_policy)?;
    let response = file_ops::create_entry(snapshot.clone(), path, request, policy).await?;
    index_upsert_ignore(&state, snapshot, &response.path).await;
    audit_ignore(&state, "create", Some(&response.path), None).await;
    Ok((StatusCode::CREATED, Json(response)))
}

async fn move_root_entry(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WriteQuery>,
    Json(request): Json<MoveEntryRequest>,
) -> Result<Json<FileOperationResponse>, AppError> {
    move_entry_at_path(state, String::new(), query, request).await
}

async fn move_entry(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(query): Query<WriteQuery>,
    Json(request): Json<MoveEntryRequest>,
) -> Result<Json<FileOperationResponse>, AppError> {
    move_entry_at_path(state, path, query, request).await
}

async fn move_entry_at_path(
    state: Arc<AppState>,
    path: String,
    query: WriteQuery,
    request: MoveEntryRequest,
) -> Result<Json<FileOperationResponse>, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let policy = write_policy(&state, &query, request.conflict_policy)?;
    let response = file_ops::move_entry(snapshot.clone(), path.clone(), request, policy).await?;
    index_move_ignore(&state, &path, &response.path).await;
    index_upsert_ignore(&state, snapshot, &response.path).await;
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
    index_remove_ignore(&state, &original_virtual_path).await;

    Ok(Json(FileOperationResponse {
        path: original_virtual_path,
    }))
}

async fn get_root_content(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ContentQuery>,
) -> Result<Response, AppError> {
    stream_content_at_path(state, String::new(), headers, query, false).await
}

async fn get_content(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    Query(query): Query<ContentQuery>,
) -> Result<Response, AppError> {
    stream_content_at_path(state, path, headers, query, false).await
}

async fn head_root_content(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ContentQuery>,
) -> Result<Response, AppError> {
    stream_content_at_path(state, String::new(), headers, query, true).await
}

async fn head_content(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    Query(query): Query<ContentQuery>,
) -> Result<Response, AppError> {
    stream_content_at_path(state, path, headers, query, true).await
}

async fn stream_content_at_path(
    state: Arc<AppState>,
    path: String,
    headers: HeaderMap,
    query: ContentQuery,
    head_only: bool,
) -> Result<Response, AppError> {
    let permit = state.limits.acquire_transfer()?;
    let snapshot = state.mapping_store.snapshot().await;
    let resolved = path_resolver::resolve_existing(snapshot, path).await?;
    let edit_policy = query
        .requires_edit_policy()?
        .then(|| edit_policy_from_state(&state));
    transfer::stream_existing_file(
        resolved,
        headers,
        false,
        head_only,
        edit_policy.as_ref(),
        permit,
    )
    .await
}

async fn save_root_content(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Body,
) -> Result<(HeaderMap, Json<FileOperationResponse>), AppError> {
    save_content_at_path(state, String::new(), headers, body).await
}

async fn save_content_by_path(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: Body,
) -> Result<(HeaderMap, Json<FileOperationResponse>), AppError> {
    save_content_at_path(state, path, headers, body).await
}

async fn save_content_at_path(
    state: Arc<AppState>,
    path: String,
    headers: HeaderMap,
    body: Body,
) -> Result<(HeaderMap, Json<FileOperationResponse>), AppError> {
    let _permit = state.limits.acquire_transfer()?;
    let snapshot = state.mapping_store.snapshot().await;
    let response = transfer::save_content(
        snapshot.clone(),
        path,
        headers,
        body,
        state.runtime_settings.max_upload_bytes,
        &edit_policy_from_state(&state),
    )
    .await?;
    index_upsert_ignore(&state, snapshot, &response.1.0.path).await;
    audit_ignore(&state, "save", Some(&response.1.0.path), None).await;
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
    let permit = state.limits.acquire_transfer()?;
    let snapshot = state.mapping_store.snapshot().await;
    let resolved = path_resolver::resolve_existing(snapshot, path).await?;
    transfer::stream_existing_file(resolved, headers, true, head_only, None, permit).await
}

async fn upload_root_file(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WriteQuery>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    upload_file_at_path(state, String::new(), query, multipart).await
}

async fn upload_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(query): Query<WriteQuery>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    upload_file_at_path(state, path, query, multipart).await
}

async fn upload_file_at_path(
    state: Arc<AppState>,
    path: String,
    query: WriteQuery,
    multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let _permit = state.limits.acquire_transfer()?;
    let snapshot = state.mapping_store.snapshot().await;
    let policy = write_policy(&state, &query, None)?;
    let response = transfer::upload_multipart(
        snapshot.clone(),
        path,
        multipart,
        state.runtime_settings.max_upload_bytes,
        policy,
    )
    .await?;
    let detail = format!("files={}", response.1.0.files.len());
    for file in &response.1.0.files {
        index_upsert_ignore(&state, snapshot.clone(), &file.path).await;
    }
    audit_ignore(&state, "upload", None, Some(&detail)).await;
    Ok(response)
}

async fn index_upsert_ignore(state: &AppState, snapshot: Arc<MappingSnapshot>, virtual_path: &str) {
    if let Err(error) = state
        .search
        .upsert_virtual_path(snapshot, virtual_path.to_string())
        .await
    {
        tracing::warn!("更新搜索索引失败: {error}");
    }
}

async fn index_remove_ignore(state: &AppState, virtual_path: &str) {
    if let Err(error) = state.search.remove_virtual_path(virtual_path).await {
        tracing::warn!("移除搜索索引失败: {error}");
    }
}

async fn index_move_ignore(state: &AppState, old_path: &str, new_path: &str) {
    if let Err(error) = state.search.move_virtual_path(old_path, new_path).await {
        tracing::warn!("移动搜索索引失败: {error}");
    }
}

async fn audit_ignore(state: &AppState, action: &str, path: Option<&str>, detail: Option<&str>) {
    if let Err(error) = state.audit.record("admin", action, path, detail).await {
        tracing::warn!("写入审计日志失败: {error}");
    }
}

fn write_policy(
    state: &AppState,
    query: &WriteQuery,
    request_policy: Option<ConflictPolicy>,
) -> Result<ConflictPolicy, AppError> {
    Ok(query
        .parse_conflict_policy()?
        .or(request_policy)
        .unwrap_or(state.runtime_settings.conflict_policy))
}

fn edit_policy_from_state(state: &AppState) -> transfer::EditablePolicy {
    transfer::EditablePolicy {
        max_bytes: state.runtime_settings.max_edit_bytes,
        extensions: state.runtime_settings.editable_extensions.clone(),
        mime_types: state.runtime_settings.editable_mime_types.clone(),
    }
}

#[cfg(test)]
mod tests {
    use serde::{Serialize, Serializer, ser::Error as _};

    use super::*;

    #[test]
    fn if_modified_since_hit_skips_metadata_serialization() {
        let last_modified = last_modified_header(Some(0)).unwrap().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            IF_MODIFIED_SINCE,
            HeaderValue::from_str(&last_modified).unwrap(),
        );

        let response =
            metadata_json_response(&headers, SerializeFails, Some(last_modified.clone())).unwrap();

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(
            response
                .headers()
                .get(LAST_MODIFIED)
                .unwrap()
                .to_str()
                .unwrap(),
            last_modified
        );
        assert!(response.headers().get(ETAG).is_none());
    }

    #[test]
    fn if_none_match_keeps_priority_over_if_modified_since() {
        let last_modified = last_modified_header(Some(0)).unwrap().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(IF_NONE_MATCH, HeaderValue::from_static("W/\"not-current\""));
        headers.insert(
            IF_MODIFIED_SINCE,
            HeaderValue::from_str(&last_modified).unwrap(),
        );

        assert!(metadata_json_response(&headers, SerializeFails, Some(last_modified)).is_err());
    }

    #[test]
    fn basic_precheck_requires_basic_ims_without_etag() {
        let mut headers = HeaderMap::new();
        headers.insert(
            IF_MODIFIED_SINCE,
            HeaderValue::from_static("Wed, 21 Oct 2015 07:28:00 GMT"),
        );

        assert!(should_precheck_basic_not_modified(
            &headers,
            DirectoryListOptions::default()
        ));

        headers.insert(IF_NONE_MATCH, HeaderValue::from_static("W/\"cached\""));
        assert!(!should_precheck_basic_not_modified(
            &headers,
            DirectoryListOptions::default()
        ));

        headers.remove(IF_NONE_MATCH);
        assert!(!should_precheck_basic_not_modified(
            &headers,
            DirectoryListOptions {
                detail: DirectoryDetail::Full,
                ..DirectoryListOptions::default()
            }
        ));
    }

    struct SerializeFails;

    impl Serialize for SerializeFails {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("不应序列化"))
        }
    }
}
