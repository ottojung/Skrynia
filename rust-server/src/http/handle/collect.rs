use super::{gate, pump, term};
use axum::body::{Body, Bytes};
use axum::http::StatusCode;
use std::io;
use std::sync::atomic::Ordering;
use tokio::time::Instant;

pub(super) async fn buffered(
    body: Body,
    max: usize,
    deadline: Instant,
    t: &term::Term,
    verdict: &gate::Verdict,
) -> Result<Bytes, StatusCode> {
    match tokio::time::timeout_at(deadline, axum::body::to_bytes(body, max)).await {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(e)) => {
            let err = io::Error::other(e);
            term::set(t, &err);
            Err(if verdict.load(Ordering::Relaxed) == 1 {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::BAD_REQUEST
            })
        }
        Err(_) => {
            let e = pump::stalled();
            term::set(t, &e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}
