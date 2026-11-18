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

const MAX_REQ: usize = 16 * 1024 * 1024;

async fn handle(
    State(app): State<App>,
    method: Method,
    uri: Uri,
    body: Bytes,
) -> impl IntoResponse {
    if method != Method::POST || uri.path() != "/" {
        return (StatusCode::METHOD_NOT_ALLOWED, "only POST /").into_response();
    }
    if body.len() > MAX_REQ {
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
