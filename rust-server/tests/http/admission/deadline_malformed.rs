use super::chunked;
use axum::{
    body::{Body, Bytes},
    http::StatusCode,
};
use futures_util::{StreamExt, stream};
use http_body_util::BodyExt;
use libskrynia::http;
use std::io;
use tokio::time::Duration;
use tower::ServiceExt;

const ECHO: &str = include_str!("../../../test-fixtures/d2_echo.wat");

#[tokio::test(start_paused = true)]
async fn buffered_chunked_still_respects_total_deadline() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(super::kernel_with_wasm("skrynia-http-deadline", &wasm));
    let parts: Vec<Result<Bytes, io::Error>> =
        (0..16).map(|_| Ok(Bytes::from_static(b"x"))).collect();
    let paced = stream::iter(parts).then(|p| async {
        tokio::time::sleep(Duration::from_secs(20)).await;
        p
    });
    let task = tokio::spawn(app.oneshot(chunked(Body::from_stream(paced))));
    super::tick(Duration::from_secs(20), 20).await;
    let res = task.await.unwrap().unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn chunked_stream_error_is_400_not_truncated_200() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(super::kernel_with_wasm("skrynia-http-malformed", &wasm));
    let parts = vec![
        Ok::<_, io::Error>(Bytes::from_static(b"abc")),
        Err(io::Error::new(io::ErrorKind::InvalidData, "bad chunk")),
    ];
    let res = app
        .oneshot(chunked(Body::from_stream(stream::iter(parts))))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(
        res.into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
}
