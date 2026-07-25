use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;
use std::path::Path;

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

/// Serve static files from embedded assets
pub async fn serve_static(path: &str) -> Response {
    let path = path.trim_start_matches('/');
    tracing::debug!("Serving static file: {}", path);

    // Try to serve the exact file
    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        tracing::debug!("Found asset: {} with mime: {}", path, mime);

        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime)],
            content.data.to_vec(),
        )
            .into_response();
    }

    // For SPA, serve index.html for any non-file path
    if !Path::new(path).extension().is_some() {
        if let Some(content) = Assets::get("index.html") {
            tracing::debug!("Serving index.html for SPA route: {}", path);
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html".to_string())],
                content.data.to_vec(),
            )
                .into_response();
        }
    }

    tracing::debug!("Asset not found: {}", path);
    StatusCode::NOT_FOUND.into_response()
}
