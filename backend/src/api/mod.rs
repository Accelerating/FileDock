pub mod files;
pub mod response;

use axum::Router;
use axum::Json;
use std::sync::Arc;

use crate::vfs::FileSystem;

/// Create API router
pub fn router(fs: Arc<dyn FileSystem>) -> Router {
    Router::new()
        .nest("/files", files::router(fs.clone()))
        .merge(health_routes())
}

/// Health check routes
fn health_routes() -> Router {
    Router::new().route("/health", axum::routing::get(health_check))
}

/// Health check endpoint
async fn health_check() -> Json<response::ApiResponse<response::HealthResponse>> {
    Json(response::ApiResponse::success(response::HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // TODO: Track actual uptime
        data_dir: std::env::var("FILEDOCK_DATA_DIR").unwrap_or_default(),
    }))
}
