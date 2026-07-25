use axum::extract::{Multipart, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use super::response::{self, ApiResponse, BatchFailure, BatchResult, PaginatedResponse};
use crate::vfs::FileSystem;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub fs: Arc<dyn FileSystem>,
}

/// Query parameters for path
#[derive(Debug, Deserialize)]
pub struct PathQuery {
    pub path: PathBuf,
}

/// Query parameters for directory listing with pagination
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub path: PathBuf,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default)]
    pub sort_by: Option<SortField>,
    #[serde(default)]
    pub sort_order: Option<SortOrder>,
}

fn default_page() -> usize {
    0
}

fn default_page_size() -> usize {
    100
}

/// Sort field options
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Name,
    Size,
    Modified,
    Created,
}

/// Sort order
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Query parameters for search
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub path: PathBuf,
    pub pattern: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

/// Query parameters for file read with range
#[derive(Debug, Deserialize)]
pub struct ReadQuery {
    pub path: PathBuf,
    pub offset: Option<u64>,
    pub length: Option<u64>,
}

/// Batch delete request
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub paths: Vec<PathBuf>,
}

/// Batch copy request
#[derive(Debug, Deserialize)]
pub struct BatchCopyRequest {
    pub operations: Vec<CopyOperation>,
}

/// Copy operation
#[derive(Debug, Deserialize)]
pub struct CopyOperation {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// File metadata response
#[derive(Debug, Serialize)]
pub struct MetadataResponse {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: std::time::SystemTime,
    pub created: Option<std::time::SystemTime>,
    pub accessed: Option<std::time::SystemTime>,
    pub permissions: Option<crate::vfs::FilePermissions>,
}

/// Directory statistics response
#[derive(Debug, Serialize)]
pub struct DirStatsResponse {
    pub path: PathBuf,
    pub stats: crate::vfs::DirStats,
}

/// Create directory request
#[derive(Debug, Deserialize)]
pub struct CreateDirRequest {
    pub path: PathBuf,
}

/// Rename request
#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Copy request
#[derive(Debug, Deserialize)]
pub struct CopyRequest {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Create API router
pub fn router(fs: Arc<dyn FileSystem>) -> Router {
    let state = AppState { fs };

    Router::new()
        .route("/", get(list_dir).post(create_dir))
        .route("/metadata", get(get_metadata))
        .route("/read", get(read_file))
        .route("/write", post(write_file))
        .route("/upload", post(upload_file))
        .route("/download", get(download_file))
        .route("/search", get(search_files))
        .route("/stats", get(get_dir_stats))
        .route("/delete", delete(delete_path))
        .route("/force-delete", delete(force_delete_path))
        .route("/rename", post(rename_path))
        .route("/copy", post(copy_path))
        .route("/batch/delete", post(batch_delete))
        .route("/batch/copy", post(batch_copy))
        .with_state(state)
}

/// List directory contents with pagination
async fn list_dir(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> crate::error::Result<Json<ApiResponse<PaginatedResponse<crate::vfs::FileEntry>>>> {
    let mut entries = state.fs.list_dir(&query.path).await?;
    let total = entries.len();

    // Sort entries if requested
    if let Some(sort_by) = &query.sort_by {
        let order = match query.sort_order {
            Some(SortOrder::Desc) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Less,
        };

        entries.sort_by(|a, b| {
            let cmp = match sort_by {
                SortField::Name => a.name.cmp(&b.name),
                SortField::Size => a.size.cmp(&b.size),
                SortField::Modified => a.modified.cmp(&b.modified),
                SortField::Created => {
                    a.created
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                        .cmp(&b.created.unwrap_or(std::time::SystemTime::UNIX_EPOCH))
                }
            };

            if order == std::cmp::Ordering::Greater {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    // Apply pagination
    let start = query.page * query.page_size;
    let end = std::cmp::min(start + query.page_size, entries.len());
    let paginated_entries = if start < entries.len() {
        entries[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        paginated_entries,
        total,
        query.page,
        query.page_size,
    ))))
}

/// Get file/directory metadata
async fn get_metadata(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
) -> crate::error::Result<Json<ApiResponse<MetadataResponse>>> {
    let metadata = state.fs.metadata(&query.path).await?;

    Ok(Json(ApiResponse::success(MetadataResponse {
        path: metadata.path,
        name: metadata.name,
        is_dir: metadata.is_dir,
        size: metadata.size,
        modified: metadata.modified,
        created: metadata.created,
        accessed: metadata.accessed,
        permissions: metadata.permissions,
    })))
}

/// Read file content
async fn read_file(
    State(state): State<AppState>,
    Query(query): Query<ReadQuery>,
) -> crate::error::Result<Vec<u8>> {
    let content = state
        .fs
        .read_file_stream(&query.path, query.offset, query.length)
        .await?;
    Ok(content)
}

/// Write file content
async fn write_file(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
    body: String,
) -> crate::error::Result<Json<ApiResponse<MetadataResponse>>> {
    state.fs.write_file(&query.path, body.as_bytes()).await?;

    // Return metadata of the written file
    let metadata = state.fs.metadata(&query.path).await?;

    Ok(Json(ApiResponse::success(MetadataResponse {
        path: metadata.path,
        name: metadata.name,
        is_dir: metadata.is_dir,
        size: metadata.size,
        modified: metadata.modified,
        created: metadata.created,
        accessed: metadata.accessed,
        permissions: metadata.permissions,
    })))
}

/// Upload file via multipart form
async fn upload_file(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
    mut multipart: Multipart,
) -> crate::error::Result<Json<ApiResponse<Vec<MetadataResponse>>>> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("uploaded_file").to_string();

        // Construct the full path
        let file_path = query.path.join(&file_name);

        // Read all bytes from the field
        let data = field.bytes().await.map_err(|e| {
            crate::error::AppError::BadRequest(format!("Failed to read file data: {}", e))
        })?;

        // Write the file
        state.fs.write_file(&file_path, &data).await?;

        // Get metadata of uploaded file
        let metadata = state.fs.metadata(&file_path).await?;

        uploaded_files.push(MetadataResponse {
            path: metadata.path,
            name: metadata.name,
            is_dir: metadata.is_dir,
            size: metadata.size,
            modified: metadata.modified,
            created: metadata.created,
            accessed: metadata.accessed,
            permissions: metadata.permissions,
        });

        tracing::info!("Uploaded file: {} ({} bytes)", name, data.len());
    }

    Ok(Json(ApiResponse::success(uploaded_files)))
}

/// Download file with proper headers
async fn download_file(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
) -> crate::error::Result<Response> {
    let metadata = state.fs.metadata(&query.path).await?;

    if metadata.is_dir {
        return Err(crate::error::AppError::NotFile(query.path));
    }

    let content = state.fs.read_file(&query.path).await?;
    let mime_type = state.fs.mime_type(&query.path);

    let headers = HeaderMap::from_iter([
        (
            axum::http::header::CONTENT_TYPE,
            mime_type.parse().unwrap(),
        ),
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", metadata.name)
                .parse()
                .unwrap(),
        ),
        (
            axum::http::header::CONTENT_LENGTH,
            content.len().to_string().parse().unwrap(),
        ),
    ]);

    Ok((headers, content).into_response())
}

/// Search files with pagination
async fn search_files(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> crate::error::Result<Json<ApiResponse<PaginatedResponse<crate::vfs::SearchResult>>>> {
    let results = state
        .fs
        .search(&query.path, &query.pattern, query.recursive)
        .await?;

    let total = results.len();

    // Apply pagination
    let start = query.page * query.page_size;
    let end = std::cmp::min(start + query.page_size, results.len());
    let paginated_results = if start < results.len() {
        results[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(Json(ApiResponse::success(PaginatedResponse::new(
        paginated_results,
        total,
        query.page,
        query.page_size,
    ))))
}

/// Get directory statistics
async fn get_dir_stats(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
) -> crate::error::Result<Json<ApiResponse<DirStatsResponse>>> {
    let stats = state.fs.dir_stats(&query.path).await?;

    Ok(Json(ApiResponse::success(DirStatsResponse {
        path: query.path,
        stats,
    })))
}

/// Create directory
async fn create_dir(
    State(state): State<AppState>,
    Json(request): Json<CreateDirRequest>,
) -> crate::error::Result<(StatusCode, Json<ApiResponse<MetadataResponse>>)> {
    state.fs.create_dir_all(&request.path).await?;

    // Get metadata of created directory
    let metadata = state.fs.metadata(&request.path).await?;

    Ok(response::created(MetadataResponse {
        path: metadata.path,
        name: metadata.name,
        is_dir: metadata.is_dir,
        size: metadata.size,
        modified: metadata.modified,
        created: metadata.created,
        accessed: metadata.accessed,
        permissions: metadata.permissions,
    }))
}

/// Delete file or directory
async fn delete_path(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
) -> crate::error::Result<Json<ApiResponse<serde_json::Value>>> {
    // Check if path exists and get info before deletion
    let metadata = state.fs.metadata(&query.path).await?;
    let is_dir = metadata.is_dir;
    let name = metadata.name.clone();

    // If it's a directory, check if it has content
    if is_dir {
        let entries = state.fs.list_dir(&query.path).await?;
        if !entries.is_empty() {
            // Return error indicating directory is not empty
            return Err(crate::error::AppError::BadRequest(format!(
                "Directory '{}' is not empty ({} items). Use force=true to delete.",
                name,
                entries.len()
            )));
        }
    }

    state.fs.delete(&query.path).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "path": query.path,
        "name": name,
        "is_dir": is_dir
    }))))
}

/// Force delete file or directory (even if not empty)
async fn force_delete_path(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
) -> crate::error::Result<Json<ApiResponse<serde_json::Value>>> {
    let metadata = state.fs.metadata(&query.path).await?;
    let name = metadata.name.clone();
    let is_dir = metadata.is_dir;

    state.fs.delete(&query.path).await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "path": query.path,
        "name": name,
        "is_dir": is_dir
    }))))
}

/// Rename/move file or directory
async fn rename_path(
    State(state): State<AppState>,
    Json(request): Json<RenameRequest>,
) -> crate::error::Result<Json<ApiResponse<MetadataResponse>>> {
    state.fs.rename(&request.from, &request.to).await?;

    // Get metadata of renamed file
    let metadata = state.fs.metadata(&request.to).await?;

    Ok(Json(ApiResponse::success(MetadataResponse {
        path: metadata.path,
        name: metadata.name,
        is_dir: metadata.is_dir,
        size: metadata.size,
        modified: metadata.modified,
        created: metadata.created,
        accessed: metadata.accessed,
        permissions: metadata.permissions,
    })))
}

/// Copy file or directory
async fn copy_path(
    State(state): State<AppState>,
    Json(request): Json<CopyRequest>,
) -> crate::error::Result<(StatusCode, Json<ApiResponse<MetadataResponse>>)> {
    state.fs.copy(&request.from, &request.to).await?;

    // Get metadata of copied file
    let metadata = state.fs.metadata(&request.to).await?;

    Ok(response::created(MetadataResponse {
        path: metadata.path,
        name: metadata.name,
        is_dir: metadata.is_dir,
        size: metadata.size,
        modified: metadata.modified,
        created: metadata.created,
        accessed: metadata.accessed,
        permissions: metadata.permissions,
    }))
}

/// Batch delete files/directories
async fn batch_delete(
    State(state): State<AppState>,
    Json(request): Json<BatchDeleteRequest>,
) -> crate::error::Result<Json<ApiResponse<BatchResult>>> {
    let mut failures = Vec::new();
    let mut success_count = 0;

    for path in &request.paths {
        match state.fs.delete(path).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                failures.push(BatchFailure {
                    path: path.to_string_lossy().to_string(),
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(Json(ApiResponse::success(BatchResult::new(
        success_count,
        failures,
    ))))
}

/// Batch copy files/directories
async fn batch_copy(
    State(state): State<AppState>,
    Json(request): Json<BatchCopyRequest>,
) -> crate::error::Result<Json<ApiResponse<BatchResult>>> {
    let mut failures = Vec::new();
    let mut success_count = 0;

    for op in &request.operations {
        match state.fs.copy(&op.from, &op.to).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                failures.push(BatchFailure {
                    path: format!("{} -> {}", op.from.display(), op.to.display()),
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(Json(ApiResponse::success(BatchResult::new(
        success_count,
        failures,
    ))))
}
