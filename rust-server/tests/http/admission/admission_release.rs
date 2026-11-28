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

#[tokio::test(start_paused = true)]
async fn chunked_timeouts_release_every_admission_permit() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(super::kernel_with_wasm("skrynia-http-permit", &wasm));
    for _ in 0..16 {
        let parts = stream::pending::<Result<Bytes, io::Error>>();
        let task = tokio::spawn(app.clone().oneshot(chunked(Body::from_stream(parts))));
        super::tick(Duration::from_secs(10), 4).await;
        assert_eq!(
            task.await.unwrap().unwrap().status(),
            StatusCode::BAD_REQUEST
        );
    }
    let ok = app
        .oneshot(Request::post("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}

#[tokio::test]
async fn chunked_oversize_releases_admission_permit() {
    let app = super::fixture::app();
    for _ in 0..16 {
        let parts: Vec<Result<Bytes, io::Error>> = (0..17)
            .map(|_| Ok(Bytes::from(vec![b'Z'; 1 << 20])))
            .collect();
        let res = app
            .clone()
            .oneshot(chunked(Body::from_stream(stream::iter(parts))))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
    let ok = app
        .oneshot(Request::post("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}
