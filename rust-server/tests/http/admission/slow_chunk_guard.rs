use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, Request, StatusCode, header::TRANSFER_ENCODING},
};
use futures_util::{StreamExt, stream};
use libskrynia::http;
use std::io;
use tokio::time::Duration;
use tower::ServiceExt;

const ECHO: &str = include_str!("../../../test-fixtures/d2_echo.wat");

fn chunked(body: Body) -> Request<Body> {
    let mut r = Request::post("/").body(body).unwrap();
    r.headers_mut()
        .insert(TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    r
}

#[tokio::test(start_paused = true)]
async fn stalled_chunked_upload_still_times_out() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(super::kernel_with_wasm("skrynia-http-slow-stall", &wasm));
    let parts =
        stream::iter(vec![Ok::<_, io::Error>(Bytes::from_static(b"a"))]).chain(stream::pending());
    let task = tokio::spawn(app.oneshot(chunked(Body::from_stream(parts))));
    super::tick(Duration::from_secs(10), 4).await;
    let res = task.await.unwrap().unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test(start_paused = true)]
async fn slow_oversize_chunked_is_413_not_truncated_200() {
    let wasm = wat::parse_str(ECHO).unwrap();
    let app = http::router(super::kernel_with_wasm("skrynia-http-slow-oversize", &wasm));
    let parts: Vec<Result<Bytes, io::Error>> = (0..17)
        .map(|_| Ok(Bytes::from(vec![b'Z'; 1 << 20])))
        .collect();
    let paced = stream::iter(parts).then(|p| async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        p
    });
    let task = tokio::spawn(app.oneshot(chunked(Body::from_stream(paced))));
    super::tick(Duration::from_secs(1), 20).await;
    let res = task.await.unwrap().unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
