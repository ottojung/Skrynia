mod admission_release;
mod backpressure;
mod cancel;
mod deadline_malformed;
mod fixture;
mod flow;
mod slow_chunk;
mod slow_chunk_guard;
mod stall;
use super::kernel_with_wasm;
use axum::{
    body::Body,
    http::{HeaderValue, Request, header::TRANSFER_ENCODING},
};

pub(super) fn chunked(body: Body) -> Request<Body> {
    let mut r = Request::post("/").body(body).unwrap();
    r.headers_mut()
        .insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    r
}

// Manual clock stepping is required: the blocking VM worker keeps the paused
// runtime runnable, so Tokio's idle auto-advance does not fire our timers.
pub(super) async fn tick(d: tokio::time::Duration, n: u32) {
    for _ in 0..n {
        tokio::time::advance(d).await;
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }
    }
}
