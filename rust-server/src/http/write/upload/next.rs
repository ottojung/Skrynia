use crate::{Fail, Result};
use axum::body::Bytes;
use axum::http::StatusCode;
use futures_util::{Stream, StreamExt};
pub async fn next(
    stream: &mut (impl Stream<Item = std::result::Result<Bytes, axum::Error>> + Unpin),
) -> Result<Option<Bytes>> {
    tokio::time::timeout(std::time::Duration::from_secs(60), stream.next())
        .await
        .map_err(|_| Fail(StatusCode::REQUEST_TIMEOUT, "request_interrupted"))?
        .transpose()
        .map_err(|_| Fail(StatusCode::BAD_REQUEST, "request_interrupted"))
}
