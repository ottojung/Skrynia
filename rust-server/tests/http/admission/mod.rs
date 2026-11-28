mod backpressure;
mod cancel;
mod fixture;
mod flow;
mod slow_chunk;
mod slow_chunk_guard;
mod stall;
use super::kernel_with_wasm;

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
