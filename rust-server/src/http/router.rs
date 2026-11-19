use axum::{Router, routing::any};
use std::sync::Arc;
use tokio::sync::Semaphore;
#[derive(Clone)]
pub struct App {
    pub kernel: Arc<crate::boot::Kernel>,
    pub permits: Arc<Semaphore>,
}
pub fn router(kernel: crate::boot::Kernel) -> Router {
    Router::new()
        .fallback(any(super::handle::handle))
        .with_state(App {
            kernel: Arc::new(kernel),
            permits: Arc::new(Semaphore::new(16)),
        })
}
pub(super) fn max_req() -> usize {
    std::env::var("SKRYNIA_MAX_REQUEST_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(16 * 1024 * 1024)
}
