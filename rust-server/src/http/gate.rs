use axum::body::{Body, Bytes};
use axum::http::Request;
use futures_util::{StreamExt, stream};
use std::io;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::time::Duration;

pub type Verdict = std::sync::Arc<AtomicU8>;

pub fn declared_too_large<T>(r: &Request<T>, max: usize) -> bool {
    r.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .is_some_and(|n| n > max)
}

pub fn chunked<T>(r: &Request<T>) -> bool {
    r.headers()
        .get("transfer-encoding")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.to_ascii_lowercase().contains("chunked"))
}

pub fn body(body: Body, max: usize, inactivity: Duration, verdict: Verdict) -> Body {
    let incoming = super::pump::inactive(body.into_data_stream(), inactivity);
    let held = stream::unfold(
        (incoming, Vec::new(), false),
        move |(mut s, mut buf, done)| {
            let verdict = verdict.clone();
            async move {
                if done {
                    return None;
                }
                loop {
                    match s.next().await {
                        Some(Ok(b)) if buf.len() + b.len() > max => {
                            verdict.store(1, Ordering::Relaxed);
                            let e = io::Error::from(io::ErrorKind::InvalidData);
                            return Some((Err(e), (s, buf, true)));
                        }
                        Some(Ok(b)) => buf.extend_from_slice(&b),
                        Some(Err(e)) => return Some((Err(io::Error::other(e)), (s, buf, true))),
                        None => {
                            let out = Bytes::from(buf);
                            return (!out.is_empty()).then(|| (Ok(out), (s, Vec::new(), true)));
                        }
                    }
                }
            }
        },
    );
    Body::from_stream(held)
}
