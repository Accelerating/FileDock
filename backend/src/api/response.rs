use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    /// Whether the request was successful
    pub success: bool,
    /// Response data (only present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error message (only present on error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Request ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            request_id: None,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    /// The data items
    pub items: Vec<T>,
    /// Total number of items
    pub total: usize,
    /// Current page number (0-indexed)
    pub page: usize,
    /// Number of items per page
    pub page_size: usize,
    /// Whether there are more pages
    pub has_more: bool,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(items: Vec<T>, total: usize, page: usize, page_size: usize) -> Self {
        let has_more = (page + 1) * page_size < total;
        Self {
            items,
            total,
            page,
            page_size,
            has_more,
        }
    }
}

/// Batch operation result
#[derive(Debug, Serialize)]
pub struct BatchResult {
    /// Number of successful operations
    pub success_count: usize,
    /// Number of failed operations
    pub failure_count: usize,
    /// Details of failed operations
    pub failures: Vec<BatchFailure>,
}

/// Details of a failed batch operation
#[derive(Debug, Serialize)]
pub struct BatchFailure {
    /// The path that failed
    pub path: String,
    /// The error message
    pub error: String,
}

impl BatchResult {
    /// Create a new batch result
    pub fn new(success_count: usize, failures: Vec<BatchFailure>) -> Self {
        Self {
            success_count,
            failure_count: failures.len(),
            failures,
        }
    }
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Data directory path
    pub data_dir: String,
}

/// Helper function to create a created response
pub fn created<T: Serialize>(data: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (StatusCode::CREATED, Json(ApiResponse::success(data)))
}
