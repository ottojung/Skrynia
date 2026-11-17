mod collect;
mod guard;
mod next;
use crate::{Fail, Result, State};
use axum::extract::Request;
use axum::http::StatusCode;
pub use guard::Upload;
use std::sync::Arc;
pub async fn read(state: &Arc<State>, req: Request) -> Result<Upload> {
    if let Some(length) = req.headers().get("content-length") {
        let size = length
            .to_str()
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or(Fail(StatusCode::BAD_REQUEST, "invalid_length"))?;
        if size > state.max_object {
            return Err(Fail(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"));
        }
    }
    collect::read(state, req.into_body()).await
}
