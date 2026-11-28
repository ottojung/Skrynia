use super::chunked;
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use libskrynia::http;
use std::io;
use tokio::time::Duration;
use tower::ServiceExt;
const ECHO: &str = include_str!("../../../test-fixtures/d2_echo.wat");
const EARLY: &str = include_str!("../../../test-fixtures/stream_early_output.wat");
// 413 is deterministic only when oversize is observed before response commit.
// Echo waits for input; early output may commit 200 before oversize is seen.
#[rustfmt::skip]
#[tokio::test]
async fn chunked_oversize_releases_admission_permit() {
    let app = http::router(super::kernel_with_wasm("skrynia-http-oversize-permit", &wat::parse_str(ECHO).unwrap()));
    for _ in 0..16 {
        let parts: Vec<Result<Bytes, io::Error>> = (0..17).map(|_| Ok(Bytes::from(vec![b'Z'; 1 << 20]))).collect();
        let res = app.clone().oneshot(chunked(Body::from_stream(stream::iter(parts)))).await.unwrap();
        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
    let ok = app.oneshot(Request::post("/").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}
#[rustfmt::skip]
#[tokio::test(start_paused = true)]
async fn chunked_timeouts_release_every_admission_permit() {
    let app = http::router(super::kernel_with_wasm("skrynia-http-timeout-permit", &wat::parse_str(ECHO).unwrap()));
    for _ in 0..16 {
        let parts = stream::pending::<Result<Bytes, io::Error>>();
        let task = tokio::spawn(app.clone().oneshot(chunked(Body::from_stream(parts))));
        super::tick(Duration::from_secs(10), 4).await;
        assert_eq!(task.await.unwrap().unwrap().status(), StatusCode::BAD_REQUEST);
    }
    let ok = app.oneshot(Request::post("/").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}
#[rustfmt::skip]
#[tokio::test] // Early output before input remains valid; committed 200 cannot become 413.
async fn early_output_contract_preserved() {
    let app = http::router(super::kernel_with_wasm("skrynia-http-early-contract", &wat::parse_str(EARLY).unwrap()));
    let res = app.oneshot(Request::post("/").body(Body::from(Bytes::from_static(b"x"))).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
