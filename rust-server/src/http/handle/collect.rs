use super::{reject::Reject, term};
use axum::body::{Body, Bytes};
use futures_util::StreamExt;
use tokio::time::{Duration, Instant};

pub(super) async fn buffered(
    body: Body,
    max: usize,
    inactivity: Duration,
    total: Duration,
    t: &term::Term,
) -> Result<Bytes, Reject> {
    let mut stream = body.into_data_stream();
    let end = Instant::now() + total;
    let mut all = Vec::new();
    loop {
        let idle = tokio::time::sleep(inactivity);
        let next = tokio::select! {
            biased;
            _ = tokio::time::sleep_until(end) => return Err(Reject::BadRequest),
            _ = idle => return Err(Reject::BadRequest),
            n = stream.next() => n,
        };
        let Some(chunk) = next else {
            return Ok(Bytes::from(all));
        };
        let b = chunk.map_err(|_| Reject::BadRequest)?;
        if all.len() + b.len() > max {
            return Err(Reject::TooLarge);
        }
        all.extend_from_slice(&b);
    }
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
