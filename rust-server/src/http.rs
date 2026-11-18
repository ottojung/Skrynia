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

async fn handle(State(_): State<App>, method: Method, uri: Uri, body: Bytes) -> impl IntoResponse {
    if method != Method::POST || uri.path() != "/" {
        return (StatusCode::METHOD_NOT_ALLOWED, "only POST /").into_response();
    }
    let _ = body;
    (StatusCode::NOT_IMPLEMENTED, "root invocation abi pending").into_response()
}
