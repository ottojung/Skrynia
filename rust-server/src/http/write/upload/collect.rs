use super::guard::Upload;
use crate::{Fail, Result, State};
use axum::body::Body;
use axum::http::StatusCode;
use futures_util::StreamExt;
use std::sync::Arc;
pub async fn read(state: &Arc<State>, body: Body) -> Result<Upload> {
    let mut upload = Upload::empty(Arc::clone(&state.upload));
    let mut chunks = body.into_data_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|_| Fail(StatusCode::BAD_REQUEST, "request_interrupted"))?;
        if chunk.len() > state.max_object.saturating_sub(upload.bytes.len()) {
            return Err(Fail(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"));
        }
        upload.append(&chunk)?;
    }
    Ok(upload)
}
