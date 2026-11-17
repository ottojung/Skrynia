use super::{guard::Upload, next::next};
use crate::{Fail, Result, State};
use axum::body::Body;
use axum::http::StatusCode;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
pub async fn read(state: &Arc<State>, body: Body) -> Result<Upload> {
    let (staged, file) =
        crate::Staged::start(&state.dir).map_err(|error| crate::error::transient_io(&error))?;
    let mut file = tokio::fs::File::from_std(file);
    let mut digest = Sha256::new();
    let mut size = 0u64;
    let mut chunks = body.into_data_stream();
    while let Some(chunk) = next(&mut chunks).await? {
        size = size
            .checked_add(chunk.len() as u64)
            .ok_or(Fail(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"))?;
        if size > state.max_object {
            return Err(Fail(StatusCode::PAYLOAD_TOO_LARGE, "request_too_large"));
        }
        let amount = u32::try_from(chunk.len()).map_err(|_| memory_full())?;
        let permit = Arc::clone(&state.upload)
            .try_acquire_many_owned(amount)
            .map_err(|_| memory_full())?;
        file.write_all(&chunk)
            .await
            .map_err(|e| crate::error::transient_io(&e))?;
        digest.update(&chunk);
        drop(permit);
    }
    file.sync_all()
        .await
        .map_err(|e| crate::error::transient_io(&e))?;
    drop(file);
    Ok(Upload {
        staged: Some(staged),
        size,
        digest: Some(hex::encode(digest.finalize())),
    })
}
fn memory_full() -> Fail {
    Fail(StatusCode::SERVICE_UNAVAILABLE, "upload_memory_full")
}
