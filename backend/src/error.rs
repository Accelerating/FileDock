use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path not found: {0}")]
    NotFound(PathBuf),

    #[error("Path already exists: {0}")]
    AlreadyExists(PathBuf),

    #[error("Path is not a directory: {0}")]
    NotDirectory(PathBuf),

    #[error("Path is not a file: {0}")]
    NotFile(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Path traversal attempt detected")]
    PathTraversal,

    #[error("Invalid request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::NotFound(p) => {
                (StatusCode::NOT_FOUND, format!("Not found: {}", p.display()))
            }
            AppError::AlreadyExists(p) => (
                StatusCode::CONFLICT,
                format!("Already exists: {}", p.display()),
            ),
            AppError::NotDirectory(p) => (
                StatusCode::BAD_REQUEST,
                format!("Not a directory: {}", p.display()),
            ),
            AppError::NotFile(p) => {
                (StatusCode::BAD_REQUEST, format!("Not a file: {}", p.display()))
            }
            AppError::InvalidPath(p) => (StatusCode::BAD_REQUEST, format!("Invalid path: {}", p)),
            AppError::PathTraversal => (
                StatusCode::FORBIDDEN,
                "Path traversal attempt detected".to_string(),
            ),
            AppError::BadRequest(e) => (StatusCode::BAD_REQUEST, e),
        };

        (status, message).into_response()
    }
}
