mod collect;
mod guard;
use crate::{Fail, Result, State};
use axum::extract::Request;
use axum::http::StatusCode;
pub use guard::Upload;
use std::sync::Arc;
pub async fn read(state: &Arc<State>, req: Request) -> Result<Upload> {
    let length = req
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());
    if length.is_some_and(|size| size > state.max_object) {
        return Err(Fail(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"));
    }
    if length.is_some_and(|size| size > state.max_buffered) {
        return Err(Fail(StatusCode::SERVICE_UNAVAILABLE, "upload_memory_full"));
    }
    tokio::time::timeout(
        std::time::Duration::from_secs(60),
        collect::read(state, req.into_body()),
    )
    .await
    .map_err(|_| Fail(StatusCode::REQUEST_TIMEOUT, "request_interrupted"))?
}

#[cfg(test)]
mod tests;
