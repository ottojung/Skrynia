use super::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::Request,
};
use futures_util::stream;
use std::io;
use tokio::sync::mpsc;
pub fn app() -> axum::Router {
    let wasm = wat::parse_str(include_str!(
        "../../../test-fixtures/stream_early_output.wat"
    ))
    .unwrap();
    libskrynia::http::router(kernel_with_wasm("skrynia-http-active-bound", &wasm))
}
pub fn input() -> (mpsc::Sender<Bytes>, Request<Body>) {
    let (tx, rx) = mpsc::channel::<Bytes>(1);
    let stream = stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|b| (Ok::<_, io::Error>(b), rx))
    });
    let request = Request::post("/").body(Body::from_stream(stream)).unwrap();
    (tx, request)
}
