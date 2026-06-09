use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    Json,
    body::Body,
    extract::{Multipart, multipart::Field},
    http::{
        HeaderMap, HeaderName, HeaderValue, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE},
    },
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom},
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{FileOperationResponse, UploadResponse},
    services::path_resolver::{
        MappingSnapshot, ResolvedPath, ensure_file, ensure_folder, ensure_not_mount_root,
        ensure_writable, join_virtual_path, normalize_child_name, resolve_existing,
    },
};

const ACCEPT_RANGES_VALUE: &str = "bytes";
const ACCEPT_RANGES_HEADER: HeaderName = HeaderName::from_static("accept-ranges");
const STREAM_BUFFER_SIZE: usize = 256 * 1024;
const WRITE_BUFFER_SIZE: usize = 256 * 1024;

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
) -> Result<Response, AppError> {
    ensure_file(&resolved.real_path, &resolved.virtual_path)?;
    let mut file = File::open(&resolved.real_path).await?;
    let size = file.metadata().await?.len();
    let range = parse_range_header(headers.get(RANGE), size)?;
    let (status, start, length) = match range {
        Some(range) => (StatusCode::PARTIAL_CONTENT, range.start, range.len()),
        None => (StatusCode::OK, 0, size),
    };

    if start > 0 {
        file.seek(SeekFrom::Start(start)).await?;
    }

    let mime = mime_guess::from_path(&resolved.real_path).first_or_octet_stream();
    let mut response = if head_only {
        Body::empty().into_response()
    } else {
        Body::from_stream(ReaderStream::with_capacity(
            file.take(length),
            STREAM_BUFFER_SIZE,
        ))
        .into_response()
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
    max_bytes: Option<u64>,
) -> Result<Json<FileOperationResponse>, AppError> {
    ensure_declared_length_within_limit(&headers, max_bytes)?;
    let resolved = resolve_existing(snapshot, path).await?;
    ensure_writable(&resolved.mapping)?;
    ensure_not_mount_root(&resolved)?;
    ensure_file(&resolved.real_path, &resolved.virtual_path)?;

    let temp_path = temp_path_for(&resolved.real_path)?;
    let result = write_body_to_file(body, &temp_path, max_bytes).await;
    if result.is_err() {
        let _ = fs::remove_file(&temp_path).await;
    }
    result?;

    if let Err(error) = replace_file(&temp_path, &resolved.real_path).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(error);
    }

    Ok(Json(FileOperationResponse {
        path: resolved.virtual_path,
    }))
}

pub async fn upload_multipart(
    snapshot: Arc<MappingSnapshot>,
    parent_path: String,
    mut multipart: Multipart,
    max_bytes: Option<u64>,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let parent = resolve_existing(snapshot, parent_path).await?;
    ensure_writable(&parent.mapping)?;
    ensure_folder(&parent.real_path, &parent.virtual_path)?;

    let mut files = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        let Some(file_name) = field.file_name().map(ToString::to_string) else {
            continue;
        };
        files.push(upload_field(&parent, file_name, field, max_bytes).await?);
    }

    Ok((StatusCode::CREATED, Json(UploadResponse { files })))
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
        .map_err(|_| AppError::range_not_satisfiable("Range 头无效"))?;
    parse_range(value, size).map(Some)
}

fn parse_range(value: &str, size: u64) -> Result<ByteRange, AppError> {
    if size == 0 {
        return Err(AppError::range_not_satisfiable("空文件不支持 Range 请求"));
    }
    let Some(spec) = value.strip_prefix("bytes=") else {
        return Err(AppError::range_not_satisfiable("仅支持 bytes Range"));
    };
    if spec.contains(',') {
        return Err(AppError::range_not_satisfiable("暂不支持多段 Range"));
    }

    let Some((start, end)) = spec.split_once('-') else {
        return Err(AppError::range_not_satisfiable("Range 格式无效"));
    };
    if start.is_empty() {
        let suffix: u64 = end
            .parse()
            .map_err(|_| AppError::range_not_satisfiable("Range 后缀无效"))?;
        if suffix == 0 {
            return Err(AppError::range_not_satisfiable("Range 后缀不能为 0"));
        }
        let length = suffix.min(size);
        return Ok(ByteRange {
            start: size - length,
            end: size - 1,
        });
    }

    let start: u64 = start
        .parse()
        .map_err(|_| AppError::range_not_satisfiable("Range 起点无效"))?;
    if start >= size {
        return Err(AppError::range_not_satisfiable("Range 起点越界"));
    }
    let end = if end.is_empty() {
        size - 1
    } else {
        end.parse()
            .map_err(|_| AppError::range_not_satisfiable("Range 终点无效"))?
    };
    if end < start {
        return Err(AppError::range_not_satisfiable("Range 终点小于起点"));
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
) -> Result<FileOperationResponse, AppError> {
    let name = normalize_child_name(&file_name)?;
    let target = parent.real_path.join(&name);
    if target.exists() {
        return Err(AppError::conflict(format!(
            "路径已存在: {}",
            join_virtual_path(&parent.virtual_path, &name)
        )));
    }

    let temp_path = temp_path_for(&target)?;
    let result = write_field_to_file(&mut field, &temp_path, max_bytes).await;
    if result.is_err() {
        let _ = fs::remove_file(&temp_path).await;
    }
    result?;
    if let Err(error) = fs::rename(&temp_path, &target).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(error.into());
    }

    Ok(FileOperationResponse {
        path: join_virtual_path(&parent.virtual_path, &name),
    })
}

async fn write_body_to_file(
    body: Body,
    temp_path: &Path,
    max_bytes: Option<u64>,
) -> Result<(), AppError> {
    let mut stream = body.into_data_stream();
    let mut file = BufWriter::with_capacity(WRITE_BUFFER_SIZE, create_temp_file(temp_path).await?);
    let mut written = 0_u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| AppError::bad_request(error.to_string()))?;
        written = checked_add_len(written, chunk.len(), max_bytes)?;
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
        .map_err(|_| AppError::bad_request("Content-Length 无效"))?
        .parse::<u64>()
        .map_err(|_| AppError::bad_request("Content-Length 无效"))?;
    if length > max_bytes {
        return Err(AppError::payload_too_large(format!(
            "上传内容超过限制: {max_bytes} bytes"
        )));
    }
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
    match fs::rename(temp_path, target).await {
        Ok(()) => Ok(()),
        Err(error) if target.exists() => {
            fs::remove_file(target).await?;
            fs::rename(temp_path, target).await.map_err(|rename_error| {
                AppError::internal(format!("替换文件失败: {error}; {rename_error}"))
            })
        }
        Err(error) => Err(error.into()),
    }
}

fn temp_path_for(target: &Path) -> Result<PathBuf, AppError> {
    let parent = target
        .parent()
        .ok_or_else(|| AppError::bad_request("目标路径没有父目录"))?;
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
    let next = current
        .checked_add(chunk_len as u64)
        .ok_or_else(|| AppError::payload_too_large("上传内容过大"))?;
    if let Some(max_bytes) = max_bytes
        && next > max_bytes
    {
        return Err(AppError::payload_too_large(format!(
            "上传内容超过限制: {max_bytes} bytes"
        )));
    }
    Ok(next)
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
    use axum::http::{HeaderMap, HeaderValue, header::CONTENT_LENGTH};

    use crate::error::AppError;

    use super::{ByteRange, ensure_declared_length_within_limit, parse_range};

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
}
