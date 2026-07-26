use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use axum::Router;
use dav_server::fakels::FakeLs;
use dav_server::localfs::LocalFs;
use dav_server::DavHandler;
use std::path::PathBuf;

/// WebDAV application state
#[derive(Clone)]
pub struct DavState {
    pub dav_handler: DavHandler,
}

/// Create WebDAV router using dav-server library
pub fn router(data_dir: PathBuf) -> Router {
    let dav_server = DavHandler::builder()
        .filesystem(LocalFs::new(data_dir.to_string_lossy().to_string(), false, false, false))
        .locksystem(FakeLs::new())
        .build_handler();

    let state = DavState {
        dav_handler: dav_server,
    };

    Router::new()
        .fallback(handle_dav)
        .with_state(state)
}

/// Handle all WebDAV requests
async fn handle_dav(
    State(state): State<DavState>,
    request: Request<Body>,
) -> Response {
    let response = state.dav_handler.handle(request).await;
    response.into_response()
}
