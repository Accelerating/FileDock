use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::method_routing;
use axum::Router;
use quick_xml::se::to_string;
use std::sync::Arc;

use super::types::*;
use crate::error::AppError;
use crate::vfs::FileSystem;

/// WebDAV application state
#[derive(Clone)]
pub struct DavState {
    pub fs: Arc<dyn FileSystem>,
}

/// Create WebDAV router
pub fn create_router(fs: Arc<dyn FileSystem>) -> Router {
    let state = DavState { fs };

    Router::new()
        .route("/", method_routing::any(handle_dav_index))
        .route("/*path", method_routing::any(handle_dav_path))
        .with_state(state)
}

/// Handle WebDAV requests to index
async fn handle_dav_index(
    State(state): State<DavState>,
    method: Method,
    headers: HeaderMap,
    body: String,
) -> Response {
    handle_dav_request(&state, "/", method, headers, body).await
}

/// Handle WebDAV requests with path
async fn handle_dav_path(
    State(state): State<DavState>,
    method: Method,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let path = format!("/{}", path);
    handle_dav_request(&state, &path, method, headers, body).await
}

/// Handle all WebDAV methods
async fn handle_dav_request(
    state: &DavState,
    path: &str,
    method: Method,
    headers: HeaderMap,
    body: String,
) -> Response {
    let method_str = method.as_str();

    tracing::debug!("WebDAV {} {}", method_str, path);

    match method_str {
        "OPTIONS" => handle_options(),
        "GET" => handle_get(&state, &path).await,
        "HEAD" => handle_head(&state, &path).await,
        "PUT" => handle_put(&state, &path, body).await,
        "DELETE" => handle_delete(&state, &path).await,
        "MKCOL" => handle_mkcol(&state, &path).await,
        "COPY" => handle_copy(&state, &path, &headers).await,
        "MOVE" => handle_move(&state, &path, &headers).await,
        "PROPFIND" => handle_propfind(&state, &path, &headers, body).await,
        "PROPPATCH" => handle_proppatch(&state, &path).await,
        "LOCK" => handle_lock(&state, &path).await,
        "UNLOCK" => handle_unlock().await,
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    }
}

/// Handle OPTIONS request
fn handle_options() -> Response {
    let mut response = StatusCode::OK.into_response();

    let headers_mut = response.headers_mut();
    headers_mut.insert("Allow", "OPTIONS, GET, HEAD, PUT, DELETE, MKCOL, COPY, MOVE, PROPFIND, PROPPATCH, LOCK, UNLOCK".parse().unwrap());
    headers_mut.insert("DAV", "1, 2".parse().unwrap());
    headers_mut.insert("MS-Author-Via", "DAV".parse().unwrap());

    response
}

/// Handle GET request
async fn handle_get(state: &DavState, path: &str) -> Response {
    let path = std::path::Path::new(path);

    match state.fs.read_file(path).await {
        Ok(content) => {
            let mime_type = state.fs.mime_type(path);
            let mut response = content.into_response();
            response.headers_mut().insert(
                "Content-Type",
                mime_type.parse().unwrap(),
            );
            response
        }
        Err(AppError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(AppError::NotFile(_)) => {
            // For directories, return a simple HTML listing
            match state.fs.list_dir(path).await {
                Ok(entries) => {
                    let html = format!(
                        "<html><body><h1>Directory: {}</h1><ul>{}</ul></body></html>",
                        path.display(),
                        entries
                            .iter()
                            .map(|e| format!(
                                "<li><a href=\"{}\">{}</a></li>",
                                e.path.display(),
                                e.name
                            ))
                            .collect::<Vec<_>>()
                            .join("")
                    );
                    (
                        StatusCode::OK,
                        [("Content-Type", "text/html")],
                        html,
                    )
                        .into_response()
                }
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle HEAD request
async fn handle_head(state: &DavState, path: &str) -> Response {
    let path = std::path::Path::new(path);

    match state.fs.metadata(path).await {
        Ok(metadata) => {
            let mime_type = state.fs.mime_type(path);
            let mut response = StatusCode::OK.into_response();
            let headers = response.headers_mut();
            headers.insert("Content-Type", mime_type.parse().unwrap());
            if !metadata.is_dir {
                headers.insert(
                    "Content-Length",
                    metadata.size.to_string().parse().unwrap(),
                );
            }
            response
        }
        Err(AppError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle PUT request
async fn handle_put(state: &DavState, path: &str, body: String) -> Response {
    let path = std::path::Path::new(path);

    match state.fs.write_file(path, body.as_bytes()).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle DELETE request
async fn handle_delete(state: &DavState, path: &str) -> Response {
    let path = std::path::Path::new(path);

    match state.fs.delete(path).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(AppError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle MKCOL request (create directory)
async fn handle_mkcol(state: &DavState, path: &str) -> Response {
    let path = std::path::Path::new(path);

    match state.fs.create_dir_all(path).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(AppError::AlreadyExists(_)) => StatusCode::METHOD_NOT_ALLOWED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle COPY request
async fn handle_copy(state: &DavState, path: &str, headers: &HeaderMap) -> Response {
    let destination = match headers.get("Destination") {
        Some(dest) => dest.to_str().unwrap_or(""),
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    // Extract path from destination URL
    let dest_path = extract_path_from_url(destination);
    let src = std::path::Path::new(path);
    let dst = std::path::Path::new(&dest_path);

    match state.fs.copy(src, dst).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(AppError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle MOVE request
async fn handle_move(state: &DavState, path: &str, headers: &HeaderMap) -> Response {
    let destination = match headers.get("Destination") {
        Some(dest) => dest.to_str().unwrap_or(""),
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    // Extract path from destination URL
    let dest_path = extract_path_from_url(destination);
    let src = std::path::Path::new(path);
    let dst = std::path::Path::new(&dest_path);

    match state.fs.rename(src, dst).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(AppError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle PROPFIND request
async fn handle_propfind(
    state: &DavState,
    path: &str,
    headers: &HeaderMap,
    body: String,
) -> Response {
    let depth = headers
        .get("Depth")
        .and_then(|v| v.to_str().ok())
        .map(Depth::from_str)
        .unwrap_or(Depth::Infinity);

    let path = std::path::Path::new(path);

    // Parse the request body to determine which properties to return
    let _prop_find: PropFind = if body.is_empty() {
        PropFind {
            prop: None,
            allprop: true,
            propname: false,
        }
    } else {
        match quick_xml::de::from_str(&body) {
            Ok(pf) => pf,
            Err(_) => {
                return StatusCode::BAD_REQUEST.into_response();
            }
        }
    };

    let mut responses = Vec::new();

    // Add the resource itself
    match build_response(state, path).await {
        Ok(resp) => responses.push(resp),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    // Add children if depth allows
    if depth == Depth::One || depth == Depth::Infinity {
        match state.fs.list_dir(path).await {
            Ok(entries) => {
                for entry in entries {
                    let child_path = path.join(&entry.name);
                    match build_response_from_entry(state, &child_path, &entry).await {
                        Ok(resp) => responses.push(resp),
                        Err(_) => continue,
                    }
                }
            }
            Err(_) => {
                // If we can't list the directory, just return the resource itself
            }
        }
    }

    let multi_status = DavMultiStatus {
        responses,
    };

    match to_string(&multi_status) {
        Ok(xml) => (
            StatusCode::MULTI_STATUS,
            [("Content-Type", "application/xml; charset=utf-8")],
            xml,
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Build a response for a path
async fn build_response(state: &DavState, path: &std::path::Path) -> Result<DavResponse, AppError> {
    let metadata = state.fs.metadata(path).await?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let path_str = path.to_string_lossy().to_string();
    let mime_type = state.fs.mime_type(path);

    let modified: chrono::DateTime<chrono::Utc> = metadata.modified.into();
    let created = metadata.created.map(|t| {
        let st: std::time::SystemTime = t;
        st.into()
    });

    let prop = metadata_to_prop(
        &name,
        &path_str,
        metadata.is_dir,
        metadata.size,
        modified,
        created,
        &mime_type,
    );

    let prop_stat = DavPropStat {
        prop,
        status: "HTTP/1.1 200 OK".to_string(),
    };

    Ok(DavResponse {
        href: path_str,
        propstat: vec![prop_stat],
        status: None,
    })
}

/// Build a response from a file entry
async fn build_response_from_entry(
    state: &DavState,
    path: &std::path::Path,
    entry: &crate::vfs::FileEntry,
) -> Result<DavResponse, AppError> {
    let mime_type = state.fs.mime_type(path);
    let path_str = path.to_string_lossy().to_string();

    let modified: chrono::DateTime<chrono::Utc> = entry.modified.into();
    let created = entry.created.map(|t| {
        let st: std::time::SystemTime = t;
        st.into()
    });

    let prop = metadata_to_prop(
        &entry.name,
        &path_str,
        entry.is_dir,
        entry.size,
        modified,
        created,
        &mime_type,
    );

    let prop_stat = DavPropStat {
        prop,
        status: "HTTP/1.1 200 OK".to_string(),
    };

    Ok(DavResponse {
        href: path_str,
        propstat: vec![prop_stat],
        status: None,
    })
}

/// Handle PROPPATCH request
async fn handle_proppatch(state: &DavState, path: &str) -> Response {
    // For now, just return success without actually modifying properties
    // In a full implementation, we would store custom properties
    let path = std::path::Path::new(path);

    match state.fs.metadata(path).await {
        Ok(_) => {
            let path_str = path.to_string_lossy().to_string();
            let multi_status = DavMultiStatus {
                responses: vec![DavResponse {
                    href: path_str,
                    propstat: vec![DavPropStat {
                        prop: DavProp {
                            creation_date: None,
                            display_name: None,
                            get_content_length: None,
                            get_content_type: None,
                            get_etag: None,
                            get_last_modified: None,
                            resource_type: None,
                            supported_lock: None,
                            lock_discovery: None,
                        },
                        status: "HTTP/1.1 200 OK".to_string(),
                    }],
                    status: None,
                }],
            };

            match to_string(&multi_status) {
                Ok(xml) => (
                    StatusCode::MULTI_STATUS,
                    [("Content-Type", "application/xml; charset=utf-8")],
                    xml,
                )
                    .into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(AppError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle LOCK request
async fn handle_lock(state: &DavState, path: &str) -> Response {
    // For now, return a simple lock response
    // In a full implementation, we would manage locks
    let path = std::path::Path::new(path);

    // Check if path exists by trying to get metadata
    let exists = state.fs.metadata(path).await.is_ok();
    if !exists {
        // Create the resource if it doesn't exist
        if state.fs.write_file(path, b"").await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    let lock_token = format!("urn:uuid:{}", uuid::Uuid::new_v4());
    let path_str = path.to_string_lossy().to_string();

    let lock_discovery = DavLockDiscovery {
        active_locks: vec![DavActiveLock {
            lock_type: DavLockTypeElement { write: true },
            lock_scope: DavLockScopeElement {
                exclusive: true,
                shared: false,
            },
            depth: "infinity".to_string(),
            owner: None,
            timeout: Some("Second-3600".to_string()),
            lock_token: DavLockToken {
                href: lock_token.clone(),
            },
        }],
    };

    let prop = DavProp {
        creation_date: None,
        display_name: None,
        get_content_length: None,
        get_content_type: None,
        get_etag: None,
        get_last_modified: None,
        resource_type: None,
        supported_lock: None,
        lock_discovery: Some(lock_discovery),
    };

    let multi_status = DavMultiStatus {
        responses: vec![DavResponse {
            href: path_str,
            propstat: vec![DavPropStat {
                prop,
                status: "HTTP/1.1 200 OK".to_string(),
            }],
            status: None,
        }],
    };

    match to_string(&multi_status) {
        Ok(xml) => {
            let mut response = (
                StatusCode::OK,
                [("Content-Type", "application/xml; charset=utf-8")],
                xml,
            )
                .into_response();

            response.headers_mut().insert(
                "Lock-Token",
                format!("<{}>", lock_token).parse().unwrap(),
            );

            response
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Handle UNLOCK request
async fn handle_unlock() -> Response {
    // For now, just return success
    // In a full implementation, we would remove the lock
    StatusCode::NO_CONTENT.into_response()
}

/// Extract path from a URL
fn extract_path_from_url(url: &str) -> String {
    // Remove scheme and host if present
    let path = if url.contains("://") {
        url.splitn(3, '/').nth(2).unwrap_or(url)
    } else {
        url
    };

    // Remove query string
    let path = path.split('?').next().unwrap_or(path);

    // Ensure it starts with /
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    }
}
