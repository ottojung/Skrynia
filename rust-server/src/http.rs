use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{Method, StatusCode, Uri},
    response::IntoResponse,
    routing::any,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct App {
    pub kernel: Arc<super::boot::Kernel>,
}

pub fn router(kernel: super::boot::Kernel) -> Router {
    let app = App {
        kernel: Arc::new(kernel),
    };
    Router::new().fallback(any(handle)).with_state(app)
}

fn max_req() -> usize {
    std::env::var("SKRYNIA_MAX_REQUEST_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16 * 1024 * 1024)
}

async fn handle(
    State(app): State<App>,
    method: Method,
    uri: Uri,
    body: Bytes,
) -> impl IntoResponse {
    if method != Method::POST || uri.path() != "/" {
        return StatusCode::NOT_FOUND.into_response();
    }
    if body.len() > max_req() {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    match app.kernel.invoke(&body) {
        Ok(out) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
            out,
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
