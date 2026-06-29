use std::{
    fs::Metadata,
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use axum::{
    Json,
    body::Body,
    extract::{Multipart, multipart::Field},
    http::{
        HeaderMap, HeaderName, HeaderValue, StatusCode,
        header::{
            CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG, IF_MATCH, RANGE,
        },
    },
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom},
    sync::OwnedSemaphorePermit,
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{ConflictPolicy, FileOperationResponse, UploadItemError, UploadResponse},
    services::{
        conflict,
        path_resolver::{
            MappingSnapshot, ResolvedPath, ensure_file, ensure_folder, ensure_not_mount_root,
            ensure_writable, join_virtual_path, normalize_child_name, resolve_existing,
        },
    },
};

const ACCEPT_RANGES_VALUE: &str = "bytes";
const ACCEPT_RANGES_HEADER: HeaderName = HeaderName::from_static("accept-ranges");
const STREAM_BUFFER_SIZE: usize = 256 * 1024;
const WRITE_BUFFER_SIZE: usize = 256 * 1024;
const TEXT_SAMPLE_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Default)]
pub struct EditablePolicy {
    pub max_bytes: u64,
    pub extensions: Vec<String>,
    pub mime_types: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    fn len(self) -> u64 {
        self.end - self.start + 1
    }
}

pub async fn stream_existing_file(
    resolved: ResolvedPath,
    headers: HeaderMap,
    attachment: bool,
    head_only: bool,
    edit_policy: Option<&EditablePolicy>,
    transfer_permit: OwnedSemaphorePermit,
) -> Result<Response, AppError> {
    let mut file = if head_only && edit_policy.is_none() {
        None
    } else {
        Some(File::open(&resolved.real_path).await?)
    };
    let metadata = match file.as_ref() {
        Some(file) => file.metadata().await?,
        None => fs::metadata(&resolved.real_path).await?,
    };
    if !metadata.is_file() {
        return Err(
            AppError::bad_request(format!("路径不是文件: {}", resolved.virtual_path))
                .with_reason("PATH_NOT_FILE")
                .with_param("path", resolved.virtual_path.clone()),
        );
    }
    if let Some(policy) = edit_policy {
        let file = file
            .as_mut()
            .ok_or_else(|| AppError::internal("编辑模式 HEAD 未打开文件"))?;
        ensure_editable_file(
            file,
            &metadata,
            &resolved.real_path,
            &resolved.virtual_path,
            policy,
        )
        .await?;
    }
    let size = metadata.len();
    let etag = content_etag(&metadata);
    let range = parse_range_header(headers.get(RANGE), size)?;
    let (status, start, length) = match range {
        Some(range) => (StatusCode::PARTIAL_CONTENT, range.start, range.len()),
        None => (StatusCode::OK, 0, size),
    };

    let mime = mime_guess::from_path(&resolved.real_path).first_or_octet_stream();
    let mut response = if head_only {
        Body::empty().into_response()
    } else {
        let mut file = file.ok_or_else(|| AppError::internal("内容读取请求未打开文件"))?;
        if start > 0 {
            file.seek(SeekFrom::Start(start)).await?;
        }
        let stream = ReaderStream::with_capacity(file.take(length), STREAM_BUFFER_SIZE);
        let stream =
            futures_util::stream::unfold((stream, transfer_permit), |(mut stream, permit)| async {
                stream.next().await.map(|chunk| (chunk, (stream, permit)))
            });
        Body::from_stream(stream).into_response()
    };
    *response.status_mut() = status;

    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        ACCEPT_RANGES_HEADER,
        HeaderValue::from_static(ACCEPT_RANGES_VALUE),
    );
    insert_header_value(headers, ETAG, &etag)?;
    headers.insert(
        CONTENT_LENGTH,
        HeaderValue::from_str(&length.to_string())
            .map_err(|error| AppError::internal(format!("生成 Content-Length 失败: {error}")))?,
    );
    if let Some(range) = range {
        headers.insert(
            CONTENT_RANGE,
            HeaderValue::from_str(&format!("bytes {}-{}/{}", range.start, range.end, size))
                .map_err(|error| AppError::internal(format!("生成 Content-Range 失败: {error}")))?,
        );
    }
    if attachment {
        headers.insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&attachment_disposition(&resolved.real_path))
                .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
        );
    }

    Ok(response)
}

pub async fn save_content(
    snapshot: Arc<MappingSnapshot>,
    path: String,
    headers: HeaderMap,
    body: Body,
    max_upload_bytes: Option<u64>,
    edit_policy: &EditablePolicy,
) -> Result<(HeaderMap, Json<FileOperationResponse>), AppError> {
    let max_bytes = effective_write_limit(max_upload_bytes, edit_policy.max_bytes);
    ensure_declared_length_within_limit(&headers, max_bytes)?;
    let if_match = required_if_match(&headers)?;
    let resolved = resolve_existing(snapshot, path).await?;
    ensure_writable(&resolved.mapping)?;
    ensure_not_mount_root(&resolved)?;
    ensure_file(&resolved.real_path, &resolved.virtual_path)?;
    ensure_editable_path(&resolved.real_path, &resolved.virtual_path, edit_policy).await?;
    verify_if_match(&if_match, &resolved.real_path, &resolved.virtual_path).await?;

    let temp_path = temp_path_for(&resolved.real_path)?;
    let result = write_body_to_file(body, &temp_path, max_bytes, true).await;
    if result.is_err() {
        let _ = fs::remove_file(&temp_path).await;
    }
    result?;
    if let Err(error) =
        verify_if_match(&if_match, &resolved.real_path, &resolved.virtual_path).await
    {
        let _ = fs::remove_file(&temp_path).await;
        return Err(error);
    }

    if let Err(error) = replace_file(&temp_path, &resolved.real_path).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(error);
    }

    let etag = current_content_etag(&resolved.real_path, &resolved.virtual_path).await?;
    let mut response_headers = HeaderMap::new();
    insert_header_value(&mut response_headers, ETAG, &etag)?;

    Ok((
        response_headers,
        Json(FileOperationResponse {
            path: resolved.virtual_path,
        }),
    ))
}

pub async fn upload_multipart(
    snapshot: Arc<MappingSnapshot>,
    parent_path: String,
    mut multipart: Multipart,
    max_bytes: Option<u64>,
    policy: ConflictPolicy,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let parent = resolve_existing(snapshot, parent_path).await?;
    ensure_writable(&parent.mapping)?;
    ensure_folder(&parent.real_path, &parent.virtual_path)?;

    let mut files = Vec::new();
    let mut errors = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        let Some(file_name) = field.file_name().map(ToString::to_string) else {
            continue;
        };
        match upload_field(&parent, file_name.clone(), field, max_bytes, policy).await {
            Ok(file) => files.push(file),
            Err(error) if files.is_empty() => return Err(error),
            Err(error) => {
                errors.push(upload_item_error(file_name, &error));
                break;
            }
        }
    }

    let success = files.len();
    let failed = errors.len();
    let status = if failed == 0 {
        StatusCode::CREATED
    } else {
        StatusCode::MULTI_STATUS
    };
    Ok((
        status,
        Json(UploadResponse {
            files,
            errors,
            success,
            failed,
        }),
    ))
}

pub fn parse_range_header(
    value: Option<&HeaderValue>,
    size: u64,
) -> Result<Option<ByteRange>, AppError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value
        .to_str()
        .map_err(|_| range_error("Range 头无效", "RANGE_HEADER_INVALID"))?;
    parse_range(value, size).map(Some)
}

fn parse_range(value: &str, size: u64) -> Result<ByteRange, AppError> {
    if size == 0 {
        return Err(range_error("空文件不支持 Range 请求", "RANGE_EMPTY_FILE"));
    }
    let Some(spec) = value.strip_prefix("bytes=") else {
        return Err(range_error("仅支持 bytes Range", "RANGE_UNIT_UNSUPPORTED"));
    };
    if spec.contains(',') {
        return Err(range_error(
            "暂不支持多段 Range",
            "RANGE_MULTIPLE_UNSUPPORTED",
        ));
    }

    let Some((start, end)) = spec.split_once('-') else {
        return Err(range_error("Range 格式无效", "RANGE_FORMAT_INVALID"));
    };
    if start.is_empty() {
        let suffix: u64 = end
            .parse()
            .map_err(|_| range_error("Range 后缀无效", "RANGE_SUFFIX_INVALID"))?;
        if suffix == 0 {
            return Err(range_error("Range 后缀不能为 0", "RANGE_SUFFIX_ZERO"));
        }
        let length = suffix.min(size);
        return Ok(ByteRange {
            start: size - length,
            end: size - 1,
        });
    }

    let start: u64 = start
        .parse()
        .map_err(|_| range_error("Range 起点无效", "RANGE_START_INVALID"))?;
    if start >= size {
        return Err(range_error("Range 起点越界", "RANGE_START_OUT_OF_BOUNDS")
            .with_param("start", start)
            .with_param("size", size));
    }
    let end = if end.is_empty() {
        size - 1
    } else {
        end.parse()
            .map_err(|_| range_error("Range 终点无效", "RANGE_END_INVALID"))?
    };
    if end < start {
        return Err(range_error("Range 终点小于起点", "RANGE_END_BEFORE_START")
            .with_param("start", start)
            .with_param("end", end));
    }

    Ok(ByteRange {
        start,
        end: end.min(size - 1),
    })
}

async fn upload_field(
    parent: &ResolvedPath,
    file_name: String,
    mut field: Field<'_>,
    max_bytes: Option<u64>,
    policy: ConflictPolicy,
) -> Result<FileOperationResponse, AppError> {
    let name = normalize_child_name(&file_name)?;
    let virtual_path = join_virtual_path(&parent.virtual_path, &name);
    let target = conflict::resolve_child(&parent.real_path, &name, &virtual_path, policy)?;
    conflict::ensure_file_overwrite_allowed(&target)?;

    let temp_path = temp_path_for(&target.path)?;
    let result = write_field_to_file(&mut field, &temp_path, max_bytes).await;
    if result.is_err() {
        let _ = fs::remove_file(&temp_path).await;
    }
    result?;
    if target.existed {
        if let Err(error) = replace_file(&temp_path, &target.path).await {
            let _ = fs::remove_file(&temp_path).await;
            return Err(error);
        }
    } else {
        if conflict::path_entry_exists(&target.path)? {
            let _ = fs::remove_file(&temp_path).await;
            let path = join_virtual_path(&parent.virtual_path, &target.name);
            return Err(AppError::conflict(format!("路径已存在: {path}"))
                .with_reason("PATH_ALREADY_EXISTS")
                .with_param("path", path));
        }
        if let Err(error) = fs::rename(&temp_path, &target.path).await {
            let _ = fs::remove_file(&temp_path).await;
            return Err(error.into());
        }
    }

    Ok(FileOperationResponse {
        path: join_virtual_path(&parent.virtual_path, &target.name),
    })
}

fn upload_item_error(file_name: String, error: &AppError) -> UploadItemError {
    UploadItemError {
        file_name,
        code: error.code().to_string(),
        reason: error.reason().to_string(),
        message: error.to_string(),
        params: error.params().cloned(),
    }
}

async fn write_body_to_file(
    body: Body,
    temp_path: &Path,
    max_bytes: Option<u64>,
    validate_text: bool,
) -> Result<(), AppError> {
    let mut stream = body.into_data_stream();
    let mut file = BufWriter::with_capacity(WRITE_BUFFER_SIZE, create_temp_file(temp_path).await?);
    let mut written = 0_u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| AppError::bad_request(error.to_string()))?;
        written = checked_add_len(written, chunk.len(), max_bytes)?;
        if validate_text {
            ensure_text_chunk(&chunk)?;
        }
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    file.shutdown().await?;
    Ok(())
}

async fn write_field_to_file(
    field: &mut Field<'_>,
    temp_path: &Path,
    max_bytes: Option<u64>,
) -> Result<(), AppError> {
    let mut file = BufWriter::with_capacity(WRITE_BUFFER_SIZE, create_temp_file(temp_path).await?);
    let mut written = 0_u64;
    while let Some(chunk) = field.chunk().await? {
        written = checked_add_len(written, chunk.len(), max_bytes)?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    file.shutdown().await?;
    Ok(())
}

fn ensure_declared_length_within_limit(
    headers: &HeaderMap,
    max_bytes: Option<u64>,
) -> Result<(), AppError> {
    let Some(max_bytes) = max_bytes else {
        return Ok(());
    };
    let Some(length) = headers.get(CONTENT_LENGTH) else {
        return Ok(());
    };
    let length = length
        .to_str()
        .map_err(|_| content_length_invalid())?
        .parse::<u64>()
        .map_err(|_| content_length_invalid())?;
    if length > max_bytes {
        return Err(
            AppError::payload_too_large(format!("上传内容超过限制: {max_bytes} bytes"))
                .with_reason("UPLOAD_SIZE_LIMIT_EXCEEDED")
                .with_param("maxBytes", max_bytes)
                .with_param("contentLength", length),
        );
    }
    Ok(())
}

fn effective_write_limit(max_upload_bytes: Option<u64>, max_edit_bytes: u64) -> Option<u64> {
    max_upload_bytes
        .map(|max_upload_bytes| max_upload_bytes.min(max_edit_bytes))
        .or(Some(max_edit_bytes))
}

async fn ensure_editable_path(
    path: &Path,
    virtual_path: &str,
    policy: &EditablePolicy,
) -> Result<(), AppError> {
    let mut file = File::open(path).await?;
    let metadata = file.metadata().await?;
    ensure_editable_file(&mut file, &metadata, path, virtual_path, policy).await
}

async fn ensure_editable_file(
    file: &mut File,
    metadata: &Metadata,
    path: &Path,
    virtual_path: &str,
    policy: &EditablePolicy,
) -> Result<(), AppError> {
    ensure_editable_kind(path, virtual_path, policy)?;

    if metadata.len() > policy.max_bytes {
        return Err(AppError::payload_too_large(format!(
            "文件超过在线编辑上限: {virtual_path}，当前 {} bytes，上限 {max_bytes} bytes",
            metadata.len(),
            max_bytes = policy.max_bytes
        ))
        .with_reason("EDIT_SIZE_LIMIT_EXCEEDED")
        .with_param("path", virtual_path)
        .with_param("size", metadata.len())
        .with_param("maxBytes", policy.max_bytes));
    }

    if metadata.len() == 0 {
        return Ok(());
    }

    file.seek(SeekFrom::Start(0)).await?;
    let sample_len = metadata.len().min(TEXT_SAMPLE_BYTES as u64) as usize;
    let mut sample = vec![0_u8; sample_len];
    file.read_exact(&mut sample).await?;
    let complete_sample = metadata.len() <= TEXT_SAMPLE_BYTES as u64;
    if !looks_like_text(&sample, complete_sample, path) {
        return Err(AppError::unsupported_media_type(format!(
            "文件看起来不是文本文件，不能在线编辑: {virtual_path}"
        ))
        .with_reason("EDIT_FILE_NOT_TEXT")
        .with_param("path", virtual_path));
    }
    file.seek(SeekFrom::Start(0)).await?;
    Ok(())
}

fn ensure_editable_kind(
    path: &Path,
    virtual_path: &str,
    policy: &EditablePolicy,
) -> Result<(), AppError> {
    if policy.extensions.is_empty() && policy.mime_types.is_empty() {
        return Ok(());
    }

    if editable_extension_matches(path, &policy.extensions)
        || editable_mime_matches(path, &policy.mime_types)
    {
        return Ok(());
    }

    Err(
        AppError::unsupported_media_type(format!("文件类型不在允许在线编辑范围内: {virtual_path}"))
            .with_reason("EDIT_FILE_TYPE_NOT_ALLOWED")
            .with_param("path", virtual_path),
    )
}

fn ensure_text_chunk(chunk: &[u8]) -> Result<(), AppError> {
    if chunk.contains(&0) {
        return Err(
            AppError::unsupported_media_type("保存内容包含二进制数据，已拒绝写入")
                .with_reason("EDIT_CONTENT_BINARY"),
        );
    }
    Ok(())
}

async fn verify_if_match(if_match: &str, path: &Path, virtual_path: &str) -> Result<(), AppError> {
    if if_match_allows(if_match, &current_content_etag(path, virtual_path).await?) {
        Ok(())
    } else {
        Err(AppError::precondition_failed(format!(
            "文件已被外部修改，请刷新后再保存: {virtual_path}"
        ))
        .with_reason("FILE_MODIFIED")
        .with_param("path", virtual_path))
    }
}

async fn current_content_etag(path: &Path, virtual_path: &str) -> Result<String, AppError> {
    let metadata = fs::metadata(path).await.map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::precondition_failed(format!("文件已不存在，请刷新后再保存: {virtual_path}"))
                .with_reason("FILE_MISSING_BEFORE_SAVE")
                .with_param("path", virtual_path)
        } else {
            AppError::from(error)
        }
    })?;
    if !metadata.is_file() {
        return Err(AppError::precondition_failed(format!(
            "路径已不再是文件，请刷新后再保存: {virtual_path}"
        ))
        .with_reason("PATH_NOT_FILE_BEFORE_SAVE")
        .with_param("path", virtual_path));
    }
    Ok(content_etag(&metadata))
}

fn required_if_match(headers: &HeaderMap) -> Result<String, AppError> {
    let value = headers.get(IF_MATCH).ok_or_else(|| {
        AppError::precondition_required("保存文件需要 If-Match 头，请重新打开文件后再保存")
            .with_reason("IF_MATCH_REQUIRED")
    })?;
    let value = value
        .to_str()
        .map_err(|_| AppError::bad_request("If-Match 头无效").with_reason("IF_MATCH_INVALID"))?
        .trim();
    if value.is_empty() {
        return Err(AppError::bad_request("If-Match 头不能为空").with_reason("IF_MATCH_EMPTY"));
    }
    Ok(value.to_string())
}

fn if_match_allows(if_match: &str, current_etag: &str) -> bool {
    if_match
        .split(',')
        .map(str::trim)
        .any(|candidate| candidate == "*" || candidate == current_etag)
}

fn content_etag(metadata: &Metadata) -> String {
    let modified_nanos = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("\"wfb-{:x}-{:x}\"", metadata.len(), modified_nanos)
}

fn insert_header_value(
    headers: &mut HeaderMap,
    name: HeaderName,
    value: &str,
) -> Result<(), AppError> {
    headers.insert(
        name,
        HeaderValue::from_str(value)
            .map_err(|error| AppError::internal(format!("生成响应头失败: {error}")))?,
    );
    Ok(())
}

async fn create_temp_file(path: &Path) -> Result<File, AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .await?)
}

async fn replace_file(temp_path: &Path, target: &Path) -> Result<(), AppError> {
    if conflict::path_entry_exists(target)? {
        conflict::ensure_file_target_replaceable(target)?;
    }
    match fs::rename(temp_path, target).await {
        Ok(()) => Ok(()),
        Err(error) => {
            if conflict::path_entry_exists(target)? {
                replace_existing_file(temp_path, target, error).await
            } else {
                Err(error.into())
            }
        }
    }
}

async fn replace_existing_file(
    temp_path: &Path,
    target: &Path,
    first_error: std::io::Error,
) -> Result<(), AppError> {
    conflict::ensure_file_target_replaceable(target)?;

    let backup = replacement_backup_path(target);
    fs::rename(target, &backup).await.map_err(|backup_error| {
        AppError::internal(format!("准备替换文件失败: {first_error}; {backup_error}"))
    })?;

    match fs::rename(temp_path, target).await {
        Ok(()) => {
            let _ = fs::remove_file(&backup).await;
            Ok(())
        }
        Err(rename_error) => match fs::rename(&backup, target).await {
            Ok(()) => Err(AppError::internal(format!(
                "替换文件失败，已恢复旧文件: {first_error}; {rename_error}"
            ))),
            Err(restore_error) => Err(AppError::internal(format!(
                "替换文件失败，且恢复旧文件失败: {first_error}; {rename_error}; {restore_error}"
            ))),
        },
    }
}

fn replacement_backup_path(target: &Path) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("target");
    parent.join(format!(".{name}.replace-backup-{}", Uuid::new_v4()))
}

fn temp_path_for(target: &Path) -> Result<PathBuf, AppError> {
    let parent = target.parent().ok_or_else(|| {
        AppError::bad_request("目标路径没有父目录").with_reason("TARGET_PARENT_MISSING")
    })?;
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    Ok(parent.join(format!(".{name}.wfb-{}.tmp", Uuid::new_v4())))
}

fn checked_add_len(
    current: u64,
    chunk_len: usize,
    max_bytes: Option<u64>,
) -> Result<u64, AppError> {
    let next = current.checked_add(chunk_len as u64).ok_or_else(|| {
        AppError::payload_too_large("上传内容过大").with_reason("UPLOAD_SIZE_LIMIT_EXCEEDED")
    })?;
    if let Some(max_bytes) = max_bytes
        && next > max_bytes
    {
        return Err(
            AppError::payload_too_large(format!("上传内容超过限制: {max_bytes} bytes"))
                .with_reason("UPLOAD_SIZE_LIMIT_EXCEEDED")
                .with_param("maxBytes", max_bytes)
                .with_param("writtenBytes", next),
        );
    }
    Ok(next)
}

fn range_error(message: &'static str, reason: &'static str) -> AppError {
    AppError::range_not_satisfiable(message).with_reason(reason)
}

fn content_length_invalid() -> AppError {
    AppError::bad_request("Content-Length 无效").with_reason("CONTENT_LENGTH_INVALID")
}

fn looks_like_text(sample: &[u8], complete_sample: bool, path: &Path) -> bool {
    if sample.is_empty() {
        return true;
    }
    if sample.contains(&0) {
        return false;
    }
    if is_known_binary_extension(path) {
        return false;
    }
    if !is_utf8_sample(sample, complete_sample) {
        return false;
    }

    let suspicious_controls = sample
        .iter()
        .filter(|byte| {
            **byte < 0x20 && !matches!(**byte, b'\n' | b'\r' | b'\t' | 0x08 | 0x0c | 0x1b)
        })
        .count();
    suspicious_controls.saturating_mul(100) <= sample.len()
}

fn editable_extension_matches(path: &Path, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return false;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|extension| extensions.iter().any(|allowed| allowed == &extension))
}

fn editable_mime_matches(path: &Path, mime_types: &[String]) -> bool {
    if mime_types.is_empty() {
        return false;
    }
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string()
        .to_ascii_lowercase();
    mime_types.iter().any(|allowed| {
        if allowed == "*/*" || allowed == &mime {
            return true;
        }
        allowed
            .strip_suffix("/*")
            .is_some_and(|group| mime.starts_with(&format!("{group}/")))
    })
}

fn is_utf8_sample(sample: &[u8], complete_sample: bool) -> bool {
    match std::str::from_utf8(sample) {
        Ok(_) => true,
        Err(error) => !complete_sample && error.error_len().is_none(),
    }
}

fn is_known_binary_extension(path: &Path) -> bool {
    let Some(extension) = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
    else {
        return false;
    };

    matches!(
        extension.as_str(),
        "7z" | "avi"
            | "bmp"
            | "class"
            | "dll"
            | "doc"
            | "docx"
            | "exe"
            | "flac"
            | "gif"
            | "gz"
            | "ico"
            | "iso"
            | "jar"
            | "jpeg"
            | "jpg"
            | "m4a"
            | "mkv"
            | "mov"
            | "mp3"
            | "mp4"
            | "o"
            | "ogg"
            | "pdf"
            | "png"
            | "ppt"
            | "pptx"
            | "rar"
            | "so"
            | "tar"
            | "wav"
            | "webm"
            | "webp"
            | "xls"
            | "xlsx"
            | "xz"
            | "zip"
    )
}

fn attachment_disposition(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download")
        .replace('"', "");
    format!("attachment; filename=\"{file_name}\"")
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use axum::{
        body::Body,
        http::{
            HeaderMap, HeaderValue,
            header::{CONTENT_LENGTH, ETAG, IF_MATCH},
        },
    };

    use crate::error::AppError;
    use crate::{models::PathMapping, services::path_resolver::MappingSnapshot};

    use super::{
        ByteRange, EditablePolicy, content_etag, editable_extension_matches, editable_mime_matches,
        ensure_declared_length_within_limit, if_match_allows, parse_range, replace_file,
        save_content,
    };

    #[test]
    fn parses_closed_range() {
        assert_eq!(
            parse_range("bytes=0-9", 100).unwrap(),
            ByteRange { start: 0, end: 9 }
        );
    }

    #[test]
    fn parses_open_ended_range() {
        assert_eq!(
            parse_range("bytes=10-", 100).unwrap(),
            ByteRange { start: 10, end: 99 }
        );
    }

    #[test]
    fn parses_suffix_range() {
        assert_eq!(
            parse_range("bytes=-20", 100).unwrap(),
            ByteRange { start: 80, end: 99 }
        );
    }

    #[test]
    fn clamps_end_range() {
        assert_eq!(
            parse_range("bytes=90-999", 100).unwrap(),
            ByteRange { start: 90, end: 99 }
        );
    }

    #[test]
    fn rejects_invalid_ranges() {
        assert!(parse_range("items=0-9", 100).is_err());
        assert!(parse_range("bytes=10-1", 100).is_err());
        assert!(parse_range("bytes=100-101", 100).is_err());
        assert!(parse_range("bytes=0-1,5-6", 100).is_err());
        assert!(parse_range("bytes=-0", 100).is_err());
    }

    #[test]
    fn rejects_declared_length_over_limit() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static("1024"));

        let error = ensure_declared_length_within_limit(&headers, Some(512)).unwrap_err();
        assert!(matches!(error, AppError::PayloadTooLarge(_)));
    }

    #[test]
    fn matches_if_match_values() {
        assert!(if_match_allows("\"a\", \"b\"", "\"b\""));
        assert!(if_match_allows("*", "\"anything\""));
        assert!(!if_match_allows("\"a\"", "\"b\""));
    }

    #[tokio::test]
    async fn save_requires_if_match() {
        let (snapshot, temp) = snapshot_with_file("save-requires-if-match", "hello").await;

        let error = save_content(
            snapshot,
            "/repo/a.txt".to_string(),
            HeaderMap::new(),
            Body::from("new"),
            None,
            &edit_policy(1024),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::PreconditionRequired(_)));
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn save_rejects_stale_if_match() {
        let (snapshot, temp) = snapshot_with_file("save-stale-if-match", "hello").await;
        let file = temp.join("a.txt");
        let old_etag = content_etag(&fs::metadata(&file).unwrap());
        fs::write(&file, "external-change").unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(IF_MATCH, HeaderValue::from_str(&old_etag).unwrap());

        let error = save_content(
            snapshot,
            "/repo/a.txt".to_string(),
            headers,
            Body::from("new"),
            None,
            &edit_policy(1024),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::PreconditionFailed(_)));
        assert_eq!(fs::read_to_string(file).unwrap(), "external-change");
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn save_returns_new_etag_after_match() {
        let (snapshot, temp) = snapshot_with_file("save-new-etag", "hello").await;
        let file = temp.join("a.txt");
        let old_etag = content_etag(&fs::metadata(&file).unwrap());
        let mut headers = HeaderMap::new();
        headers.insert(IF_MATCH, HeaderValue::from_str(&old_etag).unwrap());

        let (response_headers, response) = save_content(
            snapshot,
            "/repo/a.txt".to_string(),
            headers,
            Body::from("new-content"),
            None,
            &edit_policy(1024),
        )
        .await
        .unwrap();

        assert_eq!(response.0.path, "/repo/a.txt");
        assert_eq!(fs::read_to_string(&file).unwrap(), "new-content");
        assert!(response_headers.get(ETAG).is_some());
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn replace_file_rejects_directory_target() {
        let temp = temp_dir("replace-dir-target");
        fs::create_dir_all(&temp).unwrap();
        let temp_file = temp.join("upload.tmp");
        let target = temp.join("target");
        fs::write(&temp_file, "new").unwrap();
        fs::create_dir(&target).unwrap();

        let result = replace_file(&temp_file, &target).await;

        assert!(result.is_err());
        assert!(temp_file.exists());
        assert!(target.is_dir());
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn save_rejects_large_edit_target() {
        let (snapshot, temp) = snapshot_with_file("save-large-edit", "hello").await;
        let file = temp.join("a.txt");
        let etag = content_etag(&fs::metadata(&file).unwrap());
        let mut headers = HeaderMap::new();
        headers.insert(IF_MATCH, HeaderValue::from_str(&etag).unwrap());

        let error = save_content(
            snapshot,
            "/repo/a.txt".to_string(),
            headers,
            Body::from("new"),
            None,
            &edit_policy(4),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::PayloadTooLarge(_)));
        assert_eq!(fs::read_to_string(file).unwrap(), "hello");
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn save_rejects_binary_edit_target() {
        let (snapshot, temp) = snapshot_with_file("save-binary-edit", "hello").await;
        let file = temp.join("a.txt");
        fs::write(&file, [0_u8, 1, 2, 3]).unwrap();
        let etag = content_etag(&fs::metadata(&file).unwrap());
        let mut headers = HeaderMap::new();
        headers.insert(IF_MATCH, HeaderValue::from_str(&etag).unwrap());

        let error = save_content(
            snapshot,
            "/repo/a.txt".to_string(),
            headers,
            Body::from("new"),
            None,
            &edit_policy(1024),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::UnsupportedMediaType(_)));
        assert_eq!(fs::read(file).unwrap(), vec![0_u8, 1, 2, 3]);
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn save_rejects_unlisted_edit_extension() {
        let (snapshot, temp) = snapshot_with_file("save-edit-extension", "hello").await;
        let file = temp.join("a.txt");
        let etag = content_etag(&fs::metadata(&file).unwrap());
        let mut headers = HeaderMap::new();
        headers.insert(IF_MATCH, HeaderValue::from_str(&etag).unwrap());

        let error = save_content(
            snapshot,
            "/repo/a.txt".to_string(),
            headers,
            Body::from("new"),
            None,
            &EditablePolicy {
                max_bytes: 1024,
                extensions: vec!["rs".to_string()],
                mime_types: Vec::new(),
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::UnsupportedMediaType(_)));
        assert_eq!(fs::read_to_string(file).unwrap(), "hello");
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn matches_editable_extension_and_mime_rules() {
        assert!(editable_extension_matches(
            std::path::Path::new("README.MD"),
            &["md".to_string()]
        ));
        assert!(editable_mime_matches(
            std::path::Path::new("config.json"),
            &["application/json".to_string()]
        ));
        assert!(editable_mime_matches(
            std::path::Path::new("note.txt"),
            &["text/*".to_string()]
        ));
        assert!(!editable_extension_matches(
            std::path::Path::new("image.png"),
            &["txt".to_string()]
        ));
    }

    fn edit_policy(max_bytes: u64) -> EditablePolicy {
        EditablePolicy {
            max_bytes,
            extensions: Vec::new(),
            mime_types: Vec::new(),
        }
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("web-file-browser-{prefix}-{nonce}"))
    }

    async fn snapshot_with_file(
        prefix: &str,
        content: &str,
    ) -> (std::sync::Arc<MappingSnapshot>, std::path::PathBuf) {
        let temp = temp_dir(prefix);
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("a.txt"), content).unwrap();
        let snapshot = MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: temp.to_string_lossy().to_string(),
            remark: Some(String::new()),
            order: Some(0),
            writable: true,
        }])
        .await
        .unwrap();
        (snapshot, temp)
    }
}
