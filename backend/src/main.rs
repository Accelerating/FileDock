mod api;
mod assets;
mod config;
mod error;
mod protocol;
mod vfs;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use axum::response::IntoResponse;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;
use vfs::{FileSystem, LocalFileSystem};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Parse configuration
    let config = Config::parse();

    // Get or create data directory
    let data_dir = config.get_data_dir()?;
    tracing::info!("Data directory: {}", data_dir.display());

    // Create file system
    let fs: Arc<dyn FileSystem> = Arc::new(LocalFileSystem::new(data_dir.clone()));

    // Build Web UI application (port 18888)
    // Set max body size to 10GB for large file uploads
    let web_app = Router::new()
        .nest("/api", api::router(fs.clone()))
        .fallback(|req: axum::http::Request<axum::body::Body>| async move {
            let path = req.uri().path();
            assets::serve_static(path).await
        })
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024)) // 10GB
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Build WebDAV application (port 17777)
    // Set max body size to 10GB for large file uploads
    let webdav_app = Router::new()
        .merge(protocol::webdav::router(fs.clone()))
        .fallback(|req: axum::http::Request<axum::body::Body>| async move {
            let path = req.uri().path();
            // Redirect /protocol/webdav/ to /protocol/webdav
            if path == "/protocol/webdav/" {
                return axum::response::Redirect::permanent("/protocol/webdav").into_response();
            }
            // Check if this is a WebDAV request
            if path.starts_with("/protocol/webdav") {
                // This should have been handled by the WebDAV router
                // Return 404 if it wasn't
                return axum::http::StatusCode::NOT_FOUND.into_response();
            }
            axum::http::StatusCode::NOT_FOUND.into_response()
        })
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024)) // 10GB
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start servers
    let web_addr = config.web_bind_address();
    let webdav_addr = config.webdav_bind_address();

    tracing::info!("Starting FileDock servers...");
    tracing::info!("Web UI: http://{}", web_addr);
    tracing::info!("WebDAV: http://{}", webdav_addr);

    // Spawn Web UI server
    let web_listener = tokio::net::TcpListener::bind(&web_addr).await?;
    let web_handle = tokio::spawn(async move {
        tracing::info!("Web UI server listening on {}", web_addr);
        if let Err(e) = axum::serve(web_listener, web_app).await {
            tracing::error!("Web UI server error: {}", e);
        }
    });

    // Spawn WebDAV server
    let webdav_listener = tokio::net::TcpListener::bind(&webdav_addr).await?;
    let webdav_handle = tokio::spawn(async move {
        tracing::info!("WebDAV server listening on {}", webdav_addr);
        if let Err(e) = axum::serve(webdav_listener, webdav_app).await {
            tracing::error!("WebDAV server error: {}", e);
        }
    });

    // Wait for both servers
    tokio::select! {
        _ = web_handle => {
            tracing::error!("Web UI server stopped");
        }
        _ = webdav_handle => {
            tracing::error!("WebDAV server stopped");
        }
    }

    Ok(())
}
