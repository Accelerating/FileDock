pub mod handler;
pub mod types;

use axum::Router;
use std::sync::Arc;

use crate::vfs::FileSystem;

/// Create WebDAV router
pub fn router(fs: Arc<dyn FileSystem>) -> Router {
    handler::create_router(fs)
}
