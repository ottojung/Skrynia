use super::{pump, term};
use axum::body::{Body, Bytes};
use axum::http::StatusCode;
use futures_util::StreamExt;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::time::{Duration, Instant};

fn bad(t: &term::Term, v: &Arc<AtomicU8>, e: io::Error) -> StatusCode {
    term::set(t, &e);
    if v.load(Ordering::Relaxed) == 1 {
        StatusCode::PAYLOAD_TOO_LARGE
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub(super) async fn buffered(
    body: Body,
    max: usize,
    inactivity: Duration,
    total: Duration,
    t: &term::Term,
    verdict: &Arc<AtomicU8>,
) -> Result<Bytes, StatusCode> {
    let mut stream = body.into_data_stream();
    let end = Instant::now() + total;
    let mut all = Vec::new();
    loop {
        let idle = tokio::time::sleep(inactivity);
        let next = tokio::select! {
            biased;
            _ = tokio::time::sleep_until(end) => return Err(bad(t, verdict, pump::stalled())),
            _ = idle => return Err(bad(t, verdict, pump::stalled())),
            n = stream.next() => n,
        };
        let Some(chunk) = next else {
            return Ok(Bytes::from(all));
        };
        let b = chunk.map_err(|e| bad(t, verdict, io::Error::other(e)))?;
        if all.len() + b.len() > max {
            verdict.store(1, Ordering::Relaxed);
            return Err(bad(t, verdict, io::Error::from(io::ErrorKind::InvalidData)));
        }
        all.extend_from_slice(&b);
    }
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
