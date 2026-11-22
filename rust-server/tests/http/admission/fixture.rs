use super::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};
use tower::ServiceExt;
static NEXT: AtomicUsize = AtomicUsize::new(0);
pub fn app() -> axum::Router {
    let name = format!(
        "skrynia-http-active-bound-{}",
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let wasm = wat::parse_str(include_str!(
        "../../../test-fixtures/stream_early_output.wat"
    ))
    .unwrap();
    skrynia_server::http::router(kernel_with_wasm(&name, &wasm))
}
pub fn input() -> (mpsc::Sender<Bytes>, Request<Body>) {
    let (tx, rx) = mpsc::channel::<Bytes>(1);
    let stream = stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|b| (Ok::<_, io::Error>(b), rx))
    });
    let request = Request::post("/").body(Body::from_stream(stream)).unwrap();
    (tx, request)
}
pub async fn status(app: &axum::Router, req: Request<Body>) -> StatusCode {
    timeout(Duration::from_secs(2), app.clone().oneshot(req))
        .await
        .unwrap()
        .unwrap()
        .status()
}
